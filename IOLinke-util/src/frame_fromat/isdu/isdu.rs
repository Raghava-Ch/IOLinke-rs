//! # ISDU Message Buffer Utilities
//!
//! This module provides utilities for handling ISDU (Indexed Service Data Unit) message buffers
//! for IO-Link communication. It defines types and functions for transmitting and receiving
//! ISDU messages, including buffer management, message compilation, parsing, and checksum validation.
//!
//! ## Features
//!
//! - **Transmit Buffer (`TxIsduMessageBuffer`)**: Provides methods to compile various ISDU responses
//!   (success, failure, busy, no service) and manage the transmission buffer.
//! - **Receive Buffer (`RxIsduMessageBuffer`)**: Handles incoming ISDU requests, including parsing
//!   service codes, extracting indices, subindices, and data, and validating checksums.
//! - **Error Handling**: Defines comprehensive error types for buffer operations and parsing.
//! - **Checksum Calculation**: Implements checksum logic for message integrity verification.
//! - **Indexing Support**: Enables slice and element indexing for buffer types.
//!
//! ## Usage
//!
//! Use the transmit and receive buffer types to manage ISDU message flows in IO-Link device or master
//! implementations. The parsing and compilation functions ensure correct message formatting and validation
//! according to the IO-Link specification.
//!
//! ## Example
//!
//! ```rust
//! let mut tx_buffer = TxIsduMessageBuffer::new();
//! tx_buffer.compile_isdu_write_success_response();
//! assert!(tx_buffer.is_ready());
//!
//! let mut rx_buffer = RxIsduMessageBuffer::new();
//! rx_buffer.extend(&[/* ISDU request bytes */]);
//! if let Ok((service, index, subindex, data)) = rx_buffer.extract_isdu_data() {
//!     // Handle parsed ISDU request
//! }
//! ```
use core::ops::{Index, IndexMut, Range};
use heapless::Vec;
use iolinke_types::{
    custom::{IoLinkError, IoLinkResult},
    frame::isdu::{IsduIServiceCode, IsduLengthCode, IsduService},
    handlers::isdu::MAX_ISDU_LENGTH,
};

use core::option::{
    Option,
    Option::{None, Some},
};
use core::result::{
    Result,
    Result::{Err, Ok},
};

/// Error type for the ISDU (Indexed Service Data Unit) message buffer operations.
#[derive(Debug, PartialEq, Eq)]
pub enum IsduMessageBufferError {
    /// Buffer does not have enough memory to perform the operation
    NotEnoughMemory,
    /// Invalid parameter provided
    InvalidIndex,
    /// Invalid parameter provided
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

/// Result type for ISDU message buffer operations.
pub type IsduMessageBufferResult<T> = Result<T, IsduMessageBufferError>;

/// Message buffer for transmitting ISDU (Indexed Service Data Unit) messages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TxIsduMessageBuffer {
    tx_buffer: Vec<u8, MAX_ISDU_LENGTH>,
    tx_buffer_ready: bool,
}

/// Message buffer for receiving ISDU (Indexed Service Data Unit) messages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RxIsduMessageBuffer {
    rx_buffer: Vec<u8, MAX_ISDU_LENGTH>,
    rx_buffer_ready: bool,
}

impl TxIsduMessageBuffer {
    /// Creates a new instance of the TxIsduMessageBuffer.
    pub fn new() -> Self {
        Self {
            tx_buffer: Vec::new(),
            tx_buffer_ready: false,
        }
    }

    /// Clears the message buffer.
    pub fn clear(&mut self) {
        self.tx_buffer.clear();
        self.tx_buffer_ready = false;
    }

    /// Checks if the message buffer is ready for transmission.
    pub fn is_ready(&self) -> bool {
        self.tx_buffer_ready
    }

    /// Returns the length of the message buffer.
    pub fn len(&self) -> usize {
        self.tx_buffer.len()
    }

    /// Returns the message buffer as a slice.
    pub fn get_as_slice(&self) -> &[u8] {
        self.tx_buffer.as_slice()
    }

    /// Compiles an ISDU write success response into the message buffer.
    pub fn compile_isdu_write_success_response(&mut self) {
        self.clear();
        const BUFFER: [u8; 3] = isdu_write_success_rsp();
        let _ = self.tx_buffer.extend_from_slice(&BUFFER);
        self.tx_buffer_ready = true;
    }

    /// Compiles an ISDU busy response into the message buffer.
    pub fn compile_isdu_busy_response(&mut self) {
        self.clear();
        const BUFFER: [u8; 1] = isdu_busy_rsp();
        let _ = self.tx_buffer.extend_from_slice(&BUFFER);
        self.tx_buffer_ready = true;
    }

