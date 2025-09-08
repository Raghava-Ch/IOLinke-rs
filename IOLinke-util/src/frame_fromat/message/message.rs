use bitfields::bitfield;
use heapless::Vec;
use iolinke_derived_config::device as derived_config;
use iolinke_types::{
    custom::{IoLinkError, IoLinkResult},
    frame::msequence::{
        ChecksumMsequenceType, ChecksumStatus, ComChannel, MsequenceBaseType, MsequenceControl,
        PdStatus, RwDirection, TransmissionRate,
    },
};

pub const HEADER_SIZE_IN_FRAME: u8 = 2; // Header size is 2 bytes (MC and length)
/// Maximum message buffer size for OD
/// This is the maximum size of the message buffer used for OD messages in operating modes.
pub const MAX_POSSIBLE_OD_LEN_IN_FRAME: u8 = derived_config::on_req_data::max_possible_od_length();
/// Maximum message buffer size for PD
/// This is the maximum size of the message buffer used for PD messages.
pub const PD_IN_LENGTH: u8 = derived_config::process_data::pd_in::config_length_in_bytes();
pub const PD_OUT_LENGTH: u8 = derived_config::process_data::pd_out::config_length_in_bytes();
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
    pub pd_status: PdStatus,
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
            pd_status: PdStatus::INVALID,
        }
    }
}

pub enum MessageBufferError {
    NotEnoughMemory,
    InvalidIndex,
    InvalidLength,
    OdNotAvailable,
    PdNotAvailable,
    OdNotSet,
    PdNotSet,
    InvalidData,
    InvalidChecksum,
    InvalidMseqType,
    InvalidRwDirection,
    NotReady,
    InvalidDeviceOperationMode,
}

pub type MessageBufferResult<T> = Result<T, MessageBufferError>;
pub trait StartupTxMessageBuffer {
    fn insert_od(&mut self, od: &[u8]) -> MessageBufferResult<()>;
    fn compile_read_rsp(
        &mut self,
        event_flag: bool,
        pd_status: PdStatus,
    ) -> MessageBufferResult<&[u8]>;
    fn compile_write_rsp(
        &mut self,
        event_flag: bool,
        pd_status: PdStatus,
    ) -> MessageBufferResult<&[u8]>;
}

trait PreOperateTxMessageBuffer {
    fn insert_od(&mut self, od: &[u8]) -> MessageBufferResult<()>;
    fn compile_read_rsp(
        &mut self,
        event_flag: bool,
        pd_status: PdStatus,
    ) -> MessageBufferResult<&[u8]>;
    fn compile_write_rsp(
        &mut self,
        event_flag: bool,
        pd_status: PdStatus,
    ) -> MessageBufferResult<&[u8]>;
}

trait OperateTxMessageBuffer {
    fn insert_od(&mut self, od: &[u8]) -> MessageBufferResult<()>;
    fn insert_pd(&mut self, pd: &[u8]) -> MessageBufferResult<()>;
    fn compile_read_rsp(
        &mut self,
        event_flag: bool,
        pd_status: PdStatus,
    ) -> MessageBufferResult<&[u8]>;
    fn compile_write_rsp(
        &mut self,
        event_flag: bool,
        pd_status: PdStatus,
    ) -> MessageBufferResult<&[u8]>;
}

trait StartupRxMessageBuffer {
    fn valid_req(&mut self) -> MessageBufferResult<RwDirection>;
    fn extract_od_from_write_req(&self) -> MessageBufferResult<&[u8]>;
    fn expected_bytes(&self) -> MessageBufferResult<u8>;
}

trait PreOperateRxMessageBuffer {
    fn valid_req(&mut self) -> MessageBufferResult<RwDirection>;
    fn extract_od_from_write_req(&self) -> MessageBufferResult<&[u8]>;
    fn expected_bytes(&self) -> MessageBufferResult<u8>;
}

trait OperateRxMessageBuffer {
    fn valid_req(&mut self) -> MessageBufferResult<RwDirection>;
    fn extract_od_from_write_req(&self) -> MessageBufferResult<&[u8]>;
    fn extract_pd(&mut self) -> MessageBufferResult<&[u8]>;
    fn expected_bytes(&self) -> MessageBufferResult<u8>;
}

#[derive(Debug, Clone)]
pub struct TxMessageBuffer<const BUFF_LEN: usize> {
    buffer: Vec<u8, BUFF_LEN>,
    length: usize,
    od_ready: bool,
    pd_ready: bool,
    tx_ready: bool,
}

