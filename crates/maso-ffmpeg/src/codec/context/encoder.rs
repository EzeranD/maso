// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::{
    Context, Encoder, Opened, ReceiveFrameError, ReceiveFrameFlags, ReceivePacketError,
    SendFrameError,
};
use crate::{
    codec::packet::Packet,
    util::{
        frame::{Frame, Kind as FrameKind},
        media::Kind,
    },
};

impl<K: Kind> Context<Opened, Encoder, K> {
    pub fn send_frame(&mut self, frame: Option<&Frame<K>>) -> Result<(), SendFrameError>
    where
        K: FrameKind,
    {
        self.inner.send_frame(frame)
    }

    pub fn receive_packet(&mut self, packet: &mut Packet) -> Result<(), ReceivePacketError> {
        self.inner.receive_packet(packet)
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
