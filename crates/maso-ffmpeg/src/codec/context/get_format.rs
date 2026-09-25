// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    mem::ManuallyDrop,
    panic::{AssertUnwindSafe, catch_unwind},
    ptr::NonNull,
    slice::from_raw_parts,
    sync::Arc,
};

use ffmsys::{AVCodecContext, AVPixelFormat};

use super::inner::Inner;
use crate::{
    codec::Codec,
    util::{
        Extensions,
        hardware::{DeviceContext, FramesContext, Initialized},
        pixel::PixelFormat,
    },
};

pub struct GetFormatContext<'a> {
    inner: &'a mut Inner,
    formats: &'a [PixelFormat],
    frames_format: Option<PixelFormat>,
}

pub type GetFormatFn =
    Arc<dyn Fn(&mut GetFormatContext<'_>, &[PixelFormat]) -> PixelFormat + Send + Sync>;

impl GetFormatContext<'_> {
    pub fn formats(&self) -> &[PixelFormat] {
        self.formats
    }

    pub fn codec(&self) -> Option<Codec> {
        self.inner.codec()
    }

    pub fn device_ctx(&self) -> Option<DeviceContext> {
        self.inner.device_ctx()
    }

    pub fn set_frames_ctx(&mut self, frames: Option<FramesContext<Initialized>>) {
        self.frames_format = frames.as_ref().map(FramesContext::format);
        self.inner.set_frames_ctx(frames);
    }
}

impl GetFormatContext<'_> {
    pub fn extensions(&self) -> &Extensions {
        self.inner.extensions()
    }

    pub fn as_ptr(&self) -> *const AVCodecContext {
        self.inner.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut AVCodecContext {
        self.inner.as_mut_ptr()
    }
}

pub(super) unsafe extern "C" fn get_format(
    avctx: *mut AVCodecContext,
    mut fmt: *const AVPixelFormat,
) -> AVPixelFormat {
    let formats: &[PixelFormat] = unsafe {
        let data = fmt;
        let mut len = 0;
        while (*fmt) != AVPixelFormat::AV_PIX_FMT_NONE {
            len += 1;
            fmt = fmt.add(1);
        }
        from_raw_parts(data.cast::<PixelFormat>(), len)
    };

    let Some(cb) = ({
        let extensions = unsafe { &*(*avctx).opaque.cast::<Extensions>() };
        extensions.get::<GetFormatFn>().cloned()
    }) else {
        return AVPixelFormat::AV_PIX_FMT_NONE;
    };

    let mut inner = ManuallyDrop::new(Inner {
        ptr: unsafe { NonNull::new_unchecked(avctx) },
    });
    let mut ctx = GetFormatContext {
        inner: &mut inner,
        formats,
        frames_format: None,
    };

    let Ok(format) = catch_unwind(AssertUnwindSafe(|| (cb)(&mut ctx, formats))) else {
        return AVPixelFormat::AV_PIX_FMT_NONE;
    };
    let format = ctx.frames_format.unwrap_or(format);

    if !formats.contains(&format) {
        return AVPixelFormat::AV_PIX_FMT_NONE;
    }

    format.as_raw()
}
