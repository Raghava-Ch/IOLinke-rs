//! Test utilities for IO-Link device stack testing
//!
//! This module provides utilities and mock implementations for testing
//! the IO-Link device stack components.

use crate::{IoLinkError, IoLinkMode, IoLinkResult, PhysicalLayer, PhysicalLayerStatus};

/// Mock HAL implementation for testing
pub struct MockHal {
    mode: IoLinkMode,
    status: PhysicalLayerStatus,
    cq_state: bool,
    timer_expired: bool,
    tx_buffer: heapless::Vec<u8, 256>,
    rx_buffer: heapless::Vec<u8, 256>,
}

impl MockHal {
    /// Create a new mock HAL instance
    pub fn new() -> Self {
        Self {
            mode: IoLinkMode::Sio,
            status: PhysicalLayerStatus::NoCommunication,
            cq_state: false,
            timer_expired: false,
            tx_buffer: heapless::Vec::new(),
            rx_buffer: heapless::Vec::new(),
        }
    }

    /// Add data to the receive buffer for testing
    pub fn add_rx_data(&mut self, data: &[u8]) -> IoLinkResult<()> {
        for &byte in data {
            self.rx_buffer.push(byte).map_err(|_| IoLinkError::BufferOverflow)?;
        }
        Ok(())
    }

    /// Get transmitted data for verification
    pub fn get_tx_data(&self) -> &[u8] {
        &self.tx_buffer
    }
}


impl PhysicalLayer for MockHal {
    fn pl_set_mode(&mut self, mode: IoLinkMode) -> IoLinkResult<()> {
        self.mode = mode;
        self.status = PhysicalLayerStatus::Communication;
        Ok(())
    }

    fn pl_transfer(&mut self, tx_data: &[u8], rx_buffer: &mut [u8]) -> IoLinkResult<usize> {
        // Store transmitted data
        for &byte in tx_data {
            self.tx_buffer.push(byte).map_err(|_| IoLinkError::BufferOverflow)?;
        }

        // Copy available receive data
        let copy_len = core::cmp::min(rx_buffer.len(), self.rx_buffer.len());
        rx_buffer[..copy_len].copy_from_slice(&self.rx_buffer[..copy_len]);
        
        // Remove copied data from internal buffer
        for _ in 0..copy_len {
            self.rx_buffer.remove(0);
        }

        Ok(copy_len)
    }

    fn pl_wake_up(&mut self) -> IoLinkResult<()> {
        self.status = PhysicalLayerStatus::Communication;
        Ok(())
    }

    fn pl_status(&self) -> PhysicalLayerStatus {
        self.status
    }

    fn data_available(&self) -> bool {
        !self.rx_buffer.is_empty()
    }

    fn get_baud_rate(&self) -> u32 {
        match self.mode {
            IoLinkMode::Sio => 0,
            IoLinkMode::Com1 => 4800,
            IoLinkMode::Com2 => 38400,
            IoLinkMode::Com3 => 230400,
        }
    }
}

#[cfg(test)]
pub mod mock {
    use crate::hal::*;
    use crate::types::*;
    use super::MockHal;

    /// Extended mock HAL for comprehensive testing
    pub struct ExtendedMockHal {
        base: MockHal,
        gpio_state: bool,
        timer_duration: u32,
        timer_start_time: u32,
        current_time: u32,
    }

    impl ExtendedMockHal {
        /// Create a new extended mock HAL
        pub fn new() -> Self {
            Self {
                base: MockHal::new(),
                gpio_state: false,
                timer_duration: 0,
                timer_start_time: 0,
                current_time: 0,
            }
        }

        /// Advance the mock time
        pub fn advance_time(&mut self, microseconds: u32) {
            self.current_time += microseconds;
        }

        /// Set GPIO state for testing
        pub fn set_gpio_state(&mut self, state: bool) {
            self.gpio_state = state;
        }
    }

    impl PhysicalLayer for ExtendedMockHal {
        fn pl_set_mode(&mut self, mode: IoLinkMode) -> IoLinkResult<()> {
            self.base.pl_set_mode(mode)
        }

        fn pl_transfer(&mut self, tx_data: &[u8], rx_buffer: &mut [u8]) -> IoLinkResult<usize> {
            self.base.pl_transfer(tx_data, rx_buffer)
        }

        fn pl_wake_up(&mut self) -> IoLinkResult<()> {
            self.base.pl_wake_up()
        }

        fn pl_status(&self) -> PhysicalLayerStatus {
            self.base.pl_status()
        }

        fn data_available(&self) -> bool {
            self.base.data_available()
        }

        fn get_baud_rate(&self) -> u32 {
            self.base.get_baud_rate()
        }
    }

    impl IoLinkTimer for ExtendedMockHal {
        fn start_timer(&mut self, duration_us: u32) -> IoLinkResult<()> {
            self.timer_duration = duration_us;
            self.timer_start_time = self.current_time;
            Ok(())
        }

        fn is_timer_expired(&self) -> bool {
            (self.current_time - self.timer_start_time) >= self.timer_duration
        }

        fn stop_timer(&mut self) {
            self.timer_duration = 0;
        }

        fn get_time_us(&self) -> u32 {
            self.current_time
        }
    }

    impl IoLinkUart for ExtendedMockHal {
        fn configure(&mut self, _baud_rate: u32) -> IoLinkResult<()> {
            Ok(())
        }

        fn send(&mut self, data: &[u8]) -> IoLinkResult<()> {
            // Store in base mock
            self.base.add_rx_data(data)?;
            Ok(())
        }

        fn receive(&mut self, buffer: &mut [u8]) -> IoLinkResult<usize> {
            self.base.pl_transfer(&[], buffer)
        }

        fn is_tx_complete(&self) -> bool {
            true
        }

        fn is_rx_ready(&self) -> bool {
            self.base.data_available()
        }

        fn flush_tx(&mut self) -> IoLinkResult<()> {
            Ok(())
        }

        fn clear_rx(&mut self) {
            // Clear would be implemented here
        }

        fn set_enabled(&mut self, _enabled: bool) -> IoLinkResult<()> {
            Ok(())
        }
    }
}

/// Test helper functions
#[cfg(test)]
pub mod helpers {
    use crate::types::*;

    /// Create test process data
    pub fn create_test_process_data() -> ProcessData {
        let mut data = ProcessData::default();
        data.input.push(0xAA).unwrap();
        data.input.push(0xBB).unwrap();
        data.output.push(0xCC).unwrap();
        data.output.push(0xDD).unwrap();
        data.valid = true;
        data
    }

    /// Create test event
    pub fn create_test_event() -> Event {
        Event {
            event_type: EventType::DeviceAppears,
            qualifier: 0x01,
            mode: 0x02,
            data: heapless::Vec::new(),
        }
    }

    /// Create test device identification
    pub fn create_test_device_id() -> DeviceIdentification {
        DeviceIdentification {
            vendor_id: 0x1234,
            device_id: 0x56789ABC,
            function_id: 0xDEF0,
            reserved: 0x00,
        }
    }
}
