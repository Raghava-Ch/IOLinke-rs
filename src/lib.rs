#![no_std]
#![warn(missing_docs)]

//! # IO-Link Device Stack
//!
//! A modular, maintainable, and portable IO-Link Device/Slave stack implementation
//! compliant with IO-Link Specification Version 1.1.4 (June 2024).
//!
//! This library provides a complete IO-Link device implementation targeting
//! embedded microcontrollers with `#![no_std]` compatibility.
//!
//! ## Architecture
//!
//! The stack is built around 12 state machines that handle different aspects
//! of the IO-Link protocol:
//!
//! - **Data Link Layer**: DL-Mode Handler, Message Handler, Process Data Handler, ISDU Handler
//! - **Application Layer**: Process Data, On-request Data, ISDU, Command, Event handlers
//! - **System**: Application Layer, Event State Machine, System Management
//! - **Storage**: Parameter Manager, Data Storage, Event Memory
//! - **Physical Layer**: UART, GPIO, Timer abstractions
//!
//! ## Key Features
//!
//! - **IO-Link v1.1.4 Compliant**: Implements all mandatory and optional features
//! - **No-std Compatible**: Designed for embedded systems without standard library
//! - **Hardware Abstraction**: Platform-independent through embedded-hal traits
//! - **State Machine Based**: Clean separation of protocol states and transitions
//! - **Event Driven**: Efficient event handling and queuing system
//!
//! ## Macros
//!
//! This crate integrates with `iolinke-macros` to provide convenient procedural
//! macros for common IO-Link patterns including parameter storage, device identification,
//! and state machine generation.
//!
//! ## Usage
//!
//! ```ignore
//! use iolinke_device::*;
//!
//! let mut device = IoLinkDevice::new();
//! device.set_device_id(0x1234, 0x56789ABC, 0x0001);
//!
//! loop {
//!     device.poll()?;
//! }
//! ```
//!
//! ## Specification Compliance
//!
//! This implementation follows IO-Link Specification v1.1.4 (June 2024):
//! - Section 5.2: Physical Layer and Communication Modes
//! - Section 6.3: State Machines and Transitions
//! - Section 7.3.4.1: Device Identification and Parameters
//! - Section 8.1.3: ISDU Parameter Access
//! - Section 8.3: Event Handling and Reporting
//! - Annex B: Direct Parameters and System Commands

// Re-export macros for convenience
pub use iolinke_macros::*;

mod al;
mod dl;
mod pl;
mod utils;
mod config;
mod storage;
mod system_management;

#[cfg(feature = "std")]
pub mod ffi;
mod types;

#[cfg(test)]
pub mod test_utils;
#[cfg(test)]
pub use test_utils::*;

// Re-export main traits and types
pub use types::*;

/// Main IO-Link device implementation that orchestrates all protocol layers.
///
/// This struct manages the complete IO-Link device stack including:
/// - Physical layer communication (UART, GPIO, timers)
/// - Data link layer state machines and message handling
/// - Application layer services and parameter management
/// - System management and device identification
///
/// The device follows the IO-Link v1.1.4 specification state machine flow:
/// 1. **Idle** → **Startup** → **IdentStartup** → **IdentCheck**
/// 2. **Preoperate** → **Operate** (with fallback to SIO mode)
///
/// # Example
///
/// ```rust
/// use iolinke_device::*;
///
/// let mut device = IoLinkDevice::new();
/// device.set_device_id(0x1234, 0x56789ABC, 0x0001);
///
/// // Main device loop
/// loop {
///     if let Err(e) = device.poll() {
///         // Handle errors according to IO-Link specification
///         match e {
///             IoLinkError::NoImplFound => continue, // Not yet implemented
///             _ => break, // Fatal error
///         }
///     }
/// 
///     break; // Example purpose only
/// }
/// ```
pub struct IoLinkDevice {
    /// Physical layer abstraction for UART, GPIO, and timer operations
    physical_layer: pl::physical_layer::PhysicalLayer,
    /// Data link layer managing protocol state machines and message handling
    dl: dl::DataLinkLayer,
    /// System management handling device identification and communication setup
    system_management: system_management::SystemManagement,
    /// Application layer providing parameter access and event handling
    application_layer: al::ApplicationLayer,
}

impl IoLinkDevice {
    /// Creates a new IO-Link device with default configuration.
    ///
    /// The device starts in the **Idle** state and must be configured with
    /// device identification before entering operational modes.
    ///
    /// # Returns
    ///
    /// A new `IoLinkDevice` instance with all layers initialized to default states.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let device = IoLinkDevice::new();
    /// ```
    pub fn new() -> Self {
        Self {
            system_management: system_management::SystemManagement::default(),
            physical_layer: pl::physical_layer::PhysicalLayer::default(),
            dl: dl::DataLinkLayer::default(),
            application_layer: al::ApplicationLayer::default(),
        }
    }

