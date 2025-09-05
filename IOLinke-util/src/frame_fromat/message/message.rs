use bitfields::bitfield;
use heapless::Vec;
use iolinke_derived_config::device as derived_config;
use iolinke_types::{
    custom::{IoLinkError, IoLinkResult},
    frame::msequence::{
        ChecksumMsequenceType, ChecksumStatus, ComChannel, MsequenceBaseType, MsequenceControl,
        MsequenceType, PdStatus, RwDirection,
    },
};

pub const HEADER_SIZE_IN_FRAME: u8 = 2; // Header size is 2 bytes (MC and length)
/// Maximum message buffer size for OD
/// This is the maximum size of the message buffer used for OD messages in operating modes.
pub const MAX_POSSIBLE_OD_LEN_IN_FRAME: u8 = derived_config::on_req_data::max_possible_od_length();
/// Maximum message buffer size for PD
/// This is the maximum size of the message buffer used for PD messages.
const PD_IN_LENGTH: u8 = derived_config::process_data::pd_in::config_length_in_bytes();
const PD_OUT_LENGTH: u8 = derived_config::process_data::pd_out::config_length_in_bytes();
/// Maximum frame size for IO-Link messages
pub const MAX_RX_FRAME_SIZE: usize =
    (MAX_POSSIBLE_OD_LEN_IN_FRAME + PD_IN_LENGTH + HEADER_SIZE_IN_FRAME) as usize;
pub const MAX_TX_FRAME_SIZE: usize =
    (MAX_POSSIBLE_OD_LEN_IN_FRAME + PD_OUT_LENGTH + HEADER_SIZE_IN_FRAME) as usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceOperationMode {
    Startup,
    PreOperate,
    Operate,
}

/// IO-Link message structure
/// See IO-Link v1.1.4 Section 6.1
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IoLinkMessage {
    /// Will be used decide the frame type to be compiled
    pub frame_type: DeviceOperationMode,
    /// Read/Write direction
    pub read_write: Option<RwDirection>,
    /// Message type
    pub message_type: Option<MsequenceBaseType>,
    /// Communication channel
    pub com_channel: Option<ComChannel>,
    /// Contains the address or flow control value (see A.1.2).
    pub address_fctrl: Option<u8>,
    /// Event flag
    pub event_flag: bool,
    /// On Request Data (OD) response data
    pub od: Option<Vec<u8, { MAX_POSSIBLE_OD_LEN_IN_FRAME as usize }>>,
    /// Process Data (PD) response data
    pub pd: Option<Vec<u8, { PD_OUT_LENGTH as usize }>>,
    /// Process Data input status
    pub pd_status: Option<PdStatus>,
}

