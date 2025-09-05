use crate::custom::IoLinkResult;

/// DL indications to other modules
pub trait DlModeInd {
    /// See 7.2.1.14 DL_Mode
    /// The DL uses the DL_Mode service to report to System Management that a certain operating
    /// status has been reached. The parameters of the service primitives are listed in Table 29.
    fn dl_mode_ind(&mut self, mode: DlMode) -> IoLinkResult<()>;
}

pub trait DlReadWriteInd {
    /// See 7.2.1.5 DL_Write
    /// The DL_Write service is used by System Management to write a Device parameter value to
    /// the Device via the page communication channel. The parameters of the service primitives are
    /// listed in Table 20.
    fn dl_write_ind(&mut self, address: u8, value: u8) -> IoLinkResult<()>;

    /// 7.2.1.4 DL_Read
    /// The DL_Read service is used by System Management to read a Device parameter value via
    /// the page communication channel. The parameters of the service primitives are listed in Table 19.
    fn dl_read_ind(&mut self, address: u8) -> IoLinkResult<()>;
}

/// Data Link Layer mode as per Section 7.2.1.14.
///
/// The DL mode indicates the current state of the data link
/// layer state machine.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Section 7.2.1.14: DL_Mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DlMode {
    /// Data link layer is inactive
    Inactive,
    /// COM1 mode is established
    Com1,
    /// COM2 mode is established
    Com2,
    /// COM3 mode is established
    Com3,
    /// Communication lost
    Comlost,
    /// Handler changed to the EstablishCom state
    Estabcom,
    /// Handler changed to the STARTUP state
    Startup,
    /// Handler changed to the PREOPERATE state
    PreOperate,
    /// Handler changed to the OPERATE state
    Operate,
}
