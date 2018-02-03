#![no_std]

#![feature(abstract_platform)]
#![feature(alloc)]
#![feature(const_fn)]
#![feature(unicode)]
#![feature(str_internals)]
#![feature(try_from)]

extern crate abstract_platform as ap;
extern crate libc;
extern crate alloc;
extern crate std_unicode;

use ap::prelude::*;

#[macro_use]
mod weak;
mod thread;
mod os;
mod mutex;
mod args;
mod fd;
mod io;
mod memchr;
mod ffi;
mod pipe;
mod os_str;
mod path;
mod time;
// mod rand;
// mod stack_overflow;

use ap::traits;
use ap::io::ErrorKind;
use ap::os::raw::c_char;

use libc::signal;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
pub struct Std;

impl traits::Std for Std {
    type c_char = libc::c_char;
    type c_double = libc::c_double;
    type c_float = libc::c_float;
    type c_int = libc::c_int;
    type c_long = libc::c_long;
    type c_longlong = libc::c_longlong;
    type c_schar = libc::c_schar;
    type c_short = libc::c_short;
    type c_uchar = libc::c_uchar;
    type c_uint = libc::c_uint;
    type c_ulong = libc::c_ulong;
    type c_ulonglong = libc::c_ulonglong;
    type c_ushort = libc::c_ushort;
    type Mutex = mutex::Mutex;
    type OsString = os_str::Buf;
    type OsStr = os_str::Slice;
    type SystemTime = time::SystemTime;
    type Instant = time::Instant;

    const UNIX_EPOCH: Self::SystemTime = time::UNIX_EPOCH;

    fn empty_cstr() -> &'static [c_char<Self>] {
        &[0]
    }

    #[inline]
    fn last_os_error() -> i32 {
        os::errno()
    }

    #[inline]
    fn error_string(code: i32) -> String {
        os::error_string(code)
    }

    fn init() {
        // By default, some platforms will send a *signal* when an EPIPE error
        // would otherwise be delivered. This runtime doesn't install a SIGPIPE
        // handler, causing it to kill the program, which isn't exactly what we
        // want!
        //
        // Hence, we set SIGPIPE to ignore when the program starts up in order
        // to prevent this problem.
        unsafe {
            reset_sigpipe();
        }

        #[cfg(not(any(target_os = "emscripten", target_os="fuchsia")))]
        unsafe fn reset_sigpipe() {
            assert!(signal(libc::SIGPIPE, libc::SIG_IGN) != libc::SIG_ERR);
        }

        #[cfg(any(target_os = "emscripten", target_os="fuchsia"))]
        unsafe fn reset_sigpipe() {}
    }

    #[inline]
    unsafe fn abort_internal() -> ! {
        libc::abort()
    }

    #[inline]
    unsafe fn strlen(cs: *const c_char<Self>) -> usize {
        libc::strlen(cs) as usize
    }

    fn decode_error_kind(errno: i32) -> ErrorKind {
        match errno as libc::c_int {
            libc::ECONNREFUSED => ErrorKind::ConnectionRefused,
            libc::ECONNRESET => ErrorKind::ConnectionReset,
            libc::EPERM | libc::EACCES => ErrorKind::PermissionDenied,
            libc::EPIPE => ErrorKind::BrokenPipe,
            libc::ENOTCONN => ErrorKind::NotConnected,
            libc::ECONNABORTED => ErrorKind::ConnectionAborted,
            libc::EADDRNOTAVAIL => ErrorKind::AddrNotAvailable,
            libc::EADDRINUSE => ErrorKind::AddrInUse,
            libc::ENOENT => ErrorKind::NotFound,
            libc::EINTR => ErrorKind::Interrupted,
            libc::EINVAL => ErrorKind::InvalidInput,
            libc::ETIMEDOUT => ErrorKind::TimedOut,
            libc::EEXIST => ErrorKind::AlreadyExists,

            // These two constants can have the same value on some systems,
            // but different values on others, so we can't use a match
            // clause
            x if x == libc::EAGAIN || x == libc::EWOULDBLOCK =>
                ErrorKind::WouldBlock,

            _ => ErrorKind::Other,
        }
    }

    #[inline]
    unsafe fn thread_guard_init() -> Option<usize> {
        thread::guard::init()
    }

    #[inline]
    fn is_path_sep_byte(b: u8) -> bool {
        path::is_sep_byte(b)
    }

    #[inline]
    fn is_verbatim_path_sep(b: u8) -> bool {
        path::is_verbatim_sep(b)
    }

    #[inline]
    fn parse_path_prefix(p: &ffi::OsStr) -> Option<path::Prefix> {
        path::parse_prefix(p)
    }

    const MAIN_PATH_SEP_STR: &'static str = path::MAIN_SEP_STR;

    const MAIN_PATH_SEP: char = path::MAIN_SEP;

    fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
        memchr::memchr(needle, haystack)
    }

    fn memrchr(needle: u8, haystack: &[u8]) -> Option<usize> {
        memchr::memrchr(needle, haystack)
    }

    #[inline]
    unsafe fn args_init(args: isize, argv: *const *const u8) {
        args::init(args, argv);
    }

    // fn hashmap_random_keys() -> (u64, u64) {
    //     rand::hashmap_random_keys()
    // }
}

trait IsMinusOne {
    fn is_minus_one(&self) -> bool;
}

macro_rules! impl_is_minus_one {
    ($($t:ident)*) => ($(impl IsMinusOne for $t {
        fn is_minus_one(&self) -> bool {
            *self == -1
        }
    })*)
}

impl_is_minus_one! { i8 i16 i32 i64 isize }

fn cvt<T: IsMinusOne>(t: T) -> io::Result<T> {
    if t.is_minus_one() {
        Err(io::Error::last_os_error())
    } else {
        Ok(t)
    }
}

fn cvt_r<T, F>(mut f: F) -> io::Result<T>
    where T: IsMinusOne,
          F: FnMut() -> T,
          Std: traits::Std
{
    loop {
        match cvt(f()) {
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
            other => return other,
        }
    }
}
