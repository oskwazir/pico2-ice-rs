// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fake pins and a fake delay that write to one shared timestamped log.
//!
//! Code under test takes its pins and delay as `embedded-hal` traits. A test passes it
//! these fakes, then checks the log for what was driven, in what order and how far
//! apart.
//! All are made from one `Recorder` and write to its log.
//!
//! - `output(pin)`: an `OutputPin`
//! - `delay()`: a `DelayNs` that advances the fake clock and returns at once
//! - `cdone_after(Some(n))`: an `InputPin` that is high after `n` rising SCK edges
//! - `cdone_after(None)`: an `InputPin` that stays low
//!
//! # Example
//!
//! ```ignore
//! let rec = Recorder::new();
//! let mut cs = rec.output(Pin::Cs);
//! let mut creset = rec.output(Pin::Creset);
//! let mut delay = rec.delay();
//!
//! cs.set_low().unwrap();
//! delay.delay_us(2);
//! creset.set_high().unwrap();
//!
//! let gap = rec.time_between(Event::Set(Pin::Cs, false), Event::Set(Pin::Creset, true));
//! assert!(gap >= Some(200));
//! ```
//!
//! Queries: `entries`, `time_of`, `time_between`, `rising`, `rising_edges`, `steps`.
//!
//! # Behaviour
//!
//! - Times are in nanoseconds and change only when the fake delay is called.
//! - Pins are assumed to start low, so a first `set_high` counts as a rising edge.
//! - Every `set_low` and `set_high` is logged, including one that repeats the level.
//! - Reads of CDONE are not logged.
//! - `delay_us` and `delay_ms` above about 4.29 s are logged as more than one delay.
//!
//! Each file in `tests/` is its own crate. One that uses part of the harness would
//! warn about the rest without the `allow` below.

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

/// Pin levels at a point in time. `true` is High.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Levels {
    pub creset: bool,
    pub cs: bool,
    pub sck: bool,
    pub si: bool,
}

impl Levels {
    pub fn get(&self, pin: Pin) -> bool {
        match pin {
            Pin::Creset => self.creset,
            Pin::Cs => self.cs,
            Pin::Sck => self.sck,
            Pin::Si => self.si,
        }
    }

    pub fn set(&mut self, pin: Pin, level: bool) {
        match pin {
            Pin::Creset => self.creset = level,
            Pin::Cs => self.cs = level,
            Pin::Sck => self.sck = level,
            Pin::Si => self.si = level,
        }
    }
}

/// A log entry with pin levels immediately before and after the event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    /// Simulated timestamp (in nanoseconds) when the event occurred, copied from the log
    pub at_ns: u64,
    /// The recorded action (a pin transition or delay start), copied from the log.
    pub event: Event,
    /// State of all four pins (`CRESET`, `CS`, `SCK`, `SI`) immediately before the event.
    pub before: Levels,
    /// State of all four pins immediately after the event.
    ///
    /// Identical to `before` when `event` is a delay.
    pub after: Levels,
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

    /// A CDONE which goes high once SCK has had `edges` rising edges, or never for `None`.
    /// It counts every rising SCK edge since the log began, including the loader's initial `set_high`
    /// to idle.
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
        let log = self.0.borrow();
        let start = log.entries.iter().position(|e| e.event == from)?;
        let end = log.entries[start + 1..].iter().find(|e| e.event == to)?;
        Some(end.at_ns - log.entries[start].at_ns)
    }

    /// The log replayed with pin levels before and after each event.
    pub fn steps(&self) -> Vec<Step> {
        let mut levels = Levels::default();
        self.0
            .borrow()
            .entries
            .iter()
            .map(|e| {
                let before = levels;
                if let Event::Set(pin, level) = e.event {
                    levels.set(pin, level);
                }
                Step {
                    at_ns: e.at_ns,
                    event: e.event,
                    before,
                    after: levels,
                }
            })
            .collect()
    }

    /// All steps containing a rising edge on `pin`.
    pub fn rising(&self, pin: Pin) -> Vec<Step> {
        self.steps()
            .into_iter()
            .filter(|s| !s.before.get(pin) && s.after.get(pin))
            .collect()
    }

    /// How many times `pin` went from low to high.
    /// A repeated `set_high` or high to low is not a rising edge.
    pub fn rising_edges(&self, pin: Pin) -> u32 {
        self.rising(pin).len() as u32
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
        let high = match self.edges {
            Some(n) => self.recorder.rising_edges(Pin::Sck) >= n,
            None => false,
        };
        Ok(high)
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.is_high().map(|high| !high)
    }
}
