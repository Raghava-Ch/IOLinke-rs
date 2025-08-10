/// Bitwise macros for extracting fields from IO-Link master messages
/// Based on IO-Link Specification v1.1.4 Section 6.1

/// Construct a u8 from individual bits (bit 0 to bit 7)
/// Constructs a u8 value from 8 individual bit values.
///
/// This macro takes 8 expressions representing individual bits (from bit 7 to bit 0)
/// and combines them into a single u8 value. Each input expression is masked with 0x01
/// to ensure only the least significant bit is used, then shifted to the appropriate
/// position within the resulting byte.
///
/// # Parameters
///
/// * `$bit7` - The most significant bit (bit 7)
/// * `$bit6` - Bit 6
/// * `$bit5` - Bit 5
/// * `$bit4` - Bit 4
/// * `$bit3` - Bit 3
/// * `$bit2` - Bit 2
/// * `$bit1` - Bit 1
/// * `$bit0` - The least significant bit (bit 0)
///
/// # Returns
///
/// A `u8` value with bits set according to the input parameters.
///
/// # Examples
///
/// ```rust
/// // Create a byte with all bits set to 1 (0xFF)
/// let byte = construct_u8!(1, 1, 1, 1, 1, 1, 1, 1);
/// assert_eq!(byte, 0xFF);
///
/// // Create a byte with alternating bits (0xAA)
/// let byte = construct_u8!(1, 0, 1, 0, 1, 0, 1, 0);
/// assert_eq!(byte, 0xAA);
///
/// // Create a byte with only the most significant bit set (0x80)
/// let byte = construct_u8!(1, 0, 0, 0, 0, 0, 0, 0);
/// assert_eq!(byte, 0x80);
///
/// // Values greater than 1 are masked to use only the LSB
/// let byte = construct_u8!(5, 2, 0, 0, 0, 0, 0, 1);
/// assert_eq!(byte, 0xC1); // 5 & 1 = 1, 2 & 1 = 0
/// ```

#[macro_export]
macro_rules! construct_u8 {
    ($bit7:expr, $bit6:expr, $bit5:expr, $bit4:expr, $bit3:expr, $bit2:expr, $bit1:expr, $bit0:expr) => {
        (($bit7 & 0x01) << 7)
            | (($bit6 & 0x01) << 6)
            | (($bit5 & 0x01) << 5)
            | (($bit4 & 0x01) << 4)
            | (($bit3 & 0x01) << 3)
            | (($bit2 & 0x01) << 2)
            | (($bit1 & 0x01) << 1)
            | ($bit0 & 0x01)
    };
}

/// Extract bit 0 from byte
#[macro_export]
macro_rules! get_bit_0 {
    ($byte:expr) => {
        ($byte) & 0x01
    };
}

/// Extract bit 1 from byte
#[macro_export]
macro_rules! get_bit_1 {
    ($byte:expr) => {
        (($byte) >> 1) & 0x01
    };
}

/// Extract bit 2 from byte
#[macro_export]
macro_rules! get_bit_2 {
    ($byte:expr) => {
        (($byte) >> 2) & 0x01
    };
}

/// Extract bit 3 from byte
#[macro_export]
macro_rules! get_bit_3 {
    ($byte:expr) => {
        (($byte) >> 3) & 0x01
    };
}

/// Extract bit 4 from byte
#[macro_export]
macro_rules! get_bit_4 {
    ($byte:expr) => {
        (($byte) >> 4) & 0x01
    };
}

/// Extract bit 5 from byte
#[macro_export]
macro_rules! get_bit_5 {
    ($byte:expr) => {
        (($byte) >> 5) & 0x01
    };
}

/// Extract bit 6 from byte
#[macro_export]
macro_rules! get_bit_6 {
    ($byte:expr) => {
        (($byte) >> 6) & 0x01
    };
}

/// Extract bit 7 from byte
#[macro_export]
macro_rules! get_bit_7 {
    ($byte:expr) => {
        (($byte) >> 7) & 0x01
    };
}

/// Extract bits 5 and 6 from byte
#[macro_export]
macro_rules! get_bits_5_6 {
    ($byte:expr) => {
        (($byte) >> 5) & 0x03
    };
}