    /// Compiles an ISDU no service response into the message buffer.
    pub fn compile_isdu_no_service_response(&mut self) {
        self.clear();
        const BUFFER: [u8; 1] = isdu_no_service_rsp();
        let _ = self.tx_buffer.extend_from_slice(&BUFFER);
        self.tx_buffer_ready = true;
    }

    /// Compiles an ISDU read success response into the message buffer.
    pub fn compile_isdu_read_success_response(
        &mut self,
        length: u8,
        data: &[u8],
    ) -> IoLinkResult<()> {
        if (2..=15).contains(&(length + 2/* +2 for iservice and checksum */)) {
            // Valid data length range (excluding length byte and checksum)
            isdu_read_success_rsp(length, data, &mut self.tx_buffer)?;
        } else {
            isdu_read_success_ext_len_rsp(length, data, &mut self.tx_buffer)?;
        }
        self.tx_buffer_ready = true;
        Ok(())
    }

    /// Compiles an ISDU read failure response into the message buffer.
    pub fn compile_isdu_read_failure_response(
        &mut self,
        error_code: u8,
        additional_error_code: u8,
    ) -> IoLinkResult<()> {
        const READ_FAILURE_RSP_LEN: u8 = 1 + 1 + 1 + 1 + 1; // iservice + error code + additional error code + checksum
        let mut i_service = IsduService::new();
        i_service.set_i_service(IsduIServiceCode::ReadFailure);
        i_service.set_length(READ_FAILURE_RSP_LEN);
        self.tx_buffer
            .push(i_service.into_bits())
            .map_err(|_| IoLinkError::InvalidLength)?;
        self.tx_buffer
            .push(error_code)
            .map_err(|_| IoLinkError::InvalidLength)?;
        self.tx_buffer
            .push(additional_error_code)
            .map_err(|_| IoLinkError::InvalidLength)?;
        self.tx_buffer
            .push(0)
            .map_err(|_| IoLinkError::InvalidLength)?; // checksum byte placeholder
        let chkpdu = calculate_checksum(4, &self.tx_buffer[0..4]);
        self.tx_buffer.pop();
        self.tx_buffer
            .push(chkpdu)
            .map_err(|_| IoLinkError::InvalidLength)?;

        self.tx_buffer_ready = true;
        Ok(())
    }

    /// Compiles an ISDU write failure response into the message buffer.
    pub fn compile_isdu_write_failure_response(
        &mut self,
        error_code: u8,
        additional_error_code: u8,
    ) -> IoLinkResult<()> {
        let mut i_service = IsduService::new();
        i_service.set_i_service(IsduIServiceCode::WriteFailure);
        i_service.set_length(4); // +2 for iservice and for checksum byte + 2 for error code and additional error code
        self.tx_buffer
            .push(i_service.into_bits())
            .map_err(|_| IoLinkError::InvalidLength)?;
        self.tx_buffer
            .push(error_code)
            .map_err(|_| IoLinkError::InvalidLength)?;
        self.tx_buffer
            .push(additional_error_code)
            .map_err(|_| IoLinkError::InvalidLength)?;
        self.tx_buffer
            .push(0)
            .map_err(|_| IoLinkError::InvalidLength)?; // checksum byte placeholder
        let chkpdu = calculate_checksum(4, &self.tx_buffer[0..4]);
        self.tx_buffer.pop();
        self.tx_buffer
            .push(chkpdu)
            .map_err(|_| IoLinkError::InvalidLength)?;

        self.tx_buffer_ready = true;
        Ok(())
    }
}

// Implement Index trait for TxIsduMessageBuffer to enable slice indexing
impl Index<Range<usize>> for TxIsduMessageBuffer {
    type Output = [u8];

    fn index(&self, range: Range<usize>) -> &Self::Output {
        &self.tx_buffer[range]
    }
}

impl Index<usize> for TxIsduMessageBuffer {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.tx_buffer[index]
    }
}

// Implement IndexMut trait for TxIsduMessageBuffer to enable mutable slice indexing
impl IndexMut<Range<usize>> for TxIsduMessageBuffer {
    fn index_mut(&mut self, range: Range<usize>) -> &mut Self::Output {
        &mut self.tx_buffer[range]
    }
}

impl IndexMut<usize> for TxIsduMessageBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.tx_buffer[index]
    }
}

impl RxIsduMessageBuffer {
    /// Creates a new instance of the RxIsduMessageBuffer.
    pub fn new() -> Self {
        Self {
            rx_buffer: Vec::new(),
            rx_buffer_ready: false,
        }
    }

