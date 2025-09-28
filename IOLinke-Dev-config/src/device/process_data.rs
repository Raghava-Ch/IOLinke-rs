use core::panic;

/// Represents the process data length for IO-Link devices, as specified in IO-Link Specification v1.1.4, Section B.1.6.
///
/// This enum encodes whether the process data length is specified in bits or octets (bytes),
/// and stores the corresponding value. The interpretation of the value depends on the variant:
///
/// - `Bit(u8)`: The process data length is given in bits. Valid values are 0–16 (see Table B.6).
/// - `Octet(u8)`: The process data length is given in octets (bytes). Valid values are 0–32 (see Table B.6).
///
/// # Examples
///
/// - `ProcessDataLength::Bit(8)` means 8 bits of process data.
/// - `ProcessDataLength::Octet(3)` means 3 octets (bytes) of process data.
///
/// # Specification Reference
/// - IO-Link Interface and System Spec v1.1.4, Section B.1.6, Table B.6
pub enum ProcessDataLength {
    /// Bit-oriented process data length (0–16 bits).
    Bit(u8),
    /// Octet (byte)-oriented process data length (0–32 octets).
    Octet(u8),
}

/// See B.1.6 ProcessDataIn or
/// check the pd_in module documentation for details
/// Configure the Process Data Input length
pub const fn config_pd_in_length() -> ProcessDataLength {
    use ProcessDataLength::*;
    // Acceptable values are 0-32 for octets, 0-16 for bits
    const OP_PD_IN_LEN: ProcessDataLength = /*CONFIG:OP_PD_IN_LEN*/ Octet(3) /*ENDCONFIG*/;
    match OP_PD_IN_LEN {
        Bit(bit_length) => {
            if bit_length > 16 {
                panic!("Invalid PD length for OPERATE M-sequence configuration");
            }
            Bit(bit_length)
        }
        Octet(octet_length) => {
            if octet_length > 32 {
                panic!("Invalid PD length for OPERATE M-sequence configuration");
            }
            Octet(octet_length)
        }
    }
}

/// Returns the Process Data Input length in bytes
pub const fn config_pd_in_length_in_bytes() -> u8 {
    use ProcessDataLength::*;
    match config_pd_in_length() {
        Bit(bit_length) => {
            // The ceiling division technique:
            // Instead of using floating-point math like ceil(bits / 8.0), this uses the mathematical identity:
            // Formula is ceil(a/b) = (a + b - 1) / b
            (bit_length + 7) / 8
        }
        Octet(octet_length) => octet_length,
    }
}

/// See B.1.6 ProcessDataOut or
/// check the pd_out module documentation for details
/// Configure the Process Data Output length
/// See B.1.6 ProcessDataOut or
/// check the pd_out module documentation for details
/// Configure the Process Data Output length
pub const fn config_pd_out_length() -> ProcessDataLength {
    use ProcessDataLength::*;
    // Acceptable values are `0-32` for octets, `0-16` for bits
    const OP_PD_OUT_LEN: ProcessDataLength = /*CONFIG:OP_PD_OUT_LEN*/ Octet(4) /*ENDCONFIG*/;
    match OP_PD_OUT_LEN {
        Bit(bit_length) => {
            if bit_length > 16 {
                panic!("Invalid PD length for OPERATE M-sequence configuration");
            }
            Bit(bit_length)
        }
        Octet(octet_length) => {
            if octet_length > 32 {
                panic!("Invalid PD length for OPERATE M-sequence configuration");
            }
            Octet(octet_length)
        }
    }
}

/// Returns the Process Data Output length in bytes
pub const fn config_pd_out_length_in_bytes() -> u8 {
    use ProcessDataLength::*;
    match config_pd_out_length() {
        Bit(bit_length) => {
            // The ceiling division technique:
            // Instead of using floating-point math like ceil(bits / 8.0), this uses the mathematical identity:
            // Formula is ceil(a/b) = (a + b - 1) / b
            (bit_length + 7) / 8
        }
        Octet(octet_length) => octet_length,
    }
}
