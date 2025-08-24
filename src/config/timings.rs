/// # MinCycleTime Configuration Module
///
/// This module provides constants and functions for configuring the Device's
/// MinCycleTime parameter as specified in IO-Link Specification v1.1.4, Section B.1.3.
/// 
/// The MinCycleTime parameter informs the IO-Link Master about the shortest cycle time
/// supported by the Device. The encoding and valid ranges are defined in Table B.3 and
/// Figure B.2 of the IO-Link specification (see image above).
///
/// ## MinCycleTime Encoding (Figure B.2)
/// - **Bits 0..=5**: Multiplier (0..=63)
/// - **Bits 6..=7**: Time Base (0b00 = 0.1ms, 0b01 = 0.4ms, 0b10 = 1.6ms, 0b11 = Reserved)
///
/// ## Valid Ranges (Table B.3)
/// | Time Base | Multiplier Range | Cycle Time Range (ms) |
/// |-----------|------------------|----------------------|
/// | 0 (0.1ms) | 4..=63           | 0.4 ..= 6.3          |
/// | 1 (0.4ms) | 0..=63           | 6.4 ..= 31.6         |
/// | 2 (1.6ms) | 0..=63           | 32.0 ..= 132.8       |
///
/// The Device must not indicate a MinCycleTime shorter than the calculated M-sequence time.
///

pub mod min_cycle_time {
    /// Returns the configured minimum cycle time in milliseconds.
    ///
    /// # Specification Reference
    /// - IO-Link v1.1.4, Section B.1.3, Table B.3
    ///
    /// # Valid Ranges
    /// - 0.4 ..= 6.3 ms (Time Base 0, Multiplier 4..=63)
    /// - 6.4 ..= 31.6 ms (Time Base 1, Multiplier 0..=63)
    /// - 32.0 ..= 132.8 ms (Time Base 2, Multiplier 0..=63)
    ///
    /// # Panics
    /// Panics if the configured value is outside the valid ranges.
    const fn time_in_ms() -> f32 {
        let time_in_ms = /*CONFIG:MIN_CYCLE_TIME_IN_MS*/ 33f32 /*ENDCONFIG*/;
        let valid = 
            (time_in_ms >= 0.4 && time_in_ms <= 6.3) ||
            (time_in_ms >= 6.4 && time_in_ms <= 31.6) ||
            (time_in_ms >= 32.0 && time_in_ms <= 132.8);
        if !valid {
            panic!("Invalid min cycle time configuration in ms. Valid ranges: 0.4–6.3, 6.4–31.6, 32.0–132.8 ms (see IO-Link Spec Table B.3)");
        }
        time_in_ms
    }

    /// Returns the encoded time base for the configured MinCycleTime.
    ///
    /// # Returns
    /// - `0b00` for 0.1 ms base (0.4–6.3 ms)
    /// - `0b01` for 0.4 ms base (6.4–31.6 ms)
    /// - `0b10` for 1.6 ms base (32.0–132.8 ms)
    ///
    /// # Panics
    /// Panics if the configured value is outside the valid ranges.
    const fn time_base() -> u8 {
        match time_in_ms() {
            (0.4..=6.3) => 0b00u8,
            (6.4..=31.6) => 0b01u8,
            (32.0..=132.8) => 0b10u8,
            _ => panic!("Invalid min cycle time configuration in ms. Valid ranges: 0.4–6.3, 6.4–31.6, 32.0–132.8 ms (see IO-Link Spec Table B.3)"),
        }
    }

    /// Returns the encoded multiplier for the configured MinCycleTime.
    ///
    /// # Returns
    /// - For 0.4–6.3 ms: `multiplier = round(time_in_ms / 0.1)`
    const fn multiplier() -> u8 {
        let multiplier = match time_in_ms() {
            t if t >= 0.4 && t <= 6.3 => {
                let m = (t / 0.1 + 0.5) as u8;
                if m > 63 {
                    panic!("Multiplier for min cycle time does not fit in 6 bits (0..=63)");
                }
                m
            },
            t if t >= 6.4 && t <= 31.6 => {
                let m = ((t - 6.4) / 0.4 + 0.5) as u8;
                if m > 63 {
                    panic!("Multiplier for min cycle time does not fit in 6 bits (0..=63)");
                }
                m
            },
            t if t >= 32.0 && t <= 132.8 => {
                let m = ((t - 32.0) / 1.6 + 0.5) as u8;
                if m > 63 {
                    panic!("Multiplier for min cycle time does not fit in 6 bits (0..=63)");
                }
                m
            },
            _ => panic!("Invalid min cycle time configuration in ms. Valid ranges: 0.4–6.3, 6.4–31.6, 32.0–132.8 ms (see IO-Link Spec Table B.3)"),
        };
        multiplier
    }

    pub const fn min_cycle_time_parameter() -> crate::utils::page_params::page1::CycleTime {
        const TIME_BASE: u8 = time_base();
        const MULTIPLIER: u8 = multiplier();
        let mut cycle_time = crate::utils::page_params::page1::CycleTime::new();
        cycle_time.set_time_base(TIME_BASE);
        cycle_time.set_multiplier(MULTIPLIER);
        cycle_time
    }
}