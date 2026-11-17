// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::{
    Context, Decoder, GetFormatContext, Opened, ReceiveFrameError, ReceiveFrameFlags,
    SendPacketError, Unopened,
};
use crate::{
    codec::packet::Packet,
    util::{
        frame::{Frame, Kind as FrameKind},
        media::{Kind, Video},
        pixel::PixelFormat,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Discard {
    None,
    Default,
    NonReference,
    Bidirectional,
    NonIntra,
    NonKey,
    All,
}

impl Context<Unopened, Decoder, Video> {
    pub fn get_format<F>(&mut self, f: F)
    where
        F: Fn(&mut GetFormatContext<'_>, &[PixelFormat]) -> PixelFormat + Send + Sync + 'static,
    {
        self.inner.get_format(f);
    }
}

impl<K: Kind> Context<Opened, Decoder, K> {
    pub fn send_packet(&mut self, packet: Option<&Packet>) -> Result<(), SendPacketError> {
        self.inner.send_packet(packet)
    }

    pub fn receive_frame(
        &mut self,
        frame: &mut Frame<K>,
        flags: Option<ReceiveFrameFlags>,
    ) -> Result<(), ReceiveFrameError>
    where
        K: FrameKind,
    {
        self.inner.receive_frame(frame, flags)
    }
}

impl<State, K: Kind> Context<State, Decoder, K> {
    pub fn skip_frame(&mut self, discard: Discard) {
        self.inner.skip_frame(discard);
    }
}

impl From<Discard> for ffmsys::AVDiscard {
    fn from(discard: Discard) -> Self {
        match discard {
            Discard::None => Self::AVDISCARD_NONE,
            Discard::Default => Self::AVDISCARD_DEFAULT,
            Discard::NonReference => Self::AVDISCARD_NONREF,
            Discard::Bidirectional => Self::AVDISCARD_BIDIR,
            Discard::NonIntra => Self::AVDISCARD_NONINTRA,
            Discard::NonKey => Self::AVDISCARD_NONKEY,
            Discard::All => Self::AVDISCARD_ALL,
        }
    }
}
