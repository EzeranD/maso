// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod buffer;
pub mod frame;
pub mod hardware;
pub mod pixel;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    ffi::CStr,
    fmt,
};

use ffmsys::{AV_ERROR_MAX_STRING_SIZE, av_strerror};

pub struct Extensions {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Code(i32);

impl Extensions {
    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert<T: Any + Send + Sync>(&mut self, value: T) {
        self.map.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref())
    }
}

impl Code {
    pub const AGAIN: Self = Self(ffmsys::AVERROR(ffmsys::EAGAIN));
    pub const EOF: Self = Self(ffmsys::AVERROR_EOF);
    pub const INVALID: Self = Self(ffmsys::AVERROR(ffmsys::EINVAL));
    pub const NOT_FOUND: Self = Self(ffmsys::AVERROR(ffmsys::ENOENT));
    pub const RANGE: Self = Self(ffmsys::AVERROR(ffmsys::ERANGE));

    pub fn from_raw(code: i32) -> Self {
        Self(code)
    }

    pub fn as_raw(self) -> i32 {
        self.0
    }

    fn fmt_message(&self, buffer: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match CStr::from_bytes_until_nul(buffer) {
            Ok(message) => f.write_str(&message.to_string_lossy()),
            Err(_) => write!(f, "Error number {} occurred", self.0),
        }
    }
}

impl From<i32> for Code {
    fn from(code: i32) -> Self {
        Self::from_raw(code)
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buffer = [0u8; AV_ERROR_MAX_STRING_SIZE];
        unsafe {
            av_strerror(self.0, buffer.as_mut_ptr().cast(), buffer.len());
        }
        self.fmt_message(&buffer, f)
    }
}

impl std::error::Error for Code {}
