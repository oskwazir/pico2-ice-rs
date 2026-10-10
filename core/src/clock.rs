// SPDX-License-Identifier: MIT OR Apache-2.0

/// Where the clock for the FPGA comes from before the divider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockSource {
    /// The crystal oscillator on the board, 12 Mhz crystal used directly by RP2350.
    Xosc,
    /// The PLL (phase locked loop) that feeds USB, normally 48 Mhz.
    UsbPll,
    /// The PLL (phase locked loop) that feeds the CPU's system clock, normally 150 Mhz for RP2350.
    /// SYS PLL can be modified by a user to make the CPU run at a different frequency (lower or higher).
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

pub fn choose(
    hz: u32,
    xosc_hz: u32,
    usb_pll_hz: u32,
    sys_pll_hz: u32,
) -> Result<ClockChoice, ClockError> {
    // if hz is 0 then return an Err letting caller know this is an error
    if hz == 0 {
        return Err(ClockError::TooLow);
    }

    let (source, source_hz) = if hz > usb_pll_hz {
        (ClockSource::SysPll, sys_pll_hz)
    } else if hz <= xosc_hz {
        (ClockSource::Xosc, xosc_hz)
    } else {
        (ClockSource::UsbPll, usb_pll_hz)
    };

    // if hz is higher than the available source hz let the caller know this is an error
    if hz > source_hz {
        return Err(ClockError::TooHigh);
    }

    // u64::from widens without a loss since the shift would overflow a u32.
    // fixed >> 16 is the whole part and fixed & 0xFFFF is the fraction part.
    let fixed = (u64::from(source_hz) << 16) / u64::from(hz);

    // try_from returns a Result and map_err just replaces the error with ClockError
    let int = u16::try_from(fixed >> 16).map_err(|_| ClockError::TooLow)?;
    // The mask keeps the low 16 bits, so this does not need try_from.map_err as above did
    let frac = (fixed & 0xFFFF) as u16;

    Ok(ClockChoice { source, int, frac })
}

#[cfg(test)]
mod tests {
    use super::*;

    const XOSC: u32 = 12_000_000;
    const USB_PLL: u32 = 48_000_000;
    const SYS_PLL: u32 = 150_000_000;

    #[test]
    // validates that requested frequency is too low
    fn frequency_of_zero_returns_too_low() {
        assert_eq!(choose(0, XOSC, USB_PLL, SYS_PLL), Err(ClockError::TooLow));
    }
    #[test]
    fn usb_pll_frequency_picks_usb_pll_undivided() {
        assert_eq!(
            choose(48_000_000, XOSC, USB_PLL, SYS_PLL),
            Ok(ClockChoice {
                source: ClockSource::UsbPll,
                int: 1,
                frac: 0
            })
        );
    }
}
