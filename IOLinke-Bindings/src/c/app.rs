use heapless::Vec;
use iolinke_device::{
    AlEventCnf, ApplicationLayerServicesInd, DeviceCom, DeviceIdent, DeviceMode, DlControlCode,
    DlControlInd, IoLinkDevice,
};
use iolinke_types::{
    custom::IoLinkResult,
    handlers::{
        self,
        sm::{SmResult, SystemManagementCnf},
    },
    page,
};

pub use core::result::{Result, Result::{Ok, Err}};
use core::option::{Option, Option::{Some, None}};

use crate::c::{
    self,
    phy::BindingPhysicalLayer,
    types::{DeviceActionState, IOLinkeDeviceHandle, SmResultWrapper},
};

unsafe extern "C" {
    /// # `Integrator Implemented Function`
    /// Indicates the end of a Process Data cycle (AL_PDCycle service).
    ///
    /// This function must implement the AL_PDCycle local service as specified in IO-Link Interface Spec v1.1.4 Section 8.2.2.7.
    /// The AL_PDCycle service signals the completion of a Process Data cycle. The device application can use this service
    /// to transmit new input data to the application layer via AL_SetInput.
    ///
    /// # Parameters
    ///
    /// * `device_id` - The instance of the device which is generated from `io_linke_device_create`.
    ///
    /// # Specification Reference
    ///
    /// - IO-Link Interface Spec v1.1.4 Section 8.2.2.7: AL_PDCycle
    /// - Table 68: AL_PDCycle service parameters
    ///
    /// # Details
    ///
    /// The service-specific parameter is transmitted in the argument. The `device_id` parameter contains the port number
    /// associated with the received new Process Data. This service is typically called by the device application at the end
    /// of each Process Data cycle to notify the application layer and allow it to update its input data accordingly.
    ///
    /// # See Also
    ///
    /// - [`al_set_input_req`] for transmitting new input data to the application layer.
    ///
    fn al_pd_cycle_ind(device_id: IOLinkeDeviceHandle);

    /// # `Integrator Implemented Function`
    /// Indicates the receipt of updated output data within the Process Data of a Device (AL_NewOutput service).
    ///
    /// This function must implement the AL_NewOutput local service as specified in IO-Link Interface Spec v1.1.4 Section 8.2.2.9.
    /// The AL_NewOutput service signals that new output data has been received and is available in the Process Data area of the device.
    /// This service has no parameters according to the specification (see Table 70 – AL_NewOutput).
    ///
    /// # Parameters
    ///
    /// * `device_id` - The instance of the device which is generated from `io_linke_device_create`.
    /// * `len` - The length of the output data.
    /// * `pd_out` - Pointer to the output data buffer.
    ///
    /// # Specification Reference
    ///
    /// - IO-Link Interface Spec v1.1.4 Section 8.2.2.9: AL_NewOutput
    /// - Table 70: AL_NewOutput service parameters
    ///
    /// # Details
    ///
    /// The AL_NewOutput service is called by the device application to notify the application layer that updated output data
    /// has been received. This allows the application layer to process the new output data accordingly.
    ///
    /// # See Also
    ///
    /// - [`al_pd_cycle_ind`] for signaling the end of a Process Data cycle.
    ///
    fn al_new_output_ind(device_id: IOLinkeDeviceHandle, len: u8, pd_out: *const u8) -> bool;

    /// # `Integrator Implemented Function`
    /// Controls the Process Data qualifier status information for the device (AL_Control service).
    ///
    /// This function must implement the AL_Control local service as specified in IO-Link Interface Spec v1.1.4 Section 8.2.2.12.
    /// The AL_Control service transmits the Process Data qualifier status information to and from the Device application.
    /// It must be synchronized with AL_GetInput and AL_SetOutput services.
    ///
    /// # Parameters
    ///
    /// * `device_id` - The instance of the device generated from `io_linke_device_create`.
    /// * `control_code` - The qualifier status of the Process Data (PD). Permitted values:
    ///     - `VALID`: Input Process Data valid
    ///     - `INVALID`: Input Process Data invalid
    ///     - `PDOUTVALID`: Output Process Data valid
    ///     - `PDOUTINVALID`: Output Process Data invalid
    ///
    /// # Specification Reference
    ///
    /// - IO-Link Interface Spec v1.1.4 Section 8.2.2.12: AL_Control
    /// - Table 73: AL_Control service parameters
    ///
    /// # Details
    ///
    /// The service-specific parameters are transmitted in the argument. The `device_id` parameter contains the port number
    /// associated with the related device. The `control_code` parameter contains the qualifier status of the Process Data.
    /// This service is typically called by the device application to update the qualifier status of input or output Process Data.
    ///
    /// # See Also
    ///
    /// - [`al_get_input_req`] for reading input Process Data.
    /// - [`al_set_output_req`] for writing output Process Data.
    fn al_control_ind(device_id: IOLinkeDeviceHandle, control_code: DlControlCode) -> bool;

    /// # `Integrator Implemented Function`
    /// Indicates up to 6 pending status or error messages (AL_Event service).
    ///
    /// This function must implement the AL_Event local service as specified in IO-Link Interface Spec v1.1.4 Section 8.2.2.11.
    /// The AL_Event service signals the occurrence of status or error events, which can be triggered by the communication layer or by an application.
    /// The source of an event can be local (Master) or remote (Device).
    ///
    /// # Parameters
    ///
    /// * `device_id` - The instance of the device generated from `io_linke_device_create`.
    ///
    /// # Specification Reference
    ///
    /// - IO-Link Interface Spec v1.1.4 Section 8.2.2.11: AL_Event
    /// - Table 72: AL_Event service parameters
    ///
    /// # Details
    ///
    /// The service-specific parameters are transmitted in the argument. The AL_Event service can indicate up to 6 pending status or error messages.
    /// Each event contains the following elements:
    /// - `Instance`: Event source (Application)
    /// - `Mode`: Event mode (SINGLESHOT, APPEARS, DISAPPEARS)
    /// - `Type`: Event category (ERROR, WARNING, NOTIFICATION)
    /// - `Origin`: Indicates whether the event was generated locally or remotely (LOCAL, REMOTE)
    /// - `EventCode`: Code identifying the specific event
    ///
    /// The `Port` parameter contains the port number of the event data. The `EventCount` parameter indicates the number of events (1 to 6) in the event memory.
    ///
    /// # See Also
    ///
    /// - Table A.17 for permitted values of `Instance`
    /// - Table A.20 for permitted values of `Mode`
    /// - Table A.19 for permitted values of `Type`
    /// - Annex D for permitted values of `EventCode`
    fn al_event_cnf(device_id: IOLinkeDeviceHandle) -> bool;

    /// # `Integrator Implemented Function`
    /// This function is a response for the sm_set_device_com_req
    /// checkout `sm_set_device_com_req` for detailed documentation
    fn sm_set_device_com_cnf(device_id: IOLinkeDeviceHandle, result: SmResultWrapper);

    /// # `Integrator Implemented Function`
    /// This is a response function for the `sm_get_device_com_req`
    /// Reads the current communication properties of the device according to the IO-Link SM_GetDeviceCom service.
    ///
    /// This method retrieves the configured communication parameters from System Management, as specified in the IO-Link protocol
    /// (see Table 90 – SM_GetDeviceCom).
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the service has been executed successfully and the communication parameters have been retrieved.
    /// - `Err(SmError)` if the service failed, including error information such as state conflict.
    ///
    /// # Result Data
    ///
    /// On success, the following communication parameters are available:
    /// - `CurrentMode`: Indicates the current SIO or Communication Mode (INACTIVE, DI, DO, COM1, COM2, COM3).
    /// - `MasterCycleTime`: Contains the MasterCycleTime set by System Management (valid only in SM_Operate state).
    /// - `M-sequence Capability`: Indicates the current M-sequence capabilities (ISDU support, OPERATE, PREOPERATE types).
    /// - `RevisionID`: Current protocol revision.
    /// - `ProcessDataIn`: Current length of process data to be sent to the Master.
    /// - `ProcessDataOut`: Current length of process data to be sent by the Master.
    ///
    /// # Errors
    ///
    /// Returns an error if the service is unavailable within the current state (`STATE_CONFLICT`).
    ///
    /// # See Also
    ///
    /// - IO-Link Specification, Table 90 – SM_GetDeviceCom
    fn sm_get_device_com_cnf(device_id: IOLinkeDeviceHandle, result: SmResultWrapper);

    /// # `Integrator Implemented Function`
    /// This is a response function for the `sm_set_device_ident_req`
    /// Checkout `sm_set_device_ident_req` for detailed documentation
    fn sm_set_device_ident_cnf(device_id: IOLinkeDeviceHandle, result: SmResultWrapper);

    /// # `Integrator Implemented Function`
    /// This is a response for the function `sm_get_device_ident_req`
    /// Reads the device identification parameters according to the IO-Link SM_GetDeviceIdent service.
    ///
    /// This method retrieves the configured identification parameters from System Management, as specified in the IO-Link protocol
    /// (see Table 92 – SM_GetDeviceIdent).
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the service has been executed successfully and the identification parameters have been retrieved.
    /// - `Err(SmError)` if the service failed, including error information such as state conflict.
    ///
    /// # Result Data
    ///
    /// On success, the following identification parameters are available:
    /// - `VendorID (VID)`: The actual VendorID of the device (2 octets).
    /// - `DeviceID (DID)`: The actual DeviceID of the device (3 octets).
    /// - `FunctionID (FID)`: The actual FunctionID of the device (2 octets).
    ///
    /// # Errors
    ///
    /// Returns an error if the service is unavailable within the current state (`STATE_CONFLICT`).
    ///
    /// # See Also
    ///
    /// - IO-Link Specification, Table 92 – SM_GetDeviceIdent
    fn sm_get_device_ident_cnf(device_id: IOLinkeDeviceHandle, result: SmResultWrapper);

    /// # `Integrator Implemented Function`
    /// This is a response function for the `sm_set_device_mode_req`
    /// Checkout the `sm_set_device_mode_req` for detailed documentation
    fn sm_set_device_mode_cnf(device_id: IOLinkeDeviceHandle, result: SmResultWrapper);
}

