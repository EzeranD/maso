// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::marker::PhantomData;

use ffmsys::AVChapter;

use crate::{
    format::context::inner::Inner as ContextInner,
    util::{
        Rational,
        dictionary::{DictionaryMut, DictionaryRef},
    },
};

mod inner;
use inner::Inner;

pub struct Chapter<'a> {
    inner: Inner,
    context: &'a ContextInner,
}

pub struct ChapterMut<'a> {
    inner: Inner,
    context: PhantomData<&'a mut ContextInner>,
}

pub struct ChapterIter<'a> {
    context: &'a ContextInner,
    index: usize,
}

pub struct ChapterIterMut<'a> {
    context: &'a mut ContextInner,
    index: usize,
}

impl Chapter<'_> {
    pub fn id(&self) -> i64 {
        self.inner.id()
    }

    pub fn time_base(&self) -> Rational {
        self.inner.time_base()
    }

    pub fn start(&self) -> i64 {
        self.inner.start()
    }

    pub fn end(&self) -> i64 {
        self.inner.end()
    }

    pub fn metadata(&self) -> DictionaryRef<'_> {
        self.inner.metadata(self.context)
    }
}

impl<'a> Chapter<'a> {
    pub(crate) unsafe fn from_ptr(ptr: *mut AVChapter, context: &'a ContextInner) -> Option<Self> {
        Some(Self {
            inner: unsafe { Inner::from_ptr(ptr)? },
            context,
        })
    }

    pub fn as_ptr(&self) -> *const AVChapter {
        self.inner.as_ptr()
    }
}

unsafe impl Send for Chapter<'_> {}
unsafe impl Sync for Chapter<'_> {}

impl ChapterMut<'_> {
    pub fn id(&self) -> i64 {
        self.inner.id()
    }
    pub fn time_base(&self) -> Rational {
        self.inner.time_base()
    }
    pub fn start(&self) -> i64 {
        self.inner.start()
    }
    pub fn end(&self) -> i64 {
        self.inner.end()
    }
    pub fn metadata(&self) -> DictionaryRef<'_> {
        self.inner.metadata(&self.inner)
    }
    pub fn metadata_mut(&mut self) -> DictionaryMut<'_> {
        self.inner.metadata_mut()
    }
}

impl<'a> ChapterMut<'a> {
    pub(crate) unsafe fn from_ptr(
        ptr: *mut AVChapter,
        _context: &'a mut ContextInner,
    ) -> Option<Self> {
        Some(Self {
            inner: unsafe { Inner::from_ptr(ptr)? },
            context: PhantomData,
        })
    }

    pub fn as_ptr(&self) -> *const AVChapter {
        self.inner.as_ptr()
    }
    pub(crate) fn as_mut_ptr(&mut self) -> *mut AVChapter {
        self.inner.as_mut_ptr()
    }
}

unsafe impl Send for ChapterIter<'_> {}
unsafe impl Sync for ChapterIter<'_> {}

impl<'a> ChapterIter<'a> {
    pub(crate) fn new(context: &'a ContextInner) -> Self {
        Self { context, index: 0 }
    }
}

impl<'a> Iterator for ChapterIter<'a> {
    type Item = Chapter<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let context = unsafe { &*self.context.as_ptr() };
        if self.index >= context.nb_chapters as usize {
            return None;
        }
        let ptr = unsafe { *context.chapters.add(self.index) };
        self.index += 1;
        unsafe { Chapter::from_ptr(ptr, self.context) }
    }
}

impl<'a> ChapterIterMut<'a> {
    pub(crate) fn new(context: &'a mut ContextInner) -> Self {
        Self { context, index: 0 }
    }
}

impl<'a> Iterator for ChapterIterMut<'a> {
    type Item = ChapterMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let context = unsafe { &*self.context.as_ptr() };
        if self.index >= context.nb_chapters as usize {
            return None;
        }
        let ptr = unsafe { *context.chapters.add(self.index) };
        self.index += 1;
        Some(ChapterMut {
            inner: unsafe { Inner::from_ptr(ptr)? },
            context: PhantomData,
        })
    }
}
