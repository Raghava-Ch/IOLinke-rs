use crate::{
    clear_checksum_bits_0_to_5, compile_checksum, compile_checksum_status, compile_event_flag,
    compile_pd_status, construct_u8, extract_address_fctrl, extract_checksum_bits,
    extract_com_channel, extract_message_type, extract_rw_direction, get_bit_0, get_bit_1,
    get_bit_2, get_bit_3, get_bit_4, get_bit_5, get_bit_6, get_bit_7, get_bits_0_4, get_bits_5_6,
    get_bits_6_7, set_bit_6, set_bit_7, set_bits_0_5,
};
use crate::{config, types};
use bitfields::bitfield;
use heapless::Vec;


pub const HEADER_SIZE: usize = 2; // Header size is 2 bytes (MC and length)
/// Maximum message buffer size for OD
/// This is the maximum size of the message buffer used for OD messages.
const MAX_OD_SIZE: usize = config::on_req_data::max_possible_od_length() as usize;
/// Maximum message buffer size for PD
/// This is the maximum size of the message buffer used for PD messages.
const PD_IN_LENGTH: usize = config::process_data::pd_in::param_length() as usize;
const PD_OUT_LENGTH: usize = config::process_data::pd_out::length() as usize;
/// Maximum frame size for IO-Link messages
pub const MAX_FRAME_SIZE: usize = MAX_OD_SIZE + PD_OUT_LENGTH + HEADER_SIZE;

#[bitfield(u8)]
#[derive(Clone, Copy)]
pub struct MsequenceControl {
    #[bits(1)]
    pub read_write: types::RwDirection,
    #[bits(2)]
    pub comm_channel: types::ComChannel,
    #[bits(5)]
    pub address_fctrl: u8,
}

/// A.1.3 Checksum / M-sequence type (CKT)
/// The M-sequence type is transmitted together with the checksum in the check/type octet. The
/// structure of this octet is demonstrated in Figure A.2.
#[bitfield(u8)]
#[derive(Clone, Copy)]
pub struct ChecksumMsequenceType {
    #[bits(2)]
    pub m_seq_type: types::MsequenceBaseType,
    #[bits(6)]
    pub checksum: u8,
}

/// IO-Link message structure
/// See IO-Link v1.1.4 Section 6.1
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IoLinkMessage {
    /// Read/Write direction
    pub read_write: Option<types::RwDirection>,
    /// Message type
    pub message_type: Option<types::MessageType>,
    /// Communication channel
    pub com_channel: Option<types::ComChannel>,
    /// Contains the address or flow control value (see A.1.2).
    pub address_fctrl: Option<u8>,
    /// Event flag
    pub event_flag: bool,
    /// On Request Data (OD) response data
    pub od: Option<Vec<u8, MAX_OD_SIZE>>,
    /// Process Data (PD) response data
    pub pd: Option<Vec<u8, PD_OUT_LENGTH>>,
    /// Process Data input status
    pub pd_status: Option<types::PdStatus>,
}

impl Default for IoLinkMessage {
    fn default() -> Self {
        Self {
            read_write: None,
            message_type: None,
            com_channel: None,
            address_fctrl: None,
            event_flag: false,
            od: None,
            pd: None,
            pd_status: None,
        }
    }
}

/// Represents the M-sequence type used in IO-Link communication, based on bits 6 and 7 of the
/// Checksum / M-sequence Type (CKT) field. These bits define how the Master structures messages
/// within an M-sequence, as specified in Table A.3 of the IO-Link specification (see Section A.1.3).
///
/// This macro depends on the `operate_m_sequence` or `operate_m_sequence_legacy` macros. The selected
/// M-sequence type in `operate_m_sequence` or `operate_m_sequence_legacy` determines the value of the
/// `operate_m_sequence_base_type` macro.
///
/// The mapping is as follows:
/// - `TYPE_0`   → `0`
/// - `TYPE_1_x` → `1`
/// - `TYPE_2_x` → `2`
/// - `3` is reserved and should not be used
///
/// Ensure consistency with the selected M-sequence type when defining dependent macros.
pub const fn get_m_sequence_base_type(m_sequence_type: types::MsequenceType) -> types::MsequenceBaseType {
    match m_sequence_type {
        types::MsequenceType::Type0 => types::MsequenceBaseType::Type0,
        types::MsequenceType::Type12 => types::MsequenceBaseType::Type1,
        types::MsequenceType::Type1V => types::MsequenceBaseType::Type2,
        _ => panic!("Invalid M-sequence type"),
    }
}

pub fn compile_mc_ckt_bytes(
    buffer: &mut [u8],
) -> Result<(MsequenceControl, ChecksumMsequenceType), types::IoLinkError> {
    let mc = MsequenceControl::from(buffer[0]);
    let ckt = ChecksumMsequenceType::from(buffer[1]);
    Ok((mc, ckt))
}

