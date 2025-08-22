pub enum ProcessDataLength {
    Bit(u8),
    Octet(u8),
}
/// Constructs the `ProcessDataIn` parameter byte (see IO-Link Spec v1.1.4, Section B.1.6, Figure B.5).
///
/// This byte is structured as follows:
///
/// ```text
///  Bit 7   | Bit 6 | Bit 5 | Bits 0-4
///  BYTE    | SIO   | RES   | LENGTH
/// ```
///
/// ### Bit 7 – `BYTE`
/// Indicates the length unit for the `LENGTH` field.
/// - `0`: Length is in bits (bit-oriented process data)
/// - `1`: Length is in octets (byte-oriented process data)
///
/// See Table B.6 for valid combinations:
// Table B.6 – Permitted combinations of BYTE and Length
///
/// | BYTE | Length  | Definition  
/// |-------|----------|----------------------------------------------|
/// |   0   |   0     | no Process Data                               |
/// |   0   |   1     | 1 bit Process Data, structured in bits        |
/// |   0   | n(2–15) | n bit Process Data, structured in bits        |
/// |   0   |  16     | 16 bit Process Data, structured in bits       |
/// |   0   | 17–31   | Reserved                                      |
/// |   1   | 0, 1    | Reserved                                      |
/// |   1   |   2     | 3 octets Process Data, structured in octets   |
/// |   1   | n(3–30) | n+1 octets Process Data, structured in octets |
/// |   1   |  31     | 32 octets Process Data, structured in octets  |
///
/// ### Bit 6 – `SIO`
/// Indicates if SIO (Standard Input/Output) mode is supported.
/// - `0`: SIO mode not supported
/// - `1`: SIO mode supported
///
/// ### Bit 5 – `RES` (Reserved)
/// Reserved for future use. **Must be set to `0`.**
///
/// ### Bits 0–4 – `LENGTH`
/// `Refer the table above which is B.6 Permitted combinations of BYTE and Length`
///
/// Encodes the process data length. The meaning depends on the `BYTE` bit:
/// - If `BYTE == 0`: Length in bits (valid: 0–16, 17–31 reserved)
/// - If `BYTE == 1`: Length in octets (valid: 2–31; total data = `LENGTH + 1`)
///
/// This macro or constant will generate the final `u8` that should be sent to the IO-Link master
/// to indicate the device's process data input capability.
///
/// ### Spec Reference:
/// - IO-Link Interface and System Spec v1.1.4
/// - Section B.1.6
/// - Table B.5 (SIO Values)
/// - Table B.6 (BYTE + LENGTH combinations)
pub mod pd_in {
    use crate::utils::page_params::page1;

    /// See B.1.6 ProcessDataIn or
    /// check the pd_in module documentation for details
    /// Configure the Process Data Input length
    pub const fn config_length() -> super::ProcessDataLength {
        use super::ProcessDataLength::*;
        // Acceptable values are 0-32 for octets, 0-16 for bits
        match /*CONFIG:OP_PD_IN_LEN*/ Octet(3) /*ENDCONFIG*/ {
            Bit(bit_length) => {
                if bit_length > 16 {
                    panic!("Invalid PD length for OPERATE M-sequence configuration");
                }
                Bit(bit_length)
            },
            Octet(octet_length) => {
                if octet_length > 32 {
                    panic!("Invalid PD length for OPERATE M-sequence configuration");
                }
                Octet(octet_length)
            }
        }
    }

    /// Returns the Process Data Output length in bytes
    pub const fn config_length_in_bytes() -> u8 {
        use super::ProcessDataLength::*;
        match config_length() {
            Bit(bit_length) => {
                // The ceiling division technique:
                // Instead of using floating-point math like ceil(bits / 8.0), this uses the mathematical identity:
                // Formula is ceil(a/b) = (a + b - 1) / b
                (bit_length + 7) / 8

            },
            Octet(octet_length) => octet_length,
        }
    }

    /// Configure the Process Data Input SIO support
    /// See B.1.6 ProcessDataIn or
    /// check the pd_in module documentation for details
    pub const fn sio() -> bool {
        true // false for not supported, true for supported
    }

    /// Configure the Process Data Input BYTE value
    /// See B.1.6 ProcessDataIn or
    /// check the pd_in module documentation for details
    pub const fn byte() -> bool {
        use super::ProcessDataLength::*;
        // 0 for bit-oriented, 1 for byte-oriented
        match /*CONFIG:OP_PD_IN_BYTE*/ Bit(0) /*ENDCONFIG*/ {
            Bit(_) => false,
            Octet(_) => true,
        }
    }

    pub const fn param_length() -> u8 {
        use super::ProcessDataLength::*;
        let length = match config_length() {
            Bit(bit_length) => match bit_length {
                0 => 0,
                1 => 1,
                2..=15 => bit_length,
                16 => 16,
                _ => panic!("Invalid PD length for OPERATE M-sequence configuration"),
            },
            Octet(octet_length) => match octet_length {
                0 | 1 => panic!("Invalid PD length for OPERATE M-sequence configuration"),
                2 => 3,
                3..=31 => octet_length - 1,
                32 => 31,
                _ => panic!("Invalid PD length for OPERATE M-sequence configuration"),
            },
        };
        length
    }

    /// See B.1.6 ProcessDataIn
    /// Construct the Process Data Input configuration byte
    pub const fn page1_pd_in_parameter() -> u8 {
        const BYTE: bool = byte();
        const SIO: bool = sio();
        const LENGTH: u8 = param_length();
        let mut pd_in_page_param: page1::ProcessDataIn = page1::ProcessDataIn::new();
        pd_in_page_param.set_byte(BYTE);
        pd_in_page_param.set_sio(SIO);
        pd_in_page_param.set_length(LENGTH);
        pd_in_page_param.into_bits()
    }
}

