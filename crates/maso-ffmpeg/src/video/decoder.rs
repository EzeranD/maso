// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::mem;

use snafu::Snafu;

use crate::{
    codec::{packet::Packet, parameters::Parameters},
    util::{
        Code,
        frame::{Frame, Video},
    },
    video::{
        context::{self, Context, Discard, SendPacketError},
        hardware::Candidate,
    },
};

#[derive(Debug, Snafu)]
pub enum ReceiveFrameError {
    #[snafu(display("more packets are required to receive a frame"))]
    NeedInput { source: Code },
    #[snafu(display("decoder has no more frames"))]
    Drained { source: Code },
    #[snafu(display("failed to receive frame"))]
    Other { source: Code },
    #[snafu(display("failed to replay buffered packet"))]
    ReplayPacket { source: SendPacketError },
    #[snafu(context(false), display("failed to initialize fallback decoder"))]
    Fallback { source: context::Error },
}

pub struct Decoder {
    context: Context,
    parameters: Parameters,
    discard: Option<Discard>,
    mode: Mode,
    phase: Phase,
}

enum Mode {
    Hardware { candidates: Vec<Candidate> },
    Software,
}

enum Phase {
    Probing(Probe),
    Replaying(Replay),
    Normal,
}

#[derive(Default)]
struct Probe {
    packets: Vec<Packet>,
    next: usize,
}

struct Replay {
    packets: Vec<Packet>,
    next: usize,
}

enum ProbeOutcome {
    Ready,
    Fallback,
}

fn select_context(
    parameters: &Parameters,
    mut candidates: Vec<Candidate>,
) -> Result<(Context, Mode), context::Error> {
    while let Some(candidate) = candidates.pop() {
        if let Ok(context) = Context::new(parameters, Some(&candidate)) {
            return Ok((context, Mode::Hardware { candidates }));
        }
    }

    let context = Context::new(parameters, None)?;
    Ok((context, Mode::Software))
}

impl Decoder {
    pub fn new(parameters: Parameters, hardware: bool) -> Result<Self, context::Error> {
        let candidates = if hardware {
            Candidate::from_parameters(&parameters)
        } else {
            Vec::new()
        };

        let (context, mode) = select_context(&parameters, candidates)?;

        let phase = match mode {
            Mode::Hardware { .. } => Phase::Probing(Probe::default()),
            Mode::Software => Phase::Normal,
        };

        Ok(Self {
            context,
            parameters,
            discard: None,
            mode,
            phase,
        })
    }

    pub fn send_packet(&mut self, packet: &Packet) -> Result<(), SendPacketError> {
        if self.phase.pending().is_some() {
            return Err(SendPacketError::NeedOutput {
                source: Code::AGAIN,
            });
        }

        self.context.send_packet(packet)?;

        if let Phase::Probing(probe) = &mut self.phase {
            probe.packets.push(packet.clone());
            probe.next += 1;
        }

        Ok(())
    }

    pub fn receive_frame(&mut self, frame: &mut Frame<Video>) -> Result<(), ReceiveFrameError> {
        if matches!(self.phase, Phase::Normal) {
            self.context.receive_frame(frame)?;
            return Ok(());
        }

        self.receive_pending(frame)
    }

    pub fn skip_frame(&mut self, discard: Discard) {
        self.discard = Some(discard);
        self.context.skip_frame(discard);
    }

    pub fn flush(&mut self) {
        match &mut self.phase {
            Phase::Probing(probe) => {
                probe.packets.clear();
                probe.next = 0;
            }
            Phase::Replaying(_) => self.phase = Phase::Normal,
            Phase::Normal => {}
        }

        self.context.flush();
    }

