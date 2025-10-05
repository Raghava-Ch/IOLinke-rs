//! # IO-Link Message Buffer Module
//!
//! This module provides types and utilities for handling IO-Link message frames, including
//! transmit and receive buffers, frame encoding/decoding, and checksum validation.
//!
//! ## Features
//!
//! - **Device Operation Modes:** Support for Startup, PreOperate, and Operate modes, with
//!   mode-specific message buffer handling.
//! - **Message Buffers:** Typed transmit (`TxMessageBuffer`) and receive (`RxMessageBuffer`) buffers
//!   for IO-Link frames, using fixed-size heapless vectors.
//! - **Object Dictionary (OD) and Process Data (PD):** Insertion and extraction of OD and PD data
//!   according to the current device mode.
//! - **Checksum Calculation and Validation:** Implements IO-Link v1.1.4 Annex A checksum algorithm
//!   for frame integrity.
//! - **Trait-based Mode Handling:** Traits for mode-specific buffer operations, enabling
//!   compile-time enforcement of protocol rules.
//! - **Error Handling:** Rich error types for buffer operations, including invalid length, data,
//!   checksum, and device mode errors.
//! - **Timing Utilities:** Calculation of maximum UART frame transmission time based on baud rate.
//!
//! ## Usage
//!
//! Use the provided buffer types and trait implementations to encode, decode, and validate IO-Link
//! frames in your device firmware or protocol stack. The module is designed for embedded,
//! no_std environments and integrates with derived device configuration.
//!
//! ## References
//!
//! - IO-Link Specification v1.1.4, Annex A: General structure and encoding of M-sequences
//!
//! ## Example
//!
//! ```rust
//! use iolinke_util::frame_format::message::*;
//!
//! let mut tx_buffer = TxMessageBuffer::<MAX_TX_FRAME_SIZE>::new();
//! tx_buffer.insert_od(od_length, &od_data, DeviceOperationMode::Operate)?;
//! tx_buffer.insert_pd(&pd_data, DeviceOperationMode::Operate)?;
//! tx_buffer.compile_message_rsp(DeviceOperationMode::Operate, RwDirection::Read, true, PdStatus::Ok)?;
//! let frame = tx_buffer.get_as_slice();
//! ```
//!

use bitfields::bitfield;
use heapless::Vec;
use iolinke_derived_config::device as derived_config;
use iolinke_types::{
    custom::{IoLinkError, IoLinkResult},
    frame::msequence::{
        ChecksumMsequenceType, ChecksumStatus, MsequenceBaseType, MsequenceControl, PdStatus,
        RwDirection, TransmissionRate,
    },
};

use core::convert::From;
use core::default::Default;
use core::result::{
    Result,
    Result::{Err, Ok},
};
use core::stringify;

/// Header size in any IO-Link frame
pub const HEADER_SIZE_IN_FRAME: u8 = 2; // Header size is 2 bytes (MC and length)
/// Maximum message buffer size for OD
/// This is the maximum size of the message buffer used for OD messages in operating modes.
pub const MAX_POSSIBLE_OD_LEN_IN_FRAME: u8 = derived_config::on_req_data::max_possible_od_length();
/// Maximum message buffer size for PD
/// This is the maximum size of the message buffer used for PD messages.
pub const PD_IN_LENGTH: u8 = derived_config::process_data::pd_in::config_length_in_bytes();
/// Maximum message buffer size for PD
pub const PD_OUT_LENGTH: u8 = derived_config::process_data::pd_out::config_length_in_bytes();
/// Maximum frame size for IO-Link messages
pub const MAX_RX_FRAME_SIZE: usize =
    (MAX_POSSIBLE_OD_LEN_IN_FRAME + PD_OUT_LENGTH + HEADER_SIZE_IN_FRAME) as usize;
/// Maximum frame size for IO-Link messages
pub const MAX_TX_FRAME_SIZE: usize =
    (MAX_POSSIBLE_OD_LEN_IN_FRAME + PD_IN_LENGTH + HEADER_SIZE_IN_FRAME) as usize;

/// Device main operation modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceOperationMode {
    /// Startup mode
    Startup,
    /// Pre-operate mode
    PreOperate,
    /// Operate mode
    Operate,
}

