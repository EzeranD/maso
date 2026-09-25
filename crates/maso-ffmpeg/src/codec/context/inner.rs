// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    ffi::c_uint,
    ptr::{self, NonNull},
    sync::Arc,
};

use bitflags::bitflags;
use ffmsys::{
    AVCodecContext, avcodec_alloc_context3, avcodec_flush_buffers, avcodec_open2,
    avcodec_parameters_to_context, avcodec_receive_frame, avcodec_receive_packet,
    avcodec_send_frame, avcodec_send_packet,
};
use snafu::{OptionExt, ResultExt};

use super::{GetFormatContext, GetFormatFn};
use crate::{
    codec::{
        Codec,
        context::{
            Discard,
            error::{
                AllocContextSnafu, Error, OpenCodecSnafu, ReceiveFrameError, ReceivePacketError,
                SendFrameError, SendPacketError, SetParametersSnafu,
            },
            get_format::get_format,
        },
        packet::Packet,
        parameters::Parameters,
    },
    util::{
        Code, Extensions,
        buffer::BufferRef,
        frame::Frame,
        hardware::{DeviceContext, FramesContext, Initialized},
        pixel::PixelFormat,
    },
};

pub(super) struct Inner {
    pub(super) ptr: NonNull<AVCodecContext>,
}

bitflags! {
    #[derive(Default)]
    pub struct ReceiveFrameFlags: c_uint {
        const SYNCHRONOUS = 1 << 0;
    }
}

impl Inner {
    pub(super) fn new(codec: Option<&Codec>) -> Result<Self, Error> {
        unsafe {
            let codec_ptr = codec.map_or(ptr::null(), Codec::as_ptr);
            let ptr = avcodec_alloc_context3(codec_ptr);
            let mut inner = NonNull::new(ptr).context(AllocContextSnafu)?;
            inner.as_mut().opaque = Box::into_raw(Box::new(Extensions::new())).cast();
            Ok(Self { ptr: inner })
        }
    }

    pub(super) fn from_parameters(
        parameters: &Parameters,
        codec: Option<&Codec>,
    ) -> Result<Self, Error> {
        unsafe {
            let mut ctx = Self::new(codec)?;
            let res = avcodec_parameters_to_context(ctx.as_mut_ptr(), parameters.as_ptr());
            if res < 0 {
                return Err(res.into()).context(SetParametersSnafu);
            }
            Ok(ctx)
        }
    }

    pub(super) fn device_ctx(&self) -> Option<DeviceContext> {
        unsafe {
            let ptr = self.ptr.as_ref().hw_device_ctx;
            let buffer = BufferRef::ref_from_ptr(ptr).ok()?;
            Some(DeviceContext::from_buffer(buffer))
        }
    }

    pub(super) fn open(&mut self, codec: Option<&Codec>) -> Result<(), Error> {
        unsafe {
            let ptr = self.as_mut_ptr();
            let res = avcodec_open2(
                ptr,
                codec.map_or(ptr::null(), Codec::as_ptr),
                ptr::null_mut(),
            );
            if res < 0 {
                return Err(res.into()).context(OpenCodecSnafu);
            }
            Ok(())
        }
    }

    pub(super) fn get_format<F>(&mut self, f: F)
    where
        F: Fn(&mut GetFormatContext<'_>, &[PixelFormat]) -> PixelFormat + Send + Sync + 'static,
    {
        self.extensions_mut().insert(Arc::new(f) as GetFormatFn);
        let ptr = self.ptr.as_ptr();
        unsafe { (*ptr).get_format = Some(get_format) }
    }

    pub(super) fn set_device_ctx(&mut self, device: DeviceContext) {
        unsafe {
            let hw_device_ctx = &raw mut self.ptr.as_mut().hw_device_ctx;
            ffmsys::av_buffer_unref(hw_device_ctx);
            *hw_device_ctx = device.into_ptr();
        }
    }
}

impl Inner {
    pub(super) fn send_packet(&mut self, packet: Option<&Packet>) -> Result<(), SendPacketError> {
        unsafe {
            let res = avcodec_send_packet(
                self.as_mut_ptr(),
                packet.map_or(ptr::null(), Packet::as_ptr),
            );
            if res < 0 {
                return Err(Code::from(res).into());
            }
            Ok(())
        }
    }

