// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use snafu::Snafu;

pub use crate::codec::context::{Discard, ReceiveFrameError, SendPacketError};
use crate::{
    codec::{
        Codec,
        context::{self, Opened},
        packet::Packet,
        parameters::Parameters,
    },
    util::{
        frame::{Frame, Video},
        hardware::{self, DeviceContext, FramesContext},
        pixel::PixelFormat,
    },
    video::hardware::Candidate,
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(transparent)]
    Context { source: context::Error },
    #[snafu(transparent)]
    DeviceContext { source: hardware::DeviceError },
}

pub struct Context {
    ctx: context::Context<Opened>,
    hw_initialized: Arc<AtomicBool>,
}

impl Context {
    pub fn new(parameters: &Parameters, hw: Option<&Candidate>) -> Result<Self, Error> {
        let hw_initialized = Arc::new(AtomicBool::new(false));

        let codec = hw
            .map(|h| h.codec)
            .or_else(|| Codec::find_decoder(parameters.codec_id()));

        let mut ctx = context::Context::from_parameters(parameters, codec.as_ref())?;

        if let Some(hw) = hw {
            let device = DeviceContext::new(hw.config.device_type())?;
            ctx.set_device_ctx(device);

            let target_fmt = hw.config.format();
            let initialized = hw_initialized.clone();
            ctx.get_format(move |ctx, formats| {
                if formats.contains(&target_fmt) {
                    let Ok(frames) = FramesContext::new(ctx, target_fmt, None) else {
                        return PixelFormat::NONE;
                    };
                    let Ok(frames) = frames.init() else {
                        return PixelFormat::NONE;
                    };
                    ctx.set_frames_ctx(Some(frames));
                    initialized.store(true, Ordering::Relaxed);
                    return target_fmt;
                }
                PixelFormat::NONE
            });
        }

        let ctx = ctx.open(None)?;

        Ok(Self {
            ctx,
            hw_initialized,
        })
    }

    pub fn send_packet(&mut self, packet: &Packet) -> Result<(), SendPacketError> {
        self.ctx.send_packet(Some(packet))
    }

    pub fn receive_frame(&mut self, frame: &mut Frame<Video>) -> Result<(), ReceiveFrameError> {
        self.ctx.receive_frame(frame, None)
    }

    pub fn send_eof(&mut self) -> Result<(), SendPacketError> {
        self.ctx.send_packet(None)
    }

    pub fn skip_frame(&mut self, discard: Discard) {
        self.ctx.skip_frame(discard);
    }

    pub fn flush(&mut self) {
        self.ctx.flush();
    }

    pub fn is_hw_initialized(&self) -> bool {
        self.hw_initialized.load(Ordering::Relaxed)
    }
}
