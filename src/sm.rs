//! System Management
//!
//! This module implements the System Management state machine as defined in
//! IO-Link Specification v1.1.4

use iolinke_macros::master_command;

use crate::{dl_mode::DlInd, types::{IoLinkError, IoLinkResult}, DeviceMode, DlMode};

/// Table 95 – State transition tables of the Device System Management
#[derive(Debug, PartialEq, Eq)]
enum Transition {
    /// Nothing to transit.
    Tn,

    /// {T1} Source:0 Target:1 The Device is switched to the configured SIO mode by receiving the trigger
    /// SM_SetDeviceMode.req(SIO).
    /// Invoke PL_SetMode(DI|DO|INACTIVE)
    /// Invoke SM_DeviceMode(SIO)
    T1,

    /// {T2} Source:1 Target:2 The Device is switched to the communication mode by receiving the trigger
    /// DL_Mode.ind(ESTABCOM).
    /// Invoke PL_SetMode(COMx)
    /// Invoke SM_DeviceMode(ESTABCOM)
    T2,

    /// {T3} Source:2,3,4,5,6,7,8 Target:0 The Device is switched to SM_Idle mode by receiving the trigger
    /// DL_Mode.ind(INACTIVE) .
    /// Invoke PL_SetMode(INACTIVE)
    /// Invoke SM_DeviceMode(IDLE)
    T3,

    /// {T4} Source:2 Target:3 The Device application receives an indication on the baudrate with which
    /// the communication has been established in the DL triggered by
    /// DL_Mode.ind(COMx).
    /// Invoke SM_DeviceMode(COMx)
    T4,

    /// {T5} Source:3 Target:4 The Device identification phase is entered by receiving the trigger
    /// DL_Write.ind(MCmd_MASTERIDENT).
    /// Invoke SM_DeviceMode(IDENTSTARTUP)
    T5,

    /// {T6} Source:4 Target:5 The Device identity check phase is entered by receiving the trigger
    /// DL_Write.ind(MCmd_DEVICEIDENT).
    /// Invoke SM_DeviceMode(IDENTCHANGE)
    T6,

    /// {T7} Source:5 Target:6 The Device compatibility startup phase is entered by receiving the trigger
    /// DL_Read.ind( Direct Parameter page 1, address 0x02 = "MinCycleTime").
    T7,

    /// {T8} Source:6 Target:7 The Device's preoperate phase is entered by receiving the trigger
    /// DL_Mode.ind(PREOPERATE).
    /// Invoke SM_DeviceMode(PREOPERATE)
    T8,

    /// {T9} Source:7 Target:8 The Device's operate phase is entered by receiving the trigger
    /// DL_Mode.ind(OPERATE).
    /// Invoke SM_DeviceMode(OPERATE)
    T9,

    /// {T10} Source:4 Target:7 The Device's preoperate phase is entered by receiving the trigger
    /// DL_Mode.ind(PREOPERATE).
    /// Invoke SM_DeviceMode(PREOPERATE)
    T10,

    /// {T11} Source:3 Target:8 The Device's operate phase is entered by receiving the trigger
    /// DL_Mode.ind(OPERATE).
    /// Invoke SM_DeviceMode(OPERATE)
    T11,

    /// {T12} Source:7 Target:3 The Device's communication startup phase is entered by receiving the
    /// trigger DL_Mode.ind(STARTUP).
    /// Invoke SM_DeviceMode(STARTUP)
    T12,

    /// {T13} Source:8 Target:3 The Device's communication startup phase is entered by receiving the
    /// trigger DL_Mode.ind(STARTUP).
    /// Invoke SM_DeviceMode(STARTUP)
    T13,

    /// {T14} Source:5 Target:2 The requested Device identification requires a change of the transmission
    /// rate. Stop communication by changing the current transmission rate.
    /// Invoke PL_SetMode(COMx)
    /// Invoke SM_DeviceMode(ESTABCOM)
    T14,
}

