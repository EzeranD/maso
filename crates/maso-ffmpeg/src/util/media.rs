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