#[derive(Debug, Clone)]
pub struct RxMessageBuffer<const BUFF_LEN: usize> {
    buffer: Vec<u8, BUFF_LEN>,
    length: usize,
}

impl<const BUFF_LEN: usize> TxMessageBuffer<BUFF_LEN> {
    pub fn new() -> Self {
        let mut buffer: Vec<u8, BUFF_LEN> = Vec::new();
        let _ = buffer
            .extend_from_slice(&[0; BUFF_LEN])
            .map_err(|_| IoLinkError::NotEnoughMemory);
        Self {
            buffer: buffer,
            length: 0,
            od_ready: false,
            pd_ready: false,
            tx_ready: false,
        }
    }

    pub fn clear(&mut self) {
        self.length = 0;
        self.buffer.clear();
        let _ = self.buffer.extend_from_slice(&[0; BUFF_LEN]);
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn get_as_slice(&self) -> &[u8] {
        &self.buffer[0..self.length]
    }

    pub fn insert_od(
        &mut self,
        od: &[u8],
        device_mode: DeviceOperationMode,
    ) -> MessageBufferResult<()> {
        match device_mode {
            DeviceOperationMode::Startup => <Self as StartupTxMessageBuffer>::insert_od(self, od),
            DeviceOperationMode::PreOperate => {
                <Self as PreOperateTxMessageBuffer>::insert_od(self, od)
            }
            DeviceOperationMode::Operate => <Self as OperateTxMessageBuffer>::insert_od(self, od),
        }
    }

    pub fn insert_pd(
        &mut self,
        pd: &[u8],
        device_mode: DeviceOperationMode,
    ) -> MessageBufferResult<()> {
        match device_mode {
            DeviceOperationMode::Operate => <Self as OperateTxMessageBuffer>::insert_pd(self, pd),
            _ => Err(MessageBufferError::InvalidDeviceOperationMode),
        }
    }

    pub fn is_ready(&self, device_mode: DeviceOperationMode) -> bool {
        match device_mode {
            DeviceOperationMode::Startup => self.od_ready,
            DeviceOperationMode::PreOperate => self.od_ready,
            DeviceOperationMode::Operate => self.od_ready && self.pd_ready,
        }
    }

    /// Compile IO-Link message from buffer
    /// See IO-Link v1.1.4 `Annex A` (normative)
    /// Codings, timing constraints, and errors
    /// A.1 General structure and encoding of M-sequences
    pub fn compile_message_rsp(
        &mut self,
        operation_mode: DeviceOperationMode,
        rw_req_dir: RwDirection,
        event_flag: bool,
        pd_status: PdStatus,
    ) -> IoLinkResult<()> {
        let _length = match (operation_mode, rw_req_dir) {
            (DeviceOperationMode::Startup, RwDirection::Read) => {
                <Self as StartupTxMessageBuffer>::compile_read_rsp(self, event_flag, pd_status)
                    .map_err(|_| IoLinkError::InvalidParameter)?
            }
            (DeviceOperationMode::PreOperate, RwDirection::Read) => {
                <Self as PreOperateTxMessageBuffer>::compile_read_rsp(self, event_flag, pd_status)
                    .map_err(|_| IoLinkError::InvalidParameter)?
            }
            (DeviceOperationMode::Operate, RwDirection::Read) => {
                <Self as OperateTxMessageBuffer>::compile_read_rsp(self, event_flag, pd_status)
                    .map_err(|_| IoLinkError::InvalidParameter)?
            }
            (DeviceOperationMode::Startup, RwDirection::Write) => {
                <Self as StartupTxMessageBuffer>::compile_write_rsp(self, event_flag, pd_status)
                    .map_err(|_| IoLinkError::InvalidParameter)?
            }
            (DeviceOperationMode::PreOperate, RwDirection::Write) => {
                <Self as PreOperateTxMessageBuffer>::compile_write_rsp(self, event_flag, pd_status)
                    .map_err(|_| IoLinkError::InvalidParameter)?
            }
            (DeviceOperationMode::Operate, RwDirection::Write) => {
                <Self as OperateTxMessageBuffer>::compile_write_rsp(self, event_flag, pd_status)
                    .map_err(|_| IoLinkError::InvalidParameter)?
            }
        };
        Ok(())
    }
}

impl<const BUFF_LEN: usize> RxMessageBuffer<BUFF_LEN> {
    pub fn new() -> Self {
        let mut buffer: Vec<u8, BUFF_LEN> = Vec::new();
        let _ = buffer
            .extend_from_slice(&[0; BUFF_LEN])
            .map_err(|_| IoLinkError::NotEnoughMemory);
        Self {
            buffer: buffer,
            length: 0,
        }
    }

