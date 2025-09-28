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
    use iolinke_dev_config::device as dev_config;

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
        match dev_config::timings::time_in_ms() {
            (0.4..=6.3) => 0b00u8,
            (6.4..=31.6) => 0b01u8,
            (32.0..=132.8) => 0b10u8,
            _ => panic!(
                "Invalid min cycle time configuration in ms. Valid ranges: 0.4–6.3, 6.4–31.6, 32.0–132.8 ms (see IO-Link Spec Table B.3)"
            ),
        }
    }

    /// Returns the encoded multiplier for the configured MinCycleTime.
    ///
    /// # Returns
    /// - For 0.4–6.3 ms: `multiplier = round(time_in_ms / 0.1)`
    const fn multiplier() -> u8 {
        let multiplier = match dev_config::timings::time_in_ms() {
            t if t >= 0.4 && t <= 6.3 => {
                let m = (t / 0.1 + 0.5) as u8;
                if m > 63 {
                    panic!("Multiplier for min cycle time does not fit in 6 bits (0..=63)");
                }
                m
            }
            t if t >= 6.4 && t <= 31.6 => {
                let m = ((t - 6.4) / 0.4 + 0.5) as u8;
                if m > 63 {
                    panic!("Multiplier for min cycle time does not fit in 6 bits (0..=63)");
                }
                m
            }
            t if t >= 32.0 && t <= 132.8 => {
                let m = ((t - 32.0) / 1.6 + 0.5) as u8;
                if m > 63 {
                    panic!("Multiplier for min cycle time does not fit in 6 bits (0..=63)");
                }
                m
            }
            _ => panic!(
                "Invalid min cycle time configuration in ms. Valid ranges: 0.4–6.3, 6.4–31.6, 32.0–132.8 ms (see IO-Link Spec Table B.3)"
            ),
        };
        multiplier
    }

    /// Returns the minimum cycle time in milliseconds as configured for the device.
    ///
    /// This value determines the minimum allowed process data cycle time according to
    /// IO-Link Specification Table B.3. The value is used to encode the CycleTime parameter.
    ///
    /// # Returns
    /// The minimum cycle time in milliseconds as an `f32`.
    ///
    /// # Example
    /// ```
    /// let min_time = config::timings::time_in_ms();
    /// ```
    ///
    /// # Specification
    /// - IO-Link Spec Table B.3: MinCycleTime encoding
    ///
    /// # Panics
    /// Panics if the value is not within the valid IO-Link ranges.
    ///
    /// # Note
    /// This function is intended for use in parameter encoding and device configuration.
    pub const fn min_cycle_time_parameter() -> iolinke_types::page::page1::CycleTime {
        const TIME_BASE: u8 = time_base();
        const MULTIPLIER: u8 = multiplier();
        let mut cycle_time = iolinke_types::page::page1::CycleTime::new();
        cycle_time.set_time_base(TIME_BASE);
        cycle_time.set_multiplier(MULTIPLIER);
        cycle_time
    }
}