static NUM_OF_DEVICES: usize = 1; // Number of devices which can be created, //! For now only 1 device is supported.
pub static mut IOLINKE_DEVICES: [Option<(
    IoLinkDevice<BindingPhysicalLayer, BindingApplicationLayer>,
    DeviceActionState,
)>; NUM_OF_DEVICES] = [None; NUM_OF_DEVICES];

const PD_OUTPUT_LENGTH: usize =
    iolinke_derived_config::device::process_data::pd_out::config_length_in_bytes() as usize;

pub trait SmResultExt {
    fn to_operation_result(&self) -> crate::c::types::SmResultWrapper;
}

impl SmResultExt for DeviceCom {
    fn to_operation_result(&self) -> crate::c::types::SmResultWrapper {
        let min_cycle_time = c::types::CycleTime {
            multiplier: self.min_cycle_time.multiplier(),
            time_base: self.min_cycle_time.time_base(),
        };
        let mseq_cap = c::types::MsequenceCapability {
            isdu: self.msequence_capability.isdu(),
            preoperate_m_sequence: self.msequence_capability.preoperate_m_sequence(),
            operate_m_sequence: self.msequence_capability.operate_m_sequence(),
        };
        let revision_id = c::types::RevisionId {
            major_rev: self.revision_id.major_rev(),
            minor_rev: self.revision_id.minor_rev(),
        };
        let process_data_in = c::types::ProcessDataIn {
            byte: self.process_data_in.byte(),
            sio: self.process_data_in.sio(),
            length: self.process_data_in.length(),
        };
        let process_data_out = c::types::ProcessDataOut {
            byte: self.process_data_out.byte(),
            length: self.process_data_out.length(),
        };

        crate::c::types::SmResultWrapper {
            result_type: c::types::SmResultType::DeviceCom,
            result: c::types::SmResult {
                device_com: c::types::DeviceCom {
                    suppported_sio_mode: self.suppported_sio_mode,
                    transmission_rate: self.transmission_rate,
                    min_cycle_time: min_cycle_time,
                    msequence_capability: mseq_cap,
                    process_data_in: process_data_in,
                    process_data_out: process_data_out,
                    revision_id: revision_id,
                },
            },
        }
    }
}

