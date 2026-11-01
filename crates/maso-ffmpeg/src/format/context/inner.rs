// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    ffi::{CStr, CString, NulError},
    mem::ManuallyDrop,
    ptr::{self, NonNull},
};

use ffmsys::{
    AVFormatContext, av_find_best_stream, av_find_default_stream_index, av_read_frame,
    av_seek_frame, avformat_alloc_context, avformat_close_input, avformat_find_stream_info,
    avformat_free_context, avformat_open_input, avformat_seek_file,
};
use snafu::{OptionExt, ResultExt, Snafu};

use crate::{
    codec::packet::Packet,
    format::{
        chapter::{Chapter, ChapterMut},
        program::{ProgramIter, ProgramIterMut},
        stream::{Stream, StreamMut},
    },
    util::{Code, MediaType, dictionary::DictionaryRef},
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("failed to allocate format context"))]
    AllocContext,
    #[snafu(display("input path contains a null byte"))]
    InvalidPath { source: NulError },
    #[snafu(display("failed to open input"))]
    OpenInput { source: Code },
    #[snafu(display("failed to find stream information"))]
    FindStreamInfo { source: Code },
    #[snafu(display("failed to read packet"))]
    ReadPacket { source: Code },
    #[snafu(display("failed to seek"))]
    Seek { source: Code },
}

pub(crate) struct Inner {
    ptr: NonNull<AVFormatContext>,
    state: State,
}

enum State {
    Allocated,
    OpenedInput,
}

impl Inner {
    pub(super) fn new() -> Result<Self, Error> {
        unsafe {
            let ptr = avformat_alloc_context();
            let inner = NonNull::new(ptr).context(AllocContextSnafu)?;
            Ok(Self {
                ptr: inner,
                state: State::Allocated,
            })
        }
    }

    pub(super) fn open_input(self, path: &str) -> Result<Self, Error> {
        unsafe {
            let url = CString::new(path).context(InvalidPathSnafu)?;
            let mut inner = ManuallyDrop::new(self);
            let mut ctx = inner.as_mut_ptr();
            let res = avformat_open_input(&raw mut ctx, url.as_ptr(), ptr::null(), ptr::null_mut());
            if res < 0 {
                return Err(res.into()).context(OpenInputSnafu);
            }
            let mut inner = ManuallyDrop::into_inner(inner);
            inner.state = State::OpenedInput;
            Ok(inner)
        }
    }

