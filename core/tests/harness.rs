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
    );
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
        Some(2_000)
    );
}

#[test]
fn cdone_rises_after_n_rising_clock_edges() {
    let rec = Recorder::new();
    let mut sck = rec.output(Pin::Sck);
    let mut cdone = rec.cdone_after(Some(3));

    sck.set_high().unwrap(); // idle level - edge 1
    for _ in 0..2 {
        assert!(cdone.is_low().unwrap());
        sck.set_low().unwrap();
        sck.set_high().unwrap();
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

#[test]
fn rising_edges_ignores_a_repeated_set_high() {
    let rec = Recorder::new();
    let mut sck = rec.output(Pin::Sck);

    sck.set_high().unwrap(); // edge 1
    sck.set_high().unwrap(); // already high, not an edge
    sck.set_low().unwrap();
    sck.set_high().unwrap(); // edge 2

    assert_eq!(rec.rising_edges(Pin::Sck), 2);
}

#[test]
fn time_between_the_same_event_measures_to_the_next_one() {
    let rec = Recorder::new();
    let mut sck = rec.output(Pin::Sck);
    let mut delay = rec.delay();

    sck.set_high().unwrap();
    delay.delay_ns(500);
    sck.set_low().unwrap();
    delay.delay_ns(500);
    sck.set_high().unwrap();

    let rise = Event::Set(Pin::Sck, true);
    assert_eq!(rec.time_between(rise, rise), Some(1_000));
}

#[test]
fn time_between_is_none_when_an_event_is_missing() {
    let rec = Recorder::new();
    let mut cs = rec.output(Pin::Cs);

    cs.set_low().unwrap();

    assert_eq!(
        rec.time_between(Event::Set(Pin::Cs, false), Event::Set(Pin::Creset, true)),
        None
    );
}