impl SmResultExt for DeviceIdent {
    fn to_operation_result(&self) -> crate::c::types::SmResultWrapper {
        crate::c::types::SmResultWrapper {
            result_type: c::types::SmResultType::DeviceCom,
            result: c::types::SmResult {
                device_ident: self.clone(),
            },
        }
    }
}

pub struct BindingApplicationLayer {
    device_id: IOLinkeDeviceHandle,
}

/// Main polling function that advances all protocol state machines.
///
/// This method must be called regularly to:
/// - Process incoming messages from the master
/// - Update internal state machines
/// - Handle timeouts and state transitions
/// - Process application layer requests
///
/// The polling order follows the IO-Link specification dependency chain:
/// 1. Application Layer → Data Link Layer
/// 2. Data Link Layer → Physical Layer + System Management
/// 3. System Management → Physical Layer + Application Layer
///
#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
pub extern "C" fn iolinke_device_poll(device_id: IOLinkeDeviceHandle) -> DeviceActionState {
    let (device, state) = unsafe {
        if let Some((device, state)) = IOLINKE_DEVICES
            .get_mut(device_id as usize)
            .and_then(|opt| opt.as_mut())
        {
            (device, state)
        } else {
            return DeviceActionState::NoDevice;
        }
    };
    match state {
        DeviceActionState::Done => {
            let _ = device.poll();
            DeviceActionState::Done
        }
        _ => DeviceActionState::Busy, // Previous operation still in progress
    }
}

