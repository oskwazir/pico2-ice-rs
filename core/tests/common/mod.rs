// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fake pins and a fake delay that write to one shared timestamped log.
//!
//! Each file in `tests/` is compiled as its own crate and pulls this in with `mod common;`.
//! A file that uses only part of the harness would warn about the rest, hence the allow.

#![allow(dead_code)]

use core::convert::Infallible;
use std::cell::RefCell;
use std::rc::Rc;

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{ErrorType, InputPin, OutputPin};

/// The pins the CRAM loader drives. SI is the FPGA's input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pin {
    Creset,
    Cs,
    Sck,
    Si,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// A pin was driven to this level, `true` is High.
    Set(Pin, bool),
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn output(&self, pin: Pin) -> FakeOutput {
        FakeOutput {
            pin,
            recorder: self.clone(),
        }
    }

    pub fn delay(&self) -> FakeDelay {
        FakeDelay {
            recorder: self.clone(),
        }
    }

    /// A CDONE that goes high once SCK has had `edges` rising edges, or never for `None`
    pub fn cdone_after(&self, edges: Option<u32>) -> FakeCdone {
        FakeCdone {
            edges,
            recorder: self.clone(),
        }
    }

    pub fn entries(&self) -> Vec<Entry> {
        self.0.borrow().entries.clone()
    }

    /// Fake time of the first entry equal to `event`.
    pub fn time_of(&self, event: Event) -> Option<u64> {
        self.0
            .borrow()
            .entries
            .iter()
            .find(|e| e.event == event)
            .map(|e| e.at_ns)
    }

    /// Fake time from the first `from` to the first `to` after it.
    pub fn time_between(&self, from: Event, to: Event) -> Option<u64> {
        todo!("slice 3")
    }

    pub fn rising_edges(&self, pin: Pin) -> u32 {
        todo!("slice 3")
    }

    fn push(&self, event: Event) {
        let mut log = self.0.borrow_mut();
        let at_ns = log.now_ns;
        log.entries.push(Entry { at_ns, event });
    }
}

pub struct FakeOutput {
    pin: Pin,
    recorder: Recorder,
}

impl ErrorType for FakeOutput {
    type Error = Infallible;
}

impl OutputPin for FakeOutput {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.recorder.push(Event::Set(self.pin, false));
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.recorder.push(Event::Set(self.pin, true));
        Ok(())
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
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        todo!("slice 3")
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.is_high().map(|high| !high)
    }
}
