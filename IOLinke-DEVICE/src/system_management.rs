//! System Management implementation for IO-Link Device Stack.
//!
//! This module implements the System Management state machine as defined in
//! IO-Link Specification v1.1.4, managing device identification, communication
//! setup, and system-wide operations.
//!
//! ## Components
//!
//! - **Device Identification**: Vendor ID, Device ID, Function ID management
//! - **Communication Setup**: Mode configuration and establishment
//! - **System Commands**: Reset, restore, and control operations
//! - **Parameter Management**: Direct parameter page handling
//! - **State Coordination**: System-wide state machine management
//!
//! ## Specification Compliance
//!
//! - Section 6.3: System Management State Machine
//! - Section 7.3: Device Identification and Communication
//! - Section 10.7: System Commands and Control
//! - Annex B: Parameter Definitions and Access

use iolinke_macros::{direct_parameter_address, master_command};
use iolinke_types::custom::{IoLinkError, IoLinkResult};
use iolinke_types::frame::msequence::TransmissionRate;
use iolinke_types::handlers;
use iolinke_types::handlers::mode::{DlModeInd, DlReadWriteInd};
use iolinke_types::handlers::sm::SystemManagementCnf;
use iolinke_types::handlers::sm::SystemManagementInd;
use iolinke_types::handlers::sm::{
    DeviceCom, DeviceMode, IoLinkMode, SioMode, SmResult, SystemManagementReq,
};
use iolinke_types::page::page1::{DeviceIdent, RevisionId};
use iolinke_util::{log_state_transition, log_state_transition_error};

use crate::{al, pl};

/// System Management state transition types.
///
/// These transitions define the state machine behavior for the
/// System Management according to Table 95 of the IO-Link specification.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Table 95: State transition tables of the Device System Management
#[derive(Debug, PartialEq, Eq)]

enum Transition {
    /// No transition needed.
    Tn,

    /// T1: Source:0 Target:1 - Switch to SIO mode.
    ///
    /// The Device is switched to the configured SIO mode by receiving
    /// the trigger SM_SetDeviceMode.req(SIO).
    ///
    /// Actions:
    /// - Invoke PL_SetMode(DI|DO|INACTIVE)
    /// - Invoke SM_DeviceMode(SIO)
    T1,

    /// T2: Source:1 Target:2 - Switch to communication mode.
    ///
    /// The Device is switched to the communication mode by receiving
    /// the trigger DL_Mode.ind(ESTABCOM).
    ///
    /// Actions:
    /// - Invoke PL_SetMode(COMx)
    /// - Invoke SM_DeviceMode(ESTABCOM)
    T2,

    /// T3: Source:2,3,4,5,6,7,8 Target:0 - Switch to idle mode.
    ///
    /// The Device is switched to SM_Idle mode by receiving the trigger
    /// DL_Mode.ind(INACTIVE).
    ///
    /// Actions:
    /// - Invoke PL_SetMode(INACTIVE)
    /// - Invoke SM_DeviceMode(IDLE)
    T3,

    /// T4: Source:2 Target:3 - Communication established.
    ///
    /// The Device application receives an indication on the baudrate
    /// with which the communication has been established in the DL
    /// triggered by DL_Mode.ind(COMx).
    ///
    /// Actions:
    /// - Invoke SM_DeviceMode(COMx)
    T4(DeviceMode),

    /// T5: Source:3 Target:4 - Enter identification startup.
    ///
    /// The Device identification phase is entered by receiving the
    /// trigger DL_Write.ind(MCmd_MASTERIDENT).
    ///
    /// Actions:
    /// - Invoke SM_DeviceMode(IDENTSTARTUP)
    T5,

    /// T6: Source:4 Target:5 - Enter identity check.
    ///
    /// The Device identity check phase is entered by receiving the
    /// trigger DL_Write.ind(MCmd_DEVICEIDENT).
    ///
    /// Actions:
    /// - Invoke SM_DeviceMode(IDENTCHANGE)
    T6,

    /// T7: Source:5 Target:6 - Enter compatibility startup.
    ///
    /// The Device compatibility startup phase is entered by receiving
    /// the trigger DL_Read.ind(Direct Parameter page 1, address 0x02 = "MinCycleTime").
    T7,

    /// T8: Source:6 Target:7 - Enter preoperate mode.
    ///
    /// The Device's preoperate phase is entered by receiving the
    /// trigger DL_Mode.ind(PREOPERATE).
    ///
    /// Actions:
    /// - Invoke SM_DeviceMode(PREOPERATE)
    T8,