    /// Clears the message buffer.
    pub fn clear(&mut self) {
        self.rx_buffer.clear();
        self.rx_buffer_ready = false;
    }

    /// Returns the length of the message buffer.
    pub fn len(&self) -> usize {
        self.rx_buffer.len()
    }

    /// Checks if the message buffer is ready for processing.
    pub fn is_ready(&self) -> bool {
        self.rx_buffer_ready
    }

    /// Appends data to the message buffer.
    pub fn extend(&mut self, data: &[u8]) {
        let _ = self.rx_buffer.extend_from_slice(data);
    }

    /// Marks the message buffer as ready for processing.
    pub fn get_as_slice(&self) -> &[u8] {
        self.rx_buffer.as_slice()
    }

    /// Extracts ISDU (Indexed Service Data Unit) request data from the provided buffer.
    ///
    /// This function parses the given buffer to extract the ISDU service, index, subindex,
    /// and optionally a data slice (for write requests). It performs basic validation,
    /// including buffer length and checksum verification, and then dispatches to the
    /// appropriate parsing function based on the ISDU service code.
    ///
    /// # Arguments
    ///
    /// * `buffer` - A reference to a heapless `Vec<u8, MAX_ISDU_LENGTH>` containing the ISDU request bytes.
    ///
    /// # Returns
    ///
    /// Returns a `IoLinkResult` containing a tuple:
    /// - `IsduService`: The parsed ISDU service descriptor.
    /// - `u16`: The index value.
    /// - `u8`: The subindex value.
    /// - `Option<&[u8]>`: The data slice if it is a write request, or `None` for read requests.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The buffer is too short to contain a valid ISDU request.
    /// - The checksum is invalid.
    /// - The ISDU service code is not recognized or not supported.
    ///
    /// # Example
    ///
    /// ```rust ignore
    /// let buffer: Vec<u8, MAX_ISDU_LENGTH> = /* ... */;
    /// match extract_isdu_data(&buffer) {
    ///     Ok((service, index, subindex, data)) => {
    ///         // Handle parsed ISDU request
    ///     }
    ///     Err(e) => {
    ///         // Handle error
    ///     }
    /// }
    /// ```
    pub fn extract_isdu_data(
        &self,
    ) -> IoLinkResult<(
        IsduService,
        u16,
        u8,
        Option<&[u8]>, // If None it is a read request, otherwise it is a write request
    )> {
        if self.rx_buffer.len() < 3 {
            return Err(IoLinkError::InvalidParameter);
        }
        if calculate_checksum(self.rx_buffer.len(), self.rx_buffer.as_slice()) != 0 {
            // Invalid checksum
            return Err(IoLinkError::ChecksumError);
        }
        let i_service: IsduService = IsduService::from_bits(self.rx_buffer[0]);
        match i_service.i_service() {
            IsduIServiceCode::ReadRequestIndex => {
                let (i_service, index, sub_index) = parse_read_request_with_index(&self.rx_buffer)?;
                Ok((i_service, index, sub_index, None))
            }
            IsduIServiceCode::ReadRequestIndexSubindex => {
                let (i_service, index, sub_index) =
                    parse_read_request_with_index_subindex(&self.rx_buffer)?;
                Ok((i_service, index, sub_index, None))
            }
            IsduIServiceCode::ReadRequestIndexIndexSubindex => {
                let (i_service, index, sub_index) =
                    parse_read_request_with_index_index_subindex(&self.rx_buffer)?;
                Ok((i_service, index, sub_index, None))
            }
            IsduIServiceCode::WriteRequestIndex => {
                let (i_service, index, sub_index, range) =
                    parse_write_request_with_index(&self.rx_buffer)?;
                Ok((i_service, index, sub_index, Some(range)))
            }
            IsduIServiceCode::WriteRequestIndexSubindex => {
                let (i_service, index, sub_index, range) =
                    parse_write_request_with_index_subindex(&self.rx_buffer)?;
                Ok((i_service, index, sub_index, Some(range)))
            }
            IsduIServiceCode::WriteRequestIndexIndexSubindex => {
                let (i_service, index, sub_index, range) =
                    parse_write_request_with_index_index_subindex(&self.rx_buffer)?;
                Ok((i_service, index, sub_index, Some(range)))
            }
            _ => Err(IoLinkError::InvalidParameter),
        }
    }
}

// Implement Index trait for RxIsduMessageBuffer to enable slice indexing
impl Index<Range<usize>> for RxIsduMessageBuffer {
    type Output = [u8];

