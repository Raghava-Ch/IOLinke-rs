//! # IO-Link Physical Layer Bindings
//!
//! This module provides Rust bindings and abstractions for the IO-Link Physical Layer (PL) as specified in the IO-Link Interface Specification v1.1.4.
//! It defines the interface for configuring and interacting with the physical layer of IO-Link devices, including mode setting, data transfer, and timer management.
//!
//! ## Features
//!
//! - **Physical Layer Services:** Exposes FFI functions for PL_SetMode, PL_Transfer, PL_WakeUp, and timer operations, which must be implemented by the integrator for hardware-specific behavior.
//! - **Indication Handlers:** Provides Rust functions to handle transfer and wake-up indications from the physical layer, forwarding events to the data link layer.
//! - **Mock Implementation:** Includes a mock physical layer struct (`BindingPhysicalLayer`) for testing and integration purposes.
//! - **Device State Management:** Integrates with the IO-Link device state machine to ensure correct operation sequencing and error handling.
//!
//! ## Usage
//!
//! - Implement the required FFI functions in your hardware abstraction layer.
//! - Use the provided Rust handlers to process physical layer events and manage device communication.
//! - Leverage the mock implementation for unit testing and simulation.
//!
//! ## Specification References
//!
//! - IO-Link Interface Spec v1.1.4 Section 5.2.2.1: PL_SetMode Service
//! - IO-Link Interface Spec v1.1.4 Section 5.2.2.2: PL_WakeUp Service
//! - IO-Link Interface Spec v1.1.4 Section 5.2.2.3: PL_Transfer Service
//! - IO-Link Interface Spec v1.1.4 Section 5.3.3.3: Wake-up Sequence
//!
//! ## Safety
//!
//! Many functions in this module are marked `unsafe` due to their reliance on FFI and mutable static references. Care must be taken to ensure thread safety and correct device indexing when interacting with these APIs.
//!
//! ## Example
//!
//! ```rust,ignore
//! let mut device = IoLinkDevice::new();
//! device.pl_transfer_ind(0x55)?; // Handle received byte 0x55
//! device.pl_wake_up_ind()?;      // Handle wake-up indication
//! ```
use crate::c::app::{BindingApplicationLayer, IOLINKE_DEVICES};
use crate::c::types::{DeviceActionState, IOLinkeDeviceHandle};

use core::option::Option::Some;
pub use core::result::{
    Result,
    Result::{Err, Ok},
};

use iolinke_device::{IoLinkDevice, PhysicalLayerReq, Timer, TransmissionRate};
use iolinke_types::custom::IoLinkResult;
use iolinke_types::handlers::sm::IoLinkMode;

