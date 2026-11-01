// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{ffi::c_int, marker::PhantomData};

use bitflags::bitflags;
#[cfg(ffmpeg_7_1)]
use ffmsys::AV_DISPOSITION_MULTILAYER;
#[cfg(ffmpeg_5_1)]
use ffmsys::AV_DISPOSITION_NON_DIEGETIC;
use ffmsys::{
    AV_DISPOSITION_ATTACHED_PIC, AV_DISPOSITION_CAPTIONS, AV_DISPOSITION_CLEAN_EFFECTS,
    AV_DISPOSITION_COMMENT, AV_DISPOSITION_DEFAULT, AV_DISPOSITION_DEPENDENT,
    AV_DISPOSITION_DESCRIPTIONS, AV_DISPOSITION_DUB, AV_DISPOSITION_FORCED,
    AV_DISPOSITION_HEARING_IMPAIRED, AV_DISPOSITION_KARAOKE, AV_DISPOSITION_LYRICS,
    AV_DISPOSITION_METADATA, AV_DISPOSITION_ORIGINAL, AV_DISPOSITION_STILL_IMAGE,
    AV_DISPOSITION_TIMED_THUMBNAILS, AV_DISPOSITION_VISUAL_IMPAIRED, AVDiscard,
    AVSTREAM_EVENT_FLAG_METADATA_UPDATED, AVSTREAM_EVENT_FLAG_NEW_PACKETS, AVStream,
};

use crate::{
    codec::parameters::ParametersRef,
    format::context::inner::Inner as ContextInner,
    util::{
        Rational,
        dictionary::{DictionaryMut, DictionaryRef},
    },
};

mod inner;
use inner::Inner;

pub struct Stream<'a> {
    inner: Inner,
    context: &'a ContextInner,
}

pub struct StreamMut<'a> {
    inner: Inner,
    context: PhantomData<&'a mut ContextInner>,
}

pub struct StreamIter<'a> {
    context: &'a ContextInner,
    index: usize,
}

pub struct StreamIterMut<'a> {
    context: &'a mut ContextInner,
    index: usize,
}

bitflags! {
    pub struct Disposition: c_int {
        const DEFAULT = AV_DISPOSITION_DEFAULT;
        const DUB = AV_DISPOSITION_DUB;
        const ORIGINAL = AV_DISPOSITION_ORIGINAL;
        const COMMENT = AV_DISPOSITION_COMMENT;
        const LYRICS = AV_DISPOSITION_LYRICS;
        const KARAOKE = AV_DISPOSITION_KARAOKE;
        const FORCED = AV_DISPOSITION_FORCED;
        const HEARING_IMPAIRED = AV_DISPOSITION_HEARING_IMPAIRED;
        const VISUAL_IMPAIRED = AV_DISPOSITION_VISUAL_IMPAIRED;
        const CLEAN_EFFECTS = AV_DISPOSITION_CLEAN_EFFECTS;
        const ATTACHED_PIC = AV_DISPOSITION_ATTACHED_PIC;
        const TIMED_THUMBNAILS = AV_DISPOSITION_TIMED_THUMBNAILS;
        #[cfg(ffmpeg_5_1)]
        const NON_DIEGETIC = AV_DISPOSITION_NON_DIEGETIC;
        const CAPTIONS = AV_DISPOSITION_CAPTIONS;
        const DESCRIPTIONS = AV_DISPOSITION_DESCRIPTIONS;
        const METADATA = AV_DISPOSITION_METADATA;
        const DEPENDENT = AV_DISPOSITION_DEPENDENT;
        const STILL_IMAGE = AV_DISPOSITION_STILL_IMAGE;
        #[cfg(ffmpeg_7_1)]
        const MULTILAYER = AV_DISPOSITION_MULTILAYER;
    }
}

bitflags! {
    pub struct Events: c_int {
        const METADATA_UPDATED = AVSTREAM_EVENT_FLAG_METADATA_UPDATED;
        const NEW_PACKETS = AVSTREAM_EVENT_FLAG_NEW_PACKETS;
    }
}

impl<'a> Stream<'a> {
    pub fn index(&self) -> i32 {
        self.inner.index()
    }

    pub fn id(&self) -> i32 {
        self.inner.id()
    }