    fn index(&self, range: Range<usize>) -> &Self::Output {
        &self.rx_buffer[range]
    }
}

impl Index<usize> for RxIsduMessageBuffer {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.rx_buffer[index]
    }
}

// Implement IndexMut trait for RxIsduMessageBuffer to enable mutable slice indexing
impl IndexMut<Range<usize>> for RxIsduMessageBuffer {
    fn index_mut(&mut self, range: Range<usize>) -> &mut Self::Output {
        &mut self.rx_buffer[range]
    }
}

impl IndexMut<usize> for RxIsduMessageBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.rx_buffer[index]
    }
}

fn isdu_read_success_ext_len_rsp(
    length: u8,
    data: &[u8],
    buffer: &mut Vec<u8, MAX_ISDU_LENGTH>,
) -> Result<(), IoLinkError> {
    let mut i_service = IsduService::new();
    i_service.set_i_service(IsduIServiceCode::ReadSuccess);
    i_service.set_length(IsduLengthCode::Extended.into());
    buffer
        .push(i_service.into_bits())
        .map_err(|_| IoLinkError::InvalidLength)?;
    buffer
        .push(3 + length) // isdu service byte + Length byte + checksum byte
        .map_err(|_| IoLinkError::InvalidLength)?;
    buffer
        .extend_from_slice(&data[..length as usize])
        .map_err(|_| IoLinkError::InvalidLength)?;
    buffer.push(0).map_err(|_| IoLinkError::InvalidLength)?;
    let total_length = 3 + length as usize;
    let chkpdu = calculate_checksum(total_length, &buffer[0..total_length]);
    buffer.pop();
    buffer
        .push(chkpdu)
        .map_err(|_| IoLinkError::InvalidLength)?;
    Ok(())
}

fn isdu_read_success_rsp(
    length: u8,
    data: &[u8],
    buffer: &mut Vec<u8, 238>,
) -> Result<(), IoLinkError> {
    let mut i_service = IsduService::new();
    i_service.set_i_service(IsduIServiceCode::ReadSuccess);
    i_service.set_length(length + 2);
    buffer
        .push(i_service.into_bits())
        .map_err(|_| IoLinkError::InvalidLength)?;
    buffer
        .extend_from_slice(&data[..length as usize])
        .map_err(|_| IoLinkError::InvalidLength)?;
    let total_length = 2 + length as usize; // 2 is for iservice + data
    buffer.push(0).map_err(|_| IoLinkError::InvalidLength)?;
    let chkpdu = calculate_checksum(total_length, &buffer[0..total_length]);
    buffer.pop();
    buffer
        .push(chkpdu)
        .map_err(|_| IoLinkError::InvalidLength)?;
    Ok(())
}

const fn isdu_write_success_rsp() -> [u8; 3] {
    const WRITE_SUCCESS_RSP_LEN: u8 = 1 + 0 + 1; // iservice + no data + checksum
    let mut buffer = [0; 3];
    let mut i_service = IsduService::new();
    i_service.set_i_service(IsduIServiceCode::WriteSuccess);
    i_service.set_length(WRITE_SUCCESS_RSP_LEN);
    buffer[0] = i_service.into_bits();
    buffer[1] = 0;
    let chkpdu = calculate_checksum(2, &buffer);
    buffer[2] = chkpdu;
    buffer
}

const fn isdu_busy_rsp() -> [u8; 1] {
    const BUSY_RSP_LEN: u8 = 1 + 0 + 0; // iservice + no data + no checksum
    let mut i_service = IsduService::new();
    i_service.set_i_service(IsduIServiceCode::NoService);
    i_service.set_length(BUSY_RSP_LEN);
    let buffer = i_service.into_bits();
    [buffer]
}

const fn isdu_no_service_rsp() -> [u8; 1] {
    const NO_SERVICE_RSP_LEN_CODE: u8 = 0 + 0 + 0; // no iservice + no data + no checksum
    let mut i_service = IsduService::new();
    i_service.set_i_service(IsduIServiceCode::NoService);
    i_service.set_length(NO_SERVICE_RSP_LEN_CODE);
    let buffer = i_service.into_bits();
    [buffer]
}

fn parse_read_request_with_index(
    buffer: &Vec<u8, MAX_ISDU_LENGTH>,
) -> IoLinkResult<(IsduService, u16, u8)> {
    let i_service: IsduService = IsduService::from_bits(buffer[0]);
    if i_service.i_service() != IsduIServiceCode::ReadRequestIndex {
        return Err(IoLinkError::InvalidParameter);
    }
    let index = buffer[1];
    Ok((i_service, index as u16, 0))
}