unsafe extern "C" {
    /// # `Integrator Implemented Function`
    /// Sets up the electrical characteristics and configuration of the Physical Layer for the specified device.
    ///
    /// This function must implement the PL_SetMode service as described in IO-Link Interface Spec v1.1.4 Section 5.2.2.1.
    /// It configures the C/Q line of the device to the requested operation mode by transmitting the service-specific
    /// parameters to the physical layer.
    ///
    /// # Parameters
    /// - `mode`: The requested operation mode for the C/Q line. Permitted values are:
    ///     - `Inactive`: C/Q line in high impedance
    ///     - `DI`: C/Q line in digital input mode
    ///     - `DO`: C/Q line in digital output mode
    ///     - `COM1`: C/Q line in COM1 mode
    ///     - `COM2`: C/Q line in COM2 mode
    ///     - `COM3`: C/Q line in COM3 mode
    ///
    /// # Returns
    /// - `Ok(())` if the mode was set successfully
    /// - `Err(IoLinkError)` if an error occurred during configuration
    ///
    /// # Specification Reference
    /// - IO-Link Interface Spec v1.1.4 Section 5.2.2.1: PL_SetMode Service
    fn pl_set_mode_req(device_id: IOLinkeDeviceHandle, mode: IoLinkMode) -> bool;

    /// # `Integrator Implemented Function`
    /// Initiates a Physical Layer data transfer request for the specified device.
    ///
    /// This function implements the PL_Transfer service as described in IO-Link Interface Spec v1.1.4 Section 5.2.2.3.
    /// It transfers the provided data buffer over the SDCI interface to the device's Physical Layer.
    ///
    /// # Parameters
    /// - `tx_data`: A slice containing the data bytes to be transferred (permitted values: 0..=255 for each byte).
    ///
    /// # Returns
    /// - `Ok(())` if the transfer request was initiated successfully.
    /// - `Err(IoLinkError)` if an error occurred during the transfer request.
    ///
    /// # Specification Reference
    /// - IO-Link Interface Spec v1.1.4 Section 5.2.2.3: PL_Transfer Service
    ///
    /// # Transfer Status
    /// The result of the transfer is indicated by the Physical Layer through status parameters:
    /// - `Result (+)`: Indicates successful execution of the transfer request.
    /// - `Result (-)`: Indicates failure of the transfer request.
    /// - `Status`: Provides supplementary information on the transfer status, with possible values:
    ///     - `PARITY_ERROR`: UART detected a parity error.
    ///     - `FRAMING_ERROR`: Invalid UART stop bit detected.
    ///     - `OVERRUN`: Octet collision within the UART.
    ///
    fn pl_transfer_req(device_id: IOLinkeDeviceHandle, len: u8, data: *const u8) -> bool;

    /// # `Integrator Implemented Function`
    /// Stops the specified timer.
    ///
    /// This function must implement the function to stop the given timer and resets its state.
    ///
    /// # Parameters
    ///
    /// * `timer` - The timer to stop
    ///
    /// # Returns
    ///
    /// - `Ok(())` if timer was stopped successfully
    /// - `Err(IoLinkError)` if an error occurred
    ///
    /// # Note
    ///
    /// This function needs to be implemented with actual timer hardware.
    fn pl_stop_timer_req(device_id: IOLinkeDeviceHandle, timer: Timer);

    /// # `Integrator Implemented Function`
    /// Starts the specified timer with the given duration.
    ///
    /// This function must implement the function to start the given timer with the specified duration
    /// in microseconds.
    ///
    /// # Parameters
    ///
    /// * `timer` - The timer to start
    /// * `duration_us` - Duration in microseconds
    ///
    /// # Returns
    ///
    /// - `Ok(())` if timer was started successfully
    /// - `Err(IoLinkError)` if an error occurred
    ///
    /// # Note
    ///
    /// This function needs to be implemented with actual timer hardware.
    fn pl_start_timer_req(device_id: IOLinkeDeviceHandle, timer: Timer, duration_us: u32);

    /// # `Integrator Implemented Function`
    /// Restarts the specified timer with the given duration.
    ///
    /// This function must implement the function to restart the given timer with the specified duration
    /// in microseconds, regardless of its current state.
    ///
    /// # Parameters
    ///
    /// * `timer` - The timer to restart
    /// * `duration_us` - Duration in microseconds
    ///
    /// # Returns
    ///
    /// - `Ok(())` if timer was restarted successfully
    /// - `Err(IoLinkError)` if an error occurred
    ///
    /// # Note
    ///
    /// This function needs to be implemented with actual timer hardware.
    fn pl_restart_timer_req(device_id: IOLinkeDeviceHandle, timer: Timer, duration_us: u32);
}

/// Handles Physical Layer transfer indication with received byte data.
///
/// This function is called when the Physical Layer receives a byte from the master.
/// It forwards the received byte to the Data Link Layer for processing.
///
/// # Parameters
///
/// * `rx_byte` - The received byte from Physical Layer (value 0-255)
///
/// # Returns
///
/// * `Ok(())` if the byte was processed successfully
/// * `Err(IoLinkError)` if an error occurred during processing
///
/// # Specification Reference
///
/// - IO-Link Interface Spec v1.1.4 Section 5.2.2.3: PL_Transfer Service
///
/// # Example
///
/// ```ignore
/// let mut device = IoLinkDevice::new();
/// device.pl_transfer_ind(0x55)?; // Handle received byte 0x55
/// ```
#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
pub extern "C" fn pl_transfer_ind(device_id: IOLinkeDeviceHandle, data: u8) -> DeviceActionState {
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
            let _ = device.pl_transfer_ind(data);
            DeviceActionState::Done
        }
        _ => DeviceActionState::Busy, // Previous operation still in progress
    }
}