    /// T9: Source:7 Target:8 - Enter operate mode.
    ///
    /// The Device's operate phase is entered by receiving the
    /// trigger DL_Mode.ind(OPERATE).
    ///
    /// Actions:
    /// - Invoke SM_DeviceMode(OPERATE)
    T9,

    /// T10: Source:4 Target:7 - Enter preoperate mode from identification.
    ///
    /// The Device's preoperate phase is entered by receiving the
    /// trigger DL_Mode.ind(PREOPERATE).
    ///
    /// Actions:
    /// - Invoke SM_DeviceMode(PREOPERATE)
    T10,

    /// T11: Source:3 Target:8 - Enter operate mode from startup.
    ///
    /// The Device's operate phase is entered by receiving the
    /// trigger DL_Mode.ind(OPERATE).
    ///
    /// Actions:
    /// - Invoke SM_DeviceMode(OPERATE)
    T11,

    /// T12: Source:7 Target:3 - Return to startup from preoperate.
    ///
    /// The Device's communication startup phase is entered by receiving
    /// the trigger DL_Mode.ind(STARTUP).
    ///
    /// Actions:
    /// - Invoke SM_DeviceMode(STARTUP)
    T12,

    /// T13: Source:8 Target:3 - Return to startup from operate.
    ///
    /// The Device's communication startup phase is entered by receiving
    /// the trigger DL_Mode.ind(STARTUP).
    ///
    /// Actions:
    /// - Invoke SM_DeviceMode(STARTUP)
    T13,

    /// T14: Source:5 Target:2 - Transmission rate change required.
    ///
    /// The requested Device identification requires a change of the
    /// transmission rate. Stop communication by changing the current
    /// transmission rate.
    ///
    /// Actions:
    /// - Invoke PL_SetMode(COMx)
    /// - Invoke SM_DeviceMode(ESTABCOM)
    T14,
}

/// System Management state machine states.
///
/// These states define the System Management state machine as per
/// Section 6.3 of the IO-Link specification.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Section 6.3: System Management State Machine
/// - Table 94: SM_DeviceMode state definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemManagementState {
    /// SM_Idle_0: Waiting for configuration.
    ///
    /// In SM_Idle the SM is waiting for configuration by the Device
    /// application and to be set to SIO mode. The state is left on
    /// receiving a SM_SetDeviceMode(SIO) request from the Device application.
    ///
    /// The following sequence of services shall be executed between
    /// Device application and SM:
    /// - Invoke SM_SetDeviceCom(initial parameter list)
    /// - Invoke SM_SetDeviceIdent(VID, initial DID, FID)
    Idle,

    /// SM_SIO_1: SIO mode active.
    ///
    /// In SM_SIO the SM Line Handler is remaining in the default SIO mode.
    /// The Physical Layer is set to the SIO mode characteristics defined
    /// by the Device application via the SetDeviceMode service. The state
    /// is left on receiving a DL_Mode(ESTABCOM) indication.
    Sio,

    /// SM_ComEstablish_2: Establishing communication.
    ///
    /// In SM_ComEstablish the SM is waiting for the communication to be
    /// established in the Data Link Layer. The state is left on receiving
    /// a DL_Mode(INACTIVE) or a DL_Mode(COMx) indication, where COMx may
    /// be any of COM1, COM2 or COM3.
    ComEstablish,

    /// SM_ComStartup_3: Communication startup phase.
    ///
    /// In SM_ComStartup the communication parameter (Direct Parameter page 1,
    /// addresses 0x02 to 0x06) are read by the Master SM via DL_Read requests.
    /// The state is left upon reception of a DL_Mode(INACTIVE), a
    /// DL_Mode(OPERATE) indication (legacy Master only), or a
    /// DL_Write(MCmd_MASTERIDENT) request (Master in accordance with this standard).
    ComStartup,

    /// SM_IdentStartup_4: Identification startup phase.
    ///
    /// In SM_IdentStartup the identification data (VID, DID, FID) are read
    /// and verified by the Master. In case of incompatibilities the Master SM
    /// writes the supported SDCI Revision (RID) and configured DeviceID (DID)
    /// to the Device. The state is left upon reception of a DL_Mode(INACTIVE),
    /// a DL_Mode(PREOPERATE) indication (compatibility check passed), or a
    /// DL_Write(MCmd_DEVICEIDENT) request (new compatibility requested).
    IdentStartup,

    /// SM_IdentCheck_5: Identity check phase.
    ///
    /// In SM_IdentCheck the SM waits for new initialization of communication
    /// and identification parameters. The state is left on receiving a
    /// DL_Mode(INACTIVE) indication, a DL_Read(Direct Parameter page 1,
    /// addresses 0x02 = "MinCycleTime") request, or the SM requires a switch
    /// of the transmission rate.
    ///
    /// Within this state the Device application shall check the RID and DID
    /// parameters from the SM and set these data to the supported values.
    /// Therefore the following sequence of services shall be executed between
    /// Device application and SM:
    /// - Invoke SM_GetDeviceCom(configured RID, parameter list)
    /// - Invoke SM_GetDeviceIdent(configured DID, parameter list)
    /// - Invoke Device application checks and provides compatibility function and parameters
    /// - Invoke SM_SetDeviceCom(new supported RID, new parameter list)
    /// - Invoke SM_SetDeviceIdent(new supported DID, new parameter list)
    IdentCheck,

    /// SM_CompStartup_6: Compatibility startup phase.
    ///
    /// In SM_CompatStartup the communication and identification data are
    /// reread and verified by the Master SM. The state is left on receiving
    /// a DL_Mode(INACTIVE) or a DL_Mode(PREOPERATE) indication.
    CompStartup,

    /// SM_Preoperate_7: Preoperate mode.
    ///
    /// During SM_Preoperate the SerialNumber can be read and verified by
    /// the Master SM, as well as Data Storage and Device parameterization
    /// may be executed. The state is left on receiving a DL_Mode(INACTIVE),
    /// a DL_Mode(STARTUP) or a DL_Mode(OPERATE) indication.
    Preoperate,

    /// SM_Operate_8: Operate mode.
    ///
    /// During SM_Operate the cyclic Process Data exchange and acyclic
    /// On-request Data transfer are active. The state is left on receiving
    /// a DL_Mode(INACTIVE) or a DL_Mode(STARTUP) indication.
    Operate,
}