/// Creates a new instance of IOLinke device.
///
/// # Arguments
/// * None
///
/// # Returns
///
/// A new `BindingApplicationLayer` instance initialized with the specified index aka device ID.
///
/// # Arguments
/// * None
///
/// # Returns
///
/// A new `BindingApplicationLayer` instance initialized with the specified index aka device ID.
#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
pub extern "C" fn io_linke_device_create() -> IOLinkeDeviceHandle {
    unsafe {
        for (i, device_option) in IOLINKE_DEVICES.iter_mut().enumerate() {
            if device_option.is_none() {
                let pl = BindingPhysicalLayer::new(i as i16);
                let al = BindingApplicationLayer::new(i as i16);
                let device = IoLinkDevice::new(pl, al);
                *device_option = Some((device, DeviceActionState::Done));
                return i as i16;
            }
        }
    }
    // No available slot
    -1
}

/// Updates the input data within the Process Data of the device (AL_SetInput service).
///
/// This method implements the AL_SetInput local service, which updates the input data
/// in the device's Process Data area as specified by IO-Link. The input data is provided
/// as an octet string and transmitted to the Application Layer.
///
/// # Parameters
///
/// * `length` - The length of the input data to be transmitted.
/// * `input_data` - A slice containing the Process Data values (octet string) to be set.
///
/// # Returns
///
/// * `Ok(())` if the input data was updated successfully.
/// * `Err(IoLinkError)` if the service failed (e.g., due to a state conflict).
///
/// # Errors
///
/// - `IoLinkError::StateConflict`: Service unavailable within the current state.
///
/// # Specification Reference
///
/// - IO-Link Interface Spec v1.1.4 Section 8.2.2.6: AL_SetInput
/// - Table 67: AL_SetInput service parameters
///
#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
pub extern "C" fn al_set_input_req(
    device_id: IOLinkeDeviceHandle,
    len: u8,
    data: *const u8,
) -> DeviceActionState {
    let (device, state, pd_in_slice) = unsafe {
        if let Some((device, state)) = IOLINKE_DEVICES
            .get_mut(device_id as usize)
            .and_then(|opt| opt.as_mut())
        {
            let pd_in_slice = core::slice::from_raw_parts(data, len as usize);
            (device, state, pd_in_slice)
        } else {
            return DeviceActionState::NoDevice; // Invalid device ID
        }
    };
    match state {
        DeviceActionState::Done => {
            let _ = device.al_set_input_req(len, pd_in_slice);
            DeviceActionState::Done
        }
        _ => DeviceActionState::Busy, // Previous operation still in progress
    }
}

