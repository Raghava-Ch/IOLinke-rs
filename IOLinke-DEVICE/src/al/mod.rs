//! Application Layer module for IO-Link Device Stack.
//!
//! This module provides the application layer services and interfaces that
//! handle parameter access, process data, events, and ISDU communication
//! according to IO-Link Specification v1.1.4.
//!
//! ## Components
//!
//! - **Parameter Manager**: Handles device parameter storage and access
//! - **Event Handler**: Manages device events and reporting
//! - **Process Data Handler**: Handles real-time process data exchange
//! - **ISDU Handler**: Manages Index-based Service Data Unit communication
//! - **On-Request Data Handler**: Handles parameter read/write requests
//! - **Data Storage**: Manages persistent parameter storage
//!
//! ## Specification Compliance
//!
//! - Section 8.1: ISDU Communication and Parameters
//! - Section 8.2: Process Data Handling
//! - Section 8.3: Event Handling and Reporting
//! - Annex B: Parameter Definitions and Access

use crate::dl;

mod data_storage;
mod event_handler;
pub mod od_handler;
pub mod parameter_manager;
mod pd_handler;
pub mod services;

use heapless::Vec;
use iolinke_types::custom::IoLinkResult;
use iolinke_types::handlers;

use core::result::Result::Ok;
/// Application Layer Read/Write Interface for parameter access.
///
/// This trait defines the interface that the data link layer uses to
/// request parameter read/write operations from the application layer.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Section 8.1.3: Parameter Access
/// - Annex B.8: Index assignment of data objects
pub trait ApplicationLayerReadWriteInd {
    /// Handles a read parameter request from the master.
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

    /// Handles a write parameter request from the master.
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
    fn al_abort_ind(&mut self) -> IoLinkResult<()>;
}

/// Application Layer Process Data Interface for real-time data exchange.
///
/// This trait defines the interface that the data link layer uses to
/// handle process data operations with the application layer.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Section 8.2: Process Data
/// - Annex B.6: Process Data Descriptors
pub trait ApplicationLayerProcessDataInd {
    /// Handles input data updates from the application.
    ///
    /// This method is called when the application has new input data
    /// to send to the master during the next process data cycle.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if input data was processed successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn al_set_input_ind(&mut self) -> IoLinkResult<()>;

    /// Handles process data cycle notifications.
    ///
    /// This method is called at the beginning of each process data cycle
    /// to allow the application to prepare input/output data.
    fn al_pd_cycle_ind(&mut self);

    /// Handles output data requests from the master.
    ///
    /// This method is called when the master requests the current
    /// output data from the device.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if output data was provided successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn al_get_output_ind(&mut self) -> IoLinkResult<()>;

    /// Handles new output data from the master.
    ///
    /// This method is called when the master provides new output data
    /// to the device during a process data cycle.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if output data was processed successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn al_new_output_ind(&mut self) -> IoLinkResult<()>;

    /// Handles control commands from the master.
    ///
    /// This method is called when the master sends a control command
    /// to the device (e.g., reset, parameter download).
    ///
    /// # Parameters
    ///
    /// * `control_code` - 8-bit control command code (see Annex B.9)
    ///
    /// # Returns
    ///
    /// - `Ok(())` if control command was processed successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn al_control(&mut self, control_code: u8) -> IoLinkResult<()>;
}

/// Application Layer Event Interface for event handling.
///
/// This trait defines the interface that the data link layer uses to
/// request event operations from the application layer.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Section 8.3: Event Handling
/// - Annex A.6: Event Reporting
pub trait ApplicationLayerEventInd {
    /// Handles event requests from the data link layer.
    ///
    /// This method is called when the data link layer needs to
    /// process events from the application layer.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if events were processed successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn al_event(&mut self) -> IoLinkResult<()>;
}

/// Main Application Layer implementation that orchestrates all application services.
///
/// The Application Layer manages the complete application-side functionality
/// of the IO-Link device including parameter management, event handling,
/// process data processing, and ISDU communication.
///
/// # Architecture
///
/// The application layer follows a modular design where each component
/// handles a specific aspect of the application functionality:
///
/// - **Event Handler**: Manages device events and reporting
/// - **On-Request Data Handler**: Handles parameter read/write requests
/// - **Services**: Provides application layer services
/// - **Parameter Manager**: Manages device parameters and storage
/// - **Data Storage**: Handles persistent parameter storage
///
/// # State Management
///
/// The application layer maintains its own state and coordinates with
/// the data link layer and system management to ensure proper protocol
/// operation according to the IO-Link specification.
pub struct ApplicationLayer<
    ALS: services::ApplicationLayerServicesInd
        + handlers::sm::SystemManagementCnf
        + services::AlEventCnf,
> {
    /// Handles device events and reporting to the master
    event_handler: event_handler::EventHandler,
    /// Handles on-request data operations (parameter read/write)
    od_handler: od_handler::OnRequestDataHandler,
    /// Provides application layer services
    services: ALS,
    /// Manages device parameters and storage
    parameter_manager: parameter_manager::ParameterManager,
    /// Handles persistent parameter storage
    data_storage: data_storage::DataStorage,
    /// Handles process data communication
    pde: pd_handler::ProcessDataHandler,
}

