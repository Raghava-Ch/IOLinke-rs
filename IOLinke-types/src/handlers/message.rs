use crate::handlers::sm;

/// Trait for message handler operations in bw modules
pub trait MsgHandlerInfo {
    /// See 7.2.2.6 MHInfo
    /// The service MHInfo signals an exceptional operation within the message handler. The
    /// parameters of the service are listed in Table 39.
    fn mh_info(&mut self, mh_info: MHInfo);
}

/// All the message handler information type
/// See 7.2.2.6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MHInfo {
    /// lost communication
    COMlost,
    /// unexpected M-sequence type detected
    IllegalMessagetype,
    /// Checksum error detected
    ChecksumMismatch,
}

/// All the message handler confirmation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MhConfState {
    Com(sm::IoLinkMode),
    Active,
    Inactive,
}