/// DL-Mode Handler states
/// See IO-Link v1.1.4 Section 7.3.2.5
/// See Table 45 – State transition tables of the Device DL-mode handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemManagementState {
    /// SM_Idle_0 In SM_Idle the SM is waiting for configuration by the Device application and to be set
    /// to SIO mode. The state is left on receiving a SM_SetDeviceMode(SIO) request from the
    /// Device application
    /// The following sequence of services shall be executed between Device application and
    /// SM.
    /// Invoke SM_SetDeviceCom(initial parameter list)
    /// Invoke SM_SetDeviceIdent(VID, initial DID, FID)
    Idle,

    /// SM_SIO_1 In SM_SIO the SM Line Handler is remaining in the default SIO mode. The Physical
    /// Layer is set to the SIO mode characteristics defined by the Device application via the
    /// SetDeviceMode service. The state is left on receiving a DL_Mode(ESTABCOM)
    /// indication.
    Sio,

    /// SM_ComEstablish_2 In SM_ComEstablish the SM is waiting for the communication to be established in the
    /// Data Link Layer. The state is left on receiving a DL_Mode(INACTIVE) or a
    /// DL_Mode(COMx) indication, where COMx may be any of COM1, COM2 or COM3.
    ComEstablish,

    /// SM_ComStartup_3 In SM_ComStartup the communication parameter (Direct Parameter page 1, addresses
    /// 0x02 to 0x06) are read by the Master SM via DL_Read requests. The state is left upon
    /// reception of a DL_Mode(INACTIVE), a DL_Mode(OPERATE) indication (legacy Master
    /// only), or a DL_Write(MCmd_MASTERIDENT) request (Master in accordance with this
    /// standard).
    ComStartup,

    /// SM_IdentStartup_4 In SM_IdentStartup the identification data (VID, DID, FID) are read and verified by the
    /// Master. In case of incompatibilities the Master SM writes the supported SDCI Revision
    /// (RID) and configured DeviceID (DID) to the Device. The state is left upon reception of a
    /// DL_Mode(INACTIVE), a DL_Mode(PREOPERATE) indication (compatibility check
    /// passed), or a DL_Write(MCmd_DEVICEIDENT) request (new compatibility requested).
    IdentStartup,

    /// SM_IdentCheck_5 In SM_IdentCheck the SM waits for new initialization of communication and
    /// identification parameters. The state is left on receiving a DL_Mode(INACTIVE)
    /// indication, a DL_Read(Direct Parameter page 1, addresses 0x02 = "MinCycleTime")
    /// request, or the SM requires a switch of the transmission rate.
    /// Within this state the Device application shall check the RID and DID parameters from
    /// the SM and set these data to the supported values. Therefore the following sequence
    /// of services shall be executed between Device application and SM.
    /// Invoke SM_GetDeviceCom(configured RID, parameter list)
    /// Invoke SM_GetDeviceIdent(configured DID, parameter list)
    /// Invoke Device application checks and provides compatibility function and parameters
    /// Invoke SM_SetDeviceCom(new supported RID, new parameter list)
    /// Invoke SM_SetDeviceIdent(new supported DID, parameter list)
    IdentCheck,

    /// SM_CompStartup_6 In SM_CompatStartup the communication and identification data are reread and
    /// verified by the Master SM. The state is left on receiving a DL_Mode(INACTIVE) or a
    /// DL_Mode(PREOPERATE) indication.
    CompStartup,

    /// SM_Preoperate_7 During SM_Preoperate the SerialNumber can be read and verified by the Master SM,
    /// as well as Data Storage and Device parameterization may be executed. The state is
    /// left on receiving a DL_Mode(INACTIVE), a DL_Mode(STARTUP) or a
    /// DL_Mode(OPERATE) indication.
    Preoperate,

    /// SM_Operate_8 During SM_Operate the cyclic Process Data exchange and acyclic On-request Data
    /// transfer are active. The state is left on receiving a DL_Mode(INACTIVE) or a
    /// DL_Mode(STARTUP) indication.
    Operate,
}

/// System Management events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemManagementEvent {
    SmDeviceModeSio,
    DlModeEstabcom,
    DlModeComx,
    DlModeInactive,
    DlModeStartup,
    DlModeOperate,
    DlWriteMCmdMasterident(u8, u8),
    DlWriteMCmdDeviceident(u8, u8),
    DlModePreoperate,
    DlReadMincycletime,
    TransmissionRateChanged,
}
/// System Management implementation
pub struct SystemManagement {
    state: SystemManagementState,
    exec_transition: Transition,
}

impl SystemManagement {
    /// Create a new System Management instance
    pub fn new() -> Self {
        Self {
            state: SystemManagementState::Idle,
            exec_transition: Transition::Tn,
        }
    }

