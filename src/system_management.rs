//! System Management
//!
//! This module implements the System Management state machine as defined in
//! IO-Link Specification v1.1.4

use iolinke_macros::{direct_parameter_address, master_command};
use modular_bitfield::prelude::*;

use crate::{
    al, dl::DlInd, pl, types::{self, IoLinkError, IoLinkResult}, IoLinkDevice, IoLinkMode
};

pub enum SmError {
    ParameterConflict,
}

pub type SmResult<T> = Result<T, SmError>;

#[derive(Clone, Debug)]
pub enum SioMode {
    // {INACTIVE} (C/Q line in high impedance)
    Inactive,
    // {DI} (C/Q line in digital input mode)
    Di,
    // {DO} (C/Q line in digital output mode)
    Do,
}

impl Default for SioMode {
    fn default() -> Self {
        Self::Inactive
    }
}

#[derive(Clone, Debug)]
pub enum TransmissionRate {
    /// {COM1} (1200 baud)
    Com1,
    /// {COM2} (2400 baud)
    Com2,
    /// {COM3} (4800 baud)
    Com3,
}

impl Default for TransmissionRate {
    fn default() -> Self {
        Self::Com1
    }
}

#[bitfield]
#[derive(Clone, Debug)]
pub struct MinCycleTime {
    /// Bits 6 to 7: Time Base
    /// These bits specify the time base for the calculation of MasterCycleTime and MinCycleTime.
    /// In the following cases, when
    /// * the Device provides no MinCycleTime, which is indicated by a MinCycleTime equal zero
    /// (binary code 0x00),
    /// * or the MinCycleTime is shorter than the calculated M-sequence time with the M-sequence
    /// type used by the Device, with (t1, t2, tidle) equal zero and tA equal one bit time (see A.3.4
    /// to A.3.6)
    pub time_base: B2,
    /// Bits 0 to 5: Multiplier
    /// These bits contain a 6-bit multiplier for the calculation of MasterCycleTime and MinCycleTime.
    /// Permissible values for the multiplier are 0 to 63, further restrictions see Table B.3.
    pub multiplier: B6,
}

impl Default for MinCycleTime {
    fn default() -> Self {
        Self::new().with_time_base(0).with_multiplier(0)
    }
}

/// MsequenceCapability bitfield as per IO-Link specification v1.1.4 Annex B.1
#[bitfield]
#[derive(Clone, Debug)]
pub struct MsequenceCapability {
    /// Bit 0: ISDU
    /// This bit indicates whether or not the ISDU communication channel is supported.
    /// Permissible values:
    ///   0 = ISDU not supported
    ///   1 = ISDU supported
    pub isdu: B1,

    /// Bits 1 to 3: Coding of the OPERATE M-sequence type
    /// This parameter indicates the available M-sequence type during the OPERATE state.
    /// Permissible codes for the OPERATE M-sequence type are listed in Table A.9 (legacy Devices)
    /// and Table A.10 (Devices according to this standard).
    pub op_m_seq_code: B3,

    /// Bits 4 to 5: Coding of the PREOPERATE M-sequence type
    /// This parameter indicates the available M-sequence type during the PREOPERATE state.
    /// Permissible codes for the PREOPERATE M-sequence type are listed in Table A.8.
    pub pre_op_m_seq_code: B2,

    /// Bits 6 to 7: Reserved
    /// These bits are reserved and shall be set to zero in this version of the specification.
    #[skip] // Always set to 0, not exposed to user
    pub __: B2,
}

impl Default for MsequenceCapability {
    fn default() -> Self {
        Self::new()
            .with_isdu(0)
            .with_op_m_seq_code(0)
            .with_pre_op_m_seq_code(0)
    }
}

#[bitfield]
#[derive(Clone, Copy, Debug, Specifier)]
pub struct RevisionId {
    /// Bits 0 to 3: MinorRev
    /// These bits contain the minor digit of the version number, for example 0 for the protocol
    /// version 1.0. Permissible values for MinorRev are 0x0 to 0xF.
    minor_rev: B4,
    /// Bits 4 to 7: MajorRev
    /// These bits contain the major digit of the version number, for example 1 for the protocol
    /// version 1.0. Permissible values for MajorRev are 0x0 to 0xF.
    major_rev: B4,
}

impl Default for RevisionId {
    fn default() -> Self {
        Self::new().with_minor_rev(0).with_major_rev(0)
    }
}

