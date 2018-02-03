
#[cfg(all(not(all(target_os = "linux", not(target_env = "musl"))),
          not(target_os = "freebsd"),
          not(target_os = "macos"),
          not(target_os = "bitrig"),
          not(all(target_os = "netbsd", not(target_vendor = "rumprun"))),
          not(target_os = "openbsd"),
          not(target_os = "solaris")))]
#[cfg_attr(test, allow(dead_code))]
pub mod guard {
    pub unsafe fn current() -> Option<usize> { None }
    pub unsafe fn init() -> Option<usize> { None }
}

#[cfg(any(all(target_os = "linux", not(target_env = "musl")),
          target_os = "freebsd",
          target_os = "macos",
          target_os = "bitrig",
          all(target_os = "netbsd", not(target_vendor = "rumprun")),
          target_os = "openbsd",
          target_os = "solaris"))]
#[cfg_attr(test, allow(dead_code))]
pub mod guard {
    use libc;
    use libc::mmap;
    use libc::{PROT_NONE, MAP_PRIVATE, MAP_ANON, MAP_FAILED, MAP_FIXED};
    use os;

    #[cfg(any(target_os = "macos",
              target_os = "bitrig",
              target_os = "openbsd",
              target_os = "solaris"))]
    unsafe fn get_stack_start() -> Option<*mut libc::c_void> {
        current().map(|s| s as *mut libc::c_void)
    }

    #[cfg(any(target_os = "android", target_os = "freebsd",
              target_os = "linux", target_os = "netbsd", target_os = "l4re"))]
    unsafe fn get_stack_start() -> Option<*mut libc::c_void> {
        let mut ret = None;
        let mut attr: libc::pthread_attr_t = ::core::mem::zeroed();
        assert_eq!(libc::pthread_attr_init(&mut attr), 0);
        #[cfg(target_os = "freebsd")]
            let e = libc::pthread_attr_get_np(libc::pthread_self(), &mut attr);
        #[cfg(not(target_os = "freebsd"))]
            let e = libc::pthread_getattr_np(libc::pthread_self(), &mut attr);
        if e == 0 {
            let mut stackaddr = ::core::ptr::null_mut();
            let mut stacksize = 0;
            assert_eq!(libc::pthread_attr_getstack(&attr, &mut stackaddr,
                                                   &mut stacksize), 0);
            ret = Some(stackaddr);
        }
        assert_eq!(libc::pthread_attr_destroy(&mut attr), 0);
        ret
    }

    pub unsafe fn init() -> Option<usize> {
        let psize = os::page_size();
        let mut stackaddr = get_stack_start()?;

        // Ensure stackaddr is page aligned! A parent process might
        // have reset RLIMIT_STACK to be non-page aligned. The
        // pthread_attr_getstack() reports the usable stack area
        // stackaddr < stackaddr + stacksize, so if stackaddr is not
        // page-aligned, calculate the fix such that stackaddr <
        // new_page_aligned_stackaddr < stackaddr + stacksize
        let remainder = (stackaddr as usize) % psize;
        if remainder != 0 {
            stackaddr = ((stackaddr as usize) + psize - remainder)
                as *mut libc::c_void;
        }

        if cfg!(target_os = "linux") {
            // Linux doesn't allocate the whole stack right away, and
            // the kernel has its own stack-guard mechanism to fault
            // when growing too close to an existing mapping.  If we map
            // our own guard, then the kernel starts enforcing a rather
            // large gap above that, rendering much of the possible
            // stack space useless.  See #43052.
            //
            // Instead, we'll just note where we expect rlimit to start
            // faulting, so our handler can report "stack overflow", and
            // trust that the kernel's own stack guard will work.
            Some(stackaddr as usize)
        } else {
            // Reallocate the last page of the stack.
            // This ensures SIGBUS will be raised on
            // stack overflow.
            let result = mmap(stackaddr, psize, PROT_NONE,
                              MAP_PRIVATE | MAP_ANON | MAP_FIXED, -1, 0);

            if result != stackaddr || result == MAP_FAILED {
                panic!("failed to allocate a guard page");
            }

            let offset = if cfg!(target_os = "freebsd") {
                2
            } else {
                1
            };

            Some(stackaddr as usize + offset * psize)
        }
    }

    #[cfg(target_os = "solaris")]
    pub unsafe fn current() -> Option<usize> {
        let mut current_stack: libc::stack_t = ::mem::zeroed();
        assert_eq!(libc::stack_getbounds(&mut current_stack), 0);
        Some(current_stack.ss_sp as usize)
    }

    #[cfg(target_os = "macos")]
    pub unsafe fn current() -> Option<usize> {
        Some(libc::pthread_get_stackaddr_np(libc::pthread_self()) as usize -
             libc::pthread_get_stacksize_np(libc::pthread_self()))
    }

    #[cfg(any(target_os = "openbsd", target_os = "bitrig"))]
    pub unsafe fn current() -> Option<usize> {
        let mut current_stack: libc::stack_t = ::core::mem::zeroed();
        assert_eq!(libc::pthread_stackseg_np(libc::pthread_self(),
                                             &mut current_stack), 0);

        let extra = if cfg!(target_os = "bitrig") {3} else {1} * os::page_size();
        Some(if libc::pthread_main_np() == 1 {
            // main thread
            current_stack.ss_sp as usize - current_stack.ss_size + extra
        } else {
            // new thread
            current_stack.ss_sp as usize - current_stack.ss_size
        })
    }

    #[cfg(any(target_os = "android", target_os = "freebsd",
              target_os = "linux", target_os = "netbsd", target_os = "l4re"))]
    pub unsafe fn current() -> Option<usize> {
        let mut ret = None;
        let mut attr: libc::pthread_attr_t = ::core::mem::zeroed();
        assert_eq!(libc::pthread_attr_init(&mut attr), 0);
        #[cfg(target_os = "freebsd")]
            let e = libc::pthread_attr_get_np(libc::pthread_self(), &mut attr);
        #[cfg(not(target_os = "freebsd"))]
            let e = libc::pthread_getattr_np(libc::pthread_self(), &mut attr);
        if e == 0 {
            let mut guardsize = 0;
            assert_eq!(libc::pthread_attr_getguardsize(&attr, &mut guardsize), 0);
            if guardsize == 0 {
                panic!("there is no guard page");
            }
            let mut stackaddr = ::core::ptr::null_mut();
            let mut size = 0;
            assert_eq!(libc::pthread_attr_getstack(&attr, &mut stackaddr,
                                                   &mut size), 0);

            ret = if cfg!(target_os = "freebsd") {
                Some(stackaddr as usize - guardsize)
            } else if cfg!(target_os = "netbsd") {
                Some(stackaddr as usize)
            } else {
                Some(stackaddr as usize + guardsize)
            };
        }
        assert_eq!(libc::pthread_attr_destroy(&mut attr), 0);
        ret
    }
}