    /// Process an event
    pub fn process_event(&mut self, event: SystemManagementEvent) -> IoLinkResult<()> {
        use SystemManagementEvent as Event;
        use SystemManagementState as State;
        let new_state = match (self.state, event) {
            (State::Idle, Event::SmDeviceModeSio) => {
                self.exec_transition = Transition::T1;
                State::Sio
            }
            (State::Sio, Event::DlModeEstabcom) => {
                self.exec_transition = Transition::T2;
                State::ComEstablish
            }
            (
                State::ComEstablish
                | State::ComStartup
                | State::IdentStartup
                | State::IdentCheck
                | State::CompStartup
                | State::Preoperate
                | State::Operate,
                Event::DlModeInactive,
            ) => {
                self.exec_transition = Transition::T3;
                State::Idle
            }
            (State::ComEstablish, Event::DlModeComx) => {
                self.exec_transition = Transition::T4;
                State::ComStartup
            }
            (State::ComStartup, Event::DlWriteMCmdMasterident(addr, value)) => {
                self.exec_transition = Transition::T5;
                State::IdentStartup
            }
            (State::IdentStartup, Event::DlWriteMCmdDeviceident(addr, value)) => {
                self.exec_transition = Transition::T6;
                State::IdentCheck
            }
            (State::IdentCheck, Event::DlReadMincycletime) => {
                self.exec_transition = Transition::T7;
                State::CompStartup
            }
            (State::CompStartup, Event::DlModePreoperate) => {
                self.exec_transition = Transition::T8;
                State::Preoperate
            }
            (State::Preoperate, Event::DlModeOperate) => {
                self.exec_transition = Transition::T14;
                State::Operate
            }
            _ => {
                return Err(IoLinkError::InvalidEvent);
            }
        };

        self.state = new_state;
        Ok(())
    }

    /// Poll the system management
    pub fn poll(&mut self) -> IoLinkResult<()> {
        match self.exec_transition {
            Transition::Tn => {
                // No transition to execute
                return Ok(());
            }
            Transition::T1 => {
                // Execute T1 transition logic
                // Set device mode to SIO
            }
            Transition::T2 => {
                // Execute T2 transition logic
                // Set device mode to COMx
            }
            Transition::T3 => {
                // Execute T3 transition logic
                // Set device mode to INACTIVE
            }
            Transition::T4 => {
                // Execute T4 transition logic
                // Set device mode to COMx
            }
            Transition::T5 => {
                // Execute T5 transition logic
                // Set device mode to IDENTSTARTUP
            }
            Transition::T6 => {
                // Execute T6 transition logic
                // Set device mode to IDENTCHANGE
            }
            Transition::T7 => {
                // Execute T7 transition logic
                // Set device mode to COMPSTARTUP
            }
            Transition::T8 => {
                // Execute T8 transition logic
                // Set device mode to PREOPERATE
            }
            Transition::T9 => {
                // Execute T9 transition logic
                // Set device mode to OPERATE
            }
            Transition::T10 => {
                // Execute T10 transition logic
                // Set device mode to PREOPERATE
            }
            Transition::T11 => {
                // Execute T11 transition logic
                // Set device mode to OPERATE
            }
            Transition::T12 => {
                // Execute T12 transition logic
                // Set device mode to STARTUP
            }
            Transition::T13 => {
                // Execute T13 transition logic
                // Set device mode to STARTUP
            }
            Transition::T14 => {
                // Execute T14 transition logic
                // Change transmission rate and set device mode to ESTABCOM
            }
        }
        Ok(())
    }

    /// See 9.3.2.7 SM_DeviceMode
    /// The SM_DeviceMode service is used to indicate changes of communication states to the
    /// Device application. The parameters of the service primitives are listed in Table 94.
    pub fn device_mode_ind(&mut self, mode: DeviceMode) -> IoLinkResult<()> {
        if mode == DeviceMode::Sio {
            self.process_event(SystemManagementEvent::SmDeviceModeSio)?;
        }
        Ok(())
    }
}

impl Default for SystemManagement {
    fn default() -> Self {
        Self::new()
    }
}

impl DlInd for SystemManagement {

    fn dl_mode_ind(&mut self, mode: DlMode) -> IoLinkResult<()> {
        match mode {
            DlMode::Inactive => self.process_event(SystemManagementEvent::DlModeInactive),
            DlMode::Com1 | DlMode::Com2 | DlMode::Com3 => {
                self.process_event(SystemManagementEvent::DlModeComx)
            }
            DlMode::Estabcom => self.process_event(SystemManagementEvent::DlModeEstabcom),
            DlMode::Startup => self.process_event(SystemManagementEvent::DlModeStartup),
            DlMode::Preoperate => self.process_event(SystemManagementEvent::DlModePreoperate),
            DlMode::Operate => self.process_event(SystemManagementEvent::DlModeOperate),
            _ => Err(IoLinkError::InvalidEvent),
        }
    }

    fn dl_write_ind(&mut self, address: u8, value: u8) -> IoLinkResult<()> {
        if value == master_command!(MasterIdent) {
            return self.process_event(SystemManagementEvent::DlWriteMCmdMasterident(address, value));
        }
        else if value == master_command!(DeviceIdent) {
            return self.process_event(SystemManagementEvent::DlWriteMCmdDeviceident(address, value));
        }
        else {
            return Err(IoLinkError::InvalidEvent);
        }
    }
}
