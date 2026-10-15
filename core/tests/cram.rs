// SPDX-License-Identifier: MIT OR Apache-2.0

mod common;

use common::{Event, Pin, Recorder};
use pico2_ice_core::cram::{Pins, load};

/// Starts with a valid header. The last three bytes read differently when their bits are reversed.
const PATTERN: &[u8] = &[
    0xFF, 0x00, 0x00, 0xFF, 0x7E, 0xAA, 0x99, 0x7E, 0x80, 0x01, 0xC5,
];

/// The bytes the FPGA receives, SI at each rising clock edge while CRESET is high and CS is low.
fn decode(rec: &Recorder) -> Vec<u8> {
    rec.rising(Pin::Sck)
        .iter()
        .filter(|s| s.after.creset && !s.after.cs)
        .map(|s| s.after.si)
        .collect::<Vec<bool>>()
        .chunks(8)
        .map(|b| b.iter().fold(0, |acc, &bit| (acc << 1) | u8::from(bit)))
        .collect()
}

/// Load `PATTERN` into fake pins and return the log.
fn run(half_bit_ns: u32) -> Recorder {
    let rec = Recorder::new();
    let mut creset = rec.output(Pin::Creset);
    let mut cs = rec.output(Pin::Cs);
    let mut sck = rec.output(Pin::Sck);
    let mut si = rec.output(Pin::Si);
    let mut cdone = rec.cdone_after(Some(1));
    let mut delay = rec.delay();

    let pins = Pins {
        creset: &mut creset,
        cs: &mut cs,
        sck: &mut sck,
        si: &mut si,
        cdone: &mut cdone,
    };
    load(pins, &mut delay, PATTERN, half_bit_ns).unwrap();
    rec
}

#[test]
fn creset_goes_low_before_any_other_pin_changes() {
    let rec = run(0);
    assert_eq!(rec.entries()[0].event, Event::Set(Pin::Creset, false));
}

#[test]
fn sck_goes_high_before_creset_rises() {
    let rec = run(0);
    let first_sck_rise = rec.rising(Pin::Sck)[0];
    assert!(!first_sck_rise.after.creset);
}

#[test]
fn cs_is_low_at_least_200_ns_before_creset_rises() {
    let rec = run(0);
    let gap = rec.time_between(Event::Set(Pin::Cs, false), Event::Set(Pin::Creset, true));
    assert!(gap >= Some(200));
}

#[test]
fn at_least_1200_us_pass_between_creset_rising_and_the_first_clock_edge() {
    let rec = run(0);
    let gap = rec.time_between(Event::Set(Pin::Creset, true), Event::Set(Pin::Sck, false));
    assert!(gap >= Some(1_200_000));
}

#[test]
fn exactly_8_clock_cycles_run_with_cs_high() {
    let rec = run(0);
    let dummy = rec.rising(Pin::Sck).iter().filter(|s| s.after.cs).count();
    assert_eq!(dummy, 8);
}

#[test]
fn decoding_each_rising_edge_gives_back_the_pattern() {
    let rec = run(0);
    assert_eq!(decode(&rec), PATTERN);
}

#[test]
fn decoding_works_with_a_nonzero_half_bit_delay() {
    let rec = run(500);
    assert_eq!(decode(&rec), PATTERN);
}

#[test]
fn si_changes_only_while_the_clock_is_low() {
    let rec = run(0);
    assert!(
        rec.steps()
            .iter()
            .filter(|s| matches!(s.event, Event::Set(Pin::Si, _)))
            .all(|s| !s.after.sck)
    );
}

#[test]
fn the_clock_is_high_when_the_load_ends() {
    let rec = run(0);
    assert!(rec.steps().last().unwrap().after.sck);
}
