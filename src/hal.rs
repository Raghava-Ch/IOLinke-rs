//! Hardware Abstraction Layer (HAL) traits
//!
//! This module defines the hardware abstraction traits that must be implemented
//! for different target platforms. Based on IO-Link Specification v1.1.4 Section 5.2.

use crate::types::{IoLinkError, IoLinkMode, IoLinkResult, PhysicalLayerStatus};
use embedded_hal::digital::{InputPin, OutputPin};

/// All the timers used in IO-Link
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timer {
    /// See Table 42 – Wake-up procedure and retry characteristics
    Tdsio
}

/// Physical Layer trait for low-level UART/PHY access
/// See IO-Link v1.1.4 Section 5.2 and Figure 13
pub trait PhysicalLayer {
    /// Set the communication mode
    /// See IO-Link v1.1.4 Section 5.2.2.1
    fn pl_set_mode(&mut self, mode: IoLinkMode) -> IoLinkResult<()>;

    /// Transfer data over the physical layer
    /// See IO-Link v1.1.4 Section 5.2.2.2
    fn pl_transfer(&mut self, tx_data: &[u8], rx_buffer: &mut [u8]) -> IoLinkResult<usize>;

    /// Wake up the physical layer
    /// See IO-Link v1.1.4 Section 5.2.2.3
    fn pl_wake_up(&mut self) -> IoLinkResult<()>;

    /// Get physical layer status
    /// See IO-Link v1.1.4 Section 5.2.3
    fn pl_status(&self) -> PhysicalLayerStatus;

    /// Check if data is available for reading
    fn data_available(&self) -> bool;

    /// Get the current baud rate
    fn get_baud_rate(&self) -> u32;
}

/// GPIO abstraction for IO-Link C/Q line control
pub trait IoLinkGpio {
    /// C/Q line input pin
    type CqInput: InputPin;
    /// C/Q line output pin  
    type CqOutput: OutputPin;

    /// Get the C/Q input pin
    fn cq_input(&mut self) -> &mut Self::CqInput;

    /// Get the C/Q output pin
    fn cq_output(&mut self) -> &mut Self::CqOutput;

    /// Set C/Q line high (typically 24V)
    fn set_cq_high(&mut self) -> IoLinkResult<()>;

    /// Set C/Q line low (typically 0V)
    fn set_cq_low(&mut self) -> IoLinkResult<()>;

    /// Read C/Q line state
    fn read_cq(&self) -> IoLinkResult<bool>;
}

/// Timer abstraction for protocol timing
pub trait IoLinkTimer {
    /// Start a timer with the given duration in microseconds
    fn start_timer(&mut self, duration_us: u32) -> IoLinkResult<()>;

    /// Check if timer has expired
    fn is_timer_expired(&self) -> bool;

    /// Stop the timer
    fn stop_timer(&mut self);

    /// Get current time in microseconds
    fn get_time_us(&self) -> u32;
}

/// UART abstraction for IO-Link communication
pub trait IoLinkUart {
    /// Configure UART for the specified baud rate
    fn configure(&mut self, baud_rate: u32) -> IoLinkResult<()>;

    /// Send data over UART
    fn send(&mut self, data: &[u8]) -> IoLinkResult<()>;

    /// Receive data from UART
    fn receive(&mut self, buffer: &mut [u8]) -> IoLinkResult<usize>;

    /// Check if transmission is complete
    fn is_tx_complete(&self) -> bool;

    /// Check if data is available for reading
    fn is_rx_ready(&self) -> bool;

    /// Flush the transmit buffer
    fn flush_tx(&mut self) -> IoLinkResult<()>;

    /// Clear the receive buffer
    fn clear_rx(&mut self);

    /// Enable/disable UART
    fn set_enabled(&mut self, enabled: bool) -> IoLinkResult<()>;
}

/// Complete HAL implementation combining all required traits
pub trait IoLinkHal: PhysicalLayer + IoLinkGpio + IoLinkTimer + IoLinkUart {
    /// Initialize the hardware
    fn init(&mut self) -> IoLinkResult<()>;

    /// Perform a hardware reset
    fn reset(&mut self) -> IoLinkResult<()>;

    /// Enter low power mode
    fn enter_low_power(&mut self) -> IoLinkResult<()>;

    /// Exit low power mode
    fn exit_low_power(&mut self) -> IoLinkResult<()>;
}
