use crate::{
    frame::msequence::TransmissionRate,
    page::page1::{
        CycleTime, DeviceIdent, MsequenceCapability, ProcessDataIn, ProcessDataOut, RevisionId,
    },
};

/// System Management request interface.
///
/// This trait defines the interface that the application layer uses to
/// request system management operations.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Section 6.3: System Management State Machine
/// - Section 7.3: Device Identification and Communication
pub trait SystemManagementReq<App> {
    /// Sets the device communication parameters.
    ///
    /// This method configures the device's communication capabilities
    /// and settings.
    ///
    /// # Parameters
    ///
    /// * `device_com` - Device communication configuration
    ///
    /// # Returns
    ///
    /// - `Ok(())` if configuration was successful
    /// - `Err(SmError)` if an error occurred
    fn sm_set_device_com_req(&mut self, device_com: &DeviceCom) -> SmResult<()>;

    /// Gets the device communication parameters.
    ///
    /// This method retrieves the current device communication
    /// configuration.
    ///
    /// # Parameters
    ///
    /// * `application_layer` - Reference to application layer for coordination
    ///
    /// # Returns
    ///
    /// - `Ok(())` if request was successful
    /// - `Err(SmError)` if an error occurred
    fn sm_get_device_com_req(&mut self, application_layer: &App) -> SmResult<()>;

    /// Sets the device identification parameters.
    ///
    /// This method configures the device's identification information
    /// including vendor ID, device ID, and function ID.
    ///
    /// # Parameters
    ///
    /// * `device_ident` - Device identification parameters
    ///
    /// # Returns
    ///
    /// - `Ok(())` if configuration was successful
    /// - `Err(SmError)` if an error occurred
    fn sm_set_device_ident_req(&mut self, device_ident: &DeviceIdent) -> SmResult<()>;

    /// Gets the device identification parameters.
    ///
    /// This method retrieves the current device identification
    /// configuration.
    ///
    /// # Parameters
    ///
    /// * `application_layer` - Reference to application layer for coordination
    ///
    /// # Returns
    ///
    /// - `Ok(())` if request was successful
    /// - `Err(SmError)` if an error occurred
    fn sm_get_device_ident_req(&mut self, application_layer: &App) -> SmResult<()>;

    /// Sets the device operating mode.
    ///
    /// This method changes the device's operating mode according
    /// to the IO-Link state machine.
    ///
    /// # Parameters
    ///
    /// * `mode` - New device operating mode
    ///
    /// # Returns
    ///
    /// - `Ok(())` if mode change was successful
    /// - `Err(SmError)` if an error occurred
    fn sm_set_device_mode_req(&mut self, mode: DeviceMode) -> SmResult<()>;
}

/// System Management confirmation interface.
///
/// This trait defines the interface that the application layer uses to
/// receive confirmations for system management operations.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Section 6.3: System Management State Machine
/// - Section 7.3: Device Identification and Communication
pub trait SystemManagementCnf {
    /// Confirms device communication setup operation.
    ///
    /// This method is called when the system management confirms
    /// a device communication setup operation.
    ///
    /// # Parameters
    ///
    /// * `result` - Result of the communication setup operation
    ///
    /// # Returns
    ///
    /// - `Ok(())` if confirmation was processed successfully
    /// - `Err(SmError)` if an error occurred
    fn sm_set_device_com_cnf(&self, result: SmResult<()>) -> SmResult<()>;

    /// Confirms device communication get operation.
    ///
    /// This method is called when the system management confirms
    /// a device communication get operation.
    ///
    /// # Parameters
    ///
    /// * `result` - Result containing device communication parameters
    ///
    /// # Returns
    ///
    /// - `Ok(())` if confirmation was processed successfully
    /// - `Err(SmError)` if an error occurred
    fn sm_get_device_com_cnf(&self, result: SmResult<&DeviceCom>) -> SmResult<()>;

    /// Confirms device identification setup operation.
    ///
    /// This method is called when the system management confirms
    /// a device identification setup operation.
    ///
    /// # Parameters
    ///
    /// * `result` - Result of the identification setup operation
    ///
    /// # Returns
    ///
    /// - `Ok(())` if confirmation was processed successfully
    /// - `Err(SmError)` if an error occurred
    fn sm_set_device_ident_cnf(&self, result: SmResult<()>) -> SmResult<()>;

    /// Confirms device identification get operation.
    ///
    /// This method is called when the system management confirms
    /// a device identification get operation.
    ///
    /// # Parameters
    ///
    /// * `result` - Result containing device identification parameters
    ///
    /// # Returns
    ///
    /// - `Ok(())` if confirmation was processed successfully
    /// - `Err(SmError)` if an error occurred
    fn sm_get_device_ident_cnf(&self, result: SmResult<&DeviceIdent>) -> SmResult<()>;

    /// Confirms device mode change operation.
    ///
    /// This method is called when the system management confirms
    /// a device mode change operation.
    ///
    /// # Parameters
    ///
    /// * `result` - Result of the mode change operation
    ///
    /// # Returns
    ///
    /// - `Ok(())` if confirmation was processed successfully
    /// - `Err(SmError)` if an error occurred
    fn sm_set_device_mode_cnf(&self, result: SmResult<()>) -> SmResult<()>;
}