/// System Management events that can trigger state transitions.
///
/// These events represent the various triggers that can cause
/// the System Management state machine to change states.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Section 6.3: System Management State Machine
/// - Table 95: State transition tables
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SystemManagementEvent {
    /// Device mode changed to SIO
    SmDeviceModeSio,
    /// Data link layer mode changed to ESTABCOM
    DlModeEstabcom,
    /// Data link layer mode changed to COMx (COM1, COM2, COM3)
    DlModeComx(DeviceMode),
    /// Data link layer mode changed to INACTIVE
    DlModeInactive,
    /// Data link layer mode changed to STARTUP
    DlModeStartup,
    /// Data link layer mode changed to OPERATE
    DlModeOperate,
    /// Master command MASTERIDENT received
    DlWriteMCmdMasterident(u8, u8),
    /// Master command DEVICEIDENT received
    DlWriteMCmdDeviceident(u8, u8),
    /// Data link layer mode changed to PREOPERATE
    DlModePreoperate,
    /// MinCycleTime parameter read request
    DlReadMincycletime,
    /// Transmission rate change required
    TransmissionRateChanged,
}

/// Reconfiguration parameters for device compatibility.
///
/// This struct holds the parameters that may need to be updated
/// during device reconfiguration for compatibility with the master.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Section 6.3: System Management State Machine
/// - Section 7.3: Device Identification and Communication
#[derive(Default, Debug)]
struct ReConfig {
    /// Optional revision ID for compatibility
    revision_id: Option<RevisionId>,
    /// Optional device ID byte 1 for compatibility
    device_id1: Option<u8>,
    /// Optional device ID byte 2 for compatibility
    device_id2: Option<u8>,
    /// Optional device ID byte 3 for compatibility
    device_id3: Option<u8>,
}

