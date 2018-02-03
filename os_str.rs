// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// The underlying OsString/OsStr implementation on Unix systems: just
/// a `Vec<u8>`/`[u8]`.

use Std;
use ap::prelude::*;
use ap::traits::{self, OsString as OsStringT, OsStr as OsStrT};

use alloc::borrow::Cow;
use core::fmt;
use core::str;
use core::mem;
use alloc::rc::Rc;
use alloc::arc::Arc;
use ap::sys_common::{AsInner, IntoInner};
use ap::sys_common::bytestring::debug_fmt_bytestring;
use std_unicode::lossy::Utf8Lossy;

#[derive(Clone, Hash)]
pub struct Buf {
    pub inner: Vec<u8>
}

pub struct Slice {
    pub inner: [u8]
}

impl fmt::Debug for Slice {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        debug_fmt_bytestring(&self.inner, formatter)
    }
}

impl fmt::Display for Slice {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&Utf8Lossy::from_bytes(&self.inner), formatter)
    }
}

impl fmt::Debug for Buf {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_slice(), formatter)
    }
}

impl fmt::Display for Buf {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_slice(), formatter)
    }
}

impl IntoInner<Vec<u8>> for Buf {
    fn into_inner(self) -> Vec<u8> {
        self.inner
    }
}

impl AsInner<[u8]> for Buf {
    fn as_inner(&self) -> &[u8] {
        &self.inner
    }
}


impl traits::OsString<Std> for Buf {
    fn from_string(s: String) -> Buf {
        Buf { inner: s.into_bytes() }
    }

    #[inline]
    fn with_capacity(capacity: usize) -> Buf {
        Buf {
            inner: Vec::with_capacity(capacity)
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.inner.clear()
    }

    #[inline]
    fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    #[inline]
    fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional)
    }

    #[inline]
    fn reserve_exact(&mut self, additional: usize) {
        self.inner.reserve_exact(additional)
    }

    #[inline]
    fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit()
    }

    fn as_slice(&self) -> &Slice {
        unsafe { mem::transmute(&*self.inner) }
    }

    fn into_string(self) -> Result<String, Buf> {
        String::from_utf8(self.inner).map_err(|p| Buf { inner: p.into_bytes() } )
    }

    fn push_slice(&mut self, s: &Slice) {
        self.inner.extend_from_slice(&s.inner)
    }

    #[inline]
    fn into_box(self) -> Box<Slice> {
        unsafe { mem::transmute(self.inner.into_boxed_slice()) }
    }

    #[inline]
    fn from_box(boxed: Box<Slice>) -> Buf {
        let inner: Box<[u8]> = unsafe { mem::transmute(boxed) };
        Buf { inner: inner.into_vec() }
    }

    #[inline]
    fn into_arc(&self) -> Arc<Slice> {
        self.as_slice().into_arc()
    }

    #[inline]
    fn into_rc(&self) -> Rc<Slice> {
        self.as_slice().into_rc()
    }
}

impl Slice {
    fn from_u8_slice(s: &[u8]) -> &Slice {
        unsafe { mem::transmute(s) }
    }
}

impl traits::OsStr<Std> for Slice {
    fn from_str(s: &str) -> &Slice {
        Slice::from_u8_slice(s.as_bytes())
    }

    fn to_str(&self) -> Option<&str> {
        str::from_utf8(&self.inner).ok()
    }

    fn to_string_lossy(&self) -> Cow<str> {
        String::from_utf8_lossy(&self.inner)
    }

    fn to_owned(&self) -> Buf {
        Buf { inner: self.inner.to_vec() }
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    #[inline]
    fn into_box(&self) -> Box<Slice> {
        let boxed: Box<[u8]> = self.inner.into();
        unsafe { mem::transmute(boxed) }
    }

    fn empty_box() -> Box<Slice> {
        let boxed: Box<[u8]> = Default::default();
        unsafe { mem::transmute(boxed) }
    }

    #[inline]
    fn into_arc(&self) -> Arc<Slice> {
        let arc: Arc<[u8]> = Arc::from(&self.inner);
        unsafe { Arc::from_raw(Arc::into_raw(arc) as *const Slice) }
    }

    #[inline]
    fn into_rc(&self) -> Rc<Slice> {
        let rc: Rc<[u8]> = Rc::from(&self.inner);
        unsafe { Rc::from_raw(Rc::into_raw(rc) as *const Slice) }
    }

    fn from_bytes(b: &[u8]) -> &Self {
        Self::from_u8_slice(b)
    }
}
