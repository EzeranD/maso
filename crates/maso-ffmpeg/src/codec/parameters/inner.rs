// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{ptr::NonNull, slice};

use ffmsys::{
    AVCodecParameters, avcodec_parameters_alloc, avcodec_parameters_copy, avcodec_parameters_free,
    avcodec_parameters_from_context,
};
use snafu::{OptionExt, ResultExt};

use super::{AllocateSnafu, CopySnafu, Error, FromContextSnafu};
use crate::{
    codec::{
        CodecId,
        context::{Context, Mode},
    },
    util::{MediaType, Rational, media::Kind},
};

pub(super) struct Inner {
    ptr: NonNull<AVCodecParameters>,
}

impl Inner {
    pub(super) fn new() -> Result<Self, Error> {
        unsafe { Self::from_ptr(avcodec_parameters_alloc()).context(AllocateSnafu) }
    }

    pub(super) fn from_context<State, M: Mode, K: Kind>(
        context: &Context<State, M, K>,
    ) -> Result<Self, Error> {
        let mut parameters = Self::new()?;
        let res =
            unsafe { avcodec_parameters_from_context(parameters.as_mut_ptr(), context.as_ptr()) };
        if res < 0 {
            return Err(res.into()).context(FromContextSnafu);
        }
        unsafe { parameters.ptr.as_mut().codec_type = K::TYPE.into() };
        Ok(parameters)
    }

    pub(super) fn try_clone(&self) -> Result<Self, Error> {
        let mut copy = Self::new()?;
        unsafe {
            let res = avcodec_parameters_copy(copy.as_mut_ptr(), self.as_ptr());
            if res < 0 {
                return Err(res.into()).context(CopySnafu);
            }
        }
        Ok(copy)
    }

    pub(super) fn media_type(&self) -> MediaType {
        unsafe { self.ptr.as_ref().codec_type.into() }
    }

    pub(super) fn codec_id(&self) -> CodecId {
        unsafe { CodecId::from_raw(self.ptr.as_ref().codec_id) }
    }

    pub(super) fn codec_tag(&self) -> u32 {
        unsafe { self.ptr.as_ref().codec_tag }
    }

    pub(super) fn extradata(&self) -> Option<&[u8]> {
        unsafe {
            let parameters = self.ptr.as_ref();
            if parameters.extradata.is_null() || parameters.extradata_size <= 0 {
                return None;
            }

            Some(slice::from_raw_parts(
                parameters.extradata,
                parameters.extradata_size as usize,
            ))
        }
    }

    pub(super) fn bit_rate(&self) -> i64 {
        unsafe { self.ptr.as_ref().bit_rate }
    }

    pub(super) fn bits_per_coded_sample(&self) -> i32 {
        unsafe { self.ptr.as_ref().bits_per_coded_sample }
    }

    pub(super) fn bits_per_raw_sample(&self) -> i32 {
        unsafe { self.ptr.as_ref().bits_per_raw_sample }
    }

    pub(super) fn profile(&self) -> i32 {
        unsafe { self.ptr.as_ref().profile }
    }

    pub(super) fn level(&self) -> i32 {
        unsafe { self.ptr.as_ref().level }
    }

    pub(super) fn width(&self) -> i32 {
        unsafe { self.ptr.as_ref().width }
    }

    pub(super) fn height(&self) -> i32 {
        unsafe { self.ptr.as_ref().height }
    }

    pub(super) fn sample_aspect_ratio(&self) -> Rational {
        unsafe { self.ptr.as_ref().sample_aspect_ratio.into() }
    }

    #[cfg(ffmpeg_6_1)]
    pub(super) fn framerate(&self) -> Rational {
        unsafe { self.ptr.as_ref().framerate.into() }
    }

    pub(super) fn video_delay(&self) -> i32 {
        unsafe { self.ptr.as_ref().video_delay }
    }

    pub(super) fn sample_rate(&self) -> i32 {
        unsafe { self.ptr.as_ref().sample_rate }
    }

    pub(super) fn block_align(&self) -> i32 {
        unsafe { self.ptr.as_ref().block_align }
    }

    pub(super) fn frame_size(&self) -> i32 {
        unsafe { self.ptr.as_ref().frame_size }
    }

    pub(super) fn initial_padding(&self) -> i32 {
        unsafe { self.ptr.as_ref().initial_padding }
    }

    pub(super) fn trailing_padding(&self) -> i32 {
        unsafe { self.ptr.as_ref().trailing_padding }
    }

    pub(super) fn seek_preroll(&self) -> i32 {
        unsafe { self.ptr.as_ref().seek_preroll }
    }

    pub(super) unsafe fn from_ptr(ptr: *mut AVCodecParameters) -> Option<Self> {
        Some(Self {
            ptr: NonNull::new(ptr)?,
        })
    }

    pub(super) fn as_ptr(&self) -> *const AVCodecParameters {
        self.ptr.as_ptr()
    }

    pub(super) fn as_mut_ptr(&mut self) -> *mut AVCodecParameters {
        self.ptr.as_ptr()
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        let mut ptr = self.as_mut_ptr();
        unsafe { avcodec_parameters_free(&raw mut ptr) }
    }
}