    pub(super) fn send_frame<Kind>(
        &mut self,
        frame: Option<&Frame<Kind>>,
    ) -> Result<(), SendFrameError> {
        unsafe {
            let res =
                avcodec_send_frame(self.as_mut_ptr(), frame.map_or(ptr::null(), Frame::as_ptr));
            if res < 0 {
                return Err(Code::from(res).into());
            }
            Ok(())
        }
    }

    pub(super) fn receive_packet(&mut self, packet: &mut Packet) -> Result<(), ReceivePacketError> {
        unsafe {
            let res = avcodec_receive_packet(self.as_mut_ptr(), packet.as_mut_ptr());
            if res < 0 {
                return Err(Code::from(res).into());
            }
            Ok(())
        }
    }

    pub(super) fn receive_frame<Kind>(
        &mut self,
        frame: &mut Frame<Kind>,
        flags: Option<ReceiveFrameFlags>,
    ) -> Result<(), ReceiveFrameError> {
        #[cfg(ffmpeg_9_0)]
        if let Some(flags) = flags {
            let mut ffmpeg_flags: c_uint = 0;
            if flags.contains(ReceiveFrameFlags::SYNCHRONOUS) {
                ffmpeg_flags |= ffmsys::AV_CODEC_RECEIVE_FRAME_FLAG_SYNCHRONOUS as c_uint;
            }
            let res = unsafe {
                use ffmsys::avcodec_receive_frame_flags;
                avcodec_receive_frame_flags(self.as_mut_ptr(), frame.as_mut_ptr(), ffmpeg_flags)
            };
            if res < 0 {
                return Err(Code::from(res).into());
            }
            return Ok(());
        }

        let res = unsafe { avcodec_receive_frame(self.as_mut_ptr(), frame.as_mut_ptr()) };
        if res < 0 {
            return Err(Code::from(res).into());
        }
        Ok(())
    }

    pub(super) fn flush(&mut self) {
        unsafe { avcodec_flush_buffers(self.as_mut_ptr()) }
    }
}

impl Inner {
    pub(super) fn set_frames_ctx(&mut self, frames: Option<FramesContext<Initialized>>) {
        unsafe {
            let hw_frames_ctx = &raw mut self.ptr.as_mut().hw_frames_ctx;
            ffmsys::av_buffer_unref(hw_frames_ctx);
            *hw_frames_ctx = frames.map_or(ptr::null_mut(), FramesContext::into_ptr);
        }
    }

    pub(super) fn codec(&self) -> Option<Codec> {
        unsafe {
            let ptr = self.ptr.as_ref().codec;
            Codec::from_ptr(ptr)
        }
    }

    pub(super) fn skip_frame(&mut self, discard: Discard) {
        unsafe {
            self.ptr.as_mut().skip_frame = discard.into();
        }
    }
}

impl Inner {
    pub(super) fn extensions(&self) -> &Extensions {
        let ptr = self.ptr.as_ptr();
        unsafe { &*((*ptr).opaque.cast::<Extensions>()) }
    }

    pub(super) fn extensions_mut(&mut self) -> &mut Extensions {
        let ptr = self.ptr.as_ptr();
        unsafe { &mut *(*ptr).opaque.cast::<Extensions>() }
    }

    pub(super) fn as_ptr(&self) -> *const AVCodecContext {
        self.ptr.as_ptr()
    }

    pub(super) fn as_mut_ptr(&mut self) -> *mut AVCodecContext {
        self.ptr.as_ptr()
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            let mut avctx = self.ptr.as_ptr();
            let extensions = if (*avctx).opaque.is_null() {
                None
            } else {
                Some(Box::from_raw((*avctx).opaque.cast::<Extensions>()))
            };

            ffmsys::avcodec_free_context(&raw mut avctx);
            drop(extensions);
        }
    }
}
