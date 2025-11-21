//! # IO-Link On-request Data (OD) Handler Module
//!
//! This module defines traits and types for handling On-request Data (OD) services in IO-Link devices.
//! It provides interfaces for processing OD indications, responses, and parameter read/write operations
//! via the page communication channel.
//!
//! ## Key Traits
//! - [`OdInd`]: Handles OD.ind service primitives.
//! - [`OdRsp`]: Handles OD.rsp service primitives.
//! - [`DlWriteParamInd`]: Handles DL_WriteParam indications.
//! - [`DlParamRsp`]: Handles DL_ReadParam and DL_WriteParam responses.
//! - [`DlReadParamInd`]: Handles DL_ReadParam.ind service primitives.
//!
//! ## Key Types
//! - [`OdIndData`]: Structure representing OD indication data.
//!
//! ## Specification Reference
//! - IO-Link v1.1.4, Section 7.2.2.2 (OD)
//! - Table 35 – OD
//!
//! This module is intended for use in IO-Link device implementations to manage OD communication,
//! parameter access, and protocol compliance for on-request data operations.

use crate::{
    custom::IoLinkResult,
    frame::msequence::{ComChannel, RwDirection},
};
use heapless::Vec;
use iolinke_dev_config::device as dev_config;

/// Maximum OD length based on the maximum possible OD length from device configuration
/// This is determined by the maximum number of parameters and their sizes as defined in the
/// device configuration.
pub const OD_LENGTH: usize = dev_config::on_req_data::max_possible_od_length() as usize;

/// Trait representing the OD.ind (On-request Data indication) service handler.
///
/// This trait provides an interface for handling the IO-Link OD.ind service, which is used to set up
/// On-request Data for the next message to be sent. The confirmation of the service contains the data
/// from the receiver as specified in the IO-Link specification.
///
/// # Methods
/// - [`od_ind`]: Invokes the OD.ind service with the provided data.
pub trait OdInd {
    /// Handles the **OD.ind (Indication)** service primitive as defined in Table 35 – OD.
    ///
    /// The OD indication is used to notify the receiver that On-request Data (OD) has been received.
    /// It contains the service-specific parameters transmitted from the communication partner.
    ///
    /// # Parameters
    /// * `od_ind_data` - A structure containing the OD indication data, which includes:
    ///   - `RWDirection`: Indicates whether the service is a read (`READ`) or write (`WRITE`) operation.
    ///   - `ComChannel`: The communication channel used (e.g., `DIAGNOSIS`, `PAGE`, or `ISDU`).
    ///   - `AddressCtrl`: The address or flow control value (range: `0..=31`).
    ///   - `Length`: The length of the transmitted data (range: `0..=32`).
    ///   - `Data`: The actual transmitted octet string.
    ///
    /// # Returns
    /// * `IoLinkResult<()>` - Returns `Ok(())` if the indication is processed successfully,
    ///   or an error variant if processing fails.
    ///
    /// # Specification Reference
    /// IO-Link Interface and System Specification, Version 1.1.4, Section 7.2.2.2 (OD)
    fn od_ind(&mut self, od_ind_data: &OdIndData) -> IoLinkResult<()>;
}

/// Trait representing the OD.ind (On-request Data indication) service handler.
///
/// This trait provides an interface for handling the IO-Link OD.ind service, which is used to set up
/// On-request Data for the next message to be sent. The confirmation of the service contains the data
/// from the receiver as specified in the IO-Link specification.
///
/// # Methods
/// - [`od_rsp`]: Invokes the OD.rsp service with the provided data.
pub trait OdRsp {
    /// Sends the **OD.rsp (Response)** service primitive as defined in Table 35 – OD.
    ///
    /// The OD response confirms execution of the OD request and contains either the
    /// successfully read data (for `READ` operations) or an error indication (for `WRITE` failures).
    ///
    /// # Parameters
    /// * `length` - The length of the read data in bytes (range: `0..=32`).
    /// * `data` - The read data values as an octet string.
    ///
    /// # Returns
    /// * `IoLinkResult<()>` - Returns `Ok(())` if the response is sent successfully,
    ///   or an error variant if the service fails (e.g., `NO_COMM`, `STATE_CONFLICT`).
    ///
    /// # Specification Reference
    /// IO-Link Interface and System Specification, Version 1.1.4, Section 7.2.2.2 (OD)
    fn od_rsp(&mut self, length: u8, data: &[u8]) -> IoLinkResult<()>;
}

/// Trait defining the interface for handling DL_WriteParam indications in IO-Link communication.
pub trait DlWriteParamInd {
    /// See 7.2.1.3 DL_WriteParam
    /// The DL_WriteParam service is used by the AL to write a parameter value to the Device via
    /// the page communication channel. The parameters of the service primitives are listed in Table 18.
    fn dl_write_param_ind(&mut self, index: u8, data: u8) -> IoLinkResult<()>;
}

