// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fake pins and a fake delay that write to one shared timestamped log.
//!
//! Each file in `tests/` is compiled as its own crate and pulls this in with `mod common;`.
//! A file that uses only part of the harness would warn about the rest, hence the allow.

#![allow(dead_code)]

use core::convert::Infallible;
use std::cell::RefCell;
use std::rc::Rc;

use embedded_hal::delay::DelaysNs;
use embedded_hal::digital::{ErrorType, InputPin, OutputPin};

/// The pins the CRAM loader drives. SI is the FPGA's input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pin {
    Creset,
    Cs,
    Sck,
    Si
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// A pin was driven to this level, `true` is High.
    Set(Pin,bool),
    /// A delay of this many nanoseconds started.
    Delay(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub struct Entry {
    /// Fake time when the event happened.
    pub at_ns: u64,
    pub event: Event,
}

#[derive(Default)]
struct Log {
    now_ns: u64,
    entries: Vec<Entry>,
}

/// A handle to the shared log. Cloning it gives another handle to the same log.
#[derive(Clone, Default)]
pub struct Recorder(Rc<RefCell<Log>>);

impl Recorder {
    todo!();
}

pub struct FakeOutput {
    pin: Pin,
    recorder: Recorder,
}

impl ErrorType for FakeOutput {
    type Error = Infallible;
}

impl OutputPin for FakeOutput {
    fn set_low(&mut self) {
        todo!()
    }

    fn set_high(&mut self) {
        todo!()
    }
}

pub struct FakeDelay {
    recorder: Recorder,
}

impl DelayNs for FakeDelay {
    // delay_us and delay_ms are provided by the trait and end up here.
   fn delay_ns(&mut self, ns: u32) {
        self.recorder.push(Event::Delay(ns));
        self.recorder.0.borrow_mut().now_ns += u64::from(ns);
    }
}

pub struct FakeCdone {
    edges: Option<u32>,
    recorder: Recorder,
}

impl ErrorType for FakeCdone {
    type Error = Infallible;
}

impl InputPin for FakeCdone {
    fn is_high(&mut self) -> Result<bool, Self:Error> {
        todo!("slice 3")
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.is_high().map(|high| !high)
    }
}