#[bitfield]
#[derive(Clone, Debug)]
pub struct ProcessDataIn {
    /// ProcessDataIn bitfield as per IO-Link specification v1.1.4 Annex B.1
    ///
    /// - Bits 0 to 4: Length
    ///   These bits contain the length of the input data (Process Data from Device to Master) in the
    ///   length unit designated in the BYTE parameter bit. Permissible codes for Length are specified
    ///   in Table B.6.
    #[bits = 5]
    pub length: B5,

    /// Bit 5: Reserved
    /// This bit is reserved and shall be set to zero in this version of the specification.
    #[skip] // Always set to 0, not exposed to user
    __: B1,

    /// Bit 6: SIO
    /// This bit indicates whether the Device provides a switching signal in SIO mode.
    /// Permissible values:
    ///   0 = SIO mode not supported
    ///   1 = SIO mode supported
    pub sio: B1,

    /// Bit 7: BYTE
    /// This bit indicates the length unit for Length.
    ///   0 = Length is in bits
    ///   1 = Length is in octets (bytes)
    pub byte: B1,
}

impl Default for ProcessDataIn {
    fn default() -> Self {
        Self::new().with_length(0).with_sio(0).with_byte(0)
    }
}

#[bitfield]
#[derive(Clone, Debug)]
pub struct ProcessDataOut {
    /// ProcessDataOut bitfield as per IO-Link specification v1.1.4 Annex B.1
    ///
    /// - Bits 0 to 4: Length
    ///   These bits contain the length of the output data (Process Data from Master to Device) in the
    ///   length unit designated in the BYTE parameter bit. Permissible codes for Length are specified
    ///   in Table B.6.
    #[bits = 5]
    pub length: B5,

    /// Bit 5 to 6: Reserved
    /// This bit is reserved and shall be set to zero in this version of the specification.
    #[skip] // Always set to 0, not exposed to user
    __: B2,

    /// Bit 7: BYTE
    /// This bit indicates the length unit for Length.
    ///   0 = Length is in bits
    ///   1 = Length is in octets (bytes)
    pub byte: B1,
}

impl Default for ProcessDataOut {
    fn default() -> Self {
        Self::new().with_length(0).with_byte(0)
    }
}

pub enum DeviceMode {
    Idle,
    Sio,
}

impl Default for DeviceMode {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Clone, Debug)]
pub struct DeviceCom {
    pub suppported_sio_mode: SioMode,
    pub transmission_rate: TransmissionRate,
    pub min_cycle_time: MinCycleTime,
    pub msequence_capability: MsequenceCapability,
    pub revision_id: RevisionId,
    pub process_data_in: ProcessDataIn,
    pub process_data_out: ProcessDataOut,
}