    pub fn clear(&mut self) {
        self.length = 0;
        self.buffer.clear();
        let _ = self.buffer.extend_from_slice(&[0; BUFF_LEN]);
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn push(&mut self, data: u8) -> MessageBufferResult<()> {
        if self.length + 1 > BUFF_LEN {
            return Err(MessageBufferError::InvalidLength);
        }
        self.buffer[self.length] = data;
        self.length += 1;
        Ok(())
    }

    pub fn extract_mc(&self) -> MessageBufferResult<MsequenceControl> {
        let mc = MsequenceControl::from(self.buffer[0]);
        Ok(mc)
    }

    pub fn valid_req(&mut self, device_mode: DeviceOperationMode) -> MessageBufferResult<RwDirection> {
        match device_mode {
            DeviceOperationMode::Startup => <Self as StartupRxMessageBuffer>::valid_req(self),
            DeviceOperationMode::PreOperate => <Self as PreOperateRxMessageBuffer>::valid_req(self),
            DeviceOperationMode::Operate => <Self as OperateRxMessageBuffer>::valid_req(self),
        }
    }

    pub fn extract_od_from_write_req(&mut self, device_mode: DeviceOperationMode) -> MessageBufferResult<&[u8]> {
        match device_mode {
            DeviceOperationMode::Startup => <Self as StartupRxMessageBuffer>::extract_od_from_write_req(self),
            DeviceOperationMode::PreOperate => <Self as PreOperateRxMessageBuffer>::extract_od_from_write_req(self),
            DeviceOperationMode::Operate => <Self as OperateRxMessageBuffer>::extract_od_from_write_req(self),
        }
    }

    pub fn extract_pd(&mut self) -> MessageBufferResult<&[u8]> {
        <Self as OperateRxMessageBuffer>::extract_pd(self)
    }

    pub fn calculate_expected_rx_bytes(&self, device_mode: DeviceOperationMode) -> u8 {
        match device_mode {
            DeviceOperationMode::Startup => {
                match <Self as StartupRxMessageBuffer>::expected_bytes(self) {
                    Ok(bytes) => bytes,
                    Err(_) => HEADER_SIZE_IN_FRAME,
                }
            }
            DeviceOperationMode::PreOperate => {
                match <Self as PreOperateRxMessageBuffer>::expected_bytes(self) {
                    Ok(bytes) => bytes,
                    Err(_) => HEADER_SIZE_IN_FRAME,
                }
            }
            DeviceOperationMode::Operate => {
                match <Self as OperateRxMessageBuffer>::expected_bytes(self) {
                    Ok(bytes) => bytes,
                    Err(_) => HEADER_SIZE_IN_FRAME,
                }
            }
        }
    }

    pub fn get_as_slice(&self) -> &[u8] {
        &self.buffer[0..self.length]
    }
}

impl<const BUFF_LEN: usize> StartupTxMessageBuffer for TxMessageBuffer<BUFF_LEN> {
    fn insert_od(&mut self, od: &[u8]) -> MessageBufferResult<()> {
        if self.length + od.len() > BUFF_LEN {
            return Err(MessageBufferError::InvalidLength);
        }
        if 1 != od.len() {
            // For startup mode, only one OD byte is used
            return Err(MessageBufferError::InvalidData);
        }
        const OD_START: usize = 0;
        const OD_END: usize = 1;
        self.buffer[OD_START..OD_END].copy_from_slice(od);
        self.length += 1;
        self.od_ready = true;

        Ok(())
    }

    fn compile_read_rsp(
        &mut self,
        event_flag: bool,
        pd_status: PdStatus,
    ) -> MessageBufferResult<&[u8]> {
        if !self.od_ready {
            return Err(MessageBufferError::OdNotSet);
        }
        const CKS_INDEX: usize = 1;
        let mut cks = ChecksumStatus::new();
        cks.set_event_flag(event_flag);
        cks.set_pd_status(pd_status);
        self.buffer[CKS_INDEX] = cks.into_bits();
        self.length += 1;
        let checksum = calculate_checksum(self.length, &self.buffer);
        cks.set_checksum(checksum);
        self.buffer[CKS_INDEX] = cks.into_bits();
        self.tx_ready = true;
        Ok(&self.buffer[0..self.length])
    }

