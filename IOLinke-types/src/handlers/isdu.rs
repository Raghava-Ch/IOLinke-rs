use crate::{custom::IoLinkResult, frame::msequence::RwDirection};
use heapless::Vec;
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

pub trait DlIsduTransportInd {
    /// See 7.2.1.6 DL_ISDUTransport
    /// The DL_ISDUTransport service is used to transport an ISDU. This service is used by the
    /// Master to send a service request from the Master application layer to the Device. It is used by
    /// the Device to send a service response to the Master from the Device application layer. The
    /// parameters of the service primitives are listed in Table 21.
    fn dl_isdu_transport_ind(&mut self, isdu: IsduMessage) -> IoLinkResult<()>;
}

pub trait DlIsduTransportRsp {
    fn dl_isdu_transport_read_rsp(&mut self, length: u8, data: &[u8]) -> IoLinkResult<()>;
    fn dl_isdu_transport_write_rsp(&mut self) -> IoLinkResult<()>;
    fn dl_isdu_transport_read_error_rsp(
        &mut self,
        error: u8,
        additional_error: u8,
    ) -> IoLinkResult<()>;
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