fn parse_read_request_with_index_subindex(
    buffer: &Vec<u8, MAX_ISDU_LENGTH>,
) -> IoLinkResult<(IsduService, u16, u8)> {
    let i_service: IsduService = IsduService::from_bits(buffer[0]);
    if i_service.i_service() != IsduIServiceCode::ReadRequestIndexSubindex {
        return Err(IoLinkError::InvalidParameter);
    }
    let index = buffer[1];
    let subindex = buffer[2];
    Ok((i_service, index as u16, subindex))
}

fn parse_read_request_with_index_index_subindex(
    buffer: &Vec<u8, MAX_ISDU_LENGTH>,
) -> IoLinkResult<(IsduService, u16, u8)> {
    let i_service: IsduService = IsduService::from_bits(buffer[0]);
    if i_service.i_service() != IsduIServiceCode::ReadRequestIndexIndexSubindex {
        return Err(IoLinkError::InvalidParameter);
    }
    let index = u16::from_le_bytes([buffer[1], buffer[2]]);
    let subindex = buffer[3];
    Ok((i_service, index, subindex))
}

fn parse_write_request_with_index(
    buffer: &Vec<u8, MAX_ISDU_LENGTH>,
) -> IoLinkResult<(IsduService, u16, u8, &[u8])> {
    let i_service: IsduService = IsduService::from_bits(buffer[0]);
    if i_service.i_service() != IsduIServiceCode::WriteRequestIndex {
        return Err(IoLinkError::InvalidParameter);
    }
    let length = i_service.length();
    if !(2..=15).contains(&length) {
        return Err(IoLinkError::InvalidData);
    }
    let index = buffer[1];
    Ok((i_service, index as u16, 0, &buffer[2..3 - length as usize]))
}

fn parse_write_request_with_index_subindex(
    buffer: &Vec<u8, MAX_ISDU_LENGTH>,
) -> IoLinkResult<(
    IsduService,
    u16, // Index
    u8,  // Subindex
    &[u8],
)> {
    let isdu_service_bits = buffer.get(0).ok_or(IoLinkError::InvalidParameter)?;
    let i_service: IsduService = IsduService::from_bits(*isdu_service_bits);
    let mut length = i_service.length();
    if length > 15 {
        return Err(IoLinkError::InvalidLength);
    }
    if length == 1 {
        length = *buffer.get(1).ok_or(IoLinkError::InvalidParameter)?;
        let index = *buffer.get(2).ok_or(IoLinkError::InvalidParameter)?;
        let subindex = *buffer.get(3).ok_or(IoLinkError::InvalidParameter)?;
        return Ok((
            i_service,
            index as u16,
            subindex,
            &buffer[4..4 + length as usize],
        ));
    }
    if !(2..=15).contains(&length) {
        let index = *buffer.get(1).ok_or(IoLinkError::InvalidParameter)?;
        let subindex = *buffer.get(2).ok_or(IoLinkError::InvalidParameter)?;
        return Ok((
            i_service,
            index as u16,
            subindex,
            &buffer[3..3 + length as usize],
        ));
    }
    return Err(IoLinkError::InvalidData);
}

fn parse_write_request_with_index_index_subindex(
    buffer: &Vec<u8, MAX_ISDU_LENGTH>,
) -> IoLinkResult<(
    IsduService,
    u16, // Index
    u8,  // Subindex
    &[u8],
)> {
    let i_service: IsduService = IsduService::from_bits(buffer[0]);
    if i_service.i_service() != IsduIServiceCode::WriteRequestIndexIndexSubindex {
        return Err(IoLinkError::InvalidParameter);
    }
    if i_service.length() != 1 {
        return Err(IoLinkError::InvalidData);
    }
    let length = buffer[1];
    if !(17..=238).contains(&length) {
        return Err(IoLinkError::InvalidData);
    }
    let index = u16::from_le_bytes([buffer[2], buffer[3]]);
    let subindex = buffer[4];
    Ok((i_service, index, subindex, &buffer[5..5 + length as usize]))
}

const fn calculate_checksum(length: usize, data: &[u8]) -> u8 {
    let mut checkpdu = 0;
    let mut i = 0;
    while i < length {
        // Avoid out-of-bounds access
        if i < data.len() {
            checkpdu ^= data[i];
        }
        i += 1;
    }
    checkpdu
}

/// Calculates the checksum for testing purposes.
#[cfg(any(test, feature = "std"))]
pub fn calculate_checksum_for_testing(length: usize, data: &[u8]) -> u8 {
    calculate_checksum(length, data)
}