    pub fn parameters(&self) -> ParametersRef<'a> {
        self.inner.parameters(self.context)
    }

    pub fn time_base(&self) -> Rational {
        self.inner.time_base()
    }

    pub fn start_time(&self) -> i64 {
        self.inner.start_time()
    }

    pub fn duration(&self) -> i64 {
        self.inner.duration()
    }

    pub fn nb_frames(&self) -> i64 {
        self.inner.nb_frames()
    }

    pub fn disposition(&self) -> Disposition {
        self.inner.disposition()
    }

    pub fn discard(&self) -> AVDiscard {
        self.inner.discard()
    }

    pub fn sample_aspect_ratio(&self) -> Rational {
        self.inner.sample_aspect_ratio()
    }

    pub fn metadata(&self) -> DictionaryRef<'_> {
        self.inner.metadata(self.context)
    }

    pub fn avg_frame_rate(&self) -> Rational {
        self.inner.avg_frame_rate()
    }

    pub fn events(&self) -> Events {
        self.inner.events()
    }

    pub fn r_frame_rate(&self) -> Rational {
        self.inner.r_frame_rate()
    }

    #[cfg(ffmpeg_5_0)]
    pub fn pts_wrap_bits(&self) -> i32 {
        self.inner.pts_wrap_bits()
    }
}

impl<'a> Stream<'a> {
    pub(crate) unsafe fn from_ptr(ptr: *mut AVStream, context: &'a ContextInner) -> Option<Self> {
        Some(Self {
            inner: unsafe { Inner::from_ptr(ptr)? },
            context,
        })
    }

    pub fn as_ptr(&self) -> *const AVStream {
        self.inner.as_ptr()
    }
}

unsafe impl Send for Stream<'_> {}
unsafe impl Sync for Stream<'_> {}

impl StreamMut<'_> {
    pub fn index(&self) -> i32 {
        self.inner.index()
    }

    pub fn id(&self) -> i32 {
        self.inner.id()
    }

    pub fn parameters(&self) -> ParametersRef<'_> {
        self.inner.parameters(&self.inner)
    }

    pub fn time_base(&self) -> Rational {
        self.inner.time_base()
    }

    pub fn start_time(&self) -> i64 {
        self.inner.start_time()
    }

    pub fn duration(&self) -> i64 {
        self.inner.duration()
    }

    pub fn nb_frames(&self) -> i64 {
        self.inner.nb_frames()
    }

    pub fn disposition(&self) -> Disposition {
        self.inner.disposition()
    }

    pub fn discard(&self) -> AVDiscard {
        self.inner.discard()
    }

    pub fn sample_aspect_ratio(&self) -> Rational {
        self.inner.sample_aspect_ratio()
    }

    pub fn metadata(&self) -> DictionaryRef<'_> {
        self.inner.metadata(&self.inner)
    }

    pub fn avg_frame_rate(&self) -> Rational {
        self.inner.avg_frame_rate()
    }

    pub fn events(&self) -> Events {
        self.inner.events()
    }

    pub fn r_frame_rate(&self) -> Rational {
        self.inner.r_frame_rate()
    }

    #[cfg(ffmpeg_5_0)]
    pub fn pts_wrap_bits(&self) -> i32 {
        self.inner.pts_wrap_bits()
    }

    pub fn set_discard(&mut self, discard: AVDiscard) {
        self.inner.set_discard(discard);
    }

    pub fn metadata_mut(&mut self) -> DictionaryMut<'_> {
        self.inner.metadata_mut()
    }

    pub fn clear_events(&mut self, events: &Events) {
        self.inner.clear_events(events);
    }
}

impl<'a> StreamMut<'a> {
    pub(crate) unsafe fn from_ptr(ptr: *mut AVStream, _context: &'a mut ContextInner) -> Option<Self> {
        Some(Self {
            inner: unsafe { Inner::from_ptr(ptr)? },
            context: PhantomData,
        })
    }

    pub fn as_ptr(&self) -> *const AVStream {
        self.inner.as_ptr()
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut AVStream {
        self.inner.as_mut_ptr()
    }
}

unsafe impl Send for StreamMut<'_> {}
unsafe impl Sync for StreamMut<'_> {}

impl<'a> StreamIter<'a> {
    pub(crate) fn new(context: &'a ContextInner) -> Self {
        Self { context, index: 0 }
    }
}

impl<'a> Iterator for StreamIter<'a> {
    type Item = Stream<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let context = unsafe { &*self.context.as_ptr() };
        if self.index >= context.nb_streams as usize {
            return None;
        }
        let ptr = unsafe { *context.streams.add(self.index) };
        self.index += 1;
        unsafe { Stream::from_ptr(ptr, self.context) }
    }
}

impl<'a> StreamIterMut<'a> {
    pub(crate) fn new(context: &'a mut ContextInner) -> Self {
        Self { context, index: 0 }
    }
}

impl<'a> Iterator for StreamIterMut<'a> {
    type Item = StreamMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let context = unsafe { &*self.context.as_ptr() };
        if self.index >= context.nb_streams as usize {
            return None;
        }
        let ptr = unsafe { *context.streams.add(self.index) };
        self.index += 1;
        Some(StreamMut {
            inner: unsafe { Inner::from_ptr(ptr)? },
            context: PhantomData,
        })
    }
}
