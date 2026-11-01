// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    ffi::CStr,
    ptr::{self, NonNull, null, null_mut},
};

#[cfg(not(ffmpeg_6_0))]
use ffmsys::AV_DICT_IGNORE_SUFFIX;
#[cfg(ffmpeg_6_0)]
use ffmsys::av_dict_iterate;
use ffmsys::{
    AVDictionary, av_dict_copy, av_dict_count, av_dict_free, av_dict_get, av_dict_set,
    av_dict_set_int,
};
use snafu::ResultExt;

use super::{CloneSnafu, CopySnafu, DictFlags, Entry, Error, RemoveEntrySnafu, SetEntrySnafu};

pub(crate) struct Inner {
    ptr: Option<NonNull<AVDictionary>>,
}

impl Inner {
    pub(super) fn try_clone(&self) -> Result<Self, Error> {
        let mut ptr = null_mut();
        let res = unsafe { av_dict_copy(&raw mut ptr, self.as_ptr(), 0) };
        let dst = Self::from_ptr(ptr);
        if res < 0 {
            return Err(res.into()).context(CloneSnafu);
        }
        Ok(dst)
    }

    pub(super) fn get(
        &self,
        key: &CStr,
        prev: Option<Entry>,
        flags: &DictFlags,
    ) -> Option<Entry<'_>> {
        let prev = match prev {
            Some(entry) if ptr::eq(entry.dictionary, self) => entry.as_ptr(),
            Some(_) => return None,
            None => std::ptr::null(),
        };
        let entry = unsafe { av_dict_get(self.as_ptr(), key.as_ptr(), prev, flags.bits()) };
        unsafe { Entry::from_ptr(entry, self) }
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = Entry<'_>> {
        let mut prev = ptr::null();
        std::iter::from_fn(move || {
            #[cfg(ffmpeg_6_0)]
            let entry = unsafe { av_dict_iterate(self.as_ptr(), prev) }.cast_mut();
            #[cfg(not(ffmpeg_6_0))]
            let entry =
                unsafe { av_dict_get(self.as_ptr(), c"".as_ptr(), prev, AV_DICT_IGNORE_SUFFIX) };

            let entry = unsafe { Entry::from_ptr(entry, self)? };
            prev = entry.as_ptr();
            Some(entry)
        })
    }

    pub(super) fn len(&self) -> i32 {
        unsafe { av_dict_count(self.as_ptr()) }
    }

    pub(super) fn set(&mut self, key: &CStr, value: &CStr, flags: &DictFlags) -> Result<(), Error> {
        let flags = DictFlags::from_bits_truncate(flags.bits());
        let mut ptr = self.as_mut_ptr();
        let res = unsafe { av_dict_set(&raw mut ptr, key.as_ptr(), value.as_ptr(), flags.bits()) };
        self.ptr = NonNull::new(ptr);
        if res < 0 {
            return Err(res.into()).context(SetEntrySnafu);
        }
        Ok(())
    }

    pub(super) fn set_int(
        &mut self,
        key: &CStr,
        value: i64,
        flags: &DictFlags,
    ) -> Result<(), Error> {
        let flags = DictFlags::from_bits_truncate(flags.bits());
        let mut ptr = self.as_mut_ptr();
        let res = unsafe { av_dict_set_int(&raw mut ptr, key.as_ptr(), value, flags.bits()) };
        self.ptr = NonNull::new(ptr);
        if res < 0 {
            return Err(res.into()).context(SetEntrySnafu);
        }
        Ok(())
    }

    pub(super) fn remove(&mut self, key: &CStr) -> Result<(), Error> {
        let mut ptr = self.as_mut_ptr();
        let res = unsafe { av_dict_set(&raw mut ptr, key.as_ptr(), ptr::null(), 0) };
        self.ptr = NonNull::new(ptr);
        if res < 0 {
            return Err(res.into()).context(RemoveEntrySnafu);
        }
        Ok(())
    }

    pub(super) fn copy_from(&mut self, source: &Self, flags: &DictFlags) -> Result<(), Error> {
        let flags = DictFlags::from_bits_truncate(flags.bits());
        let mut ptr = self.as_mut_ptr();
        let res = unsafe { av_dict_copy(&raw mut ptr, source.as_ptr(), flags.bits()) };
        self.ptr = NonNull::new(ptr);
        if res < 0 {
            return Err(res.into()).context(CopySnafu);
        }
        Ok(())
    }
}

impl Inner {
    pub(super) fn from_ptr(ptr: *mut AVDictionary) -> Self {
        let ptr = NonNull::new(ptr);
        Self { ptr }
    }

    pub(super) fn as_ptr(&self) -> *const AVDictionary {
        self.ptr.map_or(null(), |p| p.as_ptr())
    }

    pub(super) fn as_mut_ptr(&mut self) -> *mut AVDictionary {
        self.ptr.map_or(null_mut(), |p| p.as_ptr())
    }

    pub(super) fn set_ptr(&mut self, ptr: *mut AVDictionary) {
        self.ptr = NonNull::new(ptr);
    }
}

impl Clone for Inner {
    fn clone(&self) -> Self {
        self.try_clone().expect("failed to clone dictionary")
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        let mut ptr = self.as_mut_ptr();
        unsafe {
            av_dict_free(&raw mut ptr);
        }
    }
}