/// Extract bits 0-4 from byte
#[macro_export]
macro_rules! get_bits_0_4 {
    ($byte:expr) => {
        ($byte) & 0x1F
    };
}

/// Extract bits 6 and 7 from byte
#[macro_export]
macro_rules! get_bits_6_7 {
    ($byte:expr) => {
        (($byte) >> 6) & 0x03
    };
}

/// Set bit 0 in byte
#[macro_export]
macro_rules! set_bit_0 {
    ($byte:expr, $value:expr) => {
        ($byte & 0xFE) | (($value & 0x01))
    };
}

/// Set bit 1 in byte
#[macro_export]
macro_rules! set_bit_1 {
    ($byte:expr, $value:expr) => {
        ($byte & 0xFD) | (($value & 0x01) << 1)
    };
}

/// Set bit 2 in byte
#[macro_export]
macro_rules! set_bit_2 {
    ($byte:expr, $value:expr) => {
        ($byte & 0xFB) | (($value & 0x01) << 2)
    };
}

/// Set bit 3 in byte
#[macro_export]
macro_rules! set_bit_3 {
    ($byte:expr, $value:expr) => {
        ($byte & 0xF7) | (($value & 0x01) << 3)
    };
}

/// Set bit 4 in byte
#[macro_export]
macro_rules! set_bit_4 {
    ($byte:expr, $value:expr) => {
        ($byte & 0xEF) | (($value & 0x01) << 4)
    };
}

/// Set bit 5 in byte
#[macro_export]
macro_rules! set_bit_5 {
    ($byte:expr, $value:expr) => {
        ($byte & 0xDF) | (($value & 0x01) << 5)
    };
}

/// Set bit 6 in byte
#[macro_export]
macro_rules! set_bit_6 {
    ($byte:expr, $value:expr) => {
        ($byte & 0xBF) | (($value & 0x01) << 6)
    };
}

/// Set bit 7 in byte
#[macro_export]
macro_rules! set_bit_7 {
    ($byte:expr, $value:expr) => {
        ($byte & 0x7F) | (($value & 0x01) << 7)
    };
}

/// Set bits 5 and 6 in byte
#[macro_export]
macro_rules! set_bits_5_6 {
    ($byte:expr, $value:expr) => {
        ($byte & 0x9F) | (($value & 0x03) << 5)
    };
}

/// Set bits 0-4 in byte
#[macro_export]
macro_rules! set_bits_0_4 {
    ($byte:expr, $value:expr) => {
        ($byte & 0xE0) | ($value & 0x1F)
    };
}

/// Set bits 0-5 in byte
#[macro_export]
macro_rules! set_bits_0_5 {
    ($byte:expr, $value:expr) => {
        ($byte & 0xE0) | ($value & 0x3F)
    };
}



/// Sets bits 6 and 7 of a byte to the specified 2-bit value.
///
/// This macro preserves the lower 6 bits (bits 0-5) of the input byte
/// and replaces bits 6 and 7 with the provided value.
///
/// # Parameters
/// * `$byte` - The byte to modify
/// * `$value` - A 2-bit value (0-3) to set in bits 6 and 7
///
/// # Returns
/// A byte with bits 6-7 set to the specified value and bits 0-5 unchanged
///
/// # Examples
/// ```rust
/// // Set bits 6-7 to binary 10 (decimal 2)
/// let original = 0b00111111; // All lower 6 bits set
/// let result = set_bits_6_7!(original, 2);
/// assert_eq!(result, 0b10111111); // Bits 6-7 now set to 10
///
/// // Set bits 6-7 to binary 01 (decimal 1)
/// let original = 0b11000000; // Bits 6-7 originally set
/// let result = set_bits_6_7!(original, 1);
/// assert_eq!(result, 0b01000000); // Lower bits cleared, 6-7 set to 01
/// ```
#[macro_export]
macro_rules! set_bits_6_7 {
    ($byte:expr, $value:expr) => {
        ($byte & 0x3F) | (($value & 0x03) << 6)
    };
}
