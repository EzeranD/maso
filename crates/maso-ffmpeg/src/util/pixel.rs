// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::ffi::CStr;

use ffmsys::{AVPixelFormat, av_get_pix_fmt_name};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelFormat(AVPixelFormat);

impl PixelFormat {
    pub const NONE: Self = Self(AVPixelFormat::AV_PIX_FMT_NONE);

    pub fn name(&self) -> Option<&str> {
        unsafe {
            let ptr = av_get_pix_fmt_name(self.0);
            if ptr.is_null() {
                return None;
            }
            CStr::from_ptr(ptr).to_str().ok()
        }
    }
}

impl PixelFormat {
    pub fn from_raw(pixel_format: AVPixelFormat) -> Self {
        Self(pixel_format)
    }

    pub fn as_raw(&self) -> AVPixelFormat {
        self.0
    }
}
