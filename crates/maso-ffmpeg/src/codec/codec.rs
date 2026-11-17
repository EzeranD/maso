// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    ffi::{CStr, CString},
    iter,
    marker::PhantomData,
    ptr::{self, NonNull},
};

use ffmsys::{
    AVCodec, AVCodecID, av_codec_is_decoder, av_codec_is_encoder, av_codec_iterate,
    avcodec_find_decoder, avcodec_find_decoder_by_name, avcodec_find_encoder,
    avcodec_find_encoder_by_name, avcodec_get_type,
};

use super::{Decoder, Encoder, Mode};
use crate::util::{
    MediaType,
    media::{Attachment, Audio, Data, Kind, Subtitle, Unknown, Video},
};

#[derive(Debug)]
pub struct Codec<M: Mode = Decoder, K: Kind = Unknown> {
    ptr: NonNull<AVCodec>,
    _kind: PhantomData<(M, K)>,
}

pub enum CodecMode {
    Decoder(CodecType<Decoder>),
    Encoder(CodecType<Encoder>),
}

pub enum CodecType<M: Mode> {
    Unknown(Codec<M, Unknown>),
    Video(Codec<M, Video>),
    Audio(Codec<M, Audio>),
    Data(Codec<M, Data>),
    Subtitle(Codec<M, Subtitle>),
    Attachment(Codec<M, Attachment>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodecId(AVCodecID);

impl Codec {
    pub fn iter() -> impl Iterator<Item = CodecMode> {
        let mut opaque = ptr::null_mut();
        iter::from_fn(move || unsafe {
            let ptr = av_codec_iterate(&raw mut opaque);
            CodecMode::from_ptr(ptr)
        })
    }
}

impl<K: Kind> Codec<Decoder, K> {
    pub fn find_decoder(id: CodecId) -> Option<Self> {
        unsafe {
            let ptr = avcodec_find_decoder(id.0);
            Self::from_ptr(ptr)
        }
    }

    pub fn find_decoder_by_name(name: &str) -> Option<Self> {
        let name = CString::new(name).ok()?;
        unsafe {
            let ptr = avcodec_find_decoder_by_name(name.as_ptr());
            Self::from_ptr(ptr)
        }
    }
}

impl<K: Kind> Codec<Encoder, K> {
    pub fn find_encoder(id: CodecId) -> Option<Self> {
        unsafe {
            let ptr = avcodec_find_encoder(id.0);
            Self::from_ptr(ptr)
        }
    }

    pub fn find_encoder_by_name(name: &str) -> Option<Self> {
        let name = CString::new(name).ok()?;
        unsafe {
            let ptr = avcodec_find_encoder_by_name(name.as_ptr());
            Self::from_ptr(ptr)
        }
    }
}

impl<M: Mode, K: Kind> Codec<M, K> {
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

impl<M: Mode, K: Kind> Codec<M, K> {
    pub unsafe fn from_ptr(ptr: *const AVCodec) -> Option<Self> {
        let ptr = NonNull::new(ptr.cast_mut())?;
        if unsafe { ptr.as_ref().type_ } != K::TYPE.into() || !unsafe { M::matches(ptr) } {
            return None;
        }
        Some(Self {
            ptr,
            _kind: PhantomData,
        })
    }

    pub fn as_ptr(&self) -> *const AVCodec {
        self.ptr.as_ptr()
    }
}

impl<M: Mode, K: Kind> Copy for Codec<M, K> {}
impl<M: Mode, K: Kind> Clone for Codec<M, K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl CodecMode {
    pub fn get<M: Mode, K: Kind>(self) -> Option<Codec<M, K>> {
        let ptr = match self {
            Self::Decoder(codec) => codec.as_ptr(),
            Self::Encoder(codec) => codec.as_ptr(),
        };
        unsafe { Codec::from_ptr(ptr) }
    }

    unsafe fn from_ptr(ptr: *const AVCodec) -> Option<Self> {
        let ptr = NonNull::new(ptr.cast_mut())?;
        unsafe {
            let kind = ptr.as_ref().type_.into();
            if av_codec_is_decoder(ptr.as_ptr()) != 0 {
                Some(Self::Decoder(CodecType::from_ptr(ptr, &kind)))
            } else if av_codec_is_encoder(ptr.as_ptr()) != 0 {
                Some(Self::Encoder(CodecType::from_ptr(ptr, &kind)))
            } else {
                None
            }
        }
    }
}

impl<M: Mode> CodecType<M> {
    fn from_ptr(ptr: NonNull<AVCodec>, media_type: &MediaType) -> Self {
        match media_type {
            MediaType::Unknown => Self::Unknown(Codec {
                ptr,
                _kind: PhantomData,
            }),
            MediaType::Video => Self::Video(Codec {
                ptr,
                _kind: PhantomData,
            }),
            MediaType::Audio => Self::Audio(Codec {
                ptr,
                _kind: PhantomData,
            }),
            MediaType::Data => Self::Data(Codec {
                ptr,
                _kind: PhantomData,
            }),
            MediaType::Subtitle => Self::Subtitle(Codec {
                ptr,
                _kind: PhantomData,
            }),
            MediaType::Attachment => Self::Attachment(Codec {
                ptr,
                _kind: PhantomData,
            }),
        }
    }

    fn as_ptr(&self) -> *const AVCodec {
        match self {
            Self::Unknown(codec) => codec.as_ptr(),
            Self::Video(codec) => codec.as_ptr(),
            Self::Audio(codec) => codec.as_ptr(),
            Self::Data(codec) => codec.as_ptr(),
            Self::Subtitle(codec) => codec.as_ptr(),
            Self::Attachment(codec) => codec.as_ptr(),
        }
    }
}

impl CodecId {
    pub fn media_type(&self) -> MediaType {
        unsafe {
            let media_type = avcodec_get_type(self.0);
            media_type.into()
        }
    }

    pub fn from_raw(id: AVCodecID) -> Self {
        Self(id)
    }
}