/// Trait representing Device Layer (DL) parameter response services for IO-Link communication.
///
/// This trait defines methods for responding to parameter read and write requests
/// from the Application Layer (AL) via the page communication channel, as specified
/// in the IO-Link protocol. Implementors of this trait are responsible for handling
/// the transmission of parameter values and reporting the result of the operation.
///
/// # Usage
/// Implement this trait for types that manage device parameter communication,
/// ensuring correct handling of service execution and error reporting.
///
/// # Errors
/// Methods return an `IoLinkResult<()>`, which may indicate communication errors
/// (e.g., `NO_COMM`) or state conflicts (`STATE_CONFLICT`) if the service cannot
/// be executed in the current device state.
pub trait DlParamRsp {
    /// See 7.2.1.2 DL_ReadParam
    /// The DL_ReadParam service is used by the AL to read a parameter value from the Device via
    /// the page communication channel. The parameters of the service primitives are listed in Table 17.
    ///
    /// # Parameters
    /// * `length` - The length of the read data in bytes (range: `0..=32`).
    /// * `data` - The read Device parameter value.
    ///
    /// # Returns
    /// * `IoLinkResult<()>` - Returns `Ok(())` if the response is sent successfully,
    ///   or an error variant if the service fails (e.g., `NO_COMM`, `STATE_CONFLICT`).
    fn dl_read_param_rsp(&mut self, length: u8, data: u8) -> IoLinkResult<()>;

    /// DL_ReadParam Response Service (see section 7.2.1.2)
    ///
    /// The DL_ReadParam service is used by the AL to read a parameter value from the Device via
    /// the page communication channel. The service-specific parameters are:
    ///
    /// # Parameters
    /// - `length`: The length of the read data in bytes. Permitted values: 0..=32.
    /// - `data`: The read Device parameter value.
    ///
    /// # Result (+)
    /// Indicates that the service has been executed successfully.
    ///
    /// # Result (-)
    /// Indicates that the service failed. Error information may include:
    /// - `NO_COMM`: No communication available.
    /// - `STATE_CONFLICT`: Service unavailable within current state.
    ///
    /// # Returns
    /// - `IoLinkResult<()>`: Returns `Ok(())` if the response is sent successfully,
    ///   or an error variant if the service fails (e.g., `NO_COMM`, `STATE_CONFLICT`).
    fn dl_write_param_rsp(&mut self) -> IoLinkResult<()>;
}

/// Trait for handling the DL_ReadParam.ind service primitive, which reads a parameter value from an IO-Link Device.
///
/// Implementors of this trait provide the logic for accessing device parameters via the DL_ReadParam.ind service.
/// This is typically used within the IO-Link communication stack to retrieve configuration or status information
/// from a device by specifying the parameter address.
///
/// See Table 17 – DL_ReadParam in the IO-Link specification for protocol details.
pub trait DlReadParamInd {
    /// Handles the DL_ReadParam.ind service primitive for reading a parameter value from the Device.
    ///
    /// # Arguments
    /// * `address` - The address of the requested Device parameter within the page communication channel.
    ///               Permitted values: 0 to 31.
    ///
    /// # Result
    /// Returns an `IoLinkResult<()>` indicating the outcome of the operation:
    /// - On success, the service has been executed and the Device parameter value has been read.
    /// - On failure, error information is provided, such as:
    ///     - `NO_COMM`: No communication available.
    ///     - `STATE_CONFLICT`: Service unavailable within current state.
    ///
    /// See Table 17 – DL_ReadParam for details.
    fn dl_read_param_ind(&mut self, address: u8) -> IoLinkResult<()>;
}

/// Structure representing the parameters for the OD.ind (On-request Data indication) service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OdIndData {
    /// Indicates the read or write direction.
    /// Permitted values: `READ` (Read operation), `WRITE` (Write operation).
    pub rw_direction: RwDirection,

    /// Indicates the selected communication channel for the transmission.
    /// Permitted values: `DIAGNOSIS`, `PAGE`, `ISDU`.
    pub com_channel: ComChannel,

    /// Contains the address or flow control value.
    /// Permitted values: 0 to 31.
    pub address_ctrl: u8,

    /// Contains the length of data to transmit.
    /// Permitted values: 0 to 32.
    pub req_length: u8,

    /// Contains the data to transmit.
    /// Data type: Octet string.
    pub data: Vec<u8, OD_LENGTH>,
}

impl OdIndData {
    /// Creates a new `OdIndData` instance with default values.
    pub fn new() -> Self {
        Self {
            rw_direction: RwDirection::Read,
            com_channel: ComChannel::Page,
            address_ctrl: 0,
            req_length: 0,
            data: Vec::new(),
        }
    }
}

impl Default for OdIndData {
    fn default() -> Self {
        Self::new()
    }
}