/// System Management indication interface.
///
/// This trait defines the interface that the application layer uses to
/// receive notifications from system management.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Section 6.3: System Management State Machine
/// - Section 7.3: Device Identification and Communication
pub trait SystemManagementInd {
    /// Indicates device mode change.
    ///
    /// This method is called when the system management changes
    /// the device operating mode.
    ///
    /// # Parameters
    ///
    /// * `mode` - New device operating mode
    ///
    /// # Returns
    ///
    /// - `Ok(())` if mode change was processed successfully
    /// - `Err(SmError)` if an error occurred
    fn sm_device_mode_ind(&mut self, mode: DeviceMode) -> SmResult<()>;
}
/// System Management error types.
///
/// These error types are used when system management operations
/// fail due to parameter conflicts or other issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmError {
    /// Parameter conflict detected during operation
    ParameterConflict,
}

/// Result type for system management operations.
///
/// This type alias provides a convenient way to handle system management
/// operation results with the appropriate error type.
pub type SmResult<T> = Result<T, SmError>;

/// SIO (Standard I/O) mode configuration options.
///
/// SIO mode provides digital input/output functionality without
/// the full IO-Link communication protocol.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Section 5.2.2: Communication Modes
/// - Section 5.3: SIO Mode Operation
#[derive(Clone, Debug)]
pub enum SioMode {
    /// C/Q line in high impedance state (no communication)
    Inactive,
    /// C/Q line configured as digital input
    Di,
    /// C/Q line configured as digital output
    Do,
}

impl Default for SioMode {
    /// Default SIO mode is inactive (high impedance).
    fn default() -> Self {
        Self::Inactive
    }
}
/// Device operating mode as per Section 6.3.
///
/// The device mode represents the current state of the IO-Link
/// device state machine.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Section 6.3: State Machines
/// - Table 94: SM_DeviceMode state definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceMode {
    /// Device is idle and waiting for configuration
    Idle,
    /// Device is in SIO mode (Standard I/O)
    Sio,
    /// Device is establishing communication with master
    Estabcom,
    /// Device is in COM1 communication mode
    Com1,
    /// Device is in COM2 communication mode
    Com2,
    /// Device is in COM3 communication mode
    Com3,
    /// Device is starting up and initializing
    Startup,
    /// Device is in identification startup phase
    Identstartup,
    /// Device is checking identification parameters
    Identchange,
    /// Device is in preoperate mode (parameterized but not operational)
    Preoperate,
    /// Device is in full operate mode (fully operational)
    Operate,
}

impl Default for DeviceMode {
    /// Default device mode is idle (waiting for configuration).
    fn default() -> Self {
        Self::Idle
    }
}

/// Device communication configuration parameters.
///
/// This struct contains all the communication-related parameters
/// that define the device's communication capabilities and settings.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Annex B.1: Direct Parameter Page 1
/// - Section B.1.1: Communication Parameters
#[derive(Clone, Debug)]
pub struct DeviceCom {
    /// Supported SIO mode configuration
    pub suppported_sio_mode: SioMode,
    /// Transmission rate for IO-Link communication
    pub transmission_rate: TransmissionRate,
    /// Minimum cycle time configuration
    pub min_cycle_time: CycleTime,
    /// M-sequence capability configuration
    pub msequence_capability: MsequenceCapability,
    /// Protocol revision identifier
    pub revision_id: RevisionId,
    /// Process data input configuration
    pub process_data_in: ProcessDataIn,
    /// Process data output configuration
    pub process_data_out: ProcessDataOut,
}

impl Default for DeviceCom {
    /// Default device communication configuration with minimal settings.
    fn default() -> Self {
        Self {
            suppported_sio_mode: SioMode::default(),
            transmission_rate: TransmissionRate::default(),
            min_cycle_time: CycleTime::default(),
            msequence_capability: MsequenceCapability::default(),
            revision_id: RevisionId::default(),
            process_data_in: ProcessDataIn::default(),
            process_data_out: ProcessDataOut::default(),
        }
    }
}

/// IO-Link communication mode as defined in Section 5.2.2.
///
/// The communication mode determines the baud rate and protocol
/// characteristics used for master-device communication.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Section 5.2.2: Communication Modes
/// - Table 5.1: Communication mode characteristics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IoLinkMode {
    /// SIO mode (Standard I/O) - Digital input/output without communication
    Sio = 0,
    /// COM1 mode - 4.8 kbaud communication (1200 baud effective)
    Com1 = 1,
    /// COM2 mode - 38.4 kbaud communication (2400 baud effective)
    Com2 = 2,
    /// COM3 mode - 230.4 kbaud communication (4800 baud effective)
    Com3 = 3,
    /// DI mode - Digital input only
    Di = 4,
    /// DO mode - Digital output only
    Do = 5,
    /// INACTIVE mode - No communication, high impedance
    Inactive = 0xFF,
}