/// Handles Physical Layer wake-up indication from the master.
///
/// This function is called when the Physical Layer detects a wake-up sequence from the master.
/// The wake-up service prepares the Physical Layer to send and receive communication requests.
/// This is an unconfirmed service with no parameters.
///
/// # Returns
///
/// * `Ok(())` if the wake-up indication was processed successfully
/// * `Err(IoLinkError)` if an error occurred during processing
///
/// # Specification Reference
///
/// - IO-Link Interface Spec v1.1.4 Section 5.2.2.2: PL_WakeUp Service
/// - IO-Link Interface Spec v1.1.4 Section 5.3.3.3: Wake-up sequence
///
/// # Note
///
/// The success of the wake-up can only be verified by the master through subsequent
/// communication attempts with the device.
///
/// # Example
///
/// ```ignore
/// let mut device = IoLinkDevice::new();
/// device.pl_wake_up_ind()?; // Handle wake-up indication
/// ```
#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
pub extern "C" fn pl_wake_up_ind(device_id: IOLinkeDeviceHandle) -> DeviceActionState {
    let (device, state) = unsafe {
        if let Some((device, state)) = IOLINKE_DEVICES
            .get_mut(device_id as usize)
            .and_then(|opt| opt.as_mut())
        {
            (device, state)
        } else {
            return DeviceActionState::NoDevice; // Invalid device ID
        }
    };
    match state {
        DeviceActionState::Done => {
            let _ = device.pl_wake_up_ind();
            DeviceActionState::Done
        }
        _ => DeviceActionState::Busy, // Previous operation still in progress
    }
}

/// This function is called when the communication is successful.
/// It will change the device mode to the corresponding communication mode.
/// # Parameters
/// * `transmission_rate` - The transmission rate of the communication
/// # Returns
/// * `Ok(())` if the communication is successful
/// * `Err(IoLinkError)` if an error occurred
#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
pub extern "C" fn pl_successful_com(
    device_id: IOLinkeDeviceHandle,
    transmission_rate: TransmissionRate,
) -> DeviceActionState {
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
            let _ = device.successful_com(transmission_rate);
            DeviceActionState::Done
        }
        _ => DeviceActionState::Busy, // Previous operation still in progress
    }
}

/// Mock physical layer implementation for testing
pub struct BindingPhysicalLayer {
    mode: IoLinkMode,
    device_id: IOLinkeDeviceHandle,
}

impl BindingPhysicalLayer {
    /// Create a new MockPhysicalLayer
    ///
    /// # Arguments
    ///
    /// * `mock_to_usr_tx` - A sender for messages to the user thread
    ///
    /// # Returns
    ///
    /// A new MockPhysicalLayer
    ///
    pub fn new(device_id: IOLinkeDeviceHandle) -> Self {
        Self {
            mode: IoLinkMode::Inactive,
            device_id: device_id,
        }
    }
}

/// Transfer the received data to the IO-Link device
pub fn transfer_ind(
    rx_buffer: &[u8],
    io_link_device_lock: &mut IoLinkDevice<BindingPhysicalLayer, BindingApplicationLayer>,
) -> IoLinkResult<()> {
    for rx_buffer_byte in rx_buffer {
        let _ = io_link_device_lock.pl_transfer_ind(*rx_buffer_byte);
    }
    Ok(())
}

impl PhysicalLayerReq for BindingPhysicalLayer {
    fn pl_set_mode_req(&mut self, mode: IoLinkMode) -> IoLinkResult<()> {
        self.mode = mode;
        let _ok = unsafe { pl_set_mode_req(self.device_id, mode) };
        Ok(())
    }

    fn pl_transfer_req(&mut self, tx_data: &[u8]) -> IoLinkResult<()> {
        unsafe {
            let _ok = pl_transfer_req(self.device_id, tx_data.len() as u8, tx_data.as_ptr());
        }
        Ok(())
    }

    fn pl_stop_timer_req(&self, timer: Timer) -> IoLinkResult<()> {
        unsafe { pl_stop_timer_req(self.device_id, timer) };
        Ok(())
    }

    fn pl_start_timer_req(&self, timer: Timer, duration_us: u32) -> IoLinkResult<()> {
        unsafe { pl_start_timer_req(self.device_id, timer, duration_us) };
        Ok(())
    }
    fn pl_restart_timer_req(&self, timer: Timer, duration_us: u32) -> IoLinkResult<()> {
        unsafe { pl_restart_timer_req(self.device_id, timer, duration_us) };
        Ok(())
    }
}