    fn receive_pending(&mut self, frame: &mut Frame<Video>) -> Result<(), ReceiveFrameError> {
        loop {
            match &mut self.phase {
                Phase::Probing(probe) => match probe.receive(&mut self.context, frame)? {
                    ProbeOutcome::Ready => {
                        let packets = mem::take(&mut probe.packets);
                        let next = probe.next;

                        self.phase = if next < packets.len() {
                            Phase::Replaying(Replay { packets, next })
                        } else {
                            Phase::Normal
                        };

                        return Ok(());
                    }
                    ProbeOutcome::Fallback => self.fallback()?,
                },
                Phase::Replaying(replay) => {
                    let res = replay.receive(&mut self.context, frame);

                    if replay.next >= replay.packets.len() {
                        self.phase = Phase::Normal;
                    }

                    return res;
                }
                Phase::Normal => {
                    self.context.receive_frame(frame)?;
                    return Ok(());
                }
            }
        }
    }

    fn fallback(&mut self) -> Result<(), context::Error> {
        let Mode::Hardware { candidates } = &mut self.mode else {
            return Ok(());
        };
        let Phase::Probing(probe) = &mut self.phase else {
            return Ok(());
        };

        let (mut context, mode) = select_context(&self.parameters, mem::take(candidates))?;

        if let Some(discard) = self.discard {
            context.skip_frame(discard);
        }

        let packets = mem::take(&mut probe.packets);
        self.phase = match mode {
            Mode::Hardware { .. } => Phase::Probing(Probe { packets, next: 0 }),
            Mode::Software if !packets.is_empty() => Phase::Replaying(Replay { packets, next: 0 }),
            Mode::Software => Phase::Normal,
        };

        self.context = context;
        self.mode = mode;

        Ok(())
    }
}

impl Phase {
    fn pending(&self) -> Option<&Packet> {
        match self {
            Self::Probing(probe) => probe.packets.get(probe.next),
            Self::Replaying(replay) => replay.packets.get(replay.next),
            Self::Normal => None,
        }
    }
}

impl Probe {
    fn receive(
        &mut self,
        context: &mut Context,
        frame: &mut Frame<Video>,
    ) -> Result<ProbeOutcome, ReceiveFrameError> {
        loop {
            match context.receive_frame(frame) {
                Ok(()) if context.is_hw_initialized() => return Ok(ProbeOutcome::Ready),
                Ok(()) => return Ok(ProbeOutcome::Fallback),
                Err(e) => match e {
                    context::ReceiveFrameError::NeedInput { .. } => {
                        let Some(packet) = self.packets.get(self.next) else {
                            return Err(e.into());
                        };

                        let Err(e) = context.send_packet(packet) else {
                            self.next += 1;
                            continue;
                        };

                        match e {
                            SendPacketError::Other { .. } => return Ok(ProbeOutcome::Fallback),
                            SendPacketError::NeedOutput { .. }
                            | SendPacketError::Flushed { .. } => {
                                return Err(e.into());
                            }
                        }
                    }
                    context::ReceiveFrameError::Drained { .. } => return Err(e.into()),
                    context::ReceiveFrameError::Other { .. } => return Ok(ProbeOutcome::Fallback),
                },
            }
        }
    }
}

impl Replay {
    fn receive(
        &mut self,
        context: &mut Context,
        frame: &mut Frame<Video>,
    ) -> Result<(), ReceiveFrameError> {
        loop {
            let Err(e) = context.receive_frame(frame) else {
                return Ok(());
            };

            match e {
                context::ReceiveFrameError::NeedInput { .. } => {
                    let Some(packet) = self.packets.get(self.next) else {
                        return Err(e.into());
                    };

                    context.send_packet(packet)?;
                    self.next += 1;
                }
                context::ReceiveFrameError::Drained { .. }
                | context::ReceiveFrameError::Other { .. } => {
                    return Err(e.into());
                }
            }
        }
    }
}

impl From<context::ReceiveFrameError> for ReceiveFrameError {
    fn from(error: context::ReceiveFrameError) -> Self {
        match error {
            context::ReceiveFrameError::NeedInput { source } => Self::NeedInput { source },
            context::ReceiveFrameError::Drained { source } => Self::Drained { source },
            context::ReceiveFrameError::Other { source } => Self::Other { source },
        }
    }
}

impl From<SendPacketError> for ReceiveFrameError {
    fn from(error: SendPacketError) -> Self {
        Self::ReplayPacket { source: error }
    }
}
