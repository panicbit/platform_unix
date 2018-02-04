// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use Std;
use ap::traits::Stdio;
use ap::io;

use libc;
use fd::FileDesc;

pub struct Stdin(());
pub struct Stdout(());
pub struct Stderr(());

impl Stdio<Std> for Stdin {
    fn new() -> io::Result<Stdin, Std> { Ok(Stdin(())) }
}

impl io::Read<Std> for Stdin {
    fn read(&mut self, data: &mut [u8]) -> io::Result<usize, Std> {
        let fd = FileDesc::new(libc::STDIN_FILENO);
        let ret = fd.read(data);
        fd.into_raw();
        ret
    }
}

impl Stdio<Std> for Stdout {
    fn new() -> io::Result<Stdout, Std> { Ok(Stdout(())) }
}

impl io::Write<Std> for Stdout {
    fn write(&mut self, data: &[u8]) -> io::Result<usize, Std> {
        let fd = FileDesc::new(libc::STDOUT_FILENO);
        let ret = fd.write(data);
        fd.into_raw();
        ret
    }

    fn flush(&mut self) -> io::Result<(), Std> {
        Ok(())
    }
}

impl Stdio<Std> for Stderr {
    fn new() -> io::Result<Stderr, Std> { Ok(Stderr(())) }
}

impl io::Write<Std> for Stderr {
    fn write(&mut self, data: &[u8]) -> io::Result<usize, Std> {
        let fd = FileDesc::new(libc::STDERR_FILENO);
        let ret = fd.write(data);
        fd.into_raw();
        ret
    }

    fn flush(&mut self) -> io::Result<(), Std> {
        Ok(())
    }
}

pub fn is_ebadf(err: &io::Error<Std>) -> bool {
    err.raw_os_error() == Some(libc::EBADF as i32)
}

pub const STDIN_BUF_SIZE: usize = ::ap::sys_common::io::DEFAULT_BUF_SIZE;