    fn compile_write_rsp(
        &mut self,
        event_flag: bool,
        pd_status: PdStatus,
    ) -> MessageBufferResult<&[u8]> {
        if !self.od_ready {
            return Err(MessageBufferError::OdNotSet);
        }
        const CKS_INDEX: usize = 0;
        let mut cks = ChecksumStatus::new();
        cks.set_event_flag(event_flag);
        cks.set_pd_status(pd_status);
        self.buffer[CKS_INDEX] = cks.into_bits();
        self.length += 1;
        let checksum = calculate_checksum(self.length, &self.buffer);
        cks.set_checksum(checksum);
        self.buffer[CKS_INDEX] = cks.into_bits();
        self.tx_ready = true;
        Ok(&self.buffer[0..self.length])
    }
}

impl<const BUFF_LEN: usize> PreOperateTxMessageBuffer for TxMessageBuffer<BUFF_LEN> {
    fn insert_od(&mut self, od: &[u8]) -> MessageBufferResult<()> {
        if self.length + od.len() > BUFF_LEN {
            return Err(MessageBufferError::InvalidLength);
        }
        const OD_LENGTH: usize = derived_config::on_req_data::pre_operate::od_length() as usize;
        if OD_LENGTH != od.len() {
            return Err(MessageBufferError::InvalidData);
        }
        const OD_START: usize = 0;
        const OD_END: usize = OD_LENGTH;
        self.buffer[OD_START..OD_END].copy_from_slice(od);
        self.length += OD_LENGTH;
        self.od_ready = true;
        Ok(())
    }

    fn compile_read_rsp(
        &mut self,
        event_flag: bool,
        pd_status: PdStatus,
    ) -> MessageBufferResult<&[u8]> {
        if !self.od_ready {
            return Err(MessageBufferError::OdNotSet);
        }
        const OD_LENGTH: usize = derived_config::on_req_data::pre_operate::od_length() as usize;
        const CKS_INDEX: usize = OD_LENGTH;
        let mut cks = ChecksumStatus::new();
        cks.set_event_flag(event_flag);
        cks.set_pd_status(pd_status);
        self.buffer[CKS_INDEX] = cks.into_bits();
        self.length += 1;
        let checksum = calculate_checksum(self.length, &self.buffer);
        cks.set_checksum(checksum);
        self.buffer[CKS_INDEX] = cks.into_bits();
        self.tx_ready = true;
        Ok(&self.buffer[0..self.length])
    }

    fn compile_write_rsp(
        &mut self,
        event_flag: bool,
        pd_status: PdStatus,
    ) -> MessageBufferResult<&[u8]> {
        <Self as StartupTxMessageBuffer>::compile_write_rsp(self, event_flag, pd_status)
    }
}

impl<const BUFF_LEN: usize> OperateTxMessageBuffer for TxMessageBuffer<BUFF_LEN> {
    fn insert_od(&mut self, od: &[u8]) -> MessageBufferResult<()> {
        if self.length + od.len() > BUFF_LEN {
            return Err(MessageBufferError::InvalidLength);
        }
        const OD_LENGTH: usize = derived_config::on_req_data::operate::od_length() as usize;
        if OD_LENGTH != od.len() {
            return Err(MessageBufferError::InvalidData);
        }
        const OD_START: usize = 0;
        const OD_END: usize = OD_LENGTH;
        self.buffer[OD_START..OD_END].copy_from_slice(od);
        self.length += OD_LENGTH;
        self.od_ready = true;
        Ok(())
    }

    fn insert_pd(&mut self, pd: &[u8]) -> MessageBufferResult<()> {
        if self.length + pd.len() > BUFF_LEN {
            return Err(MessageBufferError::InvalidLength);
        }
        const PD_LENGTH: usize =
            derived_config::process_data::pd_out::config_length_in_bytes() as usize;
        const OD_LENGTH: usize = derived_config::on_req_data::operate::od_length() as usize;
        if PD_LENGTH != pd.len() {
            return Err(MessageBufferError::InvalidData);
        }
        const PD_START: usize = OD_LENGTH;
        const PD_END: usize = OD_LENGTH + PD_LENGTH;
        self.buffer[PD_START..PD_END].copy_from_slice(pd);
        self.length += PD_LENGTH;
        self.pd_ready = true;
        Ok(())
    }

