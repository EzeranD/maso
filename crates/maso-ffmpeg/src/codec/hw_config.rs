// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{ffi::c_int, ptr::NonNull};

use bitflags::bitflags;
use ffmsys::{
    AV_CODEC_HW_CONFIG_METHOD_AD_HOC, AV_CODEC_HW_CONFIG_METHOD_HW_DEVICE_CTX,
    AV_CODEC_HW_CONFIG_METHOD_HW_FRAMES_CTX, AV_CODEC_HW_CONFIG_METHOD_INTERNAL, AVCodecHWConfig,
    avcodec_get_hw_config,
};

use crate::{
    codec::Codec,
    util::{hardware::device::DeviceType, pixel::PixelFormat},
};

#[derive(Debug, Clone, Copy)]
pub struct HwConfig {
    ptr: NonNull<AVCodecHWConfig>,
}

bitflags! {
    pub struct Methods: c_int {
        const HW_DEVICE_CTX = AV_CODEC_HW_CONFIG_METHOD_HW_DEVICE_CTX as c_int;
        const HW_FRAMES_CTX = AV_CODEC_HW_CONFIG_METHOD_HW_FRAMES_CTX as c_int;
        const INTERNAL = AV_CODEC_HW_CONFIG_METHOD_INTERNAL as c_int;
        const AD_HOC = AV_CODEC_HW_CONFIG_METHOD_AD_HOC as c_int;
    }
}

impl HwConfig {
    pub fn from_codec(codec: Codec) -> impl Iterator<Item = Self> {
        let mut index = 0;
        std::iter::from_fn(move || unsafe {
            let ptr = avcodec_get_hw_config(codec.as_ptr(), index);
            if ptr.is_null() {
                return None;
            }
            index += 1;
            Self::from_ptr(ptr)
        })
    }

    pub fn format(self) -> PixelFormat {
        let format = unsafe { self.ptr.as_ref().pix_fmt };
        PixelFormat::from_raw(format)
    }

    pub fn methods(self) -> Methods {
        let methods = unsafe { self.ptr.as_ref().methods };
        Methods::from_bits_retain(methods)
    }

    pub fn device_type(self) -> DeviceType {
        let device_type = unsafe { self.ptr.as_ref().device_type };
        DeviceType::from_raw(device_type)
    }
}

impl HwConfig {
    pub unsafe fn from_ptr(ptr: *const AVCodecHWConfig) -> Option<Self> {
        let ptr = NonNull::new(ptr.cast_mut())?;
        Some(Self { ptr })
    }

    pub fn as_ptr(&self) -> *const AVCodecHWConfig {
        self.ptr.as_ptr()
    }
}