impl<
    ALS: services::ApplicationLayerServicesInd
        + handlers::sm::SystemManagementCnf
        + services::AlEventCnf,
> ApplicationLayer<ALS>
{
    /// Polls all application layer components to advance their state.
    ///
    /// This method must be called regularly to:
    /// - Process pending events
    /// - Handle parameter requests
    /// - Update parameter manager state
    /// - Process data storage operations
    ///
    /// # Parameters
    ///
    /// * `data_link_layer` - Reference to the data link layer for communication
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all components processed successfully
    /// - `Err(IoLinkError)` if an error occurred
    ///
    /// # Errors
    ///
    /// - `IoLinkError::NoImplFound` - Feature not yet implemented
    /// - `IoLinkError::InvalidParameter` - Invalid parameter value
    /// - `IoLinkError::FuncNotAvailable` - Function not available in current state
    pub fn poll(&mut self, data_link_layer: &mut dl::DataLinkLayer) -> IoLinkResult<()> {
        // Poll all components in dependency order
        self.event_handler
            .poll(&mut self.services, data_link_layer)?;
        self.od_handler
            .poll(&mut self.parameter_manager, data_link_layer)?;
        self.parameter_manager
            .poll(&mut self.od_handler, &mut self.data_storage)?;
        self.data_storage
            .poll(&mut self.event_handler, &mut self.parameter_manager)?;
        Ok(())
    }
}

impl<
    ALS: services::ApplicationLayerServicesInd
        + handlers::sm::SystemManagementCnf
        + services::AlEventCnf,
> services::AlSetInputReq for ApplicationLayer<ALS>
{
    fn al_set_input_req(
        &mut self,
        pd_data: &[u8],
        data_link_layer: &mut dl::DataLinkLayer,
    ) -> IoLinkResult<()> {
        self.pde.al_set_input_req(pd_data, data_link_layer)
    }
}

impl<
    ALS: services::ApplicationLayerServicesInd
        + handlers::sm::SystemManagementCnf
        + services::AlEventCnf,
> services::AlEventReq for ApplicationLayer<ALS>
{
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
    ) -> IoLinkResult<()> {
        self.event_handler.al_event_req(event_count, event_entries)
    }
}

impl<
    ALS: services::ApplicationLayerServicesInd
        + handlers::sm::SystemManagementCnf
        + services::AlEventCnf,
> handlers::sm::SystemManagementInd for ApplicationLayer<ALS>
{
    /// Handles device mode change notifications from system management.
    ///
    /// This method is called when the system management changes the
    /// device operating mode.
    ///
    /// # Parameters
    ///
    /// * `mode` - New device operating mode
    ///
    /// # Returns
    ///
    /// - `Ok(())` if mode change was processed successfully
    /// - `Err(SmError)` if an error occurred
    fn sm_device_mode_ind(&mut self, mode: handlers::sm::DeviceMode) -> handlers::sm::SmResult<()> {
        let _ = self.parameter_manager.sm_device_mode_ind(mode);
        let _ = self.data_storage.sm_device_mode_ind(mode);
        Ok(())
    }
}

impl<
    ALS: services::ApplicationLayerServicesInd
        + handlers::sm::SystemManagementCnf
        + services::AlEventCnf,
> handlers::sm::SystemManagementCnf for ApplicationLayer<ALS>
{
    /// Handles device communication setup confirmations.
    ///
    /// This method is called when the system management confirms
    /// device communication setup operations.
    ///
    /// # Parameters
    ///
    /// * `result` - Result of the communication setup operation
    ///
    /// # Returns
    ///
    /// - `Ok(())` if confirmation was processed successfully
    /// - `Err(SmError)` if an error occurred
    fn sm_set_device_com_cnf(
        &self,
        result: handlers::sm::SmResult<()>,
    ) -> handlers::sm::SmResult<()> {
        self.services.sm_set_device_com_cnf(result)
    }

    /// Handles device communication get confirmations.
    ///
    /// This method is called when the system management confirms
    /// device communication get operations.
    ///
    /// # Parameters
    ///
    /// * `result` - Result containing device communication parameters
    ///
    /// # Returns
    ///
    /// - `Ok(())` if confirmation was processed successfully
    /// - `Err(SmError)` if an error occurred
    fn sm_get_device_com_cnf(
        &self,
        result: handlers::sm::SmResult<&handlers::sm::DeviceCom>,
    ) -> handlers::sm::SmResult<()> {
        self.services.sm_get_device_com_cnf(result)
    }

    /// Handles device identification setup confirmations.
    ///
    /// This method is called when the system management confirms
    /// device identification setup operations.
    ///
    /// # Parameters
    ///
    /// * `result` - Result of the identification setup operation
    ///
    /// # Returns
    ///
    /// - `Ok(())` if confirmation was processed successfully
    /// - `Err(SmError)` if an error occurred
    fn sm_set_device_ident_cnf(
        &self,
        result: handlers::sm::SmResult<()>,
    ) -> handlers::sm::SmResult<()> {
        self.services.sm_set_device_ident_cnf(result)
    }

    /// Handles device identification get confirmations.
    ///
    /// This method is called when the system management confirms
    /// device identification get operations.
    ///
    /// # Parameters
    ///
    /// * `result` - Result containing device identification parameters
    ///
    /// # Returns
    ///
    /// - `Ok(())` if confirmation was processed successfully
    /// - `Err(SmError)` if an error occurred
    fn sm_get_device_ident_cnf(
        &self,
        result: handlers::sm::SmResult<&iolinke_types::page::page1::DeviceIdent>,
    ) -> handlers::sm::SmResult<()> {
        self.services.sm_get_device_ident_cnf(result)
    }
    fn sm_set_device_mode_cnf(
        &self,
        result: handlers::sm::SmResult<()>,
    ) -> handlers::sm::SmResult<()> {
        self.services.sm_set_device_mode_cnf(result)
    }
}