/// Sets the device communication parameters according to IO-Link SM_SetDeviceCom service.
///
/// This method configures the device's communication and identification parameters
/// as specified in the IO-Link protocol (see Table 89 – SM_SetDeviceCom).
///
/// # Parameters
///
/// * `device_com` - Reference to the device communication configuration, including:
///     - Supported SIO mode (`SupportedSIOMode`): INACTIVE, DI, DO
///     - Supported transmission rate (`SupportedTransmissionrate`): COM1, COM2, COM3
///     - Minimum cycle time (`MinCycleTime`)
///     - M-sequence capability (`M-sequence Capability`)
///     - Protocol revision (`RevisionID`)
///     - Process data lengths (`ProcessDataIn`, `ProcessDataOut`)
///
/// # Returns
///
/// - `Ok(())` if the service has been executed successfully.
/// - `Err(SmError)` if the service failed, including error information such as parameter conflict.
///
/// # Errors
///
/// Returns an error if the parameter set is inconsistent or violates protocol requirements.
///
/// # See Also
///
/// - IO-Link Specification, Table 89 – SM_SetDeviceCom
#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
pub extern "C" fn sm_set_device_com_req(device_id: IOLinkeDeviceHandle, com: &c::types::DeviceCom) {
    let (device, _state) = unsafe {
        if let Some((device, state)) = IOLINKE_DEVICES
            .get_mut(device_id as usize)
            .and_then(|opt| opt.as_mut())
        {
            (device, state)
        } else {
            return; // Invalid device ID
        }
    };
    let mut min_cycle_time = page::page1::CycleTime::new();
    min_cycle_time.set_multiplier(com.min_cycle_time.multiplier);
    min_cycle_time.set_time_base(com.min_cycle_time.time_base);

    let mut m_seq_capability = page::page1::MsequenceCapability::new();
    m_seq_capability.set_isdu(com.msequence_capability.isdu);
    m_seq_capability.set_operate_m_sequence(com.msequence_capability.operate_m_sequence);
    m_seq_capability.set_preoperate_m_sequence(com.msequence_capability.preoperate_m_sequence);

    let mut revision_id = page::page1::RevisionId::new();
    revision_id.set_major_rev(com.revision_id.major_rev);
    revision_id.set_minor_rev(com.revision_id.minor_rev);

    let mut process_data_in = page::page1::ProcessDataIn::new();
    process_data_in.set_byte(com.process_data_in.byte);
    process_data_in.set_sio(com.process_data_in.sio);
    process_data_in.set_length(com.process_data_in.length);

    let mut process_data_out = page::page1::ProcessDataOut::new();
    process_data_out.set_byte(com.process_data_out.byte);
    process_data_out.set_length(com.process_data_out.length);

    let com = handlers::sm::DeviceCom {
        suppported_sio_mode: com.suppported_sio_mode.clone(),
        transmission_rate: com.transmission_rate,
        min_cycle_time: min_cycle_time,
        msequence_capability: m_seq_capability,
        revision_id: revision_id,
        process_data_in: process_data_in,
        process_data_out: process_data_out,
    };

    let _ = device.sm_set_device_com_req(&com);
}

/// Sets the device identification parameters according to IO-Link SM_SetDeviceIdent service.
///
/// This method configures the device's identification data as specified in the IO-Link protocol
/// (see Table 91 – SM_SetDeviceIdent).
///
/// # Parameters
///
/// * `device_ident` - Reference to the device identification configuration, including:
///     - VendorID (`VID`): Vendor identifier assigned to the device (2 octets)
///     - DeviceID (`DID`): Device identifier assigned to the device (3 octets)
///     - FunctionID (`FID`): Function identifier assigned to the device (2 octets)
///
/// # Returns
///
/// - `Ok(())` if the service has been executed successfully.
/// - `Err(SmError)` if the service failed, including error information such as state conflict or parameter conflict.
///
/// # Errors
///
/// Returns an error if the service is unavailable within the current state (`STATE_CONFLICT`)
/// or if the consistency of the parameter set is violated (`PARAMETER_CONFLICT`).
///
/// # See Also
///
/// - IO-Link Specification, Table 91 – SM_SetDeviceIdent
#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
pub extern "C" fn sm_set_device_ident_req(device_id: IOLinkeDeviceHandle, ident: &DeviceIdent) {
    let (device, _state) = unsafe {
        if let Some((device, state)) = IOLINKE_DEVICES
            .get_mut(device_id as usize)
            .and_then(|opt| opt.as_mut())
        {
            (device, state)
        } else {
            return; // Invalid device ID
        }
    };
    let _ = device.sm_set_device_ident_req(ident);
}

