// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::ptr::NonNull;

use ffmsys::{AVDiscard, AVStream};

use super::{Disposition, Events};
use crate::{
    codec::parameters::ParametersRef,
    util::{
        Rational,
        dictionary::{DictionaryMut, DictionaryRef},
    },
};

pub(super) struct Inner {
    ptr: NonNull<AVStream>,
}

impl Inner {
    pub(super) fn index(&self) -> i32 {
        unsafe { self.ptr.as_ref().index }
    }

    pub(super) fn id(&self) -> i32 {
        unsafe { self.ptr.as_ref().id }
    }

    pub(super) fn parameters<'a, T>(&self, context: &'a T) -> ParametersRef<'a> {
        let ptr = unsafe { self.ptr.as_ref().codecpar };
        unsafe { ParametersRef::from_ptr(ptr, context).expect("stream must have parameters") }
    }

    pub(super) fn time_base(&self) -> Rational {
        unsafe { self.ptr.as_ref().time_base.into() }
    }

    pub(super) fn start_time(&self) -> i64 {
        unsafe { self.ptr.as_ref().start_time }
    }

    pub(super) fn duration(&self) -> i64 {
        unsafe { self.ptr.as_ref().duration }
    }

    pub(super) fn nb_frames(&self) -> i64 {
        unsafe { self.ptr.as_ref().nb_frames }
    }

    pub(super) fn disposition(&self) -> Disposition {
        unsafe { Disposition::from_bits_retain(self.ptr.as_ref().disposition) }
    }

    pub(super) fn discard(&self) -> AVDiscard {
        unsafe { self.ptr.as_ref().discard }
    }

    pub(super) fn sample_aspect_ratio(&self) -> Rational {
        unsafe { self.ptr.as_ref().sample_aspect_ratio.into() }
    }

    pub(super) fn metadata<'a, T>(&self, context: &'a T) -> DictionaryRef<'a> {
        unsafe { DictionaryRef::from_ptr(self.ptr.as_ref().metadata, context) }
    }

    pub(super) fn avg_frame_rate(&self) -> Rational {
        unsafe { self.ptr.as_ref().avg_frame_rate.into() }
    }

    pub(super) fn events(&self) -> Events {
        unsafe { Events::from_bits_retain(self.ptr.as_ref().event_flags) }
    }

    pub(super) fn r_frame_rate(&self) -> Rational {
        unsafe { self.ptr.as_ref().r_frame_rate.into() }
    }

    #[cfg(ffmpeg_5_0)]
    pub(super) fn pts_wrap_bits(&self) -> i32 {
        unsafe { self.ptr.as_ref().pts_wrap_bits }
    }

    pub(super) fn set_discard(&mut self, discard: AVDiscard) {
        unsafe { self.ptr.as_mut().discard = discard }
    }

    pub(super) fn metadata_mut(&mut self) -> DictionaryMut<'_> {
        unsafe { DictionaryMut::from_ptr(&mut self.ptr.as_mut().metadata) }
    }

    pub(super) fn clear_events(&mut self, events: &Events) {
        unsafe { self.ptr.as_mut().event_flags &= !events.bits() }
    }
}

impl Inner {
    pub(super) unsafe fn from_ptr(ptr: *mut AVStream) -> Option<Self> {
        Some(Self {
            ptr: NonNull::new(ptr)?,
        })
    }

    pub(super) fn as_ptr(&self) -> *const AVStream {
        self.ptr.as_ptr()
    }

    pub(super) fn as_mut_ptr(&mut self) -> *mut AVStream {
        self.ptr.as_ptr()
    }
}
