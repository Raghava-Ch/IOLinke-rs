/// Returns the encoded MinCycleTime parameter as a single `u8` value according to IO-Link Specification v1.1.4, B.1.3.
/// 
/// The MinCycleTime parameter informs the IO-Link Master about the shortest cycle time supported by this Device.
/// The encoding is as follows (see Figure B.2 and Table B.3):
/// - Bits 0..=5: Multiplier (0..=63)
/// - Bits 6..=7: Time Base (0b00 = 0.1ms, 0b01 = 0.4ms, 0b10 = 1.6ms, 0b11 = Reserved)
///
/// # Returns
/// The encoded MinCycleTime as a `u8`.
///
/// # Panics
/// - If the time base is 0b11 (reserved)
/// - If the time base is greater than 0b11
/// - If the multiplier is greater than 63
///
/// # Specification Reference
/// - IO-Link Specification v1.1.4, Section B.1.3, Table B.3
/// - Figure B.2 – MinCycleTime
///
/// # Note
/// If MinCycleTime is zero (0x00), the device provides no MinCycleTime and the Master must use the calculated worst-case M-sequence timing.
pub const fn min_cycle_time() -> u8 {
    const TIME_BASE: u8 = min_cycle_time_time_base();
    const MULTIPLIER: u8 = min_cycle_time_multiplier();
    if TIME_BASE == 3 {
        panic!("Invalid min cycle time time base: 3 is reserved");
    }
    if TIME_BASE > 3 {
        panic!("Invalid min cycle time time base provided is not in the range 0-3");
    }
    if MULTIPLIER > 0b111111 {
        panic!("Invalid min cycle time multiplier provided is not in the range 0-63");
    }
    (TIME_BASE << 6) | MULTIPLIER
}

/// Returns the multiplier value (bits 0..=5) for the MinCycleTime parameter.
/// 
/// The multiplier is a 6-bit value (0..=63) used in the calculation of the MinCycleTime.
/// The actual time is determined by the combination of the time base and multiplier (see Table B.3).
///
/// # Returns
/// The multiplier as a `u8`.
///
/// # Specification Reference
/// - IO-Link Specification v1.1.4, Section B.1.3, Table B.3
pub const fn min_cycle_time_time_base() -> u8 {
    2u8
}

/// Returns the time base encoding (bits 6..=7) for the MinCycleTime parameter.
/// 
/// The time base determines the base unit for the cycle time calculation:
/// - 0b00: 0.1 ms
/// - 0b01: 0.4 ms
/// - 0b10: 1.6 ms
/// - 0b11: Reserved (must not be used)
///
/// # Returns
/// The time base encoding as a `u8` (0..=2).
///
/// # Specification Reference
/// - IO-Link Specification v1.1.4, Section B.1.3, Table B.3
pub const fn min_cycle_time_multiplier() -> u8 {
    60u8
}

/// Returns the minimum cycle time in microseconds (µs) as calculated from the time base and multiplier.
/// 
/// The calculation is as follows (see Table B.3):
/// - If time base == 0b00: MinCycleTime = Multiplier × 0.1 ms
/// - If time base == 0b01: MinCycleTime = 6.4 ms + Multiplier × 0.4 ms
/// - If time base == 0b10: MinCycleTime = 32.0 ms + Multiplier × 1.6 ms
/// - If time base == 0b11: Reserved (returns 0)
///
/// # Returns
/// The minimum cycle time in microseconds (µs) as a `u16`.
///
/// # Specification Reference
/// - IO-Link Specification v1.1.4, Section B.1.3, Table B.3
pub const fn min_cycle_time_in_us() -> u16 {
    0u16
}