/// Main System Management implementation that orchestrates the device state machine.
///
/// The System Management manages the complete device lifecycle including:
/// - Device identification and communication setup
/// - State machine transitions and timing
/// - Parameter management and compatibility
/// - Coordination with other protocol layers
///
/// # Architecture
///
/// The System Management follows the state machine defined in Section 6.3
/// of the IO-Link specification, with the following key components:
///
/// - **State Machine**: Manages device operating modes and transitions
/// - **Parameter Management**: Handles device communication and identification parameters
/// - **Event Processing**: Processes events from other protocol layers
/// - **Reconfiguration**: Manages device compatibility and parameter updates
///
/// # State Flow
///
/// The device follows this state progression:
/// 1. **Idle** → **SIO** → **ComEstablish** → **ComStartup**
/// 2. **IdentStartup** → **IdentCheck** → **CompStartup**
/// 3. **Preoperate** → **Operate** (with fallback paths)
///
/// # Specification Compliance
///
/// - IO-Link v1.1.4 Section 6.3: System Management State Machine
/// - Table 94: SM_DeviceMode state definitions
/// - Table 95: State transition tables
pub struct SystemManagement {
    /// Current state of the System Management state machine
    state: SystemManagementState,
    /// Currently executing transition (if any)
    exec_transition: Transition,
    /// Device communication configuration parameters
    device_com: DeviceCom,
    /// Device identification parameters
    device_ident: DeviceIdent,
    /// Current device operating mode
    device_mode: DeviceMode,
    /// Reconfiguration parameters for compatibility
    reconfig: ReConfig,
}

impl SystemManagement {
    /// Creates a new System Management instance with default configuration.
    ///
    /// The System Management starts in the **Idle** state and must be
    /// configured with device parameters before entering operational modes.
    ///
    /// # Returns
    ///
    /// A new `SystemManagement` instance ready for configuration.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut sm = SystemManagement::new();
    /// ```
    pub fn new() -> Self {
        Self {
            state: SystemManagementState::Idle,
            exec_transition: Transition::Tn,
            device_com: DeviceCom::default(),
            device_ident: DeviceIdent::default(),
            device_mode: DeviceMode::default(),
            reconfig: ReConfig::default(),
        }
    }

    /// Processes a system management event and updates the state machine.
    ///
    /// This method handles events from other protocol layers and determines
    /// the appropriate state transitions according to the IO-Link specification.
    ///
    /// # Parameters
    ///
    /// * `event` - The system management event to process
    ///
    /// # Returns
    ///
    /// - `Ok(())` if event was processed successfully
    /// - `Err(IoLinkError::InvalidEvent)` if event is not valid for current state
    ///
    /// # Specification Reference
    ///
    /// - IO-Link v1.1.4 Table 95: State transition tables
    fn process_event(&mut self, event: SystemManagementEvent) -> IoLinkResult<()> {
        use SystemManagementEvent as Event;
        use SystemManagementState as State;
        let (new_transition, new_state) = match (self.state, event) {
            (State::Idle, Event::SmDeviceModeSio) => (Transition::T1, State::Sio),
            (State::Sio, Event::DlModeEstabcom) => (Transition::T2, State::ComEstablish),
            (
                State::ComEstablish
                | State::ComStartup
                | State::IdentStartup
                | State::IdentCheck
                | State::CompStartup
                | State::Preoperate
                | State::Operate,
                Event::DlModeInactive,
            ) => (Transition::T3, State::Idle),
            (State::ComEstablish, Event::DlModeComx(mode)) => {
                (Transition::T4(mode), State::ComStartup)
            }
            (State::ComStartup, Event::DlWriteMCmdMasterident(_addr, _value)) => {
                (Transition::T5, State::IdentStartup)
            }
            (State::ComStartup, Event::DlModeOperate) => (Transition::T11, State::Preoperate),
            (State::IdentStartup, Event::DlWriteMCmdDeviceident(_addr, _value)) => {
                (Transition::T6, State::IdentCheck)
            }
            (State::IdentStartup, Event::DlModeOperate) => (Transition::T10, State::Preoperate),
            (State::IdentCheck, Event::DlReadMincycletime) => (Transition::T7, State::CompStartup),
            (State::IdentCheck, Event::TransmissionRateChanged) => {
                (Transition::T14, State::ComEstablish)
            }
            (State::CompStartup, Event::DlModePreoperate) => (Transition::T8, State::Preoperate),
            (State::Preoperate, Event::DlModeOperate) => (Transition::T9, State::Operate),
            (State::Preoperate, Event::DlModeStartup) => (Transition::T12, State::ComStartup),
            (State::Operate, Event::DlModeStartup) => (Transition::T13, State::ComStartup),
            _ => {
                log_state_transition_error!(module_path!(), "process_event", self.state, event);
                return Err(IoLinkError::InvalidEvent);
            }
        };

        log_state_transition!(
            module_path!(),
            "process_event",
            self.state,
            new_state,
            event
        );
        self.state = new_state;
        self.exec_transition = new_transition;
        Ok(())
    }

