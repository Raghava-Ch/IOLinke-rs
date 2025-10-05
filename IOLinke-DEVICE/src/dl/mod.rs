//! Data Link Layer module for IO-Link Device Stack.
//!
//! This module provides the data link layer services that handle message
//! transmission, protocol state machines, and communication with the
//! physical and application layers according to IO-Link Specification v1.1.4.
//!
//! ## Components
//!
//! - **Command Handler**: Manages master commands and responses
//! - **Mode Handler**: Handles DL mode state transitions
//! - **Event Handler**: Manages device events and reporting
//! - **Message Handler**: Handles message transmission and reception
//! - **Process Data Handler**: Manages process data exchange
//! - **ISDU Handler**: Handles Index-based Service Data Unit communication
//! - **On-Request Data Handler**: Manages parameter read/write operations
//!
//! ## Specification Compliance
//!
//! - Section 7.2: Data Link Layer Services
//! - Section 7.3: Device Identification and Communication
//! - Section 7.4: Message Handling and Transmission
//! - Annex A: Protocol Details and Timing
use crate::{al, services};
use crate::{pl, system_management};
use iolinke_types::custom::IoLinkResult;
use iolinke_types::frame;
use iolinke_types::handlers;

mod command_handler;
mod event_handler;
mod isdu_handler;
pub mod message_handler;
mod mode_handler;
mod od_handler;
mod pd_handler;

pub use handlers::command::DlControlInd;
pub use handlers::event::{DlEventReq, DlEventTriggerConf};
pub use handlers::isdu::{DlIsduAbort, DlIsduTransportInd, DlIsduTransportRsp, IsduMessage};
pub use handlers::mode::DlModeInd;
pub use handlers::od::{DlParamRsp, DlReadParamInd, DlWriteParamInd};
pub use handlers::pd::{DlPDInputUpdate, DlPDOutputTransportInd, PD_OUTPUT_LENGTH};
use iolinke_types::handlers::command::DlControlReq;

use core::default::Default;
use core::result::Result::Ok;

/// Main Data Link Layer implementation that orchestrates all DL services.
///
/// The Data Link Layer manages the complete data link functionality
/// of the IO-Link device including message handling, protocol state
/// machines, and communication coordination between layers.
///
/// # Architecture
///
/// The data link layer follows a modular design where each component
/// handles a specific aspect of the data link functionality:
///
/// - **Command Handler**: Processes master commands and generates responses
/// - **Mode Handler**: Manages DL mode state transitions and timing
/// - **Event Handler**: Handles device events and reporting to master
/// - **Message Handler**: Manages message transmission and reception
/// - **Process Data Handler**: Handles real-time process data exchange
/// - **ISDU Handler**: Manages Index-based Service Data Unit communication
/// - **On-Request Data Handler**: Manages parameter read/write operations
///
/// # State Management
///
/// The data link layer maintains the protocol state machines and
/// coordinates with the physical layer and application layer to ensure
/// proper protocol operation according to the IO-Link specification.
///
/// # Polling Order
///
/// The polling order follows the IO-Link specification dependency chain:
/// 1. Command Handler → Message Handler
/// 2. Mode Handler → All handlers + System Management
/// 3. Event Handler → Message Handler
/// 4. Process Data Handler → Message Handler + Application Layer
/// 5. ISDU Handler → Message Handler + Application Layer
/// 6. Message Handler → Physical Layer + All handlers
/// 7. On-Request Data Handler → All handlers + Application Layer + System Management
pub struct DataLinkLayer {
    /// Handles master commands and generates responses
    command_handler: command_handler::CommandHandler,
    /// Manages DL mode state transitions and timing
    mode_handler: mode_handler::DlModeHandler,
    /// Handles device events and reporting to master
    event_handler: event_handler::EventHandler,
    /// Manages message transmission and reception
    message_handler: message_handler::MessageHandler,
    /// Handles real-time process data exchange
    pd_handler: pd_handler::ProcessDataHandler,
    /// Manages Index-based Service Data Unit communication
    isdu_handler: isdu_handler::IsduHandler,
    /// Manages parameter read/write operations
    od_handler: od_handler::OnRequestDataHandler,
}

impl DataLinkLayer {
    /// This function is called when the communication is successful.
    /// It will change the DL mode to the corresponding communication mode.
    /// # Parameters
    /// * `transmission_rate` - The transmission rate of the communication
    /// # Returns
    /// * `Ok(())` if the communication is successful
    /// * `Err(IoLinkError)` if an error occurred
    pub fn successful_com(&mut self, transmission_rate: frame::msequence::TransmissionRate) {
        let _ = self.mode_handler.successful_com(transmission_rate);
    }

