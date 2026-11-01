// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{ffi::CStr, marker::PhantomData};

use crate::{
    codec::packet::Packet,
    format::{
        chapter::{Chapter, ChapterIter, ChapterIterMut, ChapterMut},
        context::inner::Inner,
        program::{ProgramIter, ProgramIterMut},
        stream::{Stream, StreamIter, StreamIterMut, StreamMut},
    },
    util::{MediaType, dictionary::DictionaryRef},
};

pub(super) mod inner;
pub use inner::Error;

pub struct Context<State, Mode> {
    inner: Inner,
    _state: PhantomData<State>,
    _mode: PhantomData<Mode>,
}

pub struct Unopened;
pub struct Opened;
pub struct Input;
pub struct Output;

pub type Demuxer = Context<Opened, Input>;

impl<Mode> Context<Unopened, Mode> {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            inner: Inner::new()?,
            _state: PhantomData,
            _mode: PhantomData,
        })
    }
}

impl<State, Mode> Context<State, Mode> {
    pub fn streams(&self) -> StreamIter<'_> {
        StreamIter::new(&self.inner)
    }

    pub fn stream(&self, index: usize) -> Option<Stream<'_>> {
        self.inner.stream(index)
    }

    pub fn best_stream(
        &self,
        media_type: MediaType,
        wanted: Option<i32>,
        related: Option<i32>,
    ) -> Option<Stream<'_>> {
        self.inner.best_stream(media_type, wanted, related)
    }

    pub fn default_stream(&self) -> Option<Stream<'_>> {
        self.inner.default_stream()
    }

    pub fn chapters(&self) -> ChapterIter<'_> {
        ChapterIter::new(&self.inner)
    }

    pub fn chapter(&self, index: usize) -> Option<Chapter<'_>> {
        self.inner.chapter(index)
    }

    pub fn url(&self) -> Option<&CStr> {
        self.inner.url()
    }

    pub fn start_time(&self) -> Option<i64> {
        self.inner.start_time()
    }

    pub fn duration(&self) -> Option<i64> {
        self.inner.duration()
    }

    pub fn bit_rate(&self) -> i64 {
        self.inner.bit_rate()
    }

    pub fn programs_for_stream(&self, index: i32) -> ProgramIter<'_> {
        self.inner.programs_for_stream(index)
    }

    pub fn metadata(&self) -> DictionaryRef<'_> {
        self.inner.metadata()
    }

    pub fn streams_mut(&mut self) -> StreamIterMut<'_> {
        StreamIterMut::new(&mut self.inner)
    }

    pub fn stream_mut(&mut self, index: usize) -> Option<StreamMut<'_>> {
        self.inner.stream_mut(index)
    }

    pub fn best_stream_mut(
        &mut self,
        media_type: MediaType,
        wanted: Option<i32>,
        related: Option<i32>,
    ) -> Option<StreamMut<'_>> {
        self.inner.best_stream_mut(media_type, wanted, related)
    }

    pub fn default_stream_mut(&mut self) -> Option<StreamMut<'_>> {
        self.inner.default_stream_mut()
    }

    pub fn chapters_mut(&mut self) -> ChapterIterMut<'_> {
        ChapterIterMut::new(&mut self.inner)
    }

    pub fn chapter_mut(&mut self, index: usize) -> Option<ChapterMut<'_>> {
        self.inner.chapter_mut(index)
    }

    pub fn programs_for_stream_mut(&mut self, index: i32) -> ProgramIterMut<'_> {
        self.inner.programs_for_stream_mut(index)
    }
}

impl Context<Unopened, Input> {
    pub fn open(self, path: &str) -> Result<Context<Opened, Input>, Error> {
        let inner = self.inner.open_input(path)?;
        Ok(Context {
            inner,
            _state: PhantomData,
            _mode: PhantomData,
        })
    }
}

impl Context<Opened, Input> {
    pub fn open(path: &str) -> Result<Self, Error> {
        let mut context = Context::<Unopened, Input>::new()?.open(path)?;
        context.find_stream_info()?;
        Ok(context)
    }

    pub fn find_stream_info(&mut self) -> Result<(), Error> {
        self.inner.find_stream_info()
    }
    pub fn read_packet(&mut self, packet: &mut Packet) -> Result<(), Error> {
        self.inner.read_packet(packet)
    }
    pub fn seek_frame(&mut self, stream: i32, timestamp: i64, flags: i32) -> Result<(), Error> {
        self.inner.seek_frame(stream, timestamp, flags)
    }
    pub fn seek_file(
        &mut self,
        stream: i32,
        min: i64,
        timestamp: i64,
        max: i64,
        flags: i32,
    ) -> Result<(), Error> {
        self.inner.seek_file(stream, min, timestamp, max, flags)
    }
}
