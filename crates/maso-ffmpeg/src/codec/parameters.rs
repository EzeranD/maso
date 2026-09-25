// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::ptr::NonNull;

use ffmsys::{
    AVCodecParameters, avcodec_parameters_alloc, avcodec_parameters_copy, avcodec_parameters_free,
    avcodec_parameters_from_context,
};
use snafu::{OptionExt, ResultExt, Snafu};

use crate::{
    codec::{CodecId, context::Context},
    util::Code,
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("failed to allocate parameters"))]
    Allocate,
    #[snafu(display("failed to copy context parameters"))]
    FromContext { source: Code },
    #[snafu(display("failed to copy parameters"))]
    Copy { source: Code },
}

pub struct Parameters {
    ptr: NonNull<AVCodecParameters>,
}

impl Parameters {
    pub fn new() -> Result<Self, Error> {
        unsafe {
            let ptr = avcodec_parameters_alloc();
            Self::from_ptr(ptr).context(AllocateSnafu)
        }
    }

    pub fn from_context<State>(context: &Context<State>) -> Result<Self, Error> {
        let mut par = Self::new()?;
        unsafe {
            let res = avcodec_parameters_from_context(par.as_mut_ptr(), context.as_ptr());
            if res < 0 {
                return Err(res.into()).context(FromContextSnafu);
            }
        }
        Ok(par)
    }

    pub fn try_clone(&self) -> Result<Self, Error> {
        let mut par = Self::new()?;
        unsafe {
            let res = avcodec_parameters_copy(par.as_mut_ptr(), self.as_ptr());
            if res < 0 {
                return Err(res.into()).context(CopySnafu);
            }
        }
        Ok(par)
    }

    pub fn codec_id(&self) -> CodecId {
        unsafe { CodecId::from_raw(self.ptr.as_ref().codec_id) }
    }
}

impl Parameters {
    pub unsafe fn from_ptr(ptr: *mut AVCodecParameters) -> Option<Self> {
        let ptr = NonNull::new(ptr)?;
        Some(Self { ptr })
    }

    pub fn as_ptr(&self) -> *const AVCodecParameters {
        self.ptr.as_ptr()
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut AVCodecParameters {
        self.ptr.as_ptr()
    }
}

impl Clone for Parameters {
    fn clone(&self) -> Self {
        self.try_clone().expect("failed to clone codec parameters")
    }
}

impl Drop for Parameters {
    fn drop(&mut self) {
        let mut ptr = self.as_mut_ptr();
        unsafe {
            avcodec_parameters_free(&raw mut ptr);
        }
    }
}
