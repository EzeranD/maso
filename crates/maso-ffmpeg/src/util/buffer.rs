// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{mem::ManuallyDrop, ptr::NonNull};

use ffmsys::{AVBufferRef, av_buffer_ref, av_buffer_unref};
use snafu::{OptionExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("buffer reference or data is null"))]
    NullBuffer,
}

pub struct BufferRef {
    ptr: NonNull<AVBufferRef>,
}

impl BufferRef {
    pub fn try_ref(&self) -> Result<Self, Error> {
        let ptr = unsafe { av_buffer_ref(self.ptr.as_ptr()) };
        Ok(Self {
            ptr: NonNull::new(ptr).context(NullBufferSnafu)?,
        })
    }

    pub(crate) unsafe fn from_ptr(ptr: *mut AVBufferRef) -> Option<Self> {
        let ptr = NonNull::new(ptr)?;
        Some(Self { ptr })
    }

    pub(crate) unsafe fn ref_from_ptr(ptr: *mut AVBufferRef) -> Result<Self, Error> {
        if ptr.is_null() {
            return NullBufferSnafu.fail();
        }
        let ptr = unsafe { av_buffer_ref(ptr) };
        Ok(Self {
            ptr: NonNull::new(ptr).context(NullBufferSnafu)?,
        })
    }

    pub(crate) fn as_ptr(&self) -> *const AVBufferRef {
        self.ptr.as_ptr()
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut AVBufferRef {
        self.ptr.as_ptr()
    }

    pub fn data_ptr<T>(&self) -> *const T {
        unsafe { self.ptr.as_ref().data.cast_const().cast() }
    }

    pub(crate) fn into_ptr(self) -> *mut AVBufferRef {
        let buffer = ManuallyDrop::new(self);
        buffer.ptr.as_ptr()
    }
}

impl Clone for BufferRef {
    fn clone(&self) -> Self {
        self.try_ref().expect("failed to clone buffer reference")
    }
}

impl Drop for BufferRef {
    fn drop(&mut self) {
        let mut ptr = self.as_mut_ptr();
        unsafe {
            av_buffer_unref(&raw mut ptr);
        }
    }
}

unsafe impl Send for BufferRef {}
unsafe impl Sync for BufferRef {}