    pub(super) fn stream(&self, index: usize) -> Option<Stream<'_>> {
        unsafe {
            let context = self.ptr.as_ref();
            if index >= context.nb_streams as usize {
                return None;
            }
            let ptr = *context.streams.add(index);
            Stream::from_ptr(ptr, self)
        }
    }

    pub(super) fn default_stream(&self) -> Option<Stream<'_>> {
        unsafe {
            let res = av_find_default_stream_index(self.as_ptr().cast_mut());
            if res < 0 {
                return None;
            }
            let ptr = *self.ptr.as_ref().streams.add(res as usize);
            Stream::from_ptr(ptr, self)
        }
    }

    pub(super) fn best_stream(
        &self,
        media_type: MediaType,
        wanted: Option<i32>,
        related: Option<i32>,
    ) -> Option<Stream<'_>> {
        unsafe {
            let res = av_find_best_stream(
                self.as_ptr().cast_mut(),
                media_type.into(),
                wanted.unwrap_or(-1),
                related.unwrap_or(-1),
                ptr::null_mut(),
                0,
            );
            if res < 0 {
                return None;
            }
            let ptr = *self.ptr.as_ref().streams.add(res as usize);
            Stream::from_ptr(ptr, self)
        }
    }

    pub(super) fn chapter(&self, index: usize) -> Option<Chapter<'_>> {
        unsafe {
            let context = self.ptr.as_ref();
            if index >= context.nb_chapters as usize {
                return None;
            }
            Chapter::from_ptr(*context.chapters.add(index), self)
        }
    }

    pub(super) fn url(&self) -> Option<&CStr> {
        unsafe {
            let url = self.ptr.as_ref().url;
            if url.is_null() {
                return None;
            }

            Some(CStr::from_ptr(url))
        }
    }

    pub(super) fn start_time(&self) -> Option<i64> {
        let value = unsafe { self.ptr.as_ref().start_time };
        (value != ffmsys::AV_NOPTS_VALUE).then_some(value)
    }

    pub(super) fn duration(&self) -> Option<i64> {
        let value = unsafe { self.ptr.as_ref().duration };
        (value != ffmsys::AV_NOPTS_VALUE).then_some(value)
    }

    pub(super) fn bit_rate(&self) -> i64 {
        unsafe { self.ptr.as_ref().bit_rate }
    }

    pub(super) fn programs_for_stream(&self, index: i32) -> ProgramIter<'_> {
        ProgramIter::new(self, index)
    }

    pub(super) fn metadata(&self) -> DictionaryRef<'_> {
        unsafe { DictionaryRef::from_ptr(self.ptr.as_ref().metadata, self) }
    }

    pub(super) fn stream_mut(&mut self, index: usize) -> Option<StreamMut<'_>> {
        unsafe {
            let context = self.ptr.as_ref();
            if index >= context.nb_streams as usize {
                return None;
            }
            let ptr = *context.streams.add(index);
            StreamMut::from_ptr(ptr, self)
        }
    }

    pub(super) fn chapter_mut(&mut self, index: usize) -> Option<ChapterMut<'_>> {
        unsafe {
            let context = self.ptr.as_ref();
            if index >= context.nb_chapters as usize {
                return None;
            }
            let ptr = *context.chapters.add(index);
            ChapterMut::from_ptr(ptr, self)
        }
    }

    pub(super) fn default_stream_mut(&mut self) -> Option<StreamMut<'_>> {
        unsafe {
            let res = av_find_default_stream_index(self.as_ptr().cast_mut());
            if res < 0 {
                return None;
            }
            let ptr = *self.ptr.as_ref().streams.add(res as usize);
            StreamMut::from_ptr(ptr, self)
        }
    }

    pub(super) fn best_stream_mut(
        &mut self,
        media_type: MediaType,
        wanted: Option<i32>,
        related: Option<i32>,
    ) -> Option<StreamMut<'_>> {
        unsafe {
            let res = av_find_best_stream(
                self.as_ptr().cast_mut(),
                media_type.into(),
                wanted.unwrap_or(-1),
                related.unwrap_or(-1),
                ptr::null_mut(),
                0,
            );
            if res < 0 {
                return None;
            }
            let ptr = *self.ptr.as_ref().streams.add(res as usize);
            StreamMut::from_ptr(ptr, self)
        }
    }

    pub(super) fn programs_for_stream_mut(&mut self, index: i32) -> ProgramIterMut<'_> {
        ProgramIterMut::new(self, index)
    }

    pub(super) fn find_stream_info(&mut self) -> Result<(), Error> {
        let res = unsafe { avformat_find_stream_info(self.as_mut_ptr(), ptr::null_mut()) };
        if res < 0 {
            return Err(res.into()).context(FindStreamInfoSnafu);
        }
        Ok(())
    }

    pub(super) fn read_packet(&mut self, packet: &mut Packet) -> Result<(), Error> {
        packet.unref();
        let res = unsafe { av_read_frame(self.as_mut_ptr(), packet.as_mut_ptr()) };
        if res < 0 {
            return Err(res.into()).context(ReadPacketSnafu);
        }
        Ok(())
    }

    pub(super) fn seek_frame(
        &mut self,
        stream: i32,
        timestamp: i64,
        flags: i32,
    ) -> Result<(), Error> {
        let res = unsafe { av_seek_frame(self.as_mut_ptr(), stream, timestamp, flags) };
        if res < 0 {
            return Err(res.into()).context(SeekSnafu);
        }
        Ok(())
    }

    pub(super) fn seek_file(
        &mut self,
        stream: i32,
        min: i64,
        timestamp: i64,
        max: i64,
        flags: i32,
    ) -> Result<(), Error> {
        let res =
            unsafe { avformat_seek_file(self.as_mut_ptr(), stream, min, timestamp, max, flags) };
        if res < 0 {
            return Err(res.into()).context(SeekSnafu);
        }
        Ok(())
    }
}

impl Inner {
    pub(crate) fn as_ptr(&self) -> *const AVFormatContext {
        self.ptr.as_ptr()
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut AVFormatContext {
        self.ptr.as_ptr()
    }
}

unsafe impl Send for Inner {}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            let mut ptr = self.as_mut_ptr();
            match self.state {
                State::Allocated => avformat_free_context(ptr),
                State::OpenedInput => avformat_close_input(&raw mut ptr),
            }
        }
    }
}
