// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::ptr::NonNull;

use ffmsys::AVChapter;

use crate::util::{
    Rational,
    dictionary::{DictionaryMut, DictionaryRef},
};

pub(super) struct Inner {
    ptr: NonNull<AVChapter>,
}

impl Inner {
    pub(super) fn id(&self) -> i64 {
        unsafe { self.ptr.as_ref().id }
    }

    pub(super) fn time_base(&self) -> Rational {
        unsafe { self.ptr.as_ref().time_base.into() }
    }

    pub(super) fn start(&self) -> i64 {
        unsafe { self.ptr.as_ref().start }
    }

    pub(super) fn end(&self) -> i64 {
        unsafe { self.ptr.as_ref().end }
    }

    pub(super) fn metadata<'a, T>(&self, context: &'a T) -> DictionaryRef<'a> {
        unsafe { DictionaryRef::from_ptr(self.ptr.as_ref().metadata, context) }
    }
    pub(super) fn metadata_mut(&mut self) -> DictionaryMut<'_> {
        unsafe { DictionaryMut::from_ptr(&mut self.ptr.as_mut().metadata) }
    }
}

impl Inner {
    pub(super) unsafe fn from_ptr(ptr: *mut AVChapter) -> Option<Self> {
        Some(Self {
            ptr: NonNull::new(ptr)?,
        })
    }

    pub(super) fn as_ptr(&self) -> *const AVChapter {
        self.ptr.as_ptr()
    }

    pub(super) fn as_mut_ptr(&mut self) -> *mut AVChapter {
        self.ptr.as_ptr()
    }
}
