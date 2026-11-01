// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{marker::PhantomData, ptr};

use ffmsys::{AVDiscard, AVProgram, av_find_program_from_stream};

use crate::{
    format::context::inner::Inner as ContextInner,
    util::dictionary::{DictionaryMut, DictionaryRef},
};

mod inner;
use inner::Inner;

pub struct Program<'a> {
    inner: Inner,
    context: &'a ContextInner,
}

pub struct ProgramMut<'a> {
    inner: Inner,
    context: PhantomData<&'a mut ContextInner>,
}

pub struct ProgramIter<'a> {
    context: &'a ContextInner,
    stream_index: i32,
    last: *mut AVProgram,
}

pub struct ProgramIterMut<'a> {
    context: &'a mut ContextInner,
    stream_index: i32,
    last: *mut AVProgram,
}

impl Program<'_> {
    pub fn id(&self) -> i32 {
        self.inner.id()
    }
    pub fn flags(&self) -> i32 {
        self.inner.flags()
    }
    pub fn discard(&self) -> AVDiscard {
        self.inner.discard()
    }
    pub fn stream_indexes(&self) -> &[u32] {
        self.inner.stream_indexes()
    }
    pub fn metadata(&self) -> DictionaryRef<'_> {
        self.inner.metadata(self.context)
    }
    pub fn program_num(&self) -> i32 {
        self.inner.program_num()
    }
    pub fn pmt_pid(&self) -> i32 {
        self.inner.pmt_pid()
    }
    pub fn pcr_pid(&self) -> i32 {
        self.inner.pcr_pid()
    }
    pub fn pmt_version(&self) -> i32 {
        self.inner.pmt_version()
    }
}

impl<'a> Program<'a> {
    pub(crate) unsafe fn from_ptr(ptr: *mut AVProgram, context: &'a ContextInner) -> Option<Self> {
        Some(Self {
            inner: unsafe { Inner::from_ptr(ptr)? },
            context,
        })
    }

    pub fn as_ptr(&self) -> *const AVProgram {
        self.inner.as_ptr()
    }
}

impl ProgramMut<'_> {
    pub fn id(&self) -> i32 {
        self.inner.id()
    }

    pub fn flags(&self) -> i32 {
        self.inner.flags()
    }

    pub fn discard(&self) -> AVDiscard {
        self.inner.discard()
    }

    pub fn stream_indexes(&self) -> &[u32] {
        self.inner.stream_indexes()
    }

    pub fn metadata(&self) -> DictionaryRef<'_> {
        self.inner.metadata(&self.inner)
    }

    pub fn program_num(&self) -> i32 {
        self.inner.program_num()
    }

    pub fn pmt_pid(&self) -> i32 {
        self.inner.pmt_pid()
    }

    pub fn pcr_pid(&self) -> i32 {
        self.inner.pcr_pid()
    }

    pub fn pmt_version(&self) -> i32 {
        self.inner.pmt_version()
    }

    pub fn set_discard(&mut self, discard: AVDiscard) {
        self.inner.set_discard(discard);
    }

    pub fn metadata_mut(&mut self) -> DictionaryMut<'_> {
        self.inner.metadata_mut()
    }
}

impl<'a> ProgramMut<'a> {
    pub(crate) unsafe fn from_ptr(
        ptr: *mut AVProgram,
        _context: &'a mut ContextInner,
    ) -> Option<Self> {
        Some(Self {
            inner: unsafe { Inner::from_ptr(ptr)? },
            context: PhantomData,
        })
    }

    pub fn as_ptr(&self) -> *const AVProgram {
        self.inner.as_ptr()
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut AVProgram {
        self.inner.as_mut_ptr()
    }
}

impl<'a> ProgramIter<'a> {
    pub(crate) fn new(context: &'a ContextInner, stream_index: i32) -> Self {
        Self {
            context,
            stream_index,
            last: ptr::null_mut(),
        }
    }
}

impl<'a> Iterator for ProgramIter<'a> {
    type Item = Program<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let ptr = unsafe {
            av_find_program_from_stream(
                self.context.as_ptr().cast_mut(),
                self.last,
                self.stream_index,
            )
        };
        if ptr.is_null() {
            return None;
        }

        self.last = ptr;
        unsafe { Program::from_ptr(ptr, self.context) }
    }
}

impl<'a> ProgramIterMut<'a> {
    pub(crate) fn new(context: &'a mut ContextInner, stream_index: i32) -> Self {
        Self {
            context,
            stream_index,
            last: ptr::null_mut(),
        }
    }
}

impl<'a> Iterator for ProgramIterMut<'a> {
    type Item = ProgramMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let ptr = unsafe {
            av_find_program_from_stream(self.context.as_mut_ptr(), self.last, self.stream_index)
        };
        if ptr.is_null() {
            return None;
        }

        self.last = ptr;
        Some(ProgramMut {
            inner: unsafe { Inner::from_ptr(ptr)? },
            context: PhantomData,
        })
    }
}
