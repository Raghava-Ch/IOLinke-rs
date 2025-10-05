//! Application Layer API implementation
//!
//! This module implements the Application Layer interface as defined in
//! IO-Link Specification v1.1.4 Section 8.4

use crate::dl;
use heapless::Vec;
use iolinke_types::custom::{IoLinkError, IoLinkResult};
use iolinke_types::handlers;

use core::result::Result::Err;

/// Application Layer trait defining all request/indication methods
/// See IO-Link v1.1.4 Section 8.4
pub trait ApplicationLayerServicesInd {
    /// Handles read requests from the master for device parameters.
    ///
    /// This method is called when the master requests to read a parameter
    /// from the device's object dictionary.
    ///
    /// # Parameters
    ///
    /// * `index` - 16-bit parameter index (see Annex B.8)
    /// * `sub_index` - 8-bit sub-index for complex parameters
    ///
    /// # Returns
    ///
    /// - `Ok(())` if read request was processed successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn al_read_ind(&mut self, index: u16, sub_index: u8) -> IoLinkResult<()>;

    /// Handles write requests from the master for device parameters.
    ///
    /// This method is called when the master requests to write a parameter
    /// to the device's object dictionary.
    ///
    /// # Parameters
    ///
    /// * `index` - 16-bit parameter index (see Annex B.8)
    /// * `sub_index` - 8-bit sub-index for complex parameters
    /// * `data` - Parameter data to be written
    ///
    /// # Returns
    ///
    /// - `Ok(())` if write request was processed successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn al_write_ind(&mut self, index: u16, sub_index: u8, data: &[u8]) -> IoLinkResult<()>;

    /// Handles an abort request from the master.
    ///
    /// This method is called when the master aborts an ongoing
    /// read or write operation.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if abort was processed successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn al_abort_ind(&mut self) -> IoLinkResult<()> {
        Err(IoLinkError::NoImplFound)
    }

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
    fn al_pd_cycle_ind(&mut self);

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
    fn al_new_output_ind(&mut self, pd_out: &Vec<u8, { dl::PD_OUTPUT_LENGTH }>)
    -> IoLinkResult<()>;

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
    fn al_control_ind(
        &mut self,
        control_code: handlers::command::DlControlCode,
    ) -> IoLinkResult<()>;
}

/// Error type for Application Layer responses
#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AlRspError {
    /// Error, operation failed
    Error(u8, u8), // (error code, additional error code)
    /// State machine is in unexpected state
    StateConflict,
    /// No data available
    NoData,
}
pub type AlResult<T> = Result<T, AlRspError>;

pub trait AlReadRsp {
    fn al_read_rsp(&mut self, result: AlResult<(u8, &[u8])>) -> IoLinkResult<()>;
}
pub trait AlWriteRsp {
    fn al_write_rsp(&mut self, result: AlResult<()>) -> IoLinkResult<()>;
}

/// Trait for handling AL_Event requests from the Application Layer.
pub trait AlEventReq {
    /// Handles event requests from the application layer services.
    ///
    /// This method is called when the services need to report events
    /// to the data link layer.
    ///
    /// # Parameters
    ///
    /// * `event_count` - Number of events to report
    /// * `event_entries` - Array of event entries to report
    ///
    /// # Returns
    ///
    /// - `Ok(())` if events were reported successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn al_event_req(
        &mut self,
        event_count: u8,
        event_entries: &[handlers::event::EventEntry],
    ) -> IoLinkResult<()>;
}

/// Trait for the AL_Control service in the Application Layer.
pub trait AlControlReq {
    /// The `AlControlReq` trait defines the interface for the AL_Control service,
    /// which transmits Process Data qualifier status information to and from the Device application.
    /// This service should be synchronized with AL_GetInput and AL_SetOutput respectively.
    ///
    /// # Parameters
    /// - `control_code`: Contains the qualifier status of the Process Data (PD).
    ///   Permitted values:
    ///   - `VALID` (Input Process Data valid)
    ///   - `INVALID` (Input Process Data invalid)
    ///   - `PDOUTVALID` (Output Process Data valid)
    ///   - `PDOUTINVALID` (Output Process Data invalid)
    ///
    /// # Returns
    /// Returns an `IoLinkResult<()>` indicating the success or failure of the operation.
    fn al_control_req(
        &mut self,
        control_code: handlers::command::DlControlCode,
    ) -> IoLinkResult<()>;
}

pub trait AlSetInputReq {
    fn al_set_input_req(
        &mut self,
        pd_data: &[u8],
        data_link_layer: &mut dl::DataLinkLayer,
    ) -> IoLinkResult<()>;
}

/// Trait for handling AL_Event confirmations from the Application Layer.
pub trait AlEventCnf {
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
    fn al_event_cnf(&mut self) -> IoLinkResult<()>;
}
