// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    ffi::{CStr, c_int},
    fmt,
    marker::PhantomData,
    mem,
    ptr::NonNull,
};

use bitflags::bitflags;
#[cfg(ffmpeg_6_1)]
use ffmsys::av_frame_replace;
use ffmsys::{
    AV_FRAME_CROP_UNALIGNED, AV_NOPTS_VALUE, AVFrame, AVFrameSideData, AVFrameSideDataType,
    av_frame_alloc, av_frame_apply_cropping, av_frame_clone, av_frame_copy, av_frame_free,
    av_frame_get_plane_buffer, av_frame_get_side_data, av_frame_is_writable,
    av_frame_make_writable, av_frame_side_data_name,
};
#[cfg(not(ffmpeg_6_1))]
use ffmsys::{av_frame_ref, av_frame_unref};
use snafu::{OptionExt, ResultExt, Snafu};

use crate::util::{Code, buffer::BufferRef, pixel::PixelFormat};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("failed to allocate frame"))]
    Allocate,
    #[snafu(display("failed to clone frame"))]
    Clone,
    #[snafu(display("failed to make frame writable"))]
    MakeWritable { source: Code },
    #[snafu(display("failed to clone into frame"))]
    CloneFrom { source: Code },
    #[snafu(display("failed to copy frame"))]
    Copy { source: Code },
    #[snafu(display("crop was invalid"))]
    InvalidCrop,
    #[snafu(display("failed to crop frame"))]
    ApplyCrop { source: Code },
}

pub struct Frame<Kind> {
    ptr: NonNull<AVFrame>,
    _kind: PhantomData<Kind>,
}

pub struct Video;
pub struct Audio;

pub struct SideData<'a> {
    ptr: NonNull<AVFrameSideData>,
    _frame: PhantomData<&'a ()>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SideDataType(AVFrameSideDataType);

bitflags! {
    #[derive(Default)]
    pub struct CropFlags: c_int {
        const UNALIGNED = AV_FRAME_CROP_UNALIGNED as c_int;
    }
}

impl Frame<Video> {
    pub fn width(&self) -> u32 {
        unsafe { self.ptr.as_ref().width.max(0) as u32 }
    }

    pub fn height(&self) -> u32 {
        unsafe { self.ptr.as_ref().height.max(0) as u32 }
    }

    pub fn format(&self) -> PixelFormat {
        unsafe {
            let format = self.ptr.as_ref().format;
            if format < 0 {
                PixelFormat::NONE
            } else {
                PixelFormat::from_raw(mem::transmute(format))
            }
        }
    }

    pub fn apply_crop(&mut self, flags: Option<CropFlags>) -> Result<(), Error> {
        unsafe {
            let res = av_frame_apply_cropping(self.as_mut_ptr(), flags.unwrap_or_default().bits());
            if res == Code::RANGE.as_raw() {
                return InvalidCropSnafu.fail();
            }
            if res < 0 {
                return Err(res.into()).context(ApplyCropSnafu);
            }
            Ok(())
        }
    }
}

impl<Kind> Frame<Kind> {
    pub fn new() -> Result<Self, Error> {
        unsafe {
            let ptr = av_frame_alloc();
            Self::from_ptr(ptr).context(AllocateSnafu)
        }
    }

    pub fn try_clone(&self) -> Result<Self, Error> {
        unsafe {
            let ptr = av_frame_clone(self.ptr.as_ptr());
            Self::from_ptr(ptr).context(CloneSnafu)
        }
    }

    pub fn plane_buffer(&self, plane: c_int) -> Option<BufferRef> {
        unsafe {
            let ptr = av_frame_get_plane_buffer(self.as_ptr(), plane);
            BufferRef::ref_from_ptr(ptr).ok()
        }
    }

    pub fn side_data(&self, data_type: SideDataType) -> Option<SideData<'_>> {
        unsafe {
            let ptr = av_frame_get_side_data(self.as_ptr(), data_type.as_raw());
            SideData::from_ptr(ptr)
        }
    }

    pub fn pts(&self) -> Option<i64> {
        unsafe {
            match self.ptr.as_ref().pts {
                AV_NOPTS_VALUE => None,
                pts => Some(pts),
            }
        }
    }

    pub fn clone_from(&mut self, source: &Frame<Kind>) -> Result<(), Error> {
        #[cfg(ffmpeg_6_1)]
        unsafe {
            let res = av_frame_replace(self.as_mut_ptr(), source.as_ptr());
            if res < 0 {
                return Err(res.into()).context(CloneFromSnafu);
            }
            Ok(())
        }

        #[cfg(not(ffmpeg_6_1))]
        unsafe {
            av_frame_unref(self.as_mut_ptr());
            let res = av_frame_ref(self.as_mut_ptr(), source.as_ptr());
            if res < 0 {
                return Err(res.into()).context(CloneFromSnafu);
            }
            Ok(())
        }
    }

    pub fn is_writable(&mut self) -> bool {
        unsafe { av_frame_is_writable(self.as_mut_ptr()) > 0 }
    }

    pub fn make_writable(&mut self) -> Result<(), Error> {
        unsafe {
            let res = av_frame_make_writable(self.as_mut_ptr());
            if res < 0 {
                return Err(res.into()).context(MakeWritableSnafu);
            }
            Ok(())
        }
    }

    pub fn copy_from(&mut self, source: &Frame<Kind>) -> Result<(), Error> {
        self.make_writable()?;
        unsafe {
            let res = av_frame_copy(self.as_mut_ptr(), source.as_ptr());
            if res < 0 {
                return Err(res.into()).context(CopySnafu);
            }
            Ok(())
        }
    }
}

impl<Kind> Frame<Kind> {
    pub unsafe fn from_ptr(ptr: *mut AVFrame) -> Option<Self> {
        let ptr = NonNull::new(ptr)?;
        Some(Self {
            ptr,
            _kind: PhantomData,
        })
    }

    pub fn as_ptr(&self) -> *const AVFrame {
        self.ptr.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut AVFrame {
        self.ptr.as_ptr()
    }
}

unsafe impl<Kind> Send for Frame<Kind> {}
unsafe impl<Kind> Sync for Frame<Kind> {}

impl<Kind> Clone for Frame<Kind> {
    fn clone(&self) -> Self {
        self.try_clone().expect("failed to clone frame")
    }
}

impl<Kind> Drop for Frame<Kind> {
    fn drop(&mut self) {
        let mut ptr = self.as_mut_ptr();
        unsafe {
            av_frame_free(&raw mut ptr);
        }
    }
}

impl SideData<'_> {
    pub unsafe fn from_ptr(ptr: *mut AVFrameSideData) -> Option<Self> {
        let ptr = NonNull::new(ptr)?;
        Some(Self {
            ptr,
            _frame: PhantomData,
        })
    }
}

impl SideDataType {
    pub fn name(&self) -> &str {
        unsafe {
            let ptr = av_frame_side_data_name(self.0);
            if ptr.is_null() {
                return "";
            }
            CStr::from_ptr(ptr).to_str().unwrap_or("")
        }
    }
}

impl SideDataType {
    pub fn from_raw(data_type: AVFrameSideDataType) -> Self {
        Self(data_type)
    }

    pub fn as_raw(self) -> AVFrameSideDataType {
        self.0
    }
}

impl fmt::Display for SideDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}