impl Default for DeviceCom {
    fn default() -> Self {
        Self {
            suppported_sio_mode: SioMode::default(),
            transmission_rate: TransmissionRate::default(),
            min_cycle_time: MinCycleTime::default(),
            msequence_capability: MsequenceCapability::default(),
            revision_id: RevisionId::default(),
            process_data_in: ProcessDataIn::default(),
            process_data_out: ProcessDataOut::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DeviceIdent {
    /// {VID} Vendor ID
    pub vendor_id: [u8; 2],
    /// {DID} Device ID
    pub device_id: [u8; 3],
    /// {FID} Function ID
    pub function_id: [u8; 2],
}

impl Default for DeviceIdent {
    fn default() -> Self {
        Self {
            vendor_id: [0, 0],
            device_id: [0, 0, 0],
            function_id: [0, 0],
        }
    }
}

pub trait SystemManagementReq {
    fn sm_set_device_com_req(&mut self, device_com: &DeviceCom) -> SmResult<()>;
    fn sm_get_device_com_req(&mut self, application_layer: &al::ApplicationLayer) -> SmResult<()>;
    fn sm_set_device_ident_req(&mut self, device_ident: &DeviceIdent) -> SmResult<()>;
    fn sm_get_device_ident_req(&mut self, application_layer: &al::ApplicationLayer) -> SmResult<()>;
    fn sm_set_device_mode_req(&mut self, mode: DeviceMode) -> SmResult<()>;
}

pub trait SystemManagementCnf {
    fn sm_set_device_com_cnf(&self, result: SmResult<()>) -> SmResult<()>;
    fn sm_get_device_com_cnf(&self, result: SmResult<&DeviceCom>) -> SmResult<()>;
    fn sm_set_device_ident_cnf(&self, result: SmResult<()>) -> SmResult<()>;
    fn sm_get_device_ident_cnf(&self, result: SmResult<&DeviceIdent>) -> SmResult<()>;
    fn sm_set_device_mode_cnf(&self, result: SmResult<()>) -> SmResult<()>;
}

pub trait SystemManagementInd {
    fn sm_device_mode_ind(&mut self, mode: types::DeviceMode) -> SmResult<()>;
}

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
    T4(types::DeviceMode),

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
    /// {SM_Idle_0} In SM_Idle the SM is waiting for configuration by the Device application and to be set
    /// to SIO mode. The state is left on receiving a SM_SetDeviceMode(SIO) request from the
    /// Device application
    /// The following sequence of services shall be executed between Device application and
    /// SM.
    /// Invoke SM_SetDeviceCom(initial parameter list)
    /// Invoke SM_SetDeviceIdent(VID, initial DID, FID)
    Idle,

    /// {SM_SIO_1} In SM_SIO the SM Line Handler is remaining in the default SIO mode. The Physical
    /// Layer is set to the SIO mode characteristics defined by the Device application via the
    /// SetDeviceMode service. The state is left on receiving a DL_Mode(ESTABCOM)
    /// indication.
    Sio,

    /// {SM_ComEstablish_2} In SM_ComEstablish the SM is waiting for the communication to be established in the
    /// Data Link Layer. The state is left on receiving a DL_Mode(INACTIVE) or a
    /// DL_Mode(COMx) indication, where COMx may be any of COM1, COM2 or COM3.
    ComEstablish,

    /// {SM_ComStartup_3} In SM_ComStartup the communication parameter (Direct Parameter page 1, addresses
    /// 0x02 to 0x06) are read by the Master SM via DL_Read requests. The state is left upon
    /// reception of a DL_Mode(INACTIVE), a DL_Mode(OPERATE) indication (legacy Master
    /// only), or a DL_Write(MCmd_MASTERIDENT) request (Master in accordance with this
    /// standard).
    ComStartup,

    /// {SM_IdentStartup_4} In SM_IdentStartup the identification data (VID, DID, FID) are read and verified by the
    /// Master. In case of incompatibilities the Master SM writes the supported SDCI Revision
    /// (RID) and configured DeviceID (DID) to the Device. The state is left upon reception of a
    /// DL_Mode(INACTIVE), a DL_Mode(PREOPERATE) indication (compatibility check
    /// passed), or a DL_Write(MCmd_DEVICEIDENT) request (new compatibility requested).
    IdentStartup,

    /// {SM_IdentCheck_5} In SM_IdentCheck the SM waits for new initialization of communication and
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

    /// {SM_CompStartup_6} In SM_CompatStartup the communication and identification data are reread and
    /// verified by the Master SM. The state is left on receiving a DL_Mode(INACTIVE) or a
    /// DL_Mode(PREOPERATE) indication.
    CompStartup,

    /// {SM_Preoperate_7} During SM_Preoperate the SerialNumber can be read and verified by the Master SM,
    /// as well as Data Storage and Device parameterization may be executed. The state is
    /// left on receiving a DL_Mode(INACTIVE), a DL_Mode(STARTUP) or a
    /// DL_Mode(OPERATE) indication.
    Preoperate,

    /// {SM_Operate_8} During SM_Operate the cyclic Process Data exchange and acyclic On-request Data
    /// transfer are active. The state is left on receiving a DL_Mode(INACTIVE) or a
    /// DL_Mode(STARTUP) indication.
    Operate,
}

/// System Management events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemManagementEvent {
    SmDeviceModeSio,
    DlModeEstabcom,
    DlModeComx(types::DeviceMode),
    DlModeInactive,
    DlModeStartup,
    DlModeOperate,
    DlWriteMCmdMasterident(u8, u8),
    DlWriteMCmdDeviceident(u8, u8),
    DlModePreoperate,
    DlReadMincycletime,
    TransmissionRateChanged,
}

#[derive(Default, Debug)]
struct ReConfig {
    revision_id: Option<RevisionId>,
    device_id1: Option<u8>,
    device_id2: Option<u8>,
    device_id3: Option<u8>,
}

/// System Management implementation
pub struct SystemManagement {
    state: SystemManagementState,
    exec_transition: Transition,
    device_com: DeviceCom,
    device_ident: DeviceIdent,
    device_mode: DeviceMode,
    reconfig: ReConfig,
}

impl SystemManagement {
    /// Create a new System Management instance
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
            (State::ComEstablish, Event::DlModeComx(mode)) => {
                self.exec_transition = Transition::T4(mode);
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
    pub fn poll(
        &mut self,
        application_layer: &mut al::ApplicationLayer,
        physical_layer: &mut pl::physical_layer::PhysicalLayer,
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

    fn poll_active_state(&mut self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
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
    fn execute_t1(
        &mut self,
        application_layer: &mut al::ApplicationLayer,
        physical_layer: &mut pl::physical_layer::PhysicalLayer,
    ) -> IoLinkResult<()> {
        // Invoke PL_SetMode(DI|DO|INACTIVE)
        let sio_mode = match self.device_com.suppported_sio_mode {
            SioMode::Inactive => IoLinkMode::Inactive,
            SioMode::Di => IoLinkMode::Di,
            SioMode::Do => IoLinkMode::Do,
        };
        physical_layer.pl_set_mode(sio_mode)?;
        // Invoke SM_DeviceMode(SIO)
        let _ = application_layer.sm_device_mode_ind(types::DeviceMode::Sio);
        Ok(())
    }

    /// Execute T2 transition: Switch to communication mode
    fn execute_t2(
        &mut self,
        application_layer: &mut al::ApplicationLayer,
        physical_layer: &mut pl::physical_layer::PhysicalLayer,
    ) -> IoLinkResult<()> {
        let com_mode = match self.device_com.transmission_rate {
            TransmissionRate::Com1 => IoLinkMode::Com1,
            TransmissionRate::Com2 => IoLinkMode::Com2,
            TransmissionRate::Com3 => IoLinkMode::Com3,
        };
        // Invoke PL_SetMode(COMx)
        physical_layer.pl_set_mode(com_mode)?;
        // Invoke SM_DeviceMode(ESTABCOM)
        let _ = application_layer.sm_device_mode_ind(types::DeviceMode::Estabcom);
        Ok(())
    }

    /// Execute T3 transition: Switch to SM_Idle mode
    fn execute_t3(
        &mut self,
        application_layer: &mut al::ApplicationLayer,
        physical_layer: &mut pl::physical_layer::PhysicalLayer,
    ) -> IoLinkResult<()> {
        // TODO: Cleanup any active communication sessions and reset state
        // Invoke PL_SetMode(INACTIVE)
        physical_layer.pl_set_mode(IoLinkMode::Inactive)?;
        // Invoke SM_DeviceMode(IDLE)
        let _ = application_layer.sm_device_mode_ind(types::DeviceMode::Idle);
        Ok(())
    }

    /// Execute T4 transition: Indicate baudrate establishment
    fn execute_t4(
        &mut self,
        mode: types::DeviceMode,
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
        let _ = application_layer.sm_device_mode_ind(types::DeviceMode::Identstartup);
        Ok(())
    }

    /// Execute T6 transition: Enter device identity check phase
    fn execute_t6(&mut self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
        // TODO: Handle master-provided RID and DID parameters for compatibility check
        // Invoke SM_DeviceMode(IDENTCHANGE)
        let _ = application_layer.sm_device_mode_ind(types::DeviceMode::Identchange);
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
        let _ = application_layer.sm_device_mode_ind(types::DeviceMode::Preoperate);
        Ok(())
    }

    /// Execute T9 transition: Enter device operate phase
    fn execute_t9(&mut self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
        // TODO: Initialize cyclic process data exchange and acyclic data transfer
        // Invoke SM_DeviceMode(OPERATE)
        let _ = application_layer.sm_device_mode_ind(types::DeviceMode::Operate);
        Ok(())
    }

    /// Execute T10 transition: Enter device preoperate phase from IdentStartup
    fn execute_t10(&mut self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
        // TODO: Handle direct transition from identification to preoperate
        // Invoke SM_DeviceMode(PREOPERATE)
        let _ = application_layer.sm_device_mode_ind(types::DeviceMode::Preoperate);
        Ok(())
    }

    /// Execute T11 transition: Enter device operate phase from ComStartup
    fn execute_t11(&mut self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
        // TODO: Handle legacy master behavior (direct transition to operate)
        // Invoke SM_DeviceMode(OPERATE)
        let _ = application_layer.sm_device_mode_ind(types::DeviceMode::Operate);
        Ok(())
    }

    /// Execute T12 transition: Enter communication startup phase from Preoperate
    fn execute_t12(&mut self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
        // TODO: Reset communication parameters and restart identification process
        // Invoke SM_DeviceMode(STARTUP)
        let _ = application_layer.sm_device_mode_ind(types::DeviceMode::Startup);
        Ok(())
    }

    /// Execute T13 transition: Enter communication startup phase from Operate
    fn execute_t13(&mut self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
        // TODO: Handle communication restart from operate mode
        // Invoke SM_DeviceMode(STARTUP)
        let _ = application_layer.sm_device_mode_ind(types::DeviceMode::Startup);
        Ok(())
    }

    /// Execute T14 transition: Change transmission rate and establish communication
    fn execute_t14(
        &mut self,
        application_layer: &mut al::ApplicationLayer,
        physical_layer: &mut pl::physical_layer::PhysicalLayer,
    ) -> IoLinkResult<()> {
        // TODO: Implement transmission rate change logic based on device identification requirements
        // Invoke PL_SetMode(COMx)
        // TODO: Change the COM1 to actual communication mode from the application
        physical_layer.pl_set_mode(IoLinkMode::Com1)?;
        // Invoke SM_DeviceMode(ESTABCOM)
        let _ = application_layer.sm_device_mode_ind(types::DeviceMode::Estabcom);
        Ok(())
    }
    /// See 9.3.2.7 SM_DeviceMode
    /// The SM_DeviceMode service is used to indicate changes of communication states to the
    /// Device application. The parameters of the service primitives are listed in Table 94.
    pub fn set_device_mode_req(&mut self, mode: types::DeviceMode) -> IoLinkResult<()> {
        // TODO: Implement proper device mode indication handling for all modes
        // TODO: Add device application notification mechanism
        if mode == types::DeviceMode::Sio {
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
    fn dl_mode_ind(&mut self, mode: types::DlMode) -> IoLinkResult<()> {
        use types::DlMode;
        match mode {
            DlMode::Inactive => self.process_event(SystemManagementEvent::DlModeInactive),
            DlMode::Com1 => {
                self.process_event(SystemManagementEvent::DlModeComx(types::DeviceMode::Com1))
            }
            DlMode::Com2 => {
                self.process_event(SystemManagementEvent::DlModeComx(types::DeviceMode::Com2))
            }
            DlMode::Com3 => {
                self.process_event(SystemManagementEvent::DlModeComx(types::DeviceMode::Com3))
            }
            DlMode::Estabcom => self.process_event(SystemManagementEvent::DlModeEstabcom),
            DlMode::Startup => self.process_event(SystemManagementEvent::DlModeStartup),
            DlMode::Preoperate => self.process_event(SystemManagementEvent::DlModePreoperate),
            DlMode::Operate => self.process_event(SystemManagementEvent::DlModeOperate),
            _ => Err(IoLinkError::InvalidEvent),
        }
    }

    fn dl_write_ind(&mut self, address: u8, value: u8) -> IoLinkResult<()> {
        match (address, value) {
            (direct_parameter_address!(MasterCommand), master_command!(MasterIdent)) => {
                self.process_event(SystemManagementEvent::DlWriteMCmdMasterident(
                    address, value,
                ));
            }
            (direct_parameter_address!(MasterCommand), master_command!(DeviceIdent)) => {
                self.process_event(SystemManagementEvent::DlWriteMCmdDeviceident(
                    address, value,
                ));
            }
            (direct_parameter_address!(RevisionID), value) => {
                self.reconfig.revision_id = Some(RevisionId::from_bytes([value]));
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

impl SystemManagementReq for SystemManagement {
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

    fn sm_get_device_ident_req(&mut self, application_layer: &al::ApplicationLayer) -> SmResult<()> {
        // Return the current device identification parameters
        let device_ident = Ok(&self.device_ident);
        application_layer.sm_get_device_ident_cnf(device_ident)?;
        Ok(())
    }

    fn sm_set_device_mode_req(&mut self, mode: DeviceMode) -> SmResult<()> {
        // Set the device mode
        self.device_mode = mode;
        Ok(())
    }
}
