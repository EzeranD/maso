// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::marker::PhantomData;

use crate::{
    codec::{Codec, context::inner::ReceiveFrameFlags, packet::Packet, parameters::Parameters},
    util::{Extensions, frame::Frame, hardware::DeviceContext, pixel::PixelFormat},
};

mod error;
mod get_format;
mod inner;

pub use error::{Error, ReceiveFrameError, ReceivePacketError, SendFrameError, SendPacketError};
use ffmsys::AVCodecContext;
pub use get_format::{GetFormatContext, GetFormatFn};
use inner::Inner;

pub struct Context<State = Unopened> {
    inner: Inner,
    _state: PhantomData<State>,
}

pub struct Unopened;
pub struct Opened;

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

impl Context<Unopened> {
    pub fn new(codec: Option<&Codec>) -> Result<Self, Error> {
        Ok(Self {
            inner: Inner::new(codec)?,
            _state: PhantomData,
        })
    }

    pub fn from_parameters(parameters: &Parameters, codec: Option<&Codec>) -> Result<Self, Error> {
        Ok(Self {
            inner: Inner::from_parameters(parameters, codec)?,
            _state: PhantomData,
        })
    }

    pub fn open(mut self, codec: Option<&Codec>) -> Result<Context<Opened>, Error> {
        self.inner.open(codec)?;
        Ok(Context {
            inner: self.inner,
            _state: PhantomData,
        })
    }

    pub fn set_device_ctx(&mut self, device: DeviceContext) {
        self.inner.set_device_ctx(device);
    }

    pub fn get_format<F>(&mut self, f: F)
    where
        F: Fn(&mut GetFormatContext<'_>, &[PixelFormat]) -> PixelFormat + Send + Sync + 'static,
    {
        self.inner.get_format(f);
    }

    pub fn extensions_mut(&mut self) -> &mut Extensions {
        self.inner.extensions_mut()
    }
}

impl Context<Opened> {
    pub fn send_packet(&mut self, packet: Option<&Packet>) -> Result<(), SendPacketError> {
        self.inner.send_packet(packet)
    }

    pub fn send_frame<Kind>(&mut self, frame: Option<&Frame<Kind>>) -> Result<(), SendFrameError> {
        self.inner.send_frame(frame)
    }

    pub fn receive_packet(&mut self, packet: &mut Packet) -> Result<(), ReceivePacketError> {
        self.inner.receive_packet(packet)
    }

    pub fn receive_frame<Kind>(
        &mut self,
        frame: &mut Frame<Kind>,
        flags: Option<ReceiveFrameFlags>,
    ) -> Result<(), ReceiveFrameError> {
        self.inner.receive_frame(frame, flags)
    }

    pub fn flush(&mut self) {
        self.inner.flush()
    }
}

impl<State> Context<State> {
    pub fn codec(&self) -> Option<Codec> {
        self.inner.codec()
    }

    pub fn skip_frame(&mut self, discard: Discard) {
        self.inner.skip_frame(discard)
    }
}

impl<State> Context<State> {
    pub fn device_ctx(&self) -> Option<DeviceContext> {
        self.inner.device_ctx()
    }

    pub fn extensions(&self) -> &Extensions {
        self.inner.extensions()
    }

    pub fn as_ptr(&self) -> *const AVCodecContext {
        self.inner.as_ptr()
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut AVCodecContext {
        self.inner.as_mut_ptr()
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
