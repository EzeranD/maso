// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    ffi::{CStr, CString},
    iter,
    ptr::{self, NonNull},
};

use ffmsys::{
    AVCodec, AVCodecID, av_codec_is_decoder, av_codec_is_encoder, av_codec_iterate,
    avcodec_find_decoder, avcodec_find_decoder_by_name, avcodec_find_encoder,
    avcodec_find_encoder_by_name,
};

#[derive(Debug, Clone, Copy)]
pub struct Codec {
    ptr: NonNull<AVCodec>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodecId(AVCodecID);

impl Codec {
    pub fn find_decoder(id: CodecId) -> Option<Codec> {
        unsafe {
            let ptr = avcodec_find_decoder(id.0);
            Self::from_ptr(ptr)
        }
    }

    pub fn find_decoder_by_name(name: &str) -> Option<Codec> {
        let name = CString::new(name).ok()?;
        unsafe {
            let ptr = avcodec_find_decoder_by_name(name.as_ptr());
            Self::from_ptr(ptr)
        }
    }

    pub fn find_encoder(id: CodecId) -> Option<Codec> {
        unsafe {
            let ptr = avcodec_find_encoder(id.0);
            Self::from_ptr(ptr)
        }
    }

    pub fn find_encoder_by_name(name: &str) -> Option<Codec> {
        let name = CString::new(name).ok()?;
        unsafe {
            let ptr = avcodec_find_encoder_by_name(name.as_ptr());
            Self::from_ptr(ptr)
        }
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        let mut opaque = ptr::null_mut();
        iter::from_fn(move || unsafe {
            let ptr = av_codec_iterate(&raw mut opaque);
            Self::from_ptr(ptr)
        })
    }

    pub fn is_decoder(&self) -> bool {
        unsafe {
            let res = av_codec_is_decoder(self.as_ptr());
            res != 0
        }
    }

    pub fn is_encoder(&self) -> bool {
        unsafe {
            let res = av_codec_is_encoder(self.as_ptr());
            res != 0
        }
    }

    pub fn name(&self) -> &str {
        unsafe {
            let name = self.ptr.as_ref().name;
            if name.is_null() {
                return "";
            }
            CStr::from_ptr(name).to_str().unwrap_or("")
        }
    }

    pub fn id(&self) -> CodecId {
        unsafe { CodecId(self.ptr.as_ref().id) }
    }
}

impl Codec {
    pub unsafe fn from_ptr(ptr: *const AVCodec) -> Option<Self> {
        let ptr = NonNull::new(ptr.cast_mut())?;
        Some(Self { ptr })
    }

    pub fn as_ptr(&self) -> *const AVCodec {
        self.ptr.as_ptr()
    }
}

impl CodecId {
    pub fn from_raw(id: AVCodecID) -> Self {
        Self(id)
    }
}
