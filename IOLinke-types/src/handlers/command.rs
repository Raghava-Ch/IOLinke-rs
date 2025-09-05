use crate::custom::IoLinkResult;
use crate::page::page1::MasterCommand;

/// MasterCommandInd
///
/// This trait defines the interface for the MasterCommand.ind service primitive as specified in
/// IO-Link v1.1.4, Section 7.2.1.18 (MasterCommand). The MasterCommand service is used by the
/// Master to convey control information to the Device application via the MasterCommand mechanism.
///
pub trait MasterCommandInd {
    /// Any MasterCommand received by the Device command handler
    /// (see Table 44 and Figure 54, state "CommandHandler_2")
    fn master_command_ind(&mut self, master_command: MasterCommand) -> IoLinkResult<()>;
}

/// DL_ControlInd
///
/// This trait defines the interface for the DL_Control.ind service primitive as specified in
/// IO-Link v1.1.4, Section 7.2.1.18 (DL_Control). The DL_Control service is used by the Master
/// to convey control information to the Device application via the MasterCommand mechanism,
/// and to receive control information via the PD status flag mechanism and the PDInStatus service
/// (see A.1.5, 7.2.2.5).
///
/// The `dl_control_ind` method is called to indicate a change in the qualifier status of the
/// Process Data (PD) to the Device application. The `control_code` parameter specifies the
/// qualifier status, which can be one of:
/// - `VALID`: Input Process Data valid (see 7.2.2.5, 8.2.2.12)
/// - `INVALID`: Input Process Data invalid
/// - `PDOUTVALID`: Output Process Data valid (see 7.3.7.1)
/// - `PDOUTINVALID`: Output Process Data invalid or missing
///
/// # Arguments
/// * `control_code` - The qualifier status of the Process Data (PD).
///
/// # Returns
/// * `IoLinkResult<()>` - Result of the indication operation.
///
/// # Specification Reference
/// - IO-Link v1.1.4, Table 33 – DL_Control
/// - Section 7.2.1.18 DL_Control
/// - Section 7.2.2.5 PDInStatus
/// - Section 8.2.2.12
/// - Section 7.3.7.1
pub trait DlControlInd {
    /// Indicate a change in the Process Data (PD) qualifier status to the Device application.
    ///
    /// This method is called by the Master to convey the current control code.
    fn dl_control_ind(&mut self, control_code: DlControlCode) -> IoLinkResult<()>;
}

/// DL_ControlCode
///
/// This enum represents the ControlCode argument used in the DL_Control service primitives,
/// as specified in IO-Link v1.1.4 (see Table 33 – DL_Control, Section 7.2.1.18).
///
/// The DL_Control service is used by the Master to convey control information to the Device
/// application via the MasterCommand mechanism, and to receive control information via the
/// PD status flag mechanism and the PDInStatus service (see A.1.5, 7.2.2.5).
///
/// The ControlCode parameter indicates the qualifier status of the Process Data (PD).
///
/// # Variants
///
/// - `VALID`: Input Process Data valid (see 7.2.2.5, 8.2.2.12)
/// - `INVALID`: Input Process Data invalid
/// - `PDOUTVALID`: Output Process Data valid (see 7.3.7.1)
/// - `PDOUTINVALID`: Output Process Data invalid or missing
///
/// # Specification Reference
/// - IO-Link v1.1.4, Table 33 – DL_Control
/// - Section 7.2.1.18 DL_Control
/// - Section 7.2.2.5 PDInStatus
/// - Section 8.2.2.12
/// - Section 7.3.7.1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DlControlCode {
    /// Input Process Data valid; see 7.2.2.5, 8.2.2.12
    VALID,
    /// Input Process Data invalid
    INVALID,
    /// Output Process Data valid; see 7.3.7.1
    PDOUTVALID,
    /// Output Process Data invalid or missing
    PDOUTINVALID,
}

/// All the Command handler configuration states used
/// See Figure 54 – State machine of the Device command handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChConfState {
    /// (Handler changed to the ACTIVE state)
    Active,
    /// (Handler changed to the INACTIVE state)
    Inactive,
}