    /// Polls the System Management to advance its state and handle transitions.
    ///
    /// This method must be called regularly to:
    /// - Process pending state transitions
    /// - Handle reconfiguration requests
    /// - Coordinate with other protocol layers
    /// - Update device operating mode
    ///
    /// # Parameters
    ///
    /// * `application_layer` - Reference to application layer for coordination
    /// * `physical_layer` - Reference to physical layer for mode changes
    ///
    /// # Returns
    ///
    /// - `Ok(())` if polling was successful
    /// - `Err(IoLinkError)` if an error occurred
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut sm = SystemManagement::new();
    /// let mut al = ApplicationLayer::default();
    /// let mut pl = PhysicalLayer::default();
    ///
    /// // Poll system management
    /// match sm.poll(&mut al, &mut pl) {
    ///     Ok(()) => {
    ///         // System management operating normally
    ///     }
    ///     Err(e) => {
    ///         // Handle errors according to IO-Link specification
    ///     }
    /// }
    /// ```
    pub fn poll<T: pl::physical_layer::PhysicalLayerReq>(
        &mut self,
        application_layer: &mut al::ApplicationLayer,
        physical_layer: &mut T,
    ) -> IoLinkResult<()> {
        if self.reconfig.revision_id.is_some()
            && self.reconfig.device_id1.is_some()
            && self.reconfig.device_id2.is_some()
            && self.reconfig.device_id3.is_some()
        {
            let _ = self.poll_active_state(application_layer);
        }
        match self.exec_transition {
            Transition::Tn => {
                // No transition to execute
                return Ok(());
            }
            Transition::T1 => {
                // Execute T1 transition logic
                // Set device mode to SIO
                self.exec_transition = Transition::Tn;
                self.execute_t1(application_layer, physical_layer)?;
            }
            Transition::T2 => {
                // Execute T2 transition logic
                // Set device mode to COMx
                self.exec_transition = Transition::Tn;
                self.execute_t2(application_layer, physical_layer)?;
            }
            Transition::T3 => {
                // Execute T3 transition logic
                // Set device mode to INACTIVE
                self.exec_transition = Transition::Tn;
                self.execute_t3(application_layer, physical_layer)?;
            }
            Transition::T4(mode) => {
                // Execute T4 transition logic
                // Set device mode to COMx
                self.exec_transition = Transition::Tn;
                self.execute_t4(mode, application_layer)?;
            }
            Transition::T5 => {
                // Execute T5 transition logic
                // Set device mode to IDENTSTARTUP
                self.exec_transition = Transition::Tn;
                self.execute_t5(application_layer)?;
            }
            Transition::T6 => {
                // Execute T6 transition logic
                // Set device mode to IDENTCHANGE
                self.exec_transition = Transition::Tn;
                self.execute_t6(application_layer)?;
            }
            Transition::T7 => {
                // Execute T7 transition logic
                // Set device mode to COMPSTARTUP
                self.exec_transition = Transition::Tn;
                self.execute_t7()?;
            }
            Transition::T8 => {
                // Execute T8 transition logic
                // Set device mode to PREOPERATE
                self.exec_transition = Transition::Tn;
                self.execute_t8(application_layer)?;
            }
            Transition::T9 => {
                // Execute T9 transition logic
                // Set device mode to OPERATE
                self.exec_transition = Transition::Tn;
                self.execute_t9(application_layer)?;
            }
            Transition::T10 => {
                // Execute T10 transition logic
                // Set device mode to PREOPERATE
                self.exec_transition = Transition::Tn;
                self.execute_t10(application_layer)?;
            }
            Transition::T11 => {
                // Execute T11 transition logic
                // Set device mode to OPERATE
                self.exec_transition = Transition::Tn;
                self.execute_t11(application_layer)?;
            }
            Transition::T12 => {
                // Execute T12 transition logic
                // Set device mode to STARTUP
                self.exec_transition = Transition::Tn;
                self.execute_t12(application_layer)?;
            }
            Transition::T13 => {
                // Execute T13 transition logic
                // Set device mode to STARTUP
                self.exec_transition = Transition::Tn;
                self.execute_t13(application_layer)?;
            }
            Transition::T14 => {
                // Execute T14 transition logic
                // Change transmission rate and set device mode to ESTABCOM
                self.exec_transition = Transition::Tn;
                self.execute_t14(application_layer, physical_layer)?;
            }
        }
        Ok(())
    }

