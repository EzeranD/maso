// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{marker::PhantomData, ptr};

use ffmsys::{
    AVBufferRef, AVHWFramesContext, av_hwframe_ctx_init, avcodec_get_hw_frames_parameters,
};
use snafu::{OptionExt, ResultExt, Snafu, ensure};

use crate::{
    codec::{
        context::GetFormatContext,
        hw_config::{HwConfig, Methods},
    },
    util::{
        Code,
        buffer::{self, BufferRef, Error::NullBuffer},
        hardware::{
            DeviceContext,
            frames::error::{
                DeviceMismatchSnafu, InitSnafu, MissingDeviceSnafu, NoConfigSnafu, NotOfferedSnafu,
            },
        },
        pixel::PixelFormat,
    },
};

#[derive(Debug, Snafu)]
#[snafu(module)]
pub enum Error {
    #[snafu(display("format is not offered by the decoder"))]
    NotOffered,
    #[snafu(display("no hardware device is available"))]
    MissingDevice,
    #[snafu(display("no hardware configuration is available for the format"))]
    NoConfig,
    #[snafu(display("device type does not match the format's hardware configuration"))]
    DeviceMismatch,
    #[snafu(transparent)]
    Buffer { source: buffer::Error },
    #[snafu(display("failed to get hardware frames parameters"))]
    GetParameters { source: Code },
    #[snafu(display("failed to initialize hardware frames context"))]
    Init { source: Code },
}

pub struct FramesContext<State = Uninitialized> {
    inner: BufferRef,
    _state: PhantomData<State>,
}

pub struct Uninitialized;
pub struct Initialized;

impl FramesContext<Uninitialized> {
    pub fn new(
        context: &mut GetFormatContext<'_>,
        format: PixelFormat,
        device: Option<DeviceContext>,
    ) -> Result<Self, Error> {
        ensure!(context.formats().contains(&format), NotOfferedSnafu);

        let mut dev_ctx = device
            .or_else(|| context.device_ctx())
            .context(MissingDeviceSnafu)?;

        let codec = context.codec().context(NoConfigSnafu)?;
        let config = HwConfig::from_codec(codec)
            .find(|config| config.format() == format)
            .filter(|config| config.methods().contains(Methods::HW_FRAMES_CTX))
            .context(NoConfigSnafu)?;

        ensure!(
            config.device_type() == dev_ctx.device_type(),
            DeviceMismatchSnafu
        );

        let mut ptr = ptr::null_mut();
        let res = unsafe {
            avcodec_get_hw_frames_parameters(
                context.as_mut_ptr(),
                dev_ctx.as_mut_ptr(),
                format.as_raw(),
                &raw mut ptr,
            )
        };
        if res < 0 {
            return Err(Error::GetParameters {
                source: Code::from(res),
            });
        }

        let inner = unsafe { BufferRef::from_ptr(ptr).ok_or(NullBuffer)? };
        if inner.data_ptr::<AVHWFramesContext>().is_null() {
            return Err(NullBuffer.into());
        }

        Ok(Self {
            inner,
            _state: PhantomData,
        })
    }

    pub fn init(mut self) -> Result<FramesContext<Initialized>, Error> {
        let res = unsafe { av_hwframe_ctx_init(self.as_mut_ptr()) };
        if res < 0 {
            return Err(res.into()).context(InitSnafu);
        }
        Ok(FramesContext {
            inner: self.inner,
            _state: PhantomData,
        })
    }
}

impl<State> FramesContext<State> {
    pub fn format(&self) -> PixelFormat {
        let frames = unsafe { &*self.inner.data_ptr::<AVHWFramesContext>() };
        PixelFormat::from_raw(frames.format)
    }
}

impl<State> FramesContext<State> {
    pub(crate) fn as_mut_ptr(&mut self) -> *mut AVBufferRef {
        self.inner.as_mut_ptr()
    }

    pub fn into_ptr(self) -> *mut AVBufferRef {
        self.inner.into_ptr()
    }
}