/// Sets the device operational mode according to the IO-Link SM_SetDeviceMode service.
///
/// This method sets the device into a defined operational state during initialization,
/// as specified in the IO-Link protocol (see Table 93 – SM_SetDeviceMode).
///
/// # Parameters
///
/// * `mode` - The desired device mode to set. Permitted values:
///     - `DeviceMode::Idle`: Device changes to waiting for configuration.
///     - `DeviceMode::Sio`: Device changes to the mode defined in the SM_SetDeviceCom service.
///
/// # Returns
///
/// - `Ok(())` if the service has been executed successfully.
/// - `Err(SmError)` if the service failed, including error information such as state conflict.
///
/// # Errors
///
/// Returns an error if the service is unavailable within the current state (`STATE_CONFLICT`).
///
/// # See Also
///
/// - IO-Link Specification, Table 93 – SM_SetDeviceMode
#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
pub extern "C" fn sm_set_device_mode_req(device_id: IOLinkeDeviceHandle, mode: DeviceMode) {
    let (device, _state) = unsafe {
        if let Some((device, state)) = IOLINKE_DEVICES
            .get_mut(device_id as usize)
            .and_then(|opt| opt.as_mut())
        {
            (device, state)
        } else {
            return; // Invalid device ID
        }
    };
    let _ = device.sm_set_device_mode_req(mode);
}

/// Reads the current communication properties of the device according to the IO-Link SM_GetDeviceCom service.
///
/// This method retrieves the configured communication parameters from System Management, as specified in the IO-Link protocol
/// (see Table 90 – SM_GetDeviceCom).
///
/// # Returns
///
/// - `Ok(())` if the service has been executed successfully and the communication parameters have been retrieved.
/// - `Err(SmError)` if the service failed, including error information such as state conflict.
///
/// # Result Data
///
/// On success, the following communication parameters are available:
/// - `CurrentMode`: Indicates the current SIO or Communication Mode (INACTIVE, DI, DO, COM1, COM2, COM3).
/// - `MasterCycleTime`: Contains the MasterCycleTime set by System Management (valid only in SM_Operate state).
/// - `M-sequence Capability`: Indicates the current M-sequence capabilities (ISDU support, OPERATE, PREOPERATE types).
/// - `RevisionID`: Current protocol revision.
/// - `ProcessDataIn`: Current length of process data to be sent to the Master.
/// - `ProcessDataOut`: Current length of process data to be sent by the Master.
///
/// # Errors
///
/// Returns an error if the service is unavailable within the current state (`STATE_CONFLICT`).
///
/// # See Also
///
/// - IO-Link Specification, Table 90 – SM_GetDeviceCom
#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
pub extern "C" fn sm_get_device_com_req(device_id: IOLinkeDeviceHandle) {
    let (device, _state) = unsafe {
        if let Some((device, state)) = IOLINKE_DEVICES
            .get_mut(device_id as usize)
            .and_then(|opt| opt.as_mut())
        {
            (device, state)
        } else {
            return; // Invalid device ID
        }
    };
    let _ = device.sm_get_device_com_req();
}

/// Reads the device identification parameters according to the IO-Link SM_GetDeviceIdent service.
///
/// This method retrieves the configured identification parameters from System Management, as specified in the IO-Link protocol
/// (see Table 92 – SM_GetDeviceIdent).
///
/// # Returns
///
/// - `Ok(())` if the service has been executed successfully and the identification parameters have been retrieved.
/// - `Err(SmError)` if the service failed, including error information such as state conflict.
///
/// # Result Data
///
/// On success, the following identification parameters are available:
/// - `VendorID (VID)`: The actual VendorID of the device (2 octets).
/// - `DeviceID (DID)`: The actual DeviceID of the device (3 octets).
/// - `FunctionID (FID)`: The actual FunctionID of the device (2 octets).
///
/// # Errors
///
/// Returns an error if the service is unavailable within the current state (`STATE_CONFLICT`).
///
/// # See Also
///
/// - IO-Link Specification, Table 92 – SM_GetDeviceIdent
#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
pub extern "C" fn sm_get_device_ident_req(device_id: IOLinkeDeviceHandle) {
    let (device, _state) = unsafe {
        if let Some((device, state)) = IOLINKE_DEVICES
            .get_mut(device_id as usize)
            .and_then(|opt| opt.as_mut())
        {
            (device, state)
        } else {
            return; // Invalid device ID
        }
    };
    let result = device.sm_get_device_ident_req();
    let _ = result;
}

impl BindingApplicationLayer {
    pub fn new(device_id: IOLinkeDeviceHandle) -> Self {
        Self { device_id }
    }
}

impl DlControlInd for BindingApplicationLayer {
    fn dl_control_ind(&mut self, control_code: DlControlCode) -> IoLinkResult<()> {
        let _ok = unsafe { al_control_ind(self.device_id, control_code) };
        Ok(())
    }
}