    /// Polls all data link layer components to advance their state.
    ///
    /// This method must be called regularly to:
    /// - Process incoming messages from the master
    /// - Update protocol state machines
    /// - Handle timeouts and state transitions
    /// - Coordinate communication between layers
    ///
    /// The polling order follows the IO-Link specification dependency chain
    /// to ensure proper operation and avoid race conditions.
    ///
    /// # Parameters
    ///
    /// * `system_management` - Reference to system management for coordination
    /// * `physical_layer` - Reference to physical layer for communication
    /// * `application_layer` - Reference to application layer for services
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all components processed successfully
    /// - `Err(IoLinkError)` if an error occurred during processing
    ///
    /// # Errors
    ///
    /// - `IoLinkError::NoImplFound` - Feature not yet implemented
    /// - `IoLinkError::InvalidParameter` - Invalid parameter value
    /// - `IoLinkError::FuncNotAvailable` - Function not available in current state
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut dl = DataLinkLayer::default();
    /// let mut sm = SystemManagement::default();
    /// let mut pl = PhysicalLayer::default();
    /// let mut al = ApplicationLayer::default();
    ///
    /// // Poll data link layer
    /// match dl.poll(&mut sm, &mut pl, &mut al) {
    ///     Ok(()) => {
    ///         // Data link layer operating normally
    ///     }
    ///     Err(e) => {
    ///         // Handle errors according to IO-Link specification
    ///     }
    /// }
    /// ```
    pub fn poll<
        PHY: pl::physical_layer::PhysicalLayerReq,
        ALS: services::ApplicationLayerServicesInd
            + handlers::sm::SystemManagementCnf
            + services::AlEventCnf,
    >(
        &mut self,
        system_management: &mut system_management::SystemManagement,
        physical_layer: &mut PHY,
        application_layer: &mut al::ApplicationLayer<ALS>,
    ) -> IoLinkResult<()> {
        // Command handler poll - handles master commands
        {
            let _ = self.command_handler.poll(
                &mut self.message_handler,
                application_layer,
                &mut self.mode_handler,
            );
        }

        // Mode handler poll - manages protocol state machines
        {
            let _ = self.mode_handler.poll(
                &mut self.isdu_handler,
                &mut self.event_handler,
                &mut self.command_handler,
                &mut self.od_handler,
                &mut self.pd_handler,
                &mut self.message_handler,
                system_management,
            );
        }

        // Event handler poll - processes device events
        {
            let _ = self.event_handler.poll(&mut self.message_handler);
        }

        // Process data handler poll - handles real-time data exchange
        {
            let _ = self
                .pd_handler
                .poll(&mut self.message_handler, application_layer);
        }

        // ISDU handler poll - manages service data unit communication
        {
            let isdu_handler = &mut self.isdu_handler;
            let _ = isdu_handler.poll(&mut self.message_handler, application_layer);
        }

        // Message handler poll - coordinates all message operations
        {
            let _ = self.message_handler.poll(
                &mut self.od_handler,
                &mut self.pd_handler,
                &mut self.mode_handler,
                physical_layer,
            );
        }

        // On-request data handler poll - manages parameter operations
        {
            let _ = self.od_handler.poll(
                &mut self.command_handler,
                &mut self.isdu_handler,
                &mut self.event_handler,
                application_layer,
                system_management,
            );
        }

        Ok(())
    }
}

impl handlers::od::DlParamRsp for DataLinkLayer {
    /// Handles parameter read responses from the application layer.
    ///
    /// This method is called when the application layer provides
    /// parameter data in response to a read request from the master.
    ///
    /// # Parameters
    ///
    /// * `length` - Length of the parameter data in bytes
    /// * `data` - Parameter data to send to the master
    ///
    /// # Returns
    ///
    /// - `Ok(())` if response was processed successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn dl_read_param_rsp(&mut self, length: u8, data: u8) -> IoLinkResult<()> {
        self.od_handler
            .dl_read_param_rsp(length, data, &mut self.message_handler)
    }

    /// Handles parameter write responses from the application layer.
    ///
    /// This method is called when the application layer confirms
    /// a parameter write operation from the master.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if response was processed successfully
    /// - `Err(IoLinkError)` if an error occurred
    ///
    /// # Note
    ///
    /// According to the IO-Link specification, no response is expected
    /// for write operations, but this method provides a hook for
    /// potential future extensions.
    fn dl_write_param_rsp(&mut self) -> IoLinkResult<()> {
        self.od_handler
            .dl_write_param_rsp(&mut self.message_handler)
    }
}