impl IoLinkMessage {
    pub fn new(device_mode: DeviceOperationMode, read_write: Option<RwDirection>) -> Self {
        Self {
            frame_type: device_mode,
            read_write: read_write,
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
pub const fn get_m_sequence_base_type(m_sequence_type: MsequenceType) -> MsequenceBaseType {
    match m_sequence_type {
        MsequenceType::Type0 => MsequenceBaseType::Type0,
        MsequenceType::Type11 => MsequenceBaseType::Type1,
        MsequenceType::Type12 => MsequenceBaseType::Type1,
        MsequenceType::Type1V => MsequenceBaseType::Type2,
        MsequenceType::Type21 => MsequenceBaseType::Type2,
        MsequenceType::Type22 => MsequenceBaseType::Type2,
        MsequenceType::Type23 => MsequenceBaseType::Type2,
        MsequenceType::Type24 => MsequenceBaseType::Type2,
        MsequenceType::Type25 => MsequenceBaseType::Type2,
        MsequenceType::Type2V => MsequenceBaseType::Type2,
    }
}

pub fn extract_mc_ckt_bytes(
    buffer: &[u8],
) -> Result<(MsequenceControl, ChecksumMsequenceType), IoLinkError> {
    let mc = MsequenceControl::from(buffer[0]);
    let ckt = ChecksumMsequenceType::from(buffer[1]);
    Ok((mc, ckt))
}

pub fn compile_iolink_startup_frame(
    tx_buffer: &mut Vec<u8, { MAX_TX_FRAME_SIZE }>,
    io_link_message: &IoLinkMessage,
) -> Result<u8, IoLinkError> {
    let mut cks = ChecksumStatus::new();
    cks.set_event_flag(io_link_message.event_flag);
    cks.set_pd_status(io_link_message.pd_status.unwrap_or(PdStatus::INVALID));
    // Index = 0
    if io_link_message.od.is_some() && io_link_message.od.as_ref().unwrap().len() > 0 {
        let od_byte = io_link_message
            .od
            .as_ref()
            .ok_or(IoLinkError::InvalidData)?[0];
        tx_buffer
            .push(od_byte)
            .map_err(|_| IoLinkError::InvalidData)?;
        // Index = 1
        tx_buffer
            .push(cks.into_bits())
            .map_err(|_| IoLinkError::InvalidData)?;
        let checksum = calculate_checksum(2, tx_buffer);
        cks.set_checksum(checksum);
        let tx_buffer_1 = match tx_buffer.get_mut(1) {
            Some(val) => val,
            None => return Err(IoLinkError::InvalidIndex),
        };
        *tx_buffer_1 = cks.into_bits();
    } else {
        tx_buffer
            .push(cks.into_bits())
            .map_err(|_| IoLinkError::InvalidData)?;
        let checksum = calculate_checksum(1, tx_buffer);
        cks.set_checksum(checksum);
        tx_buffer[0] = cks.into_bits();
    }
    Ok(tx_buffer.len() as u8)
}

pub fn compile_iolink_preoperate_frame(
    tx_buffer: &mut Vec<u8, { MAX_TX_FRAME_SIZE }>,
    io_link_message: &IoLinkMessage,
) -> Result<u8, IoLinkError> {
    tx_buffer.clear();
    const OD_LENGTH_BYTES: u8 = derived_config::on_req_data::pre_operate::od_length();
    if io_link_message.od.is_none() {
        return Err(IoLinkError::InvalidData);
    }

    if matches!(io_link_message.read_write, Some(RwDirection::Read)) {
        let od = io_link_message.od.as_ref();
        let od_len = od.map_or(0, |o| o.len());
        if od_len > OD_LENGTH_BYTES as usize {
            return Err(IoLinkError::InvalidData);
        }
        for index in 0..OD_LENGTH_BYTES as usize {
            let byte = if let Some(od) = od {
                if index < od.len() { od[index] } else { 0 }
            } else {
                0
            };
            tx_buffer.push(byte).map_err(|_| IoLinkError::InvalidData)?;
        }
    }

    let mut cks = ChecksumStatus::new();
    cks.set_event_flag(io_link_message.event_flag);
    cks.set_pd_status(io_link_message.pd_status.unwrap_or(PdStatus::INVALID));
    tx_buffer
        .push(cks.into_bits())
        .map_err(|_| IoLinkError::InvalidData)?;
    let checksum = calculate_checksum(OD_LENGTH_BYTES + 1, tx_buffer);
    cks.set_checksum(checksum);
    tx_buffer.pop();
    tx_buffer
        .push(cks.into_bits())
        .map_err(|_| IoLinkError::InvalidData)?;
    Ok(OD_LENGTH_BYTES + 1)
}

pub fn compile_iolink_operate_frame(
    tx_buffer: &mut Vec<u8, { MAX_TX_FRAME_SIZE }>,
    io_link_message: &IoLinkMessage,
) -> Result<u8, IoLinkError> {
    tx_buffer.clear();
    const OD_LENGTH: u8 = derived_config::on_req_data::operate::od_length();
    const PD_LENGTH: u8 = derived_config::process_data::pd_out::config_length_in_bytes();

    if let Some(od) = &io_link_message.od {
        if od.len() > OD_LENGTH as usize {
            return Err(IoLinkError::InvalidData);
        }
        for (i, &byte) in od.iter().enumerate() {
            if i < OD_LENGTH as usize {
                tx_buffer[i] = byte;
            } else {
                break; // Avoid out of bounds access
            }
        }
    } else {
        for i in 0..OD_LENGTH as usize {
            tx_buffer[i] = 0;
        }
    }

    if let Some(pd) = &io_link_message.pd {
        if pd.len() > PD_LENGTH as usize {
            return Err(IoLinkError::InvalidData);
        }
        for (i, &byte) in pd.iter().enumerate() {
            if i < PD_LENGTH as usize {
                tx_buffer[i] = byte;
            } else {
                break; // Avoid out of bounds access
            }
        }
    } else {
        for i in 0..PD_LENGTH as usize {
            tx_buffer[i] = 0;
        }
    }
    const TOTAL_LENGTH: u8 = OD_LENGTH + PD_LENGTH + 1;
    let mut cks = ChecksumStatus::new();
    cks.set_event_flag(io_link_message.event_flag);
    cks.set_pd_status(io_link_message.pd_status.unwrap_or(PdStatus::INVALID));
    tx_buffer[TOTAL_LENGTH as usize] = cks.into_bits();
    let checksum = calculate_checksum(TOTAL_LENGTH, tx_buffer);
    cks.set_checksum(checksum);
    tx_buffer[TOTAL_LENGTH as usize] = cks.into_bits();
    Ok(TOTAL_LENGTH)
}

/// Parse IO-Link frame using nom
/// See IO-Link v1.1.4 Section 6.1
pub fn parse_iolink_startup_frame(
    input: &Vec<u8, { MAX_RX_FRAME_SIZE }>,
) -> IoLinkResult<IoLinkMessage> {
    // Extracting `MC` byte properties
    // Extracting `CKT` byte properties
    let (mc, ckt) = extract_mc_ckt_bytes(input)?;

    if ckt.m_seq_type() != MsequenceBaseType::Type0 {
        return Err(IoLinkError::InvalidMseqType);
    }

    // check for M-sequenceCapability
    // For STARTUP, we expect TYPE_0 (0b00), thus we can check directly
    // because message sequence is always TYPE_0.
    // OD length in startup is always 1 byte
    let od: Option<Vec<u8, { MAX_POSSIBLE_OD_LEN_IN_FRAME as usize }>> = if input.len() > 2 {
        let mut vec = Vec::new();
        vec.push(input[2]).map_err(|_| IoLinkError::InvalidData)?;
        Some(vec)
    } else {
        None
    };

    Ok(IoLinkMessage {
        frame_type: DeviceOperationMode::Startup,
        read_write: Some(mc.read_write()),
        message_type: Some(ckt.m_seq_type()),
        com_channel: Some(mc.comm_channel()),
        address_fctrl: Some(mc.address_fctrl()),
        event_flag: false, // Event flag is not set in startup frame
        od,
        pd: None,        // No PD in startup frame
        pd_status: None, // No PD status in startup frame
    })
}

pub fn parse_iolink_pre_operate_frame(
    input: &Vec<u8, { MAX_RX_FRAME_SIZE }>,
) -> IoLinkResult<IoLinkMessage> {
    // On-request Data (OD) length
    const OD_LENGTH: u8 = derived_config::on_req_data::pre_operate::od_length();
    const M_SEQ_TYPE: MsequenceBaseType =
        derived_config::m_seq_capability::pre_operate_m_sequence::m_sequence_base_type();
    // Extracting `MC` byte properties
    // Extracting `CKT` byte properties
    let (mc, ckt) = extract_mc_ckt_bytes(input)?;

    if M_SEQ_TYPE != ckt.m_seq_type() {
        return Err(IoLinkError::InvalidMseqType);
    }
    let expected_frame_length = if mc.read_write() == RwDirection::Read {
        HEADER_SIZE_IN_FRAME
    } else {
        HEADER_SIZE_IN_FRAME + OD_LENGTH
    };
    if input.len() < expected_frame_length as usize {
        return Err(IoLinkError::InvalidData);
    }
    let mut od: Vec<u8, { MAX_POSSIBLE_OD_LEN_IN_FRAME as usize }> = Vec::new();
    // Extract OD data
    for i in 2..(2 + OD_LENGTH as usize) {
        if i < input.len() {
            od.push(input[i]).map_err(|_| IoLinkError::InvalidData)?;
        } else {
            break; // Avoid out of bounds access
        }
    }

    Ok(IoLinkMessage {
        frame_type: DeviceOperationMode::PreOperate,
        read_write: Some(mc.read_write()),
        message_type: Some(ckt.m_seq_type()),
        com_channel: Some(mc.comm_channel()),
        address_fctrl: Some(mc.address_fctrl()),
        event_flag: false, // Event flag is not set in pre-operate frame
        od: Some(od),
        pd: None,        // No PD in pre-operate frame
        pd_status: None, // No PD status in pre-operate frame
    })
}

pub fn parse_iolink_operate_frame(
    input: &Vec<u8, { MAX_RX_FRAME_SIZE }>,
) -> IoLinkResult<IoLinkMessage> {
    const OD_LENGTH_OCTETS: u8 = derived_config::on_req_data::operate::od_length();
    const PD_LENGTH: u8 = derived_config::process_data::pd_out::config_length_in_bytes();
    const M_SEQ_TYPE: MsequenceBaseType =
        derived_config::m_seq_capability::operate_m_sequence::m_sequence_base_type();

    // Extracting `MC` byte properties
    // Extracting `CKT` byte properties
    let (mc, ckt) = extract_mc_ckt_bytes(input)?;

    if input.len() != (HEADER_SIZE_IN_FRAME + OD_LENGTH_OCTETS + PD_LENGTH) as usize {
        return Err(IoLinkError::InvalidData);
    }
    let mut od: Vec<u8, { MAX_POSSIBLE_OD_LEN_IN_FRAME as usize }> = Vec::new();
    let mut pd: Vec<u8, { PD_OUT_LENGTH as usize }> = Vec::new();

    for i in 2..(2 + OD_LENGTH_OCTETS as usize) {
        if i < input.len() {
            od.push(input[i]).map_err(|_| IoLinkError::InvalidData)?;
        } else {
            break; // Avoid out of bounds access
        }
    }

    for i in (2 + OD_LENGTH_OCTETS as usize)..(2 + OD_LENGTH_OCTETS as usize + PD_LENGTH as usize) {
        if i < input.len() {
            pd.push(input[i]).map_err(|_| IoLinkError::InvalidData)?;
        } else {
            break; // Avoid out of bounds access
        }
    }

    Ok(IoLinkMessage {
        frame_type: DeviceOperationMode::Operate,
        read_write: Some(mc.read_write()),
        message_type: Some(ckt.m_seq_type()),
        com_channel: Some(mc.comm_channel()),
        address_fctrl: Some(mc.address_fctrl()),
        event_flag: false, // Event flag is not set in pre-operate frame
        od: Some(od),
        pd: Some(pd),
        pd_status: None, // No PD status in pre-operate frame
    })
}

pub fn validate_master_frame_checksum(length: u8, data: &mut [u8]) -> bool {
    // Validate the checksum of the received IO-Link message
    let (_, ckt) = match extract_mc_ckt_bytes(data) {
        Ok(val) => val,
        Err(_) => return false,
    };
    let checksum = ckt.checksum();
    // clear the received checksum bits (0-5), Before calculating the checksum
    data[1] = crate::clear_checksum_bits_0_to_5!(data[1]);
    let calculated_checksum = calculate_checksum(length, &data);
    calculated_checksum == checksum
}

/// See A.1.6 Calculation of the checksum
/// Calculate message checksum
#[cfg(any(test, feature = "std"))]
pub fn calculate_checksum_for_testing(length: u8, data: &[u8]) -> u8 {
    calculate_checksum(length, data)
}

fn calculate_checksum(length: u8, data: &[u8]) -> u8 {
    #[bitfield(u8)]
    struct FullChecksumBits {
        #[bits(1)]
        bit0: u8,
        #[bits(1)]
        bit1: u8,
        #[bits(1)]
        bit2: u8,
        #[bits(1)]
        bit3: u8,
        #[bits(1)]
        bit4: u8,
        #[bits(1)]
        bit5: u8,
        #[bits(1)]
        bit6: u8,
        #[bits(1)]
        bit7: u8,
    }

    #[bitfield(u8)]
    struct ChecksumBits {
        #[bits(1)]
        bit0: u8,
        #[bits(1)]
        bit1: u8,
        #[bits(1)]
        bit2: u8,
        #[bits(1)]
        bit3: u8,
        #[bits(1)]
        bit4: u8,
        #[bits(1)]
        bit5: u8,
        #[bits(2)]
        __: u8,
    }
    // Seed value as per IO-Link spec
    let mut checksum = 0x52u8;
    for i in 0..length as usize {
        if i < data.len() {
            checksum ^= data[i];
        }
    }
    let full_checksum_bits = FullChecksumBits::from(checksum);
    let d_bit0 = full_checksum_bits.bit0();
    let d_bit1 = full_checksum_bits.bit1();
    let d_bit2 = full_checksum_bits.bit2();
    let d_bit3 = full_checksum_bits.bit3();
    let d_bit4 = full_checksum_bits.bit4();
    let d_bit5 = full_checksum_bits.bit5();
    let d_bit6 = full_checksum_bits.bit6();
    let d_bit7 = full_checksum_bits.bit7();

    let mut checksum_bits = ChecksumBits::new();
    checksum_bits.set_bit0(d_bit1 ^ d_bit0);
    checksum_bits.set_bit1(d_bit3 ^ d_bit2);
    checksum_bits.set_bit2(d_bit5 ^ d_bit4);
    checksum_bits.set_bit3(d_bit7 ^ d_bit6);
    checksum_bits.set_bit4(d_bit6 ^ d_bit4 ^ d_bit2 ^ d_bit0);
    checksum_bits.set_bit5(d_bit7 ^ d_bit5 ^ d_bit3 ^ d_bit1);

    checksum_bits.into_bits()
}

/// Mask to set bits 0-5 to zero while preserving bits 6-7
/// This macro clears the revceived checksum bits (0-5) in a byte,
/// leaving bits 6 and 7 unchanged.
#[macro_export]
macro_rules! clear_checksum_bits_0_to_5 {
    ($byte:expr) => {
        ($byte) & 0b11000000u8
    };
}
