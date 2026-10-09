// SPDX-License-Identifier: MIT OR Apache-2.0

//! Blinks the pico2-ice's red RP2350 LED on GPIO 1.
//!
//! The LED is active-low: driving the pin low turns it on.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_time::Timer;
use gpio::{Level, Output};
use pico2_ice_core::bitstream;
use {defmt_rtt as _, panic_probe as _};

// Program metadata for `picotool info`.
// This isn't needed, but it's recommended to have these minimal entries.
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"pico2-ice rp_blinky"),
    embassy_rp::binary_info::rp_program_description!(c"Blinks the red RP2350 LED on GPIO 1"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_1, Level::High);
    let header: [u8; 8] = [0xFF, 0x00, 0x00, 0xFF, 0x7E, 0xAA, 0x99, 0x7E];

    defmt::info!("header ok: {}", bitstream::check(&header).is_ok());

    loop {
        info!("led on!");
        led.set_low();
        Timer::after_millis(250).await;

        info!("led off!");
        led.set_high();
        Timer::after_millis(250).await;
    }
}