impl handlers::isdu::DlIsduTransportRsp for DataLinkLayer {
    /// Handles ISDU read responses from the application layer.
    ///
    /// This method is called when the application layer provides
    /// ISDU data in response to a read request from the master.
    ///
    /// # Parameters
    ///
    /// * `length` - Length of the ISDU data in bytes
    /// * `data` - ISDU data to send to the master
    ///
    /// # Returns
    ///
    /// - `Ok(())` if response was processed successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn dl_isdu_transport_read_rsp(&mut self, length: u8, data: &[u8]) -> IoLinkResult<()> {
        self.isdu_handler.dl_isdu_transport_read_rsp(length, data)
    }

    /// Handles ISDU write responses from the application layer.
    ///
    /// This method is called when the application layer confirms
    /// an ISDU write operation from the master.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if response was processed successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn dl_isdu_transport_write_rsp(&mut self) -> IoLinkResult<()> {
        self.isdu_handler.dl_isdu_transport_write_rsp()
    }

    /// Handles ISDU read error responses from the application layer.
    ///
    /// This method is called when the application layer reports
    /// an error during an ISDU read operation.
    ///
    /// # Parameters
    ///
    /// * `error` - Primary error code (see Annex A.7)
    /// * `additional_error` - Additional error code for detailed error information
    ///
    /// # Returns
    ///
    /// - `Ok(())` if error response was processed successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn dl_isdu_transport_read_error_rsp(
        &mut self,
        error: u8,
        additional_error: u8,
    ) -> IoLinkResult<()> {
        self.isdu_handler
            .dl_isdu_transport_read_error_rsp(error, additional_error)
    }

    /// Handles ISDU write error responses from the application layer.
    ///
    /// This method is called when the application layer reports
    /// an error during an ISDU write operation.
    ///
    /// # Parameters
    ///
    /// * `error` - Primary error code (see Annex A.7)
    /// * `additional_error` - Additional error code for detailed error information
    ///
    /// # Returns
    ///
    /// - `Ok(())` if error response was processed successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn dl_isdu_transport_write_error_rsp(
        &mut self,
        error: u8,
        additional_error: u8,
    ) -> IoLinkResult<()> {
        self.isdu_handler
            .dl_isdu_transport_write_error_rsp(error, additional_error)
    }
}

impl handlers::event::DlEventReq for DataLinkLayer {
    /// Handles event requests from the application layer.
    ///
    /// This method is called when the application layer has events
    /// to report to the master.
    ///
    /// # Parameters
    ///
    /// * `event_count` - Number of events to report
    /// * `event_entries` - Array of event entries to report
    ///
    /// # Returns
    ///
    /// - `Ok(())` if events were processed successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn dl_event_req(
        &mut self,
        event_count: u8,
        event_entries: &[handlers::event::EventEntry],
    ) -> IoLinkResult<()> {
        self.event_handler.dl_event_req(event_count, event_entries)
    }

    /// Handles event trigger requests from the application layer.
    ///
    /// This method is called when the application layer requests
    /// to trigger an event transmission to the master.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if event trigger was processed successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn dl_event_trigger_req(&mut self) -> IoLinkResult<()> {
        self.event_handler.dl_event_trigger_req()
    }
}

impl handlers::pd::DlPDInputUpdate for DataLinkLayer {
    /// Handles process data input update requests from the application layer.
    ///
    /// This method is called when the application layer has new
    /// input data to send to the master during the next process data cycle.
    ///
    /// # Parameters
    ///
    /// * `length` - Length of the input data in bytes
    /// * `input_data` - Input data to send to the master
    ///
    /// # Returns
    ///
    /// - `Ok(())` if input update was processed successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn dl_pd_input_update_req(&mut self, length: u8, input_data: &[u8]) -> IoLinkResult<()> {
        self.pd_handler.dl_pd_input_update_req(length, input_data)
    }
}

impl DlModeInd for DataLinkLayer {
    fn dl_mode_ind(&mut self, mode: handlers::mode::DlMode) -> IoLinkResult<()> {
        let _ = self.message_handler.dl_mode_ind(mode);
        Ok(())
    }
}

impl DlControlReq for DataLinkLayer {
    fn dl_control_req(&mut self, control_code: handlers::command::DlControlCode) -> IoLinkResult<()> {
        self.command_handler.dl_control_req(control_code)
    }
}

impl Default for DataLinkLayer {
    /// Creates a new Data Link Layer with default configuration.
    ///
    /// All components are initialized to their default states
    /// and ready for operation.
    fn default() -> Self {
        Self {
            command_handler: command_handler::CommandHandler::new(),
            mode_handler: mode_handler::DlModeHandler::new(),
            event_handler: event_handler::EventHandler::new(),
            message_handler: message_handler::MessageHandler::new(),
            pd_handler: pd_handler::ProcessDataHandler::new(),
            isdu_handler: isdu_handler::IsduHandler::new(),
            od_handler: od_handler::OnRequestDataHandler::new(),
        }
    }
}

impl<PHY: pl::physical_layer::PhysicalLayerReq> pl::physical_layer::PhysicalLayerInd<PHY>
    for DataLinkLayer
{
    /// Handles wake-up notifications from the physical layer.
    ///
    /// This method is called when the physical layer detects
    /// a wake-up condition on the C/Q line.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if wake-up was processed successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn pl_wake_up_ind(&mut self, physical_layer: &PHY) -> IoLinkResult<()> {
        self.mode_handler.pl_wake_up_ind(physical_layer)
    }

    /// Handles data transfer notifications from the physical layer.
    ///
    /// This method is called when the physical layer receives
    /// data from the master.
    ///
    /// # Parameters
    ///
    /// * `rx_buffer` - Buffer containing received data
    ///
    /// # Returns
    ///
    /// - `Ok(())` if data transfer was processed successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn pl_transfer_ind(&mut self, physical_layer: &PHY, rx_byte: u8) -> IoLinkResult<()> {
        self.message_handler
            .pl_transfer_ind(physical_layer, rx_byte)
    }
}
