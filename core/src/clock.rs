// SPDX-License-Identifier: MIT OR Apache-2.0

/// Where the clock for the FPGA comes from before the divider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockSource {
    /// The crystal oscillator on the board, 12 MHz crystal used directly by RP2350.
    Xosc,
    /// The PLL (phase locked loop) that feeds USB, normally 48 MHz.
    UsbPll,
    /// The PLL (phase locked loop) that feeds the CPU's system clock, normally 150 MHz for RP2350.
    /// SYS PLL can be modified by a user to make the CPU run at a different frequency (lower or higher).
    SysPll,
}

/// The clock source and divider that produce a requested FPGA clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockChoice {
    /// The clock to divide down from.
    pub source: ClockSource,
    /// Whole part of the divider.
    pub int: u16,
    /// Fraction of the divider in 65,536ths.
    pub frac: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockError {
    /// The request needs a divider larger than the hardware has. Includes 0 Hz.
    TooLow,
    /// The request is above every source that could serve it.
    TooHigh,
}

/// Picks the clock source and divider for a requested FPGA clock as ice_fpga_init does in the C SDK.
///
/// The divider is 16.16 fixed point, split into the `int` and `frac` that `Gpout::set_div`
/// takes. When the division is not exact the fraction is truncated, so the output is
/// slightly fast on average and its ticks are uneven.
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

    // Widen to u64 first since the shift would overflow a u32.
    let fixed = (u64::from(source_hz) << 16) / u64::from(hz);

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

    #[test]
    fn xosc_frequency_picks_xosc_undivided() {
        assert_eq!(
            choose(12_000_000, XOSC, USB_PLL, SYS_PLL),
            Ok(ClockChoice {
                source: ClockSource::Xosc,
                int: 1,
                frac: 0
            })
        );
    }

    #[test]
    fn below_xosc_picks_xosc_divided() {
        assert_eq!(
            choose(6_000_000, XOSC, USB_PLL, SYS_PLL),
            Ok(ClockChoice {
                source: ClockSource::Xosc,
                int: 2,
                frac: 0
            })
        );
    }

    #[test]
    fn one_hz_above_xosc_picks_usb_pll() {
        assert_eq!(
            choose(12_000_001, XOSC, USB_PLL, SYS_PLL),
            Ok(ClockChoice {
                source: ClockSource::UsbPll,
                int: 3,
                frac: 65535
            })
        );
    }

    #[test]
    fn one_hz_above_usb_pll_picks_sys_pll() {
        assert_eq!(
            choose(48_000_001, XOSC, USB_PLL, SYS_PLL),
            Ok(ClockChoice {
                source: ClockSource::SysPll,
                int: 3,
                frac: 8191
            })
        );
    }

    #[test]
    fn divider_splits_into_int_and_frac() {
        assert_eq!(
            choose(100_000_000, XOSC, USB_PLL, SYS_PLL),
            Ok(ClockChoice {
                source: ClockSource::SysPll,
                int: 1,
                frac: 32768
            })
        );
    }

    #[test]
    fn inexact_division_truncates() {
        assert_eq!(
            choose(13_000_000, XOSC, USB_PLL, SYS_PLL),
            Ok(ClockChoice {
                source: ClockSource::UsbPll,
                int: 3,
                frac: 45371
            })
        );
    }

    #[test]
    fn above_sys_pll_returns_too_high() {
        assert_eq!(
            choose(150_000_001, XOSC, USB_PLL, SYS_PLL),
            Err(ClockError::TooHigh)
        );
    }

    #[test]
    fn slow_sys_pll_does_not_block_usb_pll_requests() {
        assert_eq!(
            choose(40_000_000, XOSC, USB_PLL, 36_000_000),
            Ok(ClockChoice {
                source: ClockSource::UsbPll,
                int: 1,
                frac: 13107
            })
        );
    }

    #[test]
    fn divider_too_large_returns_too_low() {
        assert_eq!(choose(100, XOSC, USB_PLL, SYS_PLL), Err(ClockError::TooLow));
    }
}
