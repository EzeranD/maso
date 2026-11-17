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
        MediaType, Rational,
        dictionary::{DictionaryMut, DictionaryRef},
        media::{Attachment, Audio, Data, Kind as MediaKind, Subtitle, Unknown, Video},
    },
};

mod inner;
use inner::Inner;

pub struct Stream<'a, K: MediaKind = Unknown> {
    inner: Inner,
    context: &'a ContextInner,
    _kind: PhantomData<K>,
}

pub struct StreamMut<'a, K: MediaKind = Unknown> {
    inner: Inner,
    context: PhantomData<&'a mut ContextInner>,
    _kind: PhantomData<K>,
}

pub enum StreamType<'a> {
    Unknown(Stream<'a, Unknown>),
    Video(Stream<'a, Video>),
    Audio(Stream<'a, Audio>),
    Data(Stream<'a, Data>),
    Subtitle(Stream<'a, Subtitle>),
    Attachment(Stream<'a, Attachment>),
}

pub enum StreamTypeMut<'a> {
    Unknown(StreamMut<'a, Unknown>),
    Video(StreamMut<'a, Video>),
    Audio(StreamMut<'a, Audio>),
    Data(StreamMut<'a, Data>),
    Subtitle(StreamMut<'a, Subtitle>),
    Attachment(StreamMut<'a, Attachment>),
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

pub trait Kind: MediaKind + Sized {
    fn get(stream: StreamType<'_>) -> Option<Stream<'_, Self>>;
    fn get_mut(stream: StreamTypeMut<'_>) -> Option<StreamMut<'_, Self>>;
}

impl<'a, K: MediaKind> Stream<'a, K> {
    fn with_kind<N: MediaKind>(self) -> Stream<'a, N> {
        Stream {
            inner: self.inner,
            context: self.context,
            _kind: PhantomData,
        }
    }

    pub fn index(&self) -> i32 {
        self.inner.index()
    }

    pub fn id(&self) -> i32 {
        self.inner.id()
    }

    pub fn parameters(&self) -> ParametersRef<'a, K> {
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

impl<'a, K: MediaKind> Stream<'a, K> {
    pub(crate) unsafe fn from_ptr(ptr: *mut AVStream, context: &'a ContextInner) -> Option<Self> {
        Some(Self {
            inner: unsafe { Inner::from_ptr(ptr)? },
            context,
            _kind: PhantomData,
        })
    }

    pub fn as_ptr(&self) -> *const AVStream {
        self.inner.as_ptr()
    }
}

unsafe impl<K: MediaKind> Send for Stream<'_, K> {}
unsafe impl<K: MediaKind> Sync for Stream<'_, K> {}

impl<'a, K: MediaKind> StreamMut<'a, K> {
    fn with_kind<N: MediaKind>(self) -> StreamMut<'a, N> {
        StreamMut {
            inner: self.inner,
            context: self.context,
            _kind: PhantomData,
        }
    }

    pub fn index(&self) -> i32 {
        self.inner.index()
    }

    pub fn id(&self) -> i32 {
        self.inner.id()
    }

    pub fn parameters(&self) -> ParametersRef<'_, K> {
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

impl<'a, K: MediaKind> StreamMut<'a, K> {
    pub(crate) unsafe fn from_ptr(
        ptr: *mut AVStream,
        _context: &'a mut ContextInner,
    ) -> Option<Self> {
        Some(Self {
            inner: unsafe { Inner::from_ptr(ptr)? },
            context: PhantomData,
            _kind: PhantomData,
        })
    }

    pub fn as_ptr(&self) -> *const AVStream {
        self.inner.as_ptr()
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut AVStream {
        self.inner.as_mut_ptr()
    }
}

unsafe impl<K: MediaKind> Send for StreamMut<'_, K> {}
unsafe impl<K: MediaKind> Sync for StreamMut<'_, K> {}

impl<'a> StreamType<'a> {
    pub fn get<K: Kind>(self) -> Option<Stream<'a, K>> {
        K::get(self)
    }
}

impl<'a> From<Stream<'a>> for StreamType<'a> {
    fn from(stream: Stream<'a>) -> Self {
        match stream.parameters().media_type() {
            MediaType::Unknown => Self::Unknown(stream.with_kind()),
            MediaType::Video => Self::Video(stream.with_kind()),
            MediaType::Audio => Self::Audio(stream.with_kind()),
            MediaType::Data => Self::Data(stream.with_kind()),
            MediaType::Subtitle => Self::Subtitle(stream.with_kind()),
            MediaType::Attachment => Self::Attachment(stream.with_kind()),
        }
    }
}

impl<'a> StreamTypeMut<'a> {
    pub fn get<K: Kind>(self) -> Option<StreamMut<'a, K>> {
        K::get_mut(self)
    }
}

impl<'a> From<StreamMut<'a>> for StreamTypeMut<'a> {
    fn from(stream: StreamMut<'a>) -> Self {
        match stream.parameters().media_type() {
            MediaType::Unknown => Self::Unknown(stream.with_kind()),
            MediaType::Video => Self::Video(stream.with_kind()),
            MediaType::Audio => Self::Audio(stream.with_kind()),
            MediaType::Data => Self::Data(stream.with_kind()),
            MediaType::Subtitle => Self::Subtitle(stream.with_kind()),
            MediaType::Attachment => Self::Attachment(stream.with_kind()),
        }
    }
}

impl<'a> StreamIter<'a> {
    pub(crate) fn new(context: &'a ContextInner) -> Self {
        Self { context, index: 0 }
    }
}

impl<'a> Iterator for StreamIter<'a> {
    type Item = StreamType<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let context = unsafe { &*self.context.as_ptr() };
        if self.index >= context.nb_streams as usize {
            return None;
        }
        let ptr = unsafe { *context.streams.add(self.index) };
        self.index += 1;
        unsafe { Stream::from_ptr(ptr, self.context).map(StreamType::from) }
    }
}

impl<'a> StreamIterMut<'a> {
    pub(crate) fn new(context: &'a mut ContextInner) -> Self {
        Self { context, index: 0 }
    }
}

impl<'a> Iterator for StreamIterMut<'a> {
    type Item = StreamTypeMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let context = unsafe { &*self.context.as_ptr() };
        if self.index >= context.nb_streams as usize {
            return None;
        }
        let ptr = unsafe { *context.streams.add(self.index) };
        self.index += 1;
        Some(StreamTypeMut::from(StreamMut {
            inner: unsafe { Inner::from_ptr(ptr)? },
            context: PhantomData,
            _kind: PhantomData,
        }))
    }
}

macro_rules! stream_media {
    ($($kind:ident),* $(,)?) => {$ (
        impl Kind for $kind {
            fn get(stream: StreamType<'_>) -> Option<Stream<'_, Self>> {
                if let StreamType::$kind(stream) = stream { Some(stream) } else { None }
            }
            fn get_mut(stream: StreamTypeMut<'_>) -> Option<StreamMut<'_, Self>> {
                if let StreamTypeMut::$kind(stream) = stream { Some(stream) } else { None }
            }
        }
    )*};
}

stream_media!(Unknown, Video, Audio, Data, Subtitle, Attachment);
