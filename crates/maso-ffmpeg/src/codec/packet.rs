// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::ptr::NonNull;

use ffmsys::{
    AVPacket, av_packet_alloc, av_packet_clone, av_packet_free, av_packet_ref, av_packet_unref,
};
use snafu::{OptionExt, ResultExt, Snafu};

use crate::util::Code;

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
