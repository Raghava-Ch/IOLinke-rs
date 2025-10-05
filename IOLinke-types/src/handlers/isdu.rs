//! # IO-Link ISDU Handler Module
//!
//! This module provides traits and types for handling Indexed Service Data Unit (ISDU) communication
//! in IO-Link devices. It defines interfaces for aborting, transporting, and responding to ISDU messages,
//! as well as structures for representing ISDU messages and handler configuration states.
//!
//! ## Key Traits
//! - [`DlIsduAbort`]: Handles ISDU abort requests.
//! - [`DlIsduTransportInd`]: Handles ISDU transport indications.
//! - [`DlIsduTransportRsp`]: Handles ISDU transport responses and error responses.
//!
//! ## Key Types
//! - [`IsduMessage`]: Structure representing an ISDU message.
//! - [`IhConfState`]: Configuration state for the ISDU handler.
//!
//! ## Specification Reference
//! - IO-Link v1.1.4, Section 7.2.1.6 (DL_ISDUAbort, DL_ISDUTransport)
//! - Section 8.4.3 – ISDU structure
//! - Figure 52 – State machine of the Device ISDU handler
//!
//! This module is intended for use in IO-Link device implementations to manage ISDU communication
//! and protocol compliance for indexed service data operations.

use crate::{custom::IoLinkResult, frame::msequence::RwDirection};
use heapless::Vec;

/// Maximum ISDU length as per IO-Link specification
/// Checkout the A.5.3 Extended length (ExtLength), Table A.14.
pub const MAX_ISDU_LENGTH: usize = 238;

/// DlIsduAbort
///
/// This trait defines the interface for the DL_ISDUAbort service primitive as specified in
/// IO-Link v1.1.4, Section 7.2.1.6 (DL_ISDUAbort). The DL_ISDUAbort service is used by the
/// Master to request the Device to abort the ISDU transmission.
///
/// # Returns
/// * `IoLinkResult<()>` - Result of the abort operation.
///
/// # Specification Reference
/// - IO-Link v1.1.4, DL_ISDUAbort
/// - Section 7.3.6.5 DL_ISDUAbort
pub trait DlIsduAbort {
    /// See 7.3.6.5 DL_ISDUAbort
    fn dl_isdu_abort(&mut self) -> IoLinkResult<()>;
}

/// Trait representing the DL_ISDUTransport indication service for IO-Link communication.
///
/// The `DlIsduTransportInd` trait defines the interface for handling the DL_ISDUTransport indication,
/// which is used to transport an ISDU (Indexed Service Data Unit) between a Master and Device.
/// This service allows the Master to send service requests to the Device, and the Device to send
/// service responses back to the Master, facilitating application layer communication.
///
/// Implementors of this trait should provide logic for processing incoming ISDU messages as defined
/// in section 7.2.1.6 of the IO-Link specification.
pub trait DlIsduTransportInd {
    /// See 7.2.1.6 DL_ISDUTransport
    /// The DL_ISDUTransport service is used to transport an ISDU. This service is used by the
    /// Master to send a service request from the Master application layer to the Device. It is used by
    /// the Device to send a service response to the Master from the Device application layer. The
    /// parameters of the service primitives are listed in Table 21.
    fn dl_isdu_transport_ind(&mut self, isdu: IsduMessage) -> IoLinkResult<()>;
}

/// Trait for handling ISDU (Indexed Service Data Unit) transport responses in IO-Link communication.
///
/// This trait defines methods for processing read and write responses, as well as error responses,
/// during ISDU transport operations. Implementors of this trait are expected to handle the
/// corresponding protocol actions and return an `IoLinkResult` indicating success or failure.
pub trait DlIsduTransportRsp {
    /// Handles a successful ISDU read response.
    ///
    /// # Arguments
    ///
    /// * `length` - The number of bytes read.
    /// * `data` - A slice containing the read data.
    ///
    /// # Returns
    ///
    /// An `IoLinkResult` indicating the outcome of the operation.
    fn dl_isdu_transport_read_rsp(&mut self, length: u8, data: &[u8]) -> IoLinkResult<()>;

    /// Handles a successful ISDU write response.
    ///
    /// # Returns
    ///
    /// An `IoLinkResult` indicating the outcome of the operation.
    fn dl_isdu_transport_write_rsp(&mut self) -> IoLinkResult<()>;

    /// Handles an ISDU read error response.
    ///
    /// # Arguments
    ///
    /// * `error` - The error code.
    /// * `additional_error` - Additional error information.
    ///
    /// # Returns
    ///
    /// An `IoLinkResult` indicating the outcome of the operation.
    fn dl_isdu_transport_read_error_rsp(
        &mut self,
        error: u8,
        additional_error: u8,
    ) -> IoLinkResult<()>;

    /// Handles an ISDU write error response.
    ///
    /// # Arguments
    ///
    /// * `error` - The error code.
    /// * `additional_error` - Additional error information.
    ///
    /// # Returns
    ///
    /// An `IoLinkResult` indicating the outcome of the operation.
    fn dl_isdu_transport_write_error_rsp(
        &mut self,
        error: u8,
        additional_error: u8,
    ) -> IoLinkResult<()>;
}

/// ISDU (Index Service Data Unit) structure
/// See IO-Link v1.1.4 Section 8.4.3
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IsduMessage {
    /// Parameter index
    pub index: u16,
    /// Sub-index
    pub sub_index: u8,
    /// Data payload
    pub data: Vec<u8, MAX_ISDU_LENGTH>,
    /// Read/Write operation flag
    pub direction: RwDirection,
}

/// All the ISDU Hanler configuration states used
/// See Figure 52 – State machine of the Device ISDU handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IhConfState {
    /// (Handler changed to the ACTIVE state)
    Active,
    /// (Handler changed to the INACTIVE state)
    Inactive,
}
