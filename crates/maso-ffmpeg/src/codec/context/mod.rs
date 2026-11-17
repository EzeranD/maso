// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::marker::PhantomData;

use crate::{
    codec::{Codec, parameters::Parameters},
    util::{
        Extensions,
        hardware::DeviceContext,
        media::{Kind, Unknown},
    },
};

mod decoder;
mod encoder;
mod error;
mod get_format;
mod inner;

pub use decoder::Discard;
pub use error::{Error, ReceiveFrameError, ReceivePacketError, SendFrameError, SendPacketError};
use ffmsys::AVCodecContext;
pub use get_format::{GetFormatContext, GetFormatFn};
use inner::Inner;
pub use inner::ReceiveFrameFlags;

pub use super::{Decoder, Encoder, Mode};

pub struct Context<State = Unopened, M: Mode = Decoder, K: Kind = Unknown> {
    inner: Inner,
    _state: PhantomData<(State, M, K)>,
}

pub struct Unopened;
pub struct Opened;

impl<M: Mode, K: Kind> Context<Unopened, M, K> {
    pub fn new(codec: Option<&Codec<M, K>>) -> Result<Self, Error> {
        Ok(Self {
            inner: Inner::new(codec)?,
            _state: PhantomData,
        })
    }

    pub fn from_parameters(
        parameters: &Parameters<K>,
        codec: Option<&Codec<M, K>>,
    ) -> Result<Self, Error> {
        Ok(Self {
            inner: Inner::from_parameters(parameters, codec)?,
            _state: PhantomData,
        })
    }

    pub fn open(mut self, codec: Option<&Codec<M, K>>) -> Result<Context<Opened, M, K>, Error> {
        self.inner.open(codec)?;
        Ok(Context {
            inner: self.inner,
            _state: PhantomData,
        })
    }

    pub fn set_device_ctx(&mut self, device: DeviceContext) {
        self.inner.set_device_ctx(device);
    }

    pub fn extensions_mut(&mut self) -> &mut Extensions {
        self.inner.extensions_mut()
    }
}

impl<M: Mode, K: Kind> Context<Opened, M, K> {
    pub fn flush(&mut self) {
        self.inner.flush();
    }
}

impl<State, M: Mode, K: Kind> Context<State, M, K> {
    pub fn codec(&self) -> Option<Codec<M, K>> {
        self.inner.codec()
    }

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