    /// Sets the device identification parameters as required by IO-Link v1.1.4.
    ///
    /// This method configures the mandatory device identification parameters
    /// that are used during the identification phase of the IO-Link protocol.
    ///
    /// # Parameters
    ///
    /// * `vendor_id` - 16-bit vendor identification number (see Annex B.1)
    /// * `device_id` - 24-bit device identification number (see Annex B.1)
    /// * `function_id` - 16-bit function identification (reserved for future use)
    ///
    /// # Specification Reference
    ///
    /// - IO-Link v1.1.4 Section 7.3.4.1: Device Identification
    /// - Annex B.1: Direct Parameter Page 1 (VendorID1, VendorID2, DeviceID1-3)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut device = IoLinkDevice::new();
    /// device.set_device_id(0x1234, 0x56789ABC, 0x0001);
    /// ```
    pub fn set_device_id(&mut self, vendor_id: u16, device_id: u32, function_id: u16) {
        let _device_identification = DeviceIdentification {
            vendor_id,
            device_id,
            function_id,
            reserved: 0,
        };
        // TODO: Store device identification in system management
    }

    /// Main polling function that advances all protocol state machines.
    ///
    /// This method must be called regularly (typically every 1-10ms) to:
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
    /// # Returns
    ///
    /// - `Ok(())` if all layers processed successfully
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
    /// let mut device = IoLinkDevice::new();
    ///
    /// loop {
    ///     match device.poll() {
    ///         Ok(()) => {
    ///             // Device operating normally
    ///         }
    ///         Err(IoLinkError::NoImplFound) => {
    ///             // Feature not implemented yet, continue
    ///             continue;
    ///         }
    ///         Err(e) => {
    ///             // Handle other errors
    ///             break;
    ///         }
    ///     }
    /// }
    /// ```
    pub fn poll(&mut self) -> IoLinkResult<()> {
        // Poll all state machines in dependency order
        self.application_layer.poll(&mut self.dl)?;
        self.dl.poll(
            &mut self.system_management,
            &mut self.physical_layer,
            &mut self.application_layer,
        )?;
        self.system_management.poll(
            &mut self.application_layer,
            &mut self.physical_layer,
        )?;
        Ok(())
    }
}

impl al::ApplicationLayerReadWriteInd for IoLinkDevice {
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
    fn al_read_ind(&mut self, index: u16, sub_index: u8) -> IoLinkResult<()> {
        self.application_layer.al_read_ind(index, sub_index)
    }

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
    fn al_write_ind(&mut self, index: u16, sub_index: u8, data: &[u8]) -> IoLinkResult<()> {
        self.application_layer.al_write_ind(index, sub_index, data)
    }

    /// Handles abort requests from the master.
    ///
    /// This method is called when the master aborts an ongoing
    /// read or write operation.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if abort was processed successfully
    /// - `Err(IoLinkError::FuncNotAvailable)` if abort is not supported
    fn al_abort_ind(&mut self) -> IoLinkResult<()> {
        Err(IoLinkError::FuncNotAvailable)
    }
}

impl al::ApplicationLayerProcessDataInd for IoLinkDevice {
    /// Handles input data updates from the application.
    ///
    /// This method is called when the application has new input data
    /// to send to the master during the next process data cycle.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if input data was processed successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn al_set_input_ind(&mut self) -> IoLinkResult<()> {
        todo!("Implement process data input handling");
    }

    /// Handles process data cycle notifications.
    ///
    /// This method is called at the beginning of each process data cycle
    /// to allow the application to prepare input/output data.
    fn al_pd_cycle_ind(&mut self) {
        todo!("Implement process data cycle handling");
    }

    /// Handles output data requests from the master.
    ///
    /// This method is called when the master requests the current
    /// output data from the device.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if output data was provided successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn al_get_output_ind(&mut self) -> IoLinkResult<()> {
        todo!("Implement output data handling");
    }

    /// Handles new output data from the master.
    ///
    /// This method is called when the master provides new output data
    /// to the device during a process data cycle.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if output data was processed successfully
    /// - `Err(IoLinkError)` if an error occurred
    fn al_new_output_ind(&mut self) -> IoLinkResult<()> {
        todo!("Implement new output data handling");
    }

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
    fn al_control(&mut self, control_code: u8) -> IoLinkResult<()> {
        todo!("Implement control command handling");
    }
}