impl<
    ALS: services::ApplicationLayerServicesInd
        + handlers::sm::SystemManagementCnf
        + services::AlEventCnf,
> dl::DlIsduAbort for ApplicationLayer<ALS>
{
    fn dl_isdu_abort(&mut self) -> IoLinkResult<()> {
        self.od_handler.dl_isdu_abort()
    }
}

impl<
    ALS: services::ApplicationLayerServicesInd
        + handlers::sm::SystemManagementCnf
        + services::AlEventCnf,
> dl::DlIsduTransportInd for ApplicationLayer<ALS>
{
    fn dl_isdu_transport_ind(&mut self, isdu: dl::IsduMessage) -> IoLinkResult<()> {
        self.od_handler.dl_isdu_transport_ind(isdu)
    }
}

impl<
    ALS: services::ApplicationLayerServicesInd
        + handlers::sm::SystemManagementCnf
        + services::AlEventCnf,
> dl::DlReadParamInd for ApplicationLayer<ALS>
{
    fn dl_read_param_ind(&mut self, address: u8) -> IoLinkResult<()> {
        self.od_handler.dl_read_param_ind(address)
    }
}

impl<
    ALS: services::ApplicationLayerServicesInd
        + handlers::sm::SystemManagementCnf
        + services::AlEventCnf,
> dl::DlWriteParamInd for ApplicationLayer<ALS>
{
    fn dl_write_param_ind(&mut self, index: u8, data: u8) -> IoLinkResult<()> {
        self.od_handler.dl_write_param_ind(index, data)
    }
}

impl<
    ALS: services::ApplicationLayerServicesInd
        + handlers::sm::SystemManagementCnf
        + services::AlEventCnf,
> dl::DlControlInd for ApplicationLayer<ALS>
{
    fn dl_control_ind(
        &mut self,
        control_code: handlers::command::DlControlCode,
    ) -> IoLinkResult<()> {
        self.services.al_control_ind(control_code)
    }
}

impl<
    ALS: services::ApplicationLayerServicesInd
        + handlers::sm::SystemManagementCnf
        + services::AlEventCnf,
> ApplicationLayer<ALS>
{
    pub fn new(al_services: ALS) -> Self {
        Self {
            event_handler: event_handler::EventHandler::new(),
            od_handler: od_handler::OnRequestDataHandler::new(),
            services: al_services,
            parameter_manager: parameter_manager::ParameterManager::new(),
            data_storage: data_storage::DataStorage::new(),
            pde: pd_handler::ProcessDataHandler::new(),
        }
    }
}

impl<
    ALS: services::ApplicationLayerServicesInd
        + handlers::sm::SystemManagementCnf
        + services::AlEventCnf,
> ApplicationLayerReadWriteInd for ApplicationLayer<ALS>
{
    fn al_read_ind(&mut self, index: u16, sub_index: u8) -> IoLinkResult<()> {
        self.parameter_manager.al_read_ind(index, sub_index)
    }

    fn al_write_ind(&mut self, index: u16, sub_index: u8, data: &[u8]) -> IoLinkResult<()> {
        self.parameter_manager.al_write_ind(index, sub_index, data)
    }

    fn al_abort_ind(&mut self) -> IoLinkResult<()> {
        self.parameter_manager.al_abort_ind()
    }
}

impl<
    ALS: services::ApplicationLayerServicesInd
        + handlers::sm::SystemManagementCnf
        + services::AlEventCnf,
> dl::DlPDOutputTransportInd for ApplicationLayer<ALS>
{
    fn dl_pd_output_transport_ind(
        &mut self,
        pd_out: &Vec<u8, { dl::PD_OUTPUT_LENGTH }>,
    ) -> IoLinkResult<()> {
        self.pde
            .dl_pd_output_transport_ind(pd_out, &mut self.services)
    }

    fn dl_pd_cycle_ind(&mut self) -> IoLinkResult<()> {
        self.pde.dl_pd_cycle_ind(&mut self.services)
    }
}