/// Constructs the `ProcessDataOut` parameter byte (see IO-Link Spec v1.1.4, Section B.1.6, Figure B.5).
///
/// This byte is structured as follows:
///
/// ```text
///  Bit 7   | Bit 6 | Bit 5 | Bits 0-4
///  BYTE    | RES   | RES   | LENGTH
/// ```
///
/// ### Bit 7 – `BYTE`
/// Indicates the length unit for the `LENGTH` field.
/// - `0`: Length is in bits (bit-oriented process data)
/// - `1`: Length is in octets (byte-oriented process data)
///
/// See Table B.6 for valid combinations:
// Table B.6 – Permitted combinations of BYTE and Length
///
/// | BYTE | Length  | Definition  
/// |-------|----------|----------------------------------------------|
/// |   0   |   0     | no Process Data                               |
/// |   0   |   1     | 1 bit Process Data, structured in bits        |
/// |   0   | n(2–15) | n bit Process Data, structured in bits        |
/// |   0   |  16     | 16 bit Process Data, structured in bits       |
/// |   0   | 17–31   | Reserved                                      |
/// |   1   | 0, 1    | Reserved                                      |
/// |   1   |   2     | 3 octets Process Data, structured in octets   |
/// |   1   | n(3–30) | n+1 octets Process Data, structured in octets |
/// |   1   |  31     | 32 octets Process Data, structured in octets  |
///
/// ### Bit 6 – `RES` (Reserved)
/// Reserved for future use. **Must be set to `0`.**
///
/// ### Bit 5 – `RES` (Reserved)
/// Reserved for future use. **Must be set to `0`.**
///
/// ### Bits 0–4 – `LENGTH`
/// `Refer the table above which is B.6 Permitted combinations of BYTE and Length`
///
/// Encodes the process data length. The meaning depends on the `BYTE` bit:
/// - If `BYTE == 0`: Length in bits (valid: 0–16, 17–31 reserved)
/// - If `BYTE == 1`: Length in octets (valid: 2–31; total data = `LENGTH + 1`)
///
/// This macro or constant will generate the final `u8` that should be sent to the IO-Link master
/// to indicate the device's process data input capability.
///
/// ### Spec Reference:
/// - IO-Link Interface and System Spec v1.1.4
/// - Section B.1.6
/// - Table B.5 (SIO Values)
/// - Table B.6 (BYTE + LENGTH combinations)
pub mod pd_out {
    use crate::utils::page_params::page1;

    /// See B.1.6 ProcessDataOut or
    /// check the pd_out module documentation for details
    /// Configure the Process Data Output length
    /// See B.1.6 ProcessDataOut or
    /// check the pd_out module documentation for details
    /// Configure the Process Data Output length
    pub const fn config_length() -> super::ProcessDataLength {
        use super::ProcessDataLength::*;
        // Acceptable values are 0-32 for octets, 0-16 for bits
        match /*CONFIG:OP_PD_OUT_LEN*/ Octet(3) /*ENDCONFIG*/ {
            Bit(bit_length) => {
                if bit_length > 16 {
                    panic!("Invalid PD length for OPERATE M-sequence configuration");
                }
                Bit(bit_length)
            },
            Octet(octet_length) => {
                if octet_length > 32 {
                    panic!("Invalid PD length for OPERATE M-sequence configuration");
                }
                Octet(octet_length)
            }
        }
    }

    /// Returns the Process Data Output length in bytes
    pub const fn config_length_in_bytes() -> u8 {
        use super::ProcessDataLength::*;
        match config_length() {
            Bit(bit_length) => {
                // The ceiling division technique:
                // Instead of using floating-point math like ceil(bits / 8.0), this uses the mathematical identity:
                // Formula is ceil(a/b) = (a + b - 1) / b
                (bit_length + 7) / 8

            },
            Octet(octet_length) => octet_length,
        }
    }

    /// Configure the Process Data Input BYTE value
    /// See B.1.6 ProcessDataIn or
    /// check the pd_in module documentation for details
    pub const fn byte() -> bool {
        use super::ProcessDataLength::*;
        // 0 for bit-oriented, 1 for byte-oriented
        match /*CONFIG:OP_PD_IN_BYTE*/ Bit(0) /*ENDCONFIG*/ {
            Bit(_) => false,
            Octet(_) => true,
        }
    }

    pub const fn length() -> u8 {
        use super::ProcessDataLength::*;
        let length = match config_length() {
            Bit(bit_length) => match bit_length {
                0 => 0,
                1 => 1,
                2..=15 => bit_length,
                16 => 16,
                _ => panic!("Invalid PD out length for OPERATE M-sequence configuration"),
            },
            Octet(octet_length) => match octet_length {
                0 | 1 => panic!("Invalid PD out length for OPERATE M-sequence configuration"),
                2 => 3,
                3..=31 => octet_length - 1,
                32 => 31,
                _ => panic!("Invalid PD  length for OPERATE M-sequence configuration"),
            },
        };
        length
    }

    /// See B.1.7 ProcessDataOut
    /// Construct the Process Data Output configuration byte
    pub const fn pd_out_parameter() -> u8 {
        let mut pd_out_page_param: page1::ProcessDataOut = page1::ProcessDataOut::new();
        pd_out_page_param.set_byte(byte());
        pd_out_page_param.set_length(length());
        pd_out_page_param.into_bits()
    }
}