/// Error type for MessageBuffer operations
pub enum MessageBufferError {
    /// Not enough memory available to allocate buffer
    NotEnoughMemory,
    /// Invalid index specified for buffer access
    InvalidIndex,
    /// Invalid length specified for buffer operation
    InvalidLength,
    /// Object Dictionary (OD) not available
    OdNotAvailable,
    /// Process Data (PD) not available
    PdNotAvailable,
    /// Object Dictionary (OD) not set
    OdNotSet,
    /// Process Data (PD) not set
    PdNotSet,
    /// Invalid data received
    InvalidData,
    /// Checksum validation failed
    InvalidChecksum,
    /// Invalid M-sequence type
    InvalidMseqType,
    /// Invalid read/write direction
    InvalidRwDirection,
    /// Not ready for communication
    NotReady,
    /// Invalid device operation mode
    InvalidDeviceOperationMode,
}

/// Result type for MessageBuffer operations
pub type MessageBufferResult<T> = Result<T, MessageBufferError>;

/// Startup mode message buffer trait
pub trait StartupTxMessageBuffer {
    /// Inserts Object Dictionary (OD) data into the message buffer.
    ///
    /// # Parameters
    /// - `od_length`: Length of the OD data to insert.
    /// - `od`: Slice containing the OD data.
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(MessageBufferError)` if an error occurs (e.g., invalid length).
    fn insert_od(&mut self, od_length: usize, od: &[u8]) -> MessageBufferResult<()>;

    /// Compiles a read response message into the buffer.
    fn compile_read_rsp(
        &mut self,
        event_flag: bool,
        pd_status: PdStatus,
    ) -> MessageBufferResult<&[u8]>;

    /// Compiles a write response message into the buffer.
    fn compile_write_rsp(
        &mut self,
        event_flag: bool,
        pd_status: PdStatus,
    ) -> MessageBufferResult<&[u8]>;
}

trait PreOperateTxMessageBuffer {
    fn insert_od(&mut self, od_length: usize, od: &[u8]) -> MessageBufferResult<()>;
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
    fn insert_od(&mut self, od_length: usize, od: &[u8]) -> MessageBufferResult<()>;
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

/// Message buffer for transmitting IO-Link messages
#[derive(Debug, Clone)]
pub struct TxMessageBuffer<const BUFF_LEN: usize> {
    buffer: Vec<u8, BUFF_LEN>,
    length: usize,
    od_ready: bool,
    pd_ready: bool,
    tx_ready: bool,
}

/// Message buffer for receiving IO-Link messages
#[derive(Debug, Clone)]
pub struct RxMessageBuffer<const BUFF_LEN: usize> {
    buffer: Vec<u8, BUFF_LEN>,
    length: usize,
}

impl<const BUFF_LEN: usize> TxMessageBuffer<BUFF_LEN> {
    /// Creates a new `TxMessageBuffer` instance.
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

    /// Clears the message buffer and resets its state.
    pub fn clear(&mut self) {
        self.length = 0;
        self.od_ready = false;
        self.pd_ready = false;
        self.tx_ready = false;
        self.buffer.clear();
        let _ = self.buffer.extend_from_slice(&[0; BUFF_LEN]);
    }

    /// Returns the current length of the message buffer.
    pub fn len(&self) -> usize {
        self.length
    }

    /// Returns the message buffer as a slice.
    pub fn get_as_slice(&self) -> &[u8] {
        &self.buffer[0..self.length]
    }

    /// Inserts Object Dictionary (OD) data into the message buffer based on the device operation mode.
    ///
    /// # Parameters
    /// - `od_length`: Length of the OD data to insert.
    /// - `od`: Slice containing the OD data.
    /// - `device_mode`: Current device operation mode.
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(MessageBufferError)` if an error occurs (e.g., invalid length or mode).
    pub fn insert_od(
        &mut self,
        od_length: usize,
        od: &[u8],
        device_mode: DeviceOperationMode,
    ) -> MessageBufferResult<()> {
        match device_mode {
            DeviceOperationMode::Startup => {
                <Self as StartupTxMessageBuffer>::insert_od(self, od_length, od)
            }
            DeviceOperationMode::PreOperate => {
                <Self as PreOperateTxMessageBuffer>::insert_od(self, od_length, od)
            }
            DeviceOperationMode::Operate => {
                <Self as OperateTxMessageBuffer>::insert_od(self, od_length, od)
            }
        }
    }

    /// Inserts Process Data (PD) into the message buffer in Operate mode.
    ///
    /// # Parameters
    /// - `pd`: Slice containing the PD data.
    /// - `device_mode`: Current device operation mode.
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(MessageBufferError)` if an error occurs (e.g., invalid length or mode).
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

    /// Checks if the message buffer is ready for transmission based on the device operation mode.
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
    /// Creates a new `RxMessageBuffer` instance.
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

    /// Clears the message buffer and resets its state.
    pub fn clear(&mut self) {
        self.length = 0;
        self.buffer.clear();
        let _ = self.buffer.extend_from_slice(&[0; BUFF_LEN]);
    }

    /// Returns the current length of the message buffer.
    pub fn len(&self) -> usize {
        self.length
    }

    /// Pushes a byte into the message buffer.
    pub fn push(&mut self, data: u8) -> MessageBufferResult<()> {
        if self.length + 1 > BUFF_LEN {
            return Err(MessageBufferError::InvalidLength);
        }
        self.buffer[self.length] = data;
        self.length += 1;
        Ok(())
    }

    /// Extracts the M-sequence control byte from the message buffer.
    pub fn extract_mc(&self) -> MessageBufferResult<MsequenceControl> {
        let mc = MsequenceControl::from(self.buffer[0]);
        Ok(mc)
    }

    /// Validates the received request based on the device operation mode.
    pub fn valid_req(
        &mut self,
        device_mode: DeviceOperationMode,
    ) -> MessageBufferResult<RwDirection> {
        match device_mode {
            DeviceOperationMode::Startup => <Self as StartupRxMessageBuffer>::valid_req(self),
            DeviceOperationMode::PreOperate => <Self as PreOperateRxMessageBuffer>::valid_req(self),
            DeviceOperationMode::Operate => <Self as OperateRxMessageBuffer>::valid_req(self),
        }
    }

    /// Extracts Object Dictionary (OD) data from a write request based on the device operation mode.
    pub fn extract_od_from_write_req(
        &mut self,
        device_mode: DeviceOperationMode,
    ) -> MessageBufferResult<&[u8]> {
        match device_mode {
            DeviceOperationMode::Startup => {
                <Self as StartupRxMessageBuffer>::extract_od_from_write_req(self)
            }
            DeviceOperationMode::PreOperate => {
                <Self as PreOperateRxMessageBuffer>::extract_od_from_write_req(self)
            }
            DeviceOperationMode::Operate => {
                <Self as OperateRxMessageBuffer>::extract_od_from_write_req(self)
            }
        }
    }

    /// Extracts Process Data (PD) from the message buffer in Operate mode.
    ///
    /// # Returns
    /// - `Ok(&[u8])` containing the PD data if successful.
    /// - `Err(MessageBufferError)` if an error occurs (e.g., invalid mode).
    pub fn extract_pd(&mut self) -> MessageBufferResult<&[u8]> {
        <Self as OperateRxMessageBuffer>::extract_pd(self)
    }

    /// Calculates the expected number of bytes to receive based on the device operation mode.
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

    /// Returns the message buffer as a slice.
    pub fn get_as_slice(&self) -> &[u8] {
        &self.buffer[0..self.length]
    }
}

impl<const BUFF_LEN: usize> StartupTxMessageBuffer for TxMessageBuffer<BUFF_LEN> {
    fn insert_od(&mut self, od_length: usize, od: &[u8]) -> MessageBufferResult<()> {
        if self.length + od.len() > BUFF_LEN {
            return Err(MessageBufferError::InvalidLength);
        }
        if od_length > 1 {
            return Err(MessageBufferError::InvalidLength);
        }
        if od_length > 0 {
            const OD_START: usize = 0;
            const OD_END: usize = 1;
            self.buffer[OD_START..OD_END].copy_from_slice(od);
            self.length += 1;
        }
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
    fn insert_od(&mut self, od_length: usize, od: &[u8]) -> MessageBufferResult<()> {
        if self.length + od.len() > BUFF_LEN {
            return Err(MessageBufferError::InvalidLength);
        }
        const OD_LENGTH: usize = derived_config::on_req_data::pre_operate::od_length() as usize;
        if od_length > 0 {
            const OD_START: usize = 0;
            const OD_END: usize = OD_LENGTH;
            for index in OD_START..OD_END {
                if index >= od.len() {
                    self.buffer[index] = 0;
                    continue;
                }
                self.buffer[index] = od[index];
            }
            self.length += OD_LENGTH;
        }
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
    fn insert_od(&mut self, od_length: usize, od: &[u8]) -> MessageBufferResult<()> {
        if self.length + od.len() > BUFF_LEN {
            return Err(MessageBufferError::InvalidLength);
        }
        const OD_LENGTH: usize = derived_config::on_req_data::operate::od_length() as usize;
        if od_length > 0 {
            const OD_START: usize = 0;
            const OD_END: usize = OD_LENGTH;
            for index in OD_START..OD_END {
                if index >= od.len() {
                    self.buffer[index] = 0;
                    continue;
                }
                self.buffer[index] = od[index];
            }
            self.length += OD_LENGTH;
        }
        self.od_ready = true;
        Ok(())
    }

    fn insert_pd(&mut self, pd: &[u8]) -> MessageBufferResult<()> {
        if self.length + pd.len() > BUFF_LEN {
            return Err(MessageBufferError::InvalidLength);
        }
        const PD_LENGTH: usize =
            derived_config::process_data::pd_in::config_length_in_bytes() as usize;
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
        const CKS_INDEX: usize = OD_LENGTH + PD_LENGTH - 1;
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
        const PD_LENGTH: usize =
            derived_config::process_data::pd_out::config_length_in_bytes() as usize;
        const CKS_INDEX: usize = PD_LENGTH - 1;
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
        if validate_master_frame_checksum(self.length, self.buffer.as_mut_slice()) {
            const PRE_OP_MSEQ_BASE_TYPE: MsequenceBaseType =
                derived_config::m_seq_capability::pre_operate_m_sequence::m_sequence_base_type();
            let (mc, ckt) = extract_mc_ckt_bytes(self.buffer.as_slice())
                .map_err(|_| MessageBufferError::InvalidChecksum)?;
            if ckt.m_seq_type() != PRE_OP_MSEQ_BASE_TYPE {
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
        if validate_master_frame_checksum(self.length, self.buffer.as_mut_slice()) {
            const OP_MSEQ_BASE_TYPE: MsequenceBaseType =
                derived_config::m_seq_capability::operate_m_sequence::m_sequence_base_type();
            let (mc, ckt) = extract_mc_ckt_bytes(self.buffer.as_slice())
                .map_err(|_| MessageBufferError::InvalidChecksum)?;
            if ckt.m_seq_type() != OP_MSEQ_BASE_TYPE {
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
        const PD_START: usize = HEADER_SIZE_IN_FRAME as usize;
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

/// Calculate the maximum time required to transmit a UART frame based on the transmission rate.
/// This function calculates the maximum time required to transmit a UART frame,
/// including the time for the transmission of a UART frame (11 TBIT) plus the maximum of t1 (1 TBIT),
/// resulting in a total of 12 TBIT.
/// # Parameters
/// - `transmission_rate`: The transmission rate (baud rate) of the UART communication.
/// # Returns
/// - The maximum time in microseconds required to transmit a UART frame.
pub fn calculate_max_uart_frame_time(transmission_rate: TransmissionRate) -> u32 {
    const NUM_OF_BITS_PER_FRAME: u32 = 12;
    // MaxUARTFrameTime Time for the transmission of a UART frame (11 TBIT) plus maximum of t1 (1 TBIT) = 12 TBIT.
    let max_uart_frame_time =
        TransmissionRate::get_t_bit_in_us(transmission_rate) * NUM_OF_BITS_PER_FRAME;
    max_uart_frame_time
}

/// Extract M-sequence control and checksum bytes from the buffer
/// # Parameters
/// - `buffer`: The input buffer containing the M-sequence control and checksum bytes.
/// # Returns
/// - `Ok((MsequenceControl, ChecksumMsequenceType))` if extraction is  successful.
/// - `Err(IoLinkError)` if the buffer is too short to contain the required bytes.
pub fn extract_mc_ckt_bytes(
    buffer: &[u8],
) -> Result<(MsequenceControl, ChecksumMsequenceType), IoLinkError> {
    let mc = MsequenceControl::from(buffer[0]);
    let ckt = ChecksumMsequenceType::from(buffer[1]);
    Ok((mc, ckt))
}

/// Validate the checksum of the received IO-Link message
/// # Parameters
/// - `length`: The length of the received message.
/// - `data`: The mutable slice containing the received message data.
/// # Returns
/// - `true` if the checksum is valid.
/// - `false` if the checksum is invalid or if extraction fails.
pub fn validate_master_frame_checksum(length: usize, data: &mut [u8]) -> bool {
    // Validate the checksum of the received IO-Link message
    let (_, mut ckt) = match extract_mc_ckt_bytes(data) {
        Ok(val) => val,
        Err(_) => return false,
    };
    let checksum = ckt.checksum();
    // clear the received checksum bits (0-5), Before calculating the checksum
    ckt.set_checksum(0);
    data[1] = ckt.into_bits();
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