pub fn compile_iolink_startup_frame(
    tx_buffer: &mut [u8],
    io_link_message: &IoLinkMessage,
) -> Result<u8, types::IoLinkError> {
    tx_buffer[0] = io_link_message
        .od
        .as_ref()
        .ok_or(types::IoLinkError::InvalidData)?[0];
    tx_buffer[1] = compile_checksum_status!(
        tx_buffer[1],
        io_link_message.event_flag as u8,
        io_link_message
            .pd_status
            .unwrap_or(types::PdStatus::INVALID) as u8,
        0 // Checksum will be calculated later
    );
    let checksum = calculate_checksum(2, &tx_buffer);
    tx_buffer[1] = compile_checksum!(tx_buffer[1], checksum);
    Ok(2)
}

pub fn compile_iolink_preoperate_frame(
    tx_buffer: &mut [u8],
    io_link_message: &IoLinkMessage,
) -> Result<u8, types::IoLinkError> {
    const OD_LENGTH: u8 = config::on_req_data::pre_operate::od_length();
    if io_link_message.od.is_none() {
        return Err(types::IoLinkError::InvalidData);
    }
    for (i, &byte) in io_link_message.od.as_ref().unwrap().iter().enumerate() {
        if i < OD_LENGTH as usize {
            tx_buffer[i] = byte;
        } else {
            break; // Avoid out of bounds access
        }
    }
    tx_buffer[OD_LENGTH as usize] = compile_checksum_status!(
        tx_buffer[OD_LENGTH as usize],
        io_link_message.event_flag as u8,
        io_link_message
            .pd_status
            .unwrap_or(types::PdStatus::INVALID) as u8,
        0 // Checksum will be calculated later
    );
    let checksum = calculate_checksum(OD_LENGTH + 1, &tx_buffer);
    tx_buffer[OD_LENGTH as usize] = compile_checksum!(tx_buffer[OD_LENGTH as usize], checksum);
    Ok(OD_LENGTH + 1)
}

pub fn compile_iolink_operate_frame(
    tx_buffer: &mut [u8],
    io_link_message: &IoLinkMessage,
) -> Result<u8, types::IoLinkError> {
    const OD_LENGTH: u8 = config::on_req_data::operate::od_length();
    const PD_LENGTH: u8 = config::process_data::pd_out::config_length_in_bytes();

    if io_link_message.od.is_none() {
        return Err(types::IoLinkError::InvalidData);
    }
    for (i, &byte) in io_link_message.od.as_ref().unwrap().iter().enumerate() {
        if i < OD_LENGTH as usize {
            tx_buffer[i] = byte;
        } else {
            break; // Avoid out of bounds access
        }
    }
    tx_buffer[OD_LENGTH as usize] = compile_checksum_status!(
        tx_buffer[OD_LENGTH as usize],
        io_link_message.event_flag as u8,
        io_link_message
            .pd_status
            .unwrap_or(types::PdStatus::INVALID) as u8,
        0 // Checksum will be calculated later
    );
    tx_buffer[PD_LENGTH as usize] = compile_checksum_status!(
        tx_buffer[PD_LENGTH as usize],
        io_link_message.event_flag as u8,
        io_link_message
            .pd_status
            .unwrap_or(types::PdStatus::INVALID) as u8,
        0 // Checksum will be calculated later
    );
    let checksum = calculate_checksum(OD_LENGTH + PD_LENGTH + 1, &tx_buffer);
    tx_buffer[OD_LENGTH as usize] =
        compile_checksum!(tx_buffer[(OD_LENGTH + PD_LENGTH) as usize], checksum);
    Ok(OD_LENGTH + PD_LENGTH + 1)
}

/// Parse IO-Link frame using nom
/// See IO-Link v1.1.4 Section 6.1
pub fn parse_iolink_startup_frame(input: &[u8]) -> types::IoLinkResult<IoLinkMessage> {
    // Extracting `MC` byte properties
    let read_write = types::RwDirection::try_from(extract_rw_direction!(input[0]))?;
    let com_channel = types::ComChannel::try_from(extract_com_channel!(input[0]))?;
    let address_fctrl = extract_address_fctrl!(input[0]);

    // Extracting `CKT` byte properties
    let message_type = types::MessageType::try_from(extract_message_type!(input[1]))?;
    // check for M-sequenceCapability
    // For STARTUP, we expect TYPE_0 (0b00), thus we can check directly
    // because message sequence is always TYPE_0.
    // OD length in startup is always 1 byte
    let od: Option<Vec<u8, MAX_OD_SIZE>> = if input.len() > 2 {
        let mut vec = Vec::new();
        vec.push(input[2])
            .map_err(|_| types::IoLinkError::InvalidData)?;
        Some(vec)
    } else {
        None
    };

    Ok(IoLinkMessage {
        read_write: Some(read_write),
        message_type: Some(message_type),
        com_channel: Some(com_channel),
        address_fctrl: Some(address_fctrl),
        event_flag: false, // Event flag is not set in startup frame
        od,
        pd: None,        // No PD in startup frame
        pd_status: None, // No PD status in startup frame
    })
}

