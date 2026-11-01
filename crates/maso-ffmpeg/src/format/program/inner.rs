// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{ptr::NonNull, slice};

use ffmsys::{AVDiscard, AVProgram};

use crate::util::dictionary::{DictionaryMut, DictionaryRef};

pub(super) struct Inner {
    ptr: NonNull<AVProgram>,
}

impl Inner {
    pub(super) fn id(&self) -> i32 {
        unsafe { self.ptr.as_ref().id }
    }

    pub(super) fn flags(&self) -> i32 {
        unsafe { self.ptr.as_ref().flags }
    }

    pub(super) fn discard(&self) -> AVDiscard {
        unsafe { self.ptr.as_ref().discard }
    }

    pub(super) fn stream_indexes(&self) -> &[u32] {
        unsafe {
            let program = self.ptr.as_ref();
            if program.nb_stream_indexes == 0 {
                return &[];
            }
            slice::from_raw_parts(program.stream_index, program.nb_stream_indexes as usize)
        }
    }

    pub(super) fn metadata<'a, T>(&self, context: &'a T) -> DictionaryRef<'a> {
        unsafe { DictionaryRef::from_ptr(self.ptr.as_ref().metadata, context) }
    }

    pub(super) fn program_num(&self) -> i32 {
        unsafe { self.ptr.as_ref().program_num }
    }

    pub(super) fn pmt_pid(&self) -> i32 {
        unsafe { self.ptr.as_ref().pmt_pid }
    }

    pub(super) fn pcr_pid(&self) -> i32 {
        unsafe { self.ptr.as_ref().pcr_pid }
    }

    pub(super) fn pmt_version(&self) -> i32 {
        unsafe { self.ptr.as_ref().pmt_version }
    }

    pub(super) fn set_discard(&mut self, discard: AVDiscard) {
        unsafe {
            self.ptr.as_mut().discard = discard;
        }
    }

    pub(super) fn metadata_mut(&mut self) -> DictionaryMut<'_> {
        unsafe { DictionaryMut::from_ptr(&mut self.ptr.as_mut().metadata) }
    }
}

impl Inner {
    pub(super) unsafe fn from_ptr(ptr: *mut AVProgram) -> Option<Self> {
        Some(Self {
            ptr: NonNull::new(ptr)?,
        })
    }

    pub(super) fn as_ptr(&self) -> *const AVProgram {
        self.ptr.as_ptr()
    }
    pub(super) fn as_mut_ptr(&mut self) -> *mut AVProgram {
        self.ptr.as_ptr()
    }
}