    fn poll_active_state(
        &mut self,
        application_layer: &mut al::ApplicationLayer,
    ) -> IoLinkResult<()> {
        use SystemManagementState as State;
        match self.state {
            State::IdentCheck => {
                let mut device_com = DeviceCom::default();
                let mut device_ident = DeviceIdent::default();
                device_com.revision_id = self.reconfig.revision_id.unwrap();
                device_ident.device_id[2] = self.reconfig.device_id1.unwrap();
                device_ident.device_id[1] = self.reconfig.device_id2.unwrap();
                device_ident.device_id[0] = self.reconfig.device_id3.unwrap();
                let device_com = Ok(&device_com);
                let device_ident = Ok(&device_ident);
                // Invoke SM_GetDeviceCom(configured RID, parameter list)
                let _ = application_layer.sm_get_device_com_cnf(device_com);
                // Invoke SM_GetDeviceIdent(configured DID, parameter list)
                let _ = application_layer.sm_get_device_ident_cnf(device_ident);
                self.reconfig = ReConfig::default();
                Ok(())
            }
            _ => {
                return Ok(());
            }
        }
    }

    /// Execute T1 transition: Switch to SIO mode
    fn execute_t1<T: pl::physical_layer::PhysicalLayerReq>(
        &mut self,
        application_layer: &mut al::ApplicationLayer,
        physical_layer: &mut T,
    ) -> IoLinkResult<()> {
        // Invoke PL_SetMode(DI|DO|INACTIVE)
        let sio_mode = match self.device_com.suppported_sio_mode {
            SioMode::Inactive => IoLinkMode::Inactive,
            SioMode::Di => IoLinkMode::Di,
            SioMode::Do => IoLinkMode::Do,
        };
        physical_layer.pl_set_mode_req(sio_mode)?;
        // Invoke SM_DeviceMode(SIO)
        let _ = application_layer.sm_device_mode_ind(DeviceMode::Sio);
        Ok(())
    }

    /// Execute T2 transition: Switch to communication mode
    fn execute_t2<T: pl::physical_layer::PhysicalLayerReq>(
        &mut self,
        application_layer: &mut al::ApplicationLayer,
        physical_layer: &mut T,
    ) -> IoLinkResult<()> {
        let com_mode = match self.device_com.transmission_rate {
            TransmissionRate::Com1 => IoLinkMode::Com1,
            TransmissionRate::Com2 => IoLinkMode::Com2,
            TransmissionRate::Com3 => IoLinkMode::Com3,
        };
        // Invoke PL_SetMode(COMx)
        let _ = physical_layer.pl_set_mode_req(com_mode);
        // Invoke SM_DeviceMode(ESTABCOM)
        let _ = application_layer.sm_device_mode_ind(DeviceMode::Estabcom);
        Ok(())
    }

    /// Execute T3 transition: Switch to SM_Idle mode
    fn execute_t3<T: pl::physical_layer::PhysicalLayerReq>(
        &mut self,
        application_layer: &mut al::ApplicationLayer,
        physical_layer: &mut T,
    ) -> IoLinkResult<()> {
        // TODO: Cleanup any active communication sessions and reset state
        // Invoke PL_SetMode(INACTIVE)
        let _ = physical_layer.pl_set_mode_req(IoLinkMode::Inactive);
        // Invoke SM_DeviceMode(IDLE)
        let _ = application_layer.sm_device_mode_ind(DeviceMode::Idle);
        Ok(())
    }

    /// Execute T4 transition: Indicate baudrate establishment
    fn execute_t4(
        &mut self,
        mode: DeviceMode,
        application_layer: &mut al::ApplicationLayer,
    ) -> IoLinkResult<()> {
        // Invoke SM_DeviceMode(COMx)
        // TODO: Invoke SM_DeviceMode(COMx)
        let _ = application_layer.sm_device_mode_ind(mode);
        Ok(())
    }

    /// Execute T5 transition: Enter device identification phase
    fn execute_t5(&mut self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
        // TODO: Prepare device identification data for master verification
        // Invoke SM_DeviceMode(IDENTSTARTUP)
        let _ = application_layer.sm_device_mode_ind(DeviceMode::Identstartup);
        Ok(())
    }

    /// Execute T6 transition: Enter device identity check phase
    fn execute_t6(&mut self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
        // TODO: Handle master-provided RID and DID parameters for compatibility check
        // Invoke SM_DeviceMode(IDENTCHANGE)
        let _ = application_layer.sm_device_mode_ind(DeviceMode::Identchange);
        Ok(())
    }

    /// Execute T7 transition: Enter device compatibility startup phase
    fn execute_t7(&mut self) -> IoLinkResult<()> {
        // TODO: Implement compatibility verification logic with master
        // Device compatibility startup phase is entered
        // No specific device mode change mentioned in spec
        Ok(())
    }