    fn compile_read_rsp(
        &mut self,
        event_flag: bool,
        pd_status: PdStatus,
    ) -> MessageBufferResult<&[u8]> {
        if !self.od_ready {
            return Err(MessageBufferError::OdNotSet);
        }
        if !self.pd_ready {
            return Err(MessageBufferError::PdNotSet);
        }
        const OD_LENGTH: usize = derived_config::on_req_data::operate::od_length() as usize;
        const PD_LENGTH: usize =
            derived_config::process_data::pd_out::config_length_in_bytes() as usize;
        const CKS_INDEX: usize = OD_LENGTH + PD_LENGTH;
        let mut cks = ChecksumStatus::new();
        cks.set_event_flag(event_flag);
        cks.set_pd_status(pd_status);
        self.buffer[CKS_INDEX] = cks.into_bits();
        self.length += 1;
        let checksum = calculate_checksum(self.length, &self.buffer);
        cks.set_checksum(checksum);
        self.buffer[CKS_INDEX] = cks.into_bits();
        self.tx_ready = true;
        Ok(&self.buffer[0..self.length])
    }

    fn compile_write_rsp(
        &mut self,
        event_flag: bool,
        pd_status: PdStatus,
    ) -> MessageBufferResult<&[u8]> {
        if !self.pd_ready {
            return Err(MessageBufferError::PdNotSet);
        }
        const OD_LENGTH: usize = derived_config::on_req_data::operate::od_length() as usize;
        const PD_LENGTH: usize =
            derived_config::process_data::pd_out::config_length_in_bytes() as usize;
        const CKS_INDEX: usize = OD_LENGTH + PD_LENGTH;
        let mut cks = ChecksumStatus::new();
        cks.set_event_flag(event_flag);
        cks.set_pd_status(pd_status);
        self.buffer[CKS_INDEX] = cks.into_bits();
        self.length += 1;
        let checksum = calculate_checksum(self.length, &self.buffer);
        cks.set_checksum(checksum);
        self.buffer[CKS_INDEX] = cks.into_bits();
        self.tx_ready = true;
        Ok(&self.buffer[0..self.length])
    }
}

impl<const BUFF_LEN: usize> StartupRxMessageBuffer for RxMessageBuffer<BUFF_LEN> {
    fn valid_req(&mut self) -> MessageBufferResult<RwDirection> {
        if validate_master_frame_checksum(self.length, self.buffer.as_mut_slice()) {
            let (mc, ckt) = extract_mc_ckt_bytes(self.buffer.as_slice())
                .map_err(|_| MessageBufferError::InvalidChecksum)?;
            if ckt.m_seq_type() != MsequenceBaseType::Type0 {
                return Err(MessageBufferError::InvalidMseqType);
            }
            Ok(mc.read_write())
        } else {
            Err(MessageBufferError::InvalidChecksum)
        }
    }
    fn extract_od_from_write_req(&self) -> MessageBufferResult<&[u8]> {
        let mc = Self::extract_mc(self).map_err(|_| MessageBufferError::InvalidMseqType)?;
        if mc.read_write() == RwDirection::Read {
            return Err(MessageBufferError::InvalidRwDirection);
        }
        const OD_LENGTH: usize = 1;

        const OD_START: usize = HEADER_SIZE_IN_FRAME as usize;
        const OD_END: usize = OD_START + OD_LENGTH;
        let od = &self.buffer[OD_START..OD_END];
        Ok(od)
    }
    fn expected_bytes(&self) -> MessageBufferResult<u8> {
        if self.length < HEADER_SIZE_IN_FRAME as usize {
            return Err(MessageBufferError::NotReady);
        }
        let mc = Self::extract_mc(self).map_err(|_| MessageBufferError::NotReady)?;
        if mc.read_write() == RwDirection::Read {
            return Ok(HEADER_SIZE_IN_FRAME);
        }
        const EXPECTED_BYTES: u8 = HEADER_SIZE_IN_FRAME + 1;
        Ok(EXPECTED_BYTES)
    }
}

impl<const BUFF_LEN: usize> PreOperateRxMessageBuffer for RxMessageBuffer<BUFF_LEN> {
    fn valid_req(&mut self) -> MessageBufferResult<RwDirection> {
        <Self as StartupRxMessageBuffer>::valid_req(self)
    }
    fn extract_od_from_write_req(&self) -> MessageBufferResult<&[u8]> {
        let mc = Self::extract_mc(self).map_err(|_| MessageBufferError::InvalidMseqType)?;
        if mc.read_write() == RwDirection::Read {
            return Err(MessageBufferError::InvalidRwDirection);
        }
        const OD_LENGTH: usize = derived_config::on_req_data::pre_operate::od_length() as usize;

        const OD_START: usize = HEADER_SIZE_IN_FRAME as usize;
        const OD_END: usize = OD_START + OD_LENGTH;
        let od = &self.buffer[OD_START..OD_END];
        Ok(od)
    }
    fn expected_bytes(&self) -> MessageBufferResult<u8> {
        if self.length < HEADER_SIZE_IN_FRAME as usize {
            return Err(MessageBufferError::NotReady);
        }
        let mc = Self::extract_mc(self).map_err(|_| MessageBufferError::NotReady)?;
        if mc.read_write() == RwDirection::Read {
            return Ok(HEADER_SIZE_IN_FRAME);
        }
        const EXPECTED_BYTES: u8 =
            HEADER_SIZE_IN_FRAME + derived_config::on_req_data::pre_operate::od_length();
        Ok(EXPECTED_BYTES)
    }
}

impl<const BUFF_LEN: usize> OperateRxMessageBuffer for RxMessageBuffer<BUFF_LEN> {
    fn valid_req(&mut self) -> MessageBufferResult<RwDirection> {
        <Self as StartupRxMessageBuffer>::valid_req(self)
    }
    fn extract_od_from_write_req(&self) -> MessageBufferResult<&[u8]> {
        let mc = Self::extract_mc(self).map_err(|_| MessageBufferError::InvalidMseqType)?;
        if mc.read_write() == RwDirection::Read {
            return Err(MessageBufferError::InvalidRwDirection);
        }
        const OD_LENGTH: usize = derived_config::on_req_data::operate::od_length() as usize;
        const PD_LENGTH: usize =
            derived_config::process_data::pd_out::config_length_in_bytes() as usize;

        const OD_START: usize = HEADER_SIZE_IN_FRAME as usize + PD_LENGTH;
        const OD_END: usize = OD_START + OD_LENGTH;
        let od = &self.buffer[OD_START..OD_END];
        Ok(od)
    }
    fn extract_pd(&mut self) -> MessageBufferResult<&[u8]> {
        const PD_LENGTH: usize =
            derived_config::process_data::pd_out::config_length_in_bytes() as usize;
        const PD_START: usize = HEADER_SIZE_IN_FRAME as usize + PD_LENGTH;
        const PD_END: usize = PD_START + PD_LENGTH;
        let pd = &self.buffer[PD_START..PD_END];
        Ok(pd)
    }
    fn expected_bytes(&self) -> MessageBufferResult<u8> {
        if self.length < HEADER_SIZE_IN_FRAME as usize {
            return Err(MessageBufferError::NotReady);
        }
        let mc = Self::extract_mc(self).map_err(|_| MessageBufferError::NotReady)?;
        const OD_LENGTH: u8 = derived_config::on_req_data::operate::od_length();
        const PD_LENGTH: u8 = derived_config::process_data::pd_out::config_length_in_bytes();
        if mc.read_write() == RwDirection::Read {
            return Ok(HEADER_SIZE_IN_FRAME + PD_LENGTH);
        }
        const EXPECTED_BYTES: u8 = HEADER_SIZE_IN_FRAME + PD_LENGTH + OD_LENGTH;
        Ok(EXPECTED_BYTES)
    }
}

pub fn calculate_max_uart_frame_time(transmission_rate: TransmissionRate) -> u32 {
    const NUM_OF_BITS_PER_FRAME: u32 = 12;
    // MaxUARTFrameTime Time for the transmission of a UART frame (11 TBIT) plus maximum of t1 (1 TBIT) = 12 TBIT.
    let max_uart_frame_time =
        TransmissionRate::get_t_bit_in_us(transmission_rate) * NUM_OF_BITS_PER_FRAME;
    max_uart_frame_time
}

pub fn extract_mc_ckt_bytes(
    buffer: &[u8],
) -> Result<(MsequenceControl, ChecksumMsequenceType), IoLinkError> {
    let mc = MsequenceControl::from(buffer[0]);
    let ckt = ChecksumMsequenceType::from(buffer[1]);
    Ok((mc, ckt))
}

pub fn validate_master_frame_checksum(length: usize, data: &mut [u8]) -> bool {
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
pub fn calculate_checksum_for_testing(length: usize, data: &[u8]) -> u8 {
    calculate_checksum(length, data)
}

fn calculate_checksum(length: usize, data: &[u8]) -> u8 {
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
    for i in 0..length {
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
