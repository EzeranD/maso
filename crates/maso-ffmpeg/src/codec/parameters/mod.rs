// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{marker::PhantomData, mem::ManuallyDrop};

use ffmsys::AVCodecParameters;
use snafu::Snafu;

use crate::{
    codec::{
        CodecId,
        context::{Context, Mode},
    },
    util::{
        Code, MediaType, Rational,
        media::{Audio, Kind, Unknown, Video},
    },
};

mod inner;
use inner::Inner;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("failed to allocate parameters"))]
    Allocate,
    #[snafu(display("failed to copy context parameters"))]
    FromContext { source: Code },
    #[snafu(display("failed to copy parameters"))]
    Copy { source: Code },
}

pub struct Parameters<K: Kind = Unknown> {
    inner: Inner,
    _kind: PhantomData<K>,
}

pub struct ParametersRef<'a, K: Kind = Unknown> {
    inner: ManuallyDrop<Inner>,
    owner: PhantomData<(&'a (), K)>,
}

impl Parameters<Unknown> {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            inner: Inner::new()?,
            _kind: PhantomData,
        })
    }
}

impl<K: Kind> Parameters<K> {
    pub fn from_context<State, M: Mode>(context: &Context<State, M, K>) -> Result<Self, Error> {
        Ok(Self {
            inner: Inner::from_context(context)?,
            _kind: PhantomData,
        })
    }

    pub fn try_clone(&self) -> Result<Self, Error> {
        Ok(Self {
            inner: self.inner.try_clone()?,
            _kind: PhantomData,
        })
    }

    pub fn media_type(&self) -> MediaType {
        self.inner.media_type()
    }

    pub fn codec_tag(&self) -> u32 {
        self.inner.codec_tag()
    }

    pub fn codec_id(&self) -> CodecId {
        self.inner.codec_id()
    }

    pub fn extradata(&self) -> Option<&[u8]> {
        self.inner.extradata()
    }

    pub fn bit_rate(&self) -> i64 {
        self.inner.bit_rate()
    }

    pub fn bits_per_coded_sample(&self) -> i32 {
        self.inner.bits_per_coded_sample()
    }

    pub fn bits_per_raw_sample(&self) -> i32 {
        self.inner.bits_per_raw_sample()
    }

    pub fn profile(&self) -> i32 {
        self.inner.profile()
    }

    pub fn level(&self) -> i32 {
        self.inner.level()
    }

    pub unsafe fn from_ptr(ptr: *mut AVCodecParameters) -> Option<Self> {
        Some(Self {
            inner: unsafe { Inner::from_ptr(ptr)? },
            _kind: PhantomData,
        })
    }

    pub fn as_ptr(&self) -> *const AVCodecParameters {
        self.inner.as_ptr()
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut AVCodecParameters {
        self.inner.as_mut_ptr()
    }
}

impl Parameters<Video> {
    pub fn width(&self) -> i32 {
        self.inner.width()
    }

    pub fn height(&self) -> i32 {
        self.inner.height()
    }

    pub fn sample_aspect_ratio(&self) -> Rational {
        self.inner.sample_aspect_ratio()
    }

    #[cfg(ffmpeg_6_1)]
    pub fn framerate(&self) -> Rational {
        self.inner.framerate()
    }

    pub fn video_delay(&self) -> i32 {
        self.inner.video_delay()
    }
}

impl Parameters<Audio> {
    pub fn sample_rate(&self) -> i32 {
        self.inner.sample_rate()
    }

    pub fn block_align(&self) -> i32 {
        self.inner.block_align()
    }

    pub fn frame_size(&self) -> i32 {
        self.inner.frame_size()
    }

    pub fn initial_padding(&self) -> i32 {
        self.inner.initial_padding()
    }

    pub fn trailing_padding(&self) -> i32 {
        self.inner.trailing_padding()
    }

    pub fn seek_preroll(&self) -> i32 {
        self.inner.seek_preroll()
    }
}

impl<'a, K: Kind> ParametersRef<'a, K> {
    pub fn try_to_owned(&self) -> Result<Parameters<K>, Error> {
        Ok(Parameters {
            inner: self.inner.try_clone()?,
            _kind: PhantomData,
        })
    }

    pub fn to_owned(&self) -> Parameters<K> {
        self.try_to_owned().expect("failed to clone parameters")
    }

    pub fn media_type(&self) -> MediaType {
        self.inner.media_type()
    }

    pub fn codec_tag(&self) -> u32 {
        self.inner.codec_tag()
    }

    pub fn codec_id(&self) -> CodecId {
        self.inner.codec_id()
    }

    pub fn extradata(&self) -> Option<&[u8]> {
        self.inner.extradata()
    }

    pub fn bit_rate(&self) -> i64 {
        self.inner.bit_rate()
    }

    pub fn bits_per_coded_sample(&self) -> i32 {
        self.inner.bits_per_coded_sample()
    }

    pub fn bits_per_raw_sample(&self) -> i32 {
        self.inner.bits_per_raw_sample()
    }

    pub fn profile(&self) -> i32 {
        self.inner.profile()
    }

    pub fn level(&self) -> i32 {
        self.inner.level()
    }

    pub(crate) unsafe fn from_ptr<T>(ptr: *mut AVCodecParameters, _owner: &'a T) -> Option<Self> {
        Some(Self {
            inner: ManuallyDrop::new(unsafe { Inner::from_ptr(ptr)? }),
            owner: PhantomData,
        })
    }

    pub fn as_ptr(&self) -> *const AVCodecParameters {
        self.inner.as_ptr()
    }
}

impl ParametersRef<'_, Video> {
    pub fn width(&self) -> i32 {
        self.inner.width()
    }

    pub fn height(&self) -> i32 {
        self.inner.height()
    }

    pub fn sample_aspect_ratio(&self) -> Rational {
        self.inner.sample_aspect_ratio()
    }

    #[cfg(ffmpeg_6_1)]
    pub fn framerate(&self) -> Rational {
        self.inner.framerate()
    }

    pub fn video_delay(&self) -> i32 {
        self.inner.video_delay()
    }
}

impl ParametersRef<'_, Audio> {
    pub fn sample_rate(&self) -> i32 {
        self.inner.sample_rate()
    }

    pub fn block_align(&self) -> i32 {
        self.inner.block_align()
    }

    pub fn frame_size(&self) -> i32 {
        self.inner.frame_size()
    }

    pub fn initial_padding(&self) -> i32 {
        self.inner.initial_padding()
    }

    pub fn trailing_padding(&self) -> i32 {
        self.inner.trailing_padding()
    }

    pub fn seek_preroll(&self) -> i32 {
        self.inner.seek_preroll()
    }
}

unsafe impl<K: Kind> Send for Parameters<K> {}
unsafe impl<K: Kind> Sync for Parameters<K> {}

impl<K: Kind> Clone for Parameters<K> {
    fn clone(&self) -> Self {
        self.try_clone().expect("failed to clone codec parameters")
    }
}
