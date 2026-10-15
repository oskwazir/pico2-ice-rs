// SPDX-License-Identifier: MIT OR Apache-2.0

//! Load a bitstream into the FPGA's configuration RAM (CRAM) by bit-banging SPI.

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};

/// Duration CS stays low before CRESET rises.
/// Lattice asks for at least 200 ns. The reference C SDK waits 2 µs.
const CS_SETUP_NS: u32 = 2_000;

/// How long to wait after CRESET rises before the first clock.
/// Lattice asks for at least 1200 µs. The reference C SDK waits 1300 µs.
const CRESET_WAIT_US: u32 = 1_300;

/// [load] borrows these pins for the duration of one load.
pub struct Pins<'a, CDONE, CRESET, CS, SCK, SI> {
    pub cdone: &'a mut CDONE,
    pub creset: &'a mut CRESET,
    pub cs: &'a mut CS,
    pub sck: &'a mut SCK,
    pub si: &'a mut SI,
}

/// Why a load failed.
#[derive(Debug)]
pub enum Error<E> {
    /// The data does not start with the bitstream header. Returned from the upcoming slice 5 feat.
    NotABitstream,
    /// CDONE did not rise after the bitstream was sent. Returned from the upcoming slice 5 feat.
    CdoneTimeout,
    /// A pin call failed.
    Pin(E),
}

/// Reset the FPGA, put it in CRAM mode and send `bitstream` most significant bit first (MSB).
///
/// `half_bit_ns` is the delay in each half of a clock cycle. It can be zero too.
pub fn load<CDONE, CRESET, CS, SCK, SI, D, E>(
    pins: Pins<'_, CDONE, CRESET, CS, SCK, SI>,
    delay: &mut D,
    bitstream: &[u8],
    half_bit_ns: u32,
) -> Result<(), Error<E>>
where
    CDONE: InputPin<Error = E>,
    CRESET: OutputPin<Error = E>,
    CS: OutputPin<Error = E>,
    SCK: OutputPin<Error = E>,
    SI: OutputPin<Error = E>,
    D: DelayNs,
{
    let Pins {
        cdone,
        creset,
        cs,
        sck,
        si,
    } = pins;
    let _ = cdone;

    creset.set_low().map_err(Error::Pin)?;
    sck.set_high().map_err(Error::Pin)?;
    cs.set_low().map_err(Error::Pin)?;
    delay.delay_ns(CS_SETUP_NS);
    creset.set_high().map_err(Error::Pin)?;
    delay.delay_us(CRESET_WAIT_US);
    // Lattice wants 8 clock cycles with CS high. The reference C SDK sends one zero byte for this.
    cs.set_high().map_err(Error::Pin)?;
    send_byte(sck, si, delay, half_bit_ns, 0x00)?;
    cs.set_low().map_err(Error::Pin)?;

    for &byte in bitstream {
        send_byte(sck, si, delay, half_bit_ns, byte)?;
    }
    Ok(())
}

/// Sends one byte with the most significant bit first.
/// SI changes while SCK is low. The FPGA reads it when SCK rises.
fn send_byte<SCK, SI, D, E>(
    sck: &mut SCK,
    si: &mut SI,
    delay: &mut D,
    half_bit_ns: u32,
    byte: u8,
) -> Result<(), Error<E>>
where
    SCK: OutputPin<Error = E>,
    SI: OutputPin<Error = E>,
    D: DelayNs,
{
    for i in 0..8 {
        let bit = byte & (0x80 >> i) != 0;
        sck.set_low().map_err(Error::Pin)?;
        si.set_state(bit.into()).map_err(Error::Pin)?;
        delay.delay_ns(half_bit_ns);
        sck.set_high().map_err(Error::Pin)?;
        delay.delay_ns(half_bit_ns);
    }
    Ok(())
}
