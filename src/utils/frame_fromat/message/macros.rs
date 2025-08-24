/// Mask to set bits 0-5 to zero while preserving bits 6-7
/// This macro clears the revceived checksum bits (0-5) in a byte,
/// leaving bits 6 and 7 unchanged.
#[macro_export]
macro_rules! clear_checksum_bits_0_to_5 {
    ($byte:expr) => {
        ($byte) & 0b11000000u8
    };
}

/// Extract checksum bits (bits 0-5) from byte
#[macro_export]
macro_rules! extract_checksum_bits {
    ($byte:expr) => {
        ($byte) & 0x3F
    };
}

/// Extract RW direction from first byte (bit 2)
/// 0 = Read, 1 = Write
#[macro_export]
macro_rules! extract_rw_direction {
    ($byte:expr) => {
        get_bit_7!($byte)
    };
}

/// Extract communication channel from first byte (bits 5-6)
/// 00 = Process, 01 = Page, 10 = Diagnosis, 11 = ISDU
#[macro_export]
macro_rules! extract_com_channel {
    ($byte:expr) => {
        get_bits_5_6!($byte)
    };
}
/// Extract address from first byte for M-sequence byte (bits 0-4)
#[macro_export]
macro_rules! extract_address_fctrl {
    ($byte:expr) => {
        get_bits_0_4!($byte)
    };
}

/// Extract the message type (TYPE_0, TYPE_1, or TYPE_2)
#[macro_export]
macro_rules! extract_message_type {
    ($msg_type:expr) => {
        get_bits_6_7!($msg_type)
    };
}

/// Compile `Event flag` for CKS byte
#[macro_export]
macro_rules! compile_event_flag {
    ($data:expr, $event_flag:expr) => {
        set_bit_7!($data, $event_flag)
    };
}

/// Compile `PD status` for CKS byte
#[macro_export]
macro_rules! compile_pd_status {
    ($data:expr, $pd_status:expr) => {
        set_bit_6!($data, $pd_status)
    };
}

/// Compile Checksum byte Bit: 0 to Bit: 5
#[macro_export]
macro_rules! compile_checksum {
    ($data:expr, $checksum:expr) => {
        set_bits_0_5!($data, $checksum)
    };
}

/// See A.1.5 Checksum / status (CKS)
/// Compile Checksum / Status (CKS) byte
#[macro_export]
macro_rules! compile_checksum_status {
    ($data:expr, $event_flag:expr, $pd_status:expr, $checksum:expr) => {{
        compile_event_flag!($data, $event_flag)
            | compile_pd_status!($data, $pd_status)
            | compile_checksum!($data, $checksum)
            | $data
    }};
}