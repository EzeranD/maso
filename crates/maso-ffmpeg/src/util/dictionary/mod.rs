// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    ffi::{CStr, CString, NulError, c_int},
    marker::PhantomData,
    mem::ManuallyDrop,
    ptr::{NonNull, null_mut},
};

use bitflags::bitflags;
#[cfg(ffmpeg_8_0)]
use ffmsys::AV_DICT_DEDUP;
use ffmsys::{
    AV_DICT_APPEND, AV_DICT_DONT_OVERWRITE, AV_DICT_IGNORE_SUFFIX, AV_DICT_MATCH_CASE,
    AV_DICT_MULTIKEY, AVDictionary, AVDictionaryEntry,
};
use snafu::ResultExt;

use crate::util::{Code, dictionary::inner::Inner};

mod inner;

#[derive(Debug, snafu::Snafu)]
pub enum Error {
    #[snafu(display("dictionary string contains a null byte"))]
    InvalidString { source: NulError },
    #[snafu(display("failed to clone dictionary"))]
    Clone { source: Code },
    #[snafu(display("failed to copy dictionary entries"))]
    Copy { source: Code },
    #[snafu(display("failed to set dictionary entry"))]
    SetEntry { source: Code },
    #[snafu(display("failed to remove dictionary entry"))]
    RemoveEntry { source: Code },
}

pub struct Dictionary {
    inner: Inner,
}

pub struct DictionaryRef<'a> {
    inner: ManuallyDrop<Inner>,
    owner: PhantomData<&'a ()>,
}

pub struct DictionaryMut<'a> {
    inner: ManuallyDrop<Inner>,
    owner_ptr: &'a mut *mut AVDictionary,
}

pub struct Entry<'a> {
    ptr: NonNull<AVDictionaryEntry>,
    dictionary: &'a Inner,
}

bitflags! {
    pub struct DictFlags: c_int {
        const MATCH_CASE = AV_DICT_MATCH_CASE;
        const IGNORE_SUFFIX = AV_DICT_IGNORE_SUFFIX;
        const DONT_OVERWRITE = AV_DICT_DONT_OVERWRITE;
        const APPEND = AV_DICT_APPEND;
        const MULTIKEY = AV_DICT_MULTIKEY;
        #[cfg(ffmpeg_8_0)]
        const DEDUP = AV_DICT_DEDUP;
    }
}

impl Dictionary {
    pub fn new() -> Self {
        Self {
            inner: Inner::from_ptr(null_mut()),
        }
    }

    pub fn try_clone(&self) -> Result<Self, Error> {
        Ok(Self {
            inner: self.inner.try_clone()?,
        })
    }

    pub fn as_ref(&self) -> DictionaryRef<'_> {
        unsafe { DictionaryRef::from_ptr(self.inner.as_ptr().cast_mut(), self) }
    }

    pub fn get(
        &self,
        key: &str,
        prev: Option<Entry<'_>>,
        flags: &DictFlags,
    ) -> Result<Option<Entry<'_>>, Error> {
        let key = CString::new(key).context(InvalidStringSnafu)?;
        Ok(self.inner.get(&key, prev, flags))
    }

    pub fn iter(&self) -> impl Iterator<Item = Entry<'_>> {
        self.inner.iter()
    }

    pub fn len(&self) -> i32 {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn set(&mut self, key: &str, value: &str, flags: &DictFlags) -> Result<(), Error> {
        let key = CString::new(key).context(InvalidStringSnafu)?;
        let value = CString::new(value).context(InvalidStringSnafu)?;
        self.inner.set(&key, &value, flags)
    }

    pub fn set_int(&mut self, key: &str, value: i64, flags: &DictFlags) -> Result<(), Error> {
        let key = CString::new(key).context(InvalidStringSnafu)?;
        self.inner.set_int(&key, value, flags)
    }

    pub fn remove(&mut self, key: &str) -> Result<(), Error> {
        let key = CString::new(key).context(InvalidStringSnafu)?;
        self.inner.remove(&key)
    }

    pub fn copy_from(&mut self, source: &Dictionary, flags: &DictFlags) -> Result<(), Error> {
        self.inner.copy_from(&source.inner, flags)
    }

    pub unsafe fn from_ptr(ptr: *mut AVDictionary) -> Self {
        Self {
            inner: Inner::from_ptr(ptr),
        }
    }

    pub fn as_ptr(&self) -> *const AVDictionary {
        self.inner.as_ptr()
    }

    pub(crate) unsafe fn set_ptr(&mut self, ptr: *mut AVDictionary) {
        self.inner.set_ptr(ptr);
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut AVDictionary {
        self.inner.as_mut_ptr()
    }
}

unsafe impl Send for Dictionary {}
unsafe impl Sync for Dictionary {}

impl Clone for Dictionary {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<'a> DictionaryRef<'a> {
    pub fn try_to_owned(&self) -> Result<Dictionary, Error> {
        Ok(Dictionary {
            inner: self.inner.try_clone()?,
        })
    }

    pub fn to_owned(&self) -> Dictionary {
        self.try_to_owned().expect("failed to clone dictionary")
    }

    pub fn get(
        &self,
        key: &str,
        prev: Option<Entry<'_>>,
        flags: &DictFlags,
    ) -> Result<Option<Entry<'_>>, Error> {
        let key = CString::new(key).context(InvalidStringSnafu)?;
        Ok(self.inner.get(&key, prev, flags))
    }

    pub fn iter(&self) -> impl Iterator<Item = Entry<'_>> {
        self.inner.iter()
    }

    pub fn len(&self) -> i32 {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub unsafe fn from_ptr<T: ?Sized>(ptr: *mut AVDictionary, _owner: &'a T) -> Self {
        Self {
            inner: ManuallyDrop::new(Inner::from_ptr(ptr)),
            owner: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *const AVDictionary {
        self.inner.as_ptr()
    }
}

unsafe impl Send for DictionaryRef<'_> {}
unsafe impl Sync for DictionaryRef<'_> {}

impl<'a> DictionaryMut<'a> {
    pub fn try_to_owned(&self) -> Result<Dictionary, Error> {
        Ok(Dictionary {
            inner: self.inner.try_clone()?,
        })
    }

    pub fn to_owned(&self) -> Dictionary {
        self.try_to_owned().expect("failed to clone dictionary")
    }

    pub fn get(
        &self,
        key: &str,
        prev: Option<Entry<'_>>,
        flags: &DictFlags,
    ) -> Result<Option<Entry<'_>>, Error> {
        let key = CString::new(key).context(InvalidStringSnafu)?;
        Ok(self.inner.get(&key, prev, flags))
    }

    pub fn iter(&self) -> impl Iterator<Item = Entry<'_>> {
        self.inner.iter()
    }

    pub fn len(&self) -> i32 {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn set(&mut self, key: &str, value: &str, flags: &DictFlags) -> Result<(), Error> {
        let key = CString::new(key).context(InvalidStringSnafu)?;
        let value = CString::new(value).context(InvalidStringSnafu)?;
        let result = self.inner.set(&key, &value, flags);
        *self.owner_ptr = self.inner.as_mut_ptr();
        result
    }

    pub fn set_int(&mut self, key: &str, value: i64, flags: &DictFlags) -> Result<(), Error> {
        let key = CString::new(key).context(InvalidStringSnafu)?;
        let result = self.inner.set_int(&key, value, flags);
        *self.owner_ptr = self.inner.as_mut_ptr();
        result
    }

    pub fn remove(&mut self, key: &str) -> Result<(), Error> {
        let key = CString::new(key).context(InvalidStringSnafu)?;
        let result = self.inner.remove(&key);
        *self.owner_ptr = self.inner.as_mut_ptr();
        result
    }

    pub fn copy_from(&mut self, source: &Dictionary, flags: &DictFlags) -> Result<(), Error> {
        let result = self.inner.copy_from(&source.inner, flags);
        *self.owner_ptr = self.inner.as_mut_ptr();
        result
    }

    pub unsafe fn from_ptr(owner_ptr: &'a mut *mut AVDictionary) -> Self {
        Self {
            inner: ManuallyDrop::new(Inner::from_ptr(*owner_ptr)),
            owner_ptr,
        }
    }

    pub fn as_ptr(&self) -> *const AVDictionary {
        self.inner.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut AVDictionary {
        self.inner.as_mut_ptr()
    }
}

unsafe impl Send for DictionaryMut<'_> {}
unsafe impl Sync for DictionaryMut<'_> {}

impl Entry<'_> {
    pub fn key(&self) -> &CStr {
        unsafe {
            let key = self.ptr.as_ref().key;
            CStr::from_ptr(key)
        }
    }

    pub fn value(&self) -> &CStr {
        unsafe {
            let value = self.ptr.as_ref().value;
            CStr::from_ptr(value)
        }
    }
}

impl<'a> Entry<'a> {
    unsafe fn from_ptr(ptr: *mut AVDictionaryEntry, dictionary: &'a Inner) -> Option<Self> {
        Some(Self {
            ptr: NonNull::new(ptr)?,
            dictionary,
        })
    }

    pub fn as_ptr(&self) -> *const AVDictionaryEntry {
        self.ptr.as_ptr()
    }

    pub fn as_mut_ptr(&self) -> *mut AVDictionaryEntry {
        self.ptr.as_ptr()
    }
}

unsafe impl Send for Entry<'_> {}
unsafe impl Sync for Entry<'_> {}
