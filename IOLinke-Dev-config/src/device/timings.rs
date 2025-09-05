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
    const TIME_IN_MS: f32 = /*CONFIG:MIN_CYCLE_TIME_IN_MS*/ 33 /*ENDCONFIG*/ as f32;
    let valid = (TIME_IN_MS >= 0.4 && TIME_IN_MS <= 6.3)
        || (TIME_IN_MS >= 6.4 && TIME_IN_MS <= 31.6)
        || (TIME_IN_MS >= 32.0 && TIME_IN_MS <= 132.8);
    if !valid {
        panic!(
            "Invalid min cycle time configuration. Valid ranges: 0.4–6.3, 6.4–31.6, 32.0–132.8 milli seconds (see IO-Link Spec Table B.3)"
        );
    }
    TIME_IN_MS
}