    /// Execute T8 transition: Enter device preoperate phase
    fn execute_t8(&mut self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
        // TODO: Prepare device for parameterization and data storage operations
        // Invoke SM_DeviceMode(PREOPERATE)
        let _ = application_layer.sm_device_mode_ind(DeviceMode::Preoperate);
        Ok(())
    }

    /// Execute T9 transition: Enter device operate phase
    fn execute_t9(&mut self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
        // TODO: Initialize cyclic process data exchange and acyclic data transfer
        // Invoke SM_DeviceMode(OPERATE)
        let _ = application_layer.sm_device_mode_ind(DeviceMode::Operate);
        Ok(())
    }

    /// Execute T10 transition: Enter device preoperate phase from IdentStartup
    fn execute_t10(&mut self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
        // TODO: Handle direct transition from identification to preoperate
        // Invoke SM_DeviceMode(PREOPERATE)
        let _ = application_layer.sm_device_mode_ind(DeviceMode::Preoperate);
        Ok(())
    }

    /// Execute T11 transition: Enter device operate phase from ComStartup
    fn execute_t11(&mut self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
        // TODO: Handle legacy master behavior (direct transition to operate)
        // Invoke SM_DeviceMode(OPERATE)
        let _ = application_layer.sm_device_mode_ind(DeviceMode::Operate);
        Ok(())
    }

    /// Execute T12 transition: Enter communication startup phase from Preoperate
    fn execute_t12(&mut self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
        // TODO: Reset communication parameters and restart identification process
        // Invoke SM_DeviceMode(STARTUP)
        let _ = application_layer.sm_device_mode_ind(DeviceMode::Startup);
        Ok(())
    }

    /// Execute T13 transition: Enter communication startup phase from Operate
    fn execute_t13(&mut self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
        // TODO: Handle communication restart from operate mode
        // Invoke SM_DeviceMode(STARTUP)
        let _ = application_layer.sm_device_mode_ind(DeviceMode::Startup);
        Ok(())
    }

    /// Execute T14 transition: Change transmission rate and establish communication
    fn execute_t14<T: pl::physical_layer::PhysicalLayerReq>(
        &mut self,
        application_layer: &mut al::ApplicationLayer,
        physical_layer: &mut T,
    ) -> IoLinkResult<()> {
        // TODO: Implement transmission rate change logic based on device identification requirements
        // Invoke PL_SetMode(COMx)
        let mode = match self.device_com.transmission_rate {
            TransmissionRate::Com1 => IoLinkMode::Com1,
            TransmissionRate::Com2 => IoLinkMode::Com2,
            TransmissionRate::Com3 => IoLinkMode::Com3,
        };
        let _ = physical_layer.pl_set_mode_req(mode);
        // Invoke SM_DeviceMode(ESTABCOM)
        let _ = application_layer.sm_device_mode_ind(DeviceMode::Estabcom);
        Ok(())
    }
}

impl Default for SystemManagement {
    fn default() -> Self {
        Self::new()
    }
}

impl DlModeInd for SystemManagement {
    fn dl_mode_ind(&mut self, mode: handlers::mode::DlMode) -> IoLinkResult<()> {
        use handlers::mode::DlMode;
        match mode {
            DlMode::Inactive => self.process_event(SystemManagementEvent::DlModeInactive),
            DlMode::Com1 => {
                let _ = self.process_event(SystemManagementEvent::DlModeComx(DeviceMode::Com1));
                self.process_event(SystemManagementEvent::TransmissionRateChanged)
            }
            DlMode::Com2 => {
                let _ = self.process_event(SystemManagementEvent::DlModeComx(DeviceMode::Com2));
                self.process_event(SystemManagementEvent::TransmissionRateChanged)
            }
            DlMode::Com3 => {
                let _ = self.process_event(SystemManagementEvent::DlModeComx(DeviceMode::Com3));
                self.process_event(SystemManagementEvent::TransmissionRateChanged)
            }
            DlMode::Estabcom => self.process_event(SystemManagementEvent::DlModeEstabcom),
            DlMode::Startup => self.process_event(SystemManagementEvent::DlModeStartup),
            DlMode::PreOperate => self.process_event(SystemManagementEvent::DlModePreoperate),
            DlMode::Operate => self.process_event(SystemManagementEvent::DlModeOperate),
            _ => Err(IoLinkError::InvalidEvent),
        }
    }
}

