// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use ffmsys::{AVRational, av_rescale_q};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational {
    pub numerator: i32,
    pub denominator: i32,
}

pub trait Rescale {
    fn rescale<S: Into<Rational>, D: Into<Rational>>(&self, source: S, destination: D) -> i64;
}

impl Rational {
    pub const fn new(numerator: i32, denominator: i32) -> Self {
        Self {
            numerator,
            denominator,
        }
    }
}

impl From<AVRational> for Rational {
    fn from(value: AVRational) -> Self {
        Self::new(value.num, value.den)
    }
}

impl From<(i32, i32)> for Rational {
    fn from((numerator, denominator): (i32, i32)) -> Self {
        Self::new(numerator, denominator)
    }
}

impl From<Rational> for AVRational {
    fn from(value: Rational) -> Self {
        Self {
            num: value.numerator,
            den: value.denominator,
        }
    }
}

impl<T: Into<i64> + Clone> Rescale for T {
    fn rescale<S: Into<Rational>, D: Into<Rational>>(&self, source: S, destination: D) -> i64 {
        unsafe {
            av_rescale_q(
                self.clone().into(),
                source.into().into(),
                destination.into().into(),
            )
        }
    }
}