pub fn parse_iolink_pre_operate_frame(input: &[u8]) -> types::IoLinkResult<IoLinkMessage> {
    // On-request Data (OD) length
    const OD_LENGTH: u8 = config::on_req_data::pre_operate::od_length();
    // Extracting `MC` byte properties
    let read_write = types::RwDirection::try_from(extract_rw_direction!(input[0]))?;
    let com_channel = types::ComChannel::try_from(extract_com_channel!(input[0]))?;
    let address_fctrl = extract_address_fctrl!(input[0]);

    // Extracting `CKT` byte properties
    let rxed_message_type = types::MessageType::try_from(extract_message_type!(input[1]))?;
    let od = if input.len() > 2 {
        let mut vec: Vec<u8, MAX_OD_SIZE> = Vec::new();
        // Extract OD data
        for i in 2..(2 + OD_LENGTH as usize) {
            if i < input.len() {
                vec.push(input[i])
                    .map_err(|_| types::IoLinkError::InvalidData)?;
            } else {
                break; // Avoid out of bounds access
            }
        }
        Some(vec)
    } else {
        None
    };

    Ok(IoLinkMessage {
        read_write: Some(read_write),
        message_type: Some(rxed_message_type),
        com_channel: Some(com_channel),
        address_fctrl: Some(address_fctrl),
        event_flag: false, // Event flag is not set in pre-operate frame
        od,
        pd: None,        // No PD in pre-operate frame
        pd_status: None, // No PD status in pre-operate frame
    })
}

pub fn parse_iolink_operate_frame(
    input: &[u8],
) -> types::IoLinkResult<IoLinkMessage> {
    const OD_LENGTH_OCTETS: u8 = config::on_req_data::operate::od_length();
    const PD_LENGTH: u8 = config::process_data::pd_out::config_length_in_bytes();

    let read_write = types::RwDirection::try_from(extract_rw_direction!(input[0]))?;
    let com_channel = types::ComChannel::try_from(extract_com_channel!(input[0]))?;
    let address_fctrl = extract_address_fctrl!(input[0]);
    let rxed_message_type = types::MessageType::try_from(extract_message_type!(input[1]))?;

    if input.len() != HEADER_SIZE + OD_LENGTH_OCTETS as usize + PD_LENGTH as usize {
        return Err(types::IoLinkError::InvalidData);
    }
    let mut od: Vec<u8, MAX_OD_SIZE> = Vec::new();
    let mut pd: Vec<u8, PD_OUT_LENGTH> = Vec::new();

    for i in 2..(2 + OD_LENGTH_OCTETS as usize) {
        if i < input.len() {
            od.push(input[i])
                .map_err(|_| types::IoLinkError::InvalidData)?;
        } else {
            break; // Avoid out of bounds access
        }
    }

    for i in (2 + OD_LENGTH_OCTETS as usize)..(2 + OD_LENGTH_OCTETS as usize + PD_LENGTH as usize) {
        if i < input.len() {
            pd.push(input[i])
                .map_err(|_| types::IoLinkError::InvalidData)?;
        } else {
            break; // Avoid out of bounds access
        }
    }

    Ok(IoLinkMessage {
        read_write: Some(read_write),
        message_type: Some(rxed_message_type),
        com_channel: Some(com_channel),
        address_fctrl: Some(address_fctrl),
        event_flag: false, // Event flag is not set in pre-operate frame
        od: Some(od),
        pd: Some(pd),
        pd_status: None, // No PD status in pre-operate frame
    })
}

pub fn validate_checksum(length: u8, data: &mut [u8]) -> bool {
    // Validate the checksum of the received IO-Link message
    let received_checksum = extract_checksum_bits!(data[1]);
    // clear the received checksum bits (0-5), Before calculating the checksum
    data[1] = clear_checksum_bits_0_to_5!(data[1]);
    let calculated_checksum = calculate_checksum(length, &data);
    calculated_checksum == received_checksum
}

/// See A.1.6 Calculation of the checksum
/// Calculate message checksum
pub fn calculate_checksum(length: u8, data: &[u8]) -> u8 {
    // Seed value as per IO-Link spec
    let mut checksum = 0x52u8;
    for i in 0..length as usize {
        if i < data.len() {
            checksum ^= data[i];
        }
    }
    let d_bit0 = get_bit_0!(checksum);
    let d_bit1 = get_bit_1!(checksum);
    let d_bit2 = get_bit_2!(checksum);
    let d_bit3 = get_bit_3!(checksum);
    let d_bit4 = get_bit_4!(checksum);
    let d_bit5 = get_bit_5!(checksum);
    let d_bit6 = get_bit_6!(checksum);
    let d_bit7 = get_bit_7!(checksum);

    let checksum_bit0 = d_bit1 ^ d_bit0;
    let checksum_bit1 = d_bit3 ^ d_bit2;
    let checksum_bit2 = d_bit5 ^ d_bit4;
    let checksum_bit3 = d_bit7 ^ d_bit6;
    let checksum_bit4 = d_bit6 ^ d_bit4 ^ d_bit2 ^ d_bit0;
    let checksum_bit5 = d_bit7 ^ d_bit5 ^ d_bit3 ^ d_bit1;

    let checksum = construct_u8!(
        0,
        0,
        checksum_bit5,
        checksum_bit4,
        checksum_bit3,
        checksum_bit2,
        checksum_bit1,
        checksum_bit0
    );

    checksum
}
