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

use crate::dl::DlPDOutputTransportInd;
use crate::{dl, storage, system_management, types, IoLinkResult};

mod event_handler;
pub mod od_handler;
mod parameter_manager;
mod pd_handler;
pub mod services;
mod data_storage;

use heapless::Vec;
pub use parameter_manager::DeviceParametersIndex;
pub use parameter_manager::SubIndex;
pub use parameter_manager::DirectParameterPage1SubIndex;
pub use parameter_manager::DataStorageIndexSubIndex;

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
pub struct ApplicationLayer {
    /// Handles device events and reporting to the master
    event_handler: event_handler::EventHandler,
    /// Handles on-request data operations (parameter read/write)
    od_handler: od_handler::OnRequestDataHandler,
    /// Provides application layer services
    services: services::ApplicationLayerServices,
    /// Manages device parameters and storage
    parameter_manager: parameter_manager::ParameterManager,
    /// Handles persistent parameter storage
    data_storage: data_storage::DataStorage,
}

impl ApplicationLayer {
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
        self.od_handler.poll(&mut self.services, data_link_layer)?;
        self.parameter_manager
            .poll(&mut self.od_handler, &mut self.data_storage)?;
        self.data_storage.poll(&mut self.event_handler, &mut self.parameter_manager)?;
        Ok(())
    }
}

impl services::AlEventReq for ApplicationLayer {
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
    fn al_event_req(&mut self, event_count: u8, event_entries: &[storage::event_memory::EventEntry]) -> IoLinkResult<()> {
        self.event_handler.al_event_req(event_count, event_entries)
    }
}

impl system_management::SystemManagementInd for ApplicationLayer {
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
    fn sm_device_mode_ind(&mut self, mode: types::DeviceMode) -> system_management::SmResult<()> {
        let _ = self.parameter_manager.sm_device_mode_ind(mode);
        let _ = self.data_storage.sm_device_mode_ind(mode);

        Ok(())
    }
}

impl system_management::SystemManagementCnf for ApplicationLayer {
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
        result: system_management::SmResult<()>,
    ) -> system_management::SmResult<()> {
        todo!("Implement device communication setup confirmation");
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
        result: system_management::SmResult<&system_management::DeviceCom>,
    ) -> system_management::SmResult<()> {
        todo!("Implement device communication get confirmation");
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
        result: system_management::SmResult<()>,
    ) -> system_management::SmResult<()> {
        todo!("Implement device identification setup confirmation");
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
        result: system_management::SmResult<&system_management::DeviceIdent>,
    ) -> system_management::SmResult<()> {
        todo!("Implement device identification get confirmation");
    }
    fn sm_set_device_mode_cnf(
        &self,
        result: system_management::SmResult<()>,
    ) -> system_management::SmResult<()> {
        todo!()
    }
}

impl dl::DlIsduAbort for ApplicationLayer {
    fn dl_isdu_abort(&mut self) -> IoLinkResult<()> {
        self.od_handler.dl_isdu_abort()
    }
}

impl dl::DlIsduTransportInd for ApplicationLayer {
    fn dl_isdu_transport_ind(&mut self, isdu: dl::Isdu) -> IoLinkResult<()> {
        self.od_handler.dl_isdu_transport_ind(isdu)
    }
}

impl dl::DlReadParamInd for ApplicationLayer {
    fn dl_read_param_ind(&mut self, address: u8) -> IoLinkResult<()> {
        self.od_handler.dl_read_param_ind(address)
    }
}

impl dl::DlWriteParamInd for ApplicationLayer {
    fn dl_write_param_ind(&mut self, index: u8, data: u8) -> IoLinkResult<()> {
        self.od_handler.dl_write_param_ind(index, data)
    }
}

impl dl::DlControlInd for ApplicationLayer {
    fn dl_control_ind(&mut self, control_code: types::DlControlCode) -> IoLinkResult<()> {
        self.services.dl_control_ind(control_code)
    }
}

impl Default for ApplicationLayer {
    fn default() -> Self {
        Self {
            event_handler: event_handler::EventHandler::new(),
            od_handler: od_handler::OnRequestDataHandler::new(),
            services: services::ApplicationLayerServices::new(),
            parameter_manager: parameter_manager::ParameterManager::new(),
            data_storage: data_storage::DataStorage::new(),
        }
    }
}

impl ApplicationLayerReadWriteInd for ApplicationLayer {
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

impl dl::DlPDOutputTransportInd for ApplicationLayer {
    fn dl_pd_output_transport_ind(&mut self, pd_out: &Vec<u8, {dl::PD_OUTPUT_LENGTH}>) -> IoLinkResult<()> {
        todo!()
    }

    fn dl_pd_cycle_ind(&mut self) -> IoLinkResult<()> {
        todo!()
    }
}
