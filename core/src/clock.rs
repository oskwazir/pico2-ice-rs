// SPDX-License-Identifier: MIT OR Apache-2.0

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockSource {
    Xosc,
    UsbPll,
    SysPll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockChoice {
    pub source: ClockSource,
    pub int: u16,
    pub frac: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockError {
    /// The request needs a divider larger than the hardware has. Includes 0 Hz.
    TooLow,
    /// The request is above the SYS PLL frequency.
    TooHigh,
}
