//! Basic IO-Link Device Example
//!
//! This example demonstrates how to use the IO-Link device stack
//! with a mock HAL implementation.

use iolink_device_stack::{
    ApplicationLayerImpl,
    DlModeHandler, DlModeState,
    MessageHandler,
    MockHal, PhysicalLayer,
    types::*,
};

/// Simple IO-Link device implementation
pub struct SimpleIoLinkDevice {
    dl_mode: DlModeHandler,
    message_handler: MessageHandler,
    application: ApplicationLayerImpl,
}

impl SimpleIoLinkDevice {
    /// Create a new simple IO-Link device
    pub fn new() -> Self {
        Self {
            dl_mode: DlModeHandler::new(),
            message_handler: MessageHandler::new(),
            application: ApplicationLayerImpl::new(),
        }
    }

    /// Main device polling function
    pub fn poll<H>(&mut self, hal: &mut H) -> IoLinkResult<()>
    where
        H: PhysicalLayer,
    {
        // Poll all state machines
        self.dl_mode.poll(hal)?;
        self.message_handler.poll()?;
        self.application.poll()?;

        // Handle inter-module communication
        self.handle_communication()?;

        Ok(())
    }

    /// Handle communication between modules
    fn handle_communication(&mut self) -> IoLinkResult<()> {
        // Example: Forward received messages to application layer
        // In a real implementation, this would route messages based on type
        Ok(())
    }

    /// Set device identification
    pub fn set_device_id(&mut self, vendor_id: u16, device_id: u32, function_id: u16) {
        let device_identification = DeviceIdentification {
            vendor_id,
            device_id,
            function_id,
            reserved: 0,
        };
        self.application.set_device_id(device_identification);
    }

    /// Get current DL-Mode state
    pub fn get_dl_mode_state(&self) -> DlModeState {
        self.dl_mode.state()
    }

    /// Request mode change
    pub fn request_mode_change(&mut self, mode: IoLinkMode) -> IoLinkResult<()> {
        self.dl_mode.request_mode_change(mode)
    }
}

fn example_usage() -> IoLinkResult<()> {
    // Create device and HAL
    let mut device = SimpleIoLinkDevice::new();
    let mut hal = MockHal::new();

    // Configure device
    device.set_device_id(0x1234, 0x56789ABC, 0xDEF0);

    // Simulate some operation cycles
    for i in 0..10 {
        device.poll(&mut hal)?;
        
        // Request mode change on first iteration
        if i == 0 {
            device.request_mode_change(IoLinkMode::Com2)?;
        }
    }

    // Verify device state
    println!("DL-Mode state: {:?}", device.get_dl_mode_state());
    
    Ok(())
}

fn main() {
    example_usage().expect("Example should run without errors");
    println!("Basic device example completed successfully!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_device_operation() {
        example_usage().expect("Example should run without errors");
    }

    #[test]
    fn test_device_initialization() {
        let device = SimpleIoLinkDevice::new();
        assert_eq!(device.get_dl_mode_state(), DlModeState::Idle);
    }

    #[test]
    fn test_mode_change_request() {
        let mut device = SimpleIoLinkDevice::new();
        device.request_mode_change(IoLinkMode::Com2).unwrap();
    }
}
