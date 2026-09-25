// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    ffi::{CStr, CString},
    fmt, iter,
    ptr::{self},
};

use ffmsys::{
    AVBufferRef, AVHWDeviceContext, AVHWDeviceType, av_hwdevice_ctx_create,
    av_hwdevice_find_type_by_name, av_hwdevice_get_type_name, av_hwdevice_iterate_types,
};
use snafu::{ResultExt, Snafu};

use crate::util::{
    Code,
    buffer::{self, BufferRef, Error::NullBuffer},
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("failed to create device context"))]
    CreateDevice { source: Code },
    #[snafu(transparent)]
    Buffer { source: buffer::Error },
}

#[derive(Clone)]
pub struct DeviceContext {
    inner: BufferRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceType(AVHWDeviceType);

impl DeviceContext {
    pub fn new(device_type: DeviceType) -> Result<Self, Error> {
        let mut ptr = ptr::null_mut();
        let res = unsafe {
            av_hwdevice_ctx_create(
                &raw mut ptr,
                device_type.as_raw(),
                ptr::null(),
                ptr::null_mut(),
                0,
            )
        };
        if res < 0 {
            return Err(res.into()).context(CreateDeviceSnafu);
        }

        let inner = unsafe { BufferRef::from_ptr(ptr).ok_or(NullBuffer)? };
        if inner.data_ptr::<AVHWDeviceContext>().is_null() {
            return Err(NullBuffer.into());
        }

        Ok(Self { inner })
    }

    pub fn device_type(&self) -> DeviceType {
        let device = unsafe { &*self.inner.data_ptr::<AVHWDeviceContext>() };
        DeviceType::from_raw(device.type_)
    }
}

impl DeviceContext {
    pub unsafe fn from_buffer(inner: BufferRef) -> Self {
        Self { inner }
    }

    pub unsafe fn from_ptr(ptr: *mut AVBufferRef) -> Option<Self> {
        let inner = unsafe { BufferRef::from_ptr(ptr)? };
        Some(Self { inner })
    }

    pub fn as_ptr(&self) -> *const AVBufferRef {
        self.inner.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut AVBufferRef {
        self.inner.as_mut_ptr()
    }

    pub fn into_ptr(self) -> *mut AVBufferRef {
        self.inner.into_ptr()
    }
}

impl DeviceType {
    pub fn iter() -> impl Iterator<Item = Self> {
        let mut prev = AVHWDeviceType::AV_HWDEVICE_TYPE_NONE;
        iter::from_fn(move || {
            let device = unsafe { av_hwdevice_iterate_types(prev) };
            if device == AVHWDeviceType::AV_HWDEVICE_TYPE_NONE {
                return None;
            }
            prev = device;
            Some(Self::from_raw(device))
        })
    }

    pub fn find_by_name(name: &str) -> Option<Self> {
        let name = CString::new(name).ok()?;
        let device = unsafe { av_hwdevice_find_type_by_name(name.as_ptr()) };
        if device == AVHWDeviceType::AV_HWDEVICE_TYPE_NONE {
            None
        } else {
            Some(Self::from_raw(device))
        }
    }

    pub fn name(&self) -> Option<&str> {
        unsafe {
            let ptr = av_hwdevice_get_type_name(self.0);
            if ptr.is_null() {
                return None;
            }
            CStr::from_ptr(ptr).to_str().ok()
        }
    }
}

impl DeviceType {
    pub fn from_raw(device_type: AVHWDeviceType) -> Self {
        Self(device_type)
    }

    pub fn as_raw(&self) -> AVHWDeviceType {
        self.0
    }
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name().unwrap_or("unknown"))
    }
}
