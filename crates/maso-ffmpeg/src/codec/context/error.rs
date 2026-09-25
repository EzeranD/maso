// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use snafu::Snafu;

use crate::util::Code;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum Error {
    #[snafu(display("failed to allocate codec context"))]
    AllocContext,
    #[snafu(display("failed to set parameters"))]
    SetParameters { source: Code },
    #[snafu(display("failed to open codec"))]
    OpenCodec { source: Code },
}

#[derive(Debug, Snafu)]
#[snafu(module)]
pub enum SendPacketError {
    #[snafu(display("receive frames before sending another packet"))]
    NeedOutput { source: Code },
    #[snafu(display("decoder cannot accept packets after flushing"))]
    Flushed { source: Code },
    #[snafu(display("failed to send packet"))]
    Other { source: Code },
}

#[derive(Debug, Snafu)]
#[snafu(module)]
pub enum SendFrameError {
    #[snafu(display("receive packets before sending another frame"))]
    NeedOutput { source: Code },
    #[snafu(display("encoder cannot accept frames after flushing"))]
    Flushed { source: Code },
    #[snafu(display("failed to send frame"))]
    Other { source: Code },
}

#[derive(Debug, Snafu)]
#[snafu(module)]
pub enum ReceiveFrameError {
    #[snafu(display("more packets are required to receive a frame"))]
    NeedInput { source: Code },
    #[snafu(display("decoder has no more frames"))]
    Drained { source: Code },
    #[snafu(display("failed to receive frame"))]
    Other { source: Code },
}

#[derive(Debug, Snafu)]
#[snafu(module)]
pub enum ReceivePacketError {
    #[snafu(display("send more frames to receive a packet"))]
    NeedInput { source: Code },
    #[snafu(display("encoder has no more packets"))]
    Drained { source: Code },
    #[snafu(display("failed to receive packet"))]
    Other { source: Code },
}

impl From<Code> for SendPacketError {
    fn from(code: Code) -> Self {
        match code {
            Code::AGAIN => Self::NeedOutput { source: code },
            Code::EOF => Self::Flushed { source: code },
            source => Self::Other { source },
        }
    }
}

impl From<Code> for SendFrameError {
    fn from(code: Code) -> Self {
        match code {
            Code::AGAIN => Self::NeedOutput { source: code },
            Code::EOF => Self::Flushed { source: code },
            source => Self::Other { source },
        }
    }
}

impl From<Code> for ReceiveFrameError {
    fn from(code: Code) -> Self {
        match code {
            Code::AGAIN => Self::NeedInput { source: code },
            Code::EOF => Self::Drained { source: code },
            source => Self::Other { source },
        }
    }
}

impl From<Code> for ReceivePacketError {
    fn from(code: Code) -> Self {
        match code {
            Code::AGAIN => Self::NeedInput { source: code },
            Code::EOF => Self::Drained { source: code },
            source => Self::Other { source },
        }
    }
}
