// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#[allow(clippy::module_inception)]
mod codec;
pub mod context;
pub mod hw_config;
pub mod packet;
pub mod parameters;

pub use codec::{Codec, CodecId, CodecMode, CodecType};

pub struct Decoder;
pub struct Encoder;

pub trait Mode: private::Sealed {}

impl Mode for Decoder {}
impl Mode for Encoder {}

mod private {
    use std::ptr::NonNull;

    use ffmsys::{AVCodec, av_codec_is_decoder, av_codec_is_encoder};

    pub trait Sealed {
        unsafe fn matches(codec: NonNull<AVCodec>) -> bool;
    }

    impl Sealed for super::Decoder {
        unsafe fn matches(codec: NonNull<AVCodec>) -> bool {
            unsafe { av_codec_is_decoder(codec.as_ptr()) != 0 }
        }
    }
    impl Sealed for super::Encoder {
        unsafe fn matches(codec: NonNull<AVCodec>) -> bool {
            unsafe { av_codec_is_encoder(codec.as_ptr()) != 0 }
        }
    }
}