impl ApplicationLayerServicesInd for BindingApplicationLayer {
    fn al_read_ind(&mut self, _index: u16, _sub_index: u8) -> IoLinkResult<()> {
        // This api is here for the future use, currently not expected to be called
        Err(iolinke_types::custom::IoLinkError::NoImplFound)
    }

    fn al_write_ind(&mut self, _index: u16, _sub_index: u8, _data: &[u8]) -> IoLinkResult<()> {
        // This api is here for the future use, currently not expected to be called
        Err(iolinke_types::custom::IoLinkError::NoImplFound)
    }

    fn al_abort_ind(&mut self) -> IoLinkResult<()> {
        // This api is here for the future use, currently not expected to be called
        Err(iolinke_types::custom::IoLinkError::NoImplFound)
    }

    fn al_pd_cycle_ind(&mut self) {
        unsafe { al_pd_cycle_ind(self.device_id) };
    }

    fn al_new_output_ind(&mut self, pd_out: &Vec<u8, { PD_OUTPUT_LENGTH }>) -> IoLinkResult<()> {
        let len = pd_out.len() as u8;
        let _ok = unsafe { al_new_output_ind(self.device_id, len, pd_out.as_ptr()) };
        Ok(())
    }

    fn al_control_ind(&mut self, control_code: DlControlCode) -> IoLinkResult<()> {
        let _ok = unsafe { al_control_ind(self.device_id, control_code) };
        Ok(())
    }
}

impl AlEventCnf for BindingApplicationLayer {
    fn al_event_cnf(&mut self) -> IoLinkResult<()> {
        let _ok = unsafe { al_event_cnf(self.device_id) };
        Ok(())
    }
}

impl SystemManagementCnf for BindingApplicationLayer {
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
    fn sm_set_device_com_cnf(&self, result: SmResult<()>) -> SmResult<()> {
        let op_result = match result {
            Ok(_) => c::types::SmResultWrapper {
                result_type: c::types::SmResultType::Ok,
                result: c::types::SmResult { err_code: 0 },
            },
            Err(_) => c::types::SmResultWrapper {
                result_type: c::types::SmResultType::Err,
                result: c::types::SmResult { err_code: -1 },
            },
        };
        unsafe { sm_set_device_com_cnf(self.device_id, op_result) };
        Ok(())
    }

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
    fn sm_get_device_com_cnf(&self, result: SmResult<&handlers::sm::DeviceCom>) -> SmResult<()> {
        let op_result = match result {
            Ok(device_com) => device_com.to_operation_result(),
            Err(_) => c::types::SmResultWrapper {
                result_type: c::types::SmResultType::Err,
                result: c::types::SmResult { err_code: -1 },
            },
        };
        unsafe { sm_get_device_com_cnf(self.device_id, op_result) };
        Ok(())
    }

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
    fn sm_set_device_ident_cnf(&self, result: SmResult<()>) -> SmResult<()> {
        let op_result = match result {
            Ok(_) => c::types::SmResultWrapper {
                result_type: c::types::SmResultType::Ok,
                result: c::types::SmResult { err_code: 0 },
            },
            Err(_) => c::types::SmResultWrapper {
                result_type: c::types::SmResultType::Err,
                result: c::types::SmResult { err_code: -1 },
            },
        };
        unsafe { sm_set_device_ident_cnf(self.device_id, op_result) };
        Ok(())
    }

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
    fn sm_get_device_ident_cnf(&self, result: SmResult<&DeviceIdent>) -> SmResult<()> {
        let op_result = match result {
            Ok(device_com) => device_com.to_operation_result(),
            Err(_) => c::types::SmResultWrapper {
                result_type: c::types::SmResultType::Err,
                result: c::types::SmResult { err_code: -1 },
            },
        };
        unsafe { sm_get_device_ident_cnf(self.device_id, op_result) };
        Ok(())
    }

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
    fn sm_set_device_mode_cnf(&self, result: SmResult<()>) -> SmResult<()> {
        let op_result = match result {
            Ok(_) => c::types::SmResultWrapper {
                result_type: c::types::SmResultType::Ok,
                result: c::types::SmResult { err_code: 0 },
            },
            Err(_) => c::types::SmResultWrapper {
                result_type: c::types::SmResultType::Err,
                result: c::types::SmResult { err_code: -1 },
            },
        };
        unsafe { sm_set_device_mode_cnf(self.device_id, op_result) };
        Ok(())
    }
}