impl DlReadWriteInd for SystemManagement {
    fn dl_write_ind(&mut self, address: u8, value: u8) -> IoLinkResult<()> {
        match (address, value) {
            (direct_parameter_address!(MasterCommand), master_command!(MasterIdent)) => {
                self.process_event(SystemManagementEvent::DlWriteMCmdMasterident(
                    address, value,
                ))?;
            }
            (direct_parameter_address!(MasterCommand), master_command!(DeviceIdent)) => {
                self.process_event(SystemManagementEvent::DlWriteMCmdDeviceident(
                    address, value,
                ))?;
            }
            (direct_parameter_address!(RevisionID), value) => {
                self.reconfig.revision_id = Some(RevisionId::from_bits(value));
            }
            (direct_parameter_address!(DeviceID1), value) => {
                self.reconfig.device_id1 = Some(value);
            }
            (direct_parameter_address!(DeviceID2), value) => {
                self.reconfig.device_id2 = Some(value);
            }
            (direct_parameter_address!(DeviceID3), value) => {
                self.reconfig.device_id3 = Some(value);
            }
            _ => return Err(IoLinkError::InvalidEvent),
        }
        Ok(())
    }

    fn dl_read_ind(&mut self, address: u8) -> IoLinkResult<()> {
        if address == direct_parameter_address!(MinCycleTime) {
            // MinCycleTime address
            return self.process_event(SystemManagementEvent::DlReadMincycletime);
        }
        Err(IoLinkError::InvalidEvent)
    }
}

impl SystemManagementReq<al::ApplicationLayer> for SystemManagement {
    fn sm_set_device_com_req(&mut self, device_com: &DeviceCom) -> SmResult<()> {
        // Set the device communication parameters
        self.device_com = device_com.clone();
        Ok(())
    }

    fn sm_get_device_com_req(&mut self, application_layer: &al::ApplicationLayer) -> SmResult<()> {
        // Return the current device communication parameters
        // Typically, this would trigger a confirmation callback
        // Here, we just return Ok(())
        let device_com = Ok(&self.device_com);
        application_layer.sm_get_device_com_cnf(device_com)?;
        Ok(())
    }

    fn sm_set_device_ident_req(&mut self, device_ident: &DeviceIdent) -> SmResult<()> {
        // Set the device identification parameters
        self.device_ident = device_ident.clone();
        Ok(())
    }

    fn sm_get_device_ident_req(
        &mut self,
        application_layer: &al::ApplicationLayer,
    ) -> SmResult<()> {
        // Return the current device identification parameters
        let device_ident = Ok(&self.device_ident);
        application_layer.sm_get_device_ident_cnf(device_ident)?;
        Ok(())
    }

    fn sm_set_device_mode_req(&mut self, mode: DeviceMode) -> SmResult<()> {
        // Set the device mode
        self.device_mode = mode;
        // TODO: Implement proper device mode indication handling for all modes
        // TODO: Add device application notification mechanism
        match mode {
            DeviceMode::Sio => {
                let _ = self.process_event(SystemManagementEvent::SmDeviceModeSio);
            }
            DeviceMode::Estabcom => {
                let _ = self.process_event(SystemManagementEvent::DlModeEstabcom);
            }
            DeviceMode::Com1 => {
                let _ = self.process_event(SystemManagementEvent::DlModeComx(DeviceMode::Com1));
                let _ = self.process_event(SystemManagementEvent::TransmissionRateChanged);
            }
            DeviceMode::Com2 => {
                let _ = self.process_event(SystemManagementEvent::DlModeComx(DeviceMode::Com2));
                let _ = self.process_event(SystemManagementEvent::TransmissionRateChanged);
            }
            DeviceMode::Com3 => {
                let _ = self.process_event(SystemManagementEvent::DlModeComx(DeviceMode::Com3));
                let _ = self.process_event(SystemManagementEvent::TransmissionRateChanged);
            }
            DeviceMode::Idle => {
                let _ = self.process_event(SystemManagementEvent::DlModeInactive);
            }
            DeviceMode::Startup => {
                let _ = self.process_event(SystemManagementEvent::DlModeStartup);
            }
            DeviceMode::Preoperate => {
                let _ = self.process_event(SystemManagementEvent::DlModePreoperate);
            }
            DeviceMode::Operate => {
                let _ = self.process_event(SystemManagementEvent::DlModeOperate);
            }
            DeviceMode::Identstartup => {}
            DeviceMode::Identchange => {}
        }
        Ok(())
    }
}
