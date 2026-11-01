// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{ffi::c_int, ptr::NonNull};

use bitflags::bitflags;
use ffmsys::{
    AV_PKT_FLAG_CORRUPT, AV_PKT_FLAG_DISCARD, AV_PKT_FLAG_DISPOSABLE, AV_PKT_FLAG_KEY,
    AV_PKT_FLAG_TRUSTED, AVPacket, av_packet_alloc, av_packet_clone, av_packet_free, av_packet_ref,
    av_packet_rescale_ts, av_packet_unref,
};
use snafu::{OptionExt, ResultExt, Snafu};

use crate::util::{Code, Rational};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("failed to allocate packet"))]
    Allocate,
    #[snafu(display("failed to clone packet"))]
    Clone,
    #[snafu(display("failed to clone into packet"))]
    CloneFrom { source: Code },
}

pub struct Packet {
    ptr: NonNull<AVPacket>,
}

bitflags! {
    pub struct Flags: c_int {
        const KEY = AV_PKT_FLAG_KEY;
        const CORRUPT = AV_PKT_FLAG_CORRUPT;
        const DISCARD = AV_PKT_FLAG_DISCARD;
        const TRUSTED = AV_PKT_FLAG_TRUSTED;
        const DISPOSABLE = AV_PKT_FLAG_DISPOSABLE;
    }
}

impl Packet {
    pub fn new() -> Result<Self, Error> {
        unsafe {
            let ptr = av_packet_alloc();
            Self::from_ptr(ptr).context(AllocateSnafu)
        }
    }

    pub fn try_clone(&self) -> Result<Self, Error> {
        unsafe {
            let ptr = av_packet_clone(self.ptr.as_ptr());
            Self::from_ptr(ptr).context(CloneSnafu)
        }
    }

    pub fn pts(&self) -> Option<i64> {
        let value = unsafe { self.ptr.as_ref().pts };
        (value != ffmsys::AV_NOPTS_VALUE).then_some(value)
    }

    pub fn dts(&self) -> Option<i64> {
        let value = unsafe { self.ptr.as_ref().dts };
        (value != ffmsys::AV_NOPTS_VALUE).then_some(value)
    }

    pub fn size(&self) -> i32 {
        unsafe { self.ptr.as_ref().size }
    }

    pub fn stream_index(&self) -> i32 {
        unsafe { self.ptr.as_ref().stream_index }
    }

    pub fn flags(&self) -> Flags {
        unsafe { Flags::from_bits_retain(self.ptr.as_ref().flags) }
    }

    pub fn duration(&self) -> Option<i64> {
        unsafe {
            match self.ptr.as_ref().duration {
                0 => None,
                duration => Some(duration),
            }
        }
    }

    pub fn pos(&self) -> Option<i64> {
        unsafe {
            match self.ptr.as_ref().pos {
                -1 => None,
                pos => Some(pos),
            }
        }
    }

    #[cfg(ffmpeg_5_0)]
    pub fn time_base(&self) -> Rational {
        unsafe { self.ptr.as_ref().time_base.into() }
    }

    pub fn rescale_ts(&mut self, source: Rational, destination: Rational) {
        unsafe {
            av_packet_rescale_ts(self.as_mut_ptr(), source.into(), destination.into());
        }
    }

    pub fn unref(&mut self) {
        unsafe { av_packet_unref(self.as_mut_ptr()) }
    }

    pub fn clone_from(&mut self, source: &Packet) -> Result<(), Error> {
        unsafe {
            av_packet_unref(self.as_mut_ptr());
            let res = av_packet_ref(self.as_mut_ptr(), source.as_ptr());
            if res < 0 {
                return Err(res.into()).context(CloneFromSnafu);
            }
            Ok(())
        }
    }
}

impl Packet {
    pub unsafe fn from_ptr(ptr: *mut AVPacket) -> Option<Self> {
        let ptr = NonNull::new(ptr)?;
        Some(Self { ptr })
    }

    pub fn as_ptr(&self) -> *const AVPacket {
        self.ptr.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut AVPacket {
        self.ptr.as_ptr()
    }
}

unsafe impl Send for Packet {}
unsafe impl Sync for Packet {}

impl Clone for Packet {
    fn clone(&self) -> Self {
        self.try_clone().expect("failed to clone packet")
    }
}

impl Drop for Packet {
    fn drop(&mut self) {
        let mut ptr = self.as_mut_ptr();
        unsafe {
            av_packet_free(&raw mut ptr);
        }
    }
}
