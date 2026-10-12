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
