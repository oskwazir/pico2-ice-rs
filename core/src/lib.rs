// SPDX-License-Identifier: MIT OR Apache-2.0

#![no_std]
#![forbid(unsafe_code)]
//! Hardware-independent logic for the pico2-ice crate; no_std, testable on the host.

pub mod bitstream;
pub mod clock;
pub mod cram;
