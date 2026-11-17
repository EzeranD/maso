// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use ffmsys::AVMediaType;

pub enum MediaType {
    Unknown,
    Video,
    Audio,
    Data,
    Subtitle,
    Attachment,
}

pub trait Kind: private::Sealed {
    const TYPE: MediaType;
}

pub struct Unknown;
pub struct Video;
pub struct Audio;
pub struct Data;
pub struct Subtitle;
pub struct Attachment;

impl From<AVMediaType> for MediaType {
    fn from(media_type: AVMediaType) -> Self {
        match media_type {
            AVMediaType::AVMEDIA_TYPE_UNKNOWN | AVMediaType::AVMEDIA_TYPE_NB => Self::Unknown,
            AVMediaType::AVMEDIA_TYPE_VIDEO => Self::Video,
            AVMediaType::AVMEDIA_TYPE_AUDIO => Self::Audio,
            AVMediaType::AVMEDIA_TYPE_DATA => Self::Data,
            AVMediaType::AVMEDIA_TYPE_SUBTITLE => Self::Subtitle,
            AVMediaType::AVMEDIA_TYPE_ATTACHMENT => Self::Attachment,
        }
    }
}

impl From<MediaType> for AVMediaType {
    fn from(value: MediaType) -> Self {
        match value {
            MediaType::Unknown => Self::AVMEDIA_TYPE_UNKNOWN,
            MediaType::Video => Self::AVMEDIA_TYPE_VIDEO,
            MediaType::Audio => Self::AVMEDIA_TYPE_AUDIO,
            MediaType::Data => Self::AVMEDIA_TYPE_DATA,
            MediaType::Subtitle => Self::AVMEDIA_TYPE_SUBTITLE,
            MediaType::Attachment => Self::AVMEDIA_TYPE_ATTACHMENT,
        }
    }
}

impl Kind for Unknown {
    const TYPE: MediaType = MediaType::Unknown;
}

impl Kind for Video {
    const TYPE: MediaType = MediaType::Video;
}

impl Kind for Audio {
    const TYPE: MediaType = MediaType::Audio;
}

impl Kind for Data {
    const TYPE: MediaType = MediaType::Data;
}

impl Kind for Subtitle {
    const TYPE: MediaType = MediaType::Subtitle;
}

impl Kind for Attachment {
    const TYPE: MediaType = MediaType::Attachment;
}

mod private {
    pub trait Sealed {}

    impl Sealed for super::Unknown {}
    impl Sealed for super::Video {}
    impl Sealed for super::Audio {}
    impl Sealed for super::Data {}
    impl Sealed for super::Subtitle {}
    impl Sealed for super::Attachment {}
}
