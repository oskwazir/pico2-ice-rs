// SPDX-License-Identifier: MIT OR Apache-2.0

mod common;

use common::{Entry, Event, Pin, Recorder};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};

#[test]
fn two_pins_around_a_delay_are_logged_in_order_with_times() {
    let rec = Recorder::new();
    let mut cs = rec.output(Pin::Cs);
    let mut creset = rec.output(Pin::Creset);
    let mut delay = rec.delay();

    cs.set_low().unwrap();
    delay.delay_us(2);
    creset.set_high().unwrap();

    assert_eq!(
        rec.entries(),
        [
            Entry {
                at_ns: 0,
                event: Event::Set(Pin::Cs, false)
            },
            Entry {
                at_ns: 0,
                event: Event::Delay(2_000)
            },
            Entry {
                at_ns: 2_000,
                event: Event::Set(Pin::Creset, true)
            },
        ]
    )
}

#[test]
fn time_between_two_events() {
    let rec = Recorder::new();
    let mut cs = rec.output(Pin::Cs);
    let mut creset = rec.output(Pin::Creset);
    let mut delay = rec.delay();

    cs.set_low().unwrap();
    delay.delay_us(2);
    creset.set_high().unwrap();

    assert_eq!(
        rec.time_between(Event::Set(Pin::Cs, false), Event::Set(Pin::Creset, true)),
        Some(2000)
    );
}

#[test]
fn cdone_rises_after_n_rising_clock_edges() {
    let rec = Recorder::new();
    let mut sck = rec.output(Pin::Sck);
    let mut cdone = rec.cdone_after(Some(3));

    sck.set_high().unwrap(); // idel level - edge 1
    for _ in 0..2 {
        assert!(cdone.is_low().unwrap());
        sck.set_low().unwrap();
        sck.set_high().unwrap()
    }

    assert!(cdone.is_high().unwrap());
}

#[test]
fn cdone_set_to_never_stays_low() {
    let rec = Recorder::new();
    let mut sck = rec.output(Pin::Sck);
    let mut cdone = rec.cdone_after(None);

    for _ in 0..200 {
        sck.set_low().unwrap();
        sck.set_high().unwrap();
    }

    assert!(cdone.is_low().unwrap());
}
