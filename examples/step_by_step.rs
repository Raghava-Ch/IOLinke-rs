// //! Basic IO-Link Device Example - Step by Step

// use iolink_device_stack::{
//     ApplicationLayerImpl,
//     DlModeHandler, DlModeState,
//     MessageHandler,
//     types::*,
// };

// /// Simple IO-Link device implementation
// pub struct SimpleIoLinkDevice {
//     dl_mode: DlModeHandler,
//     message_handler: MessageHandler,
//     application: ApplicationLayerImpl,
// }

// impl SimpleIoLinkDevice {
//     /// Create a new simple IO-Link device
//     pub fn new() -> Self {
//         Self {
//             dl_mode: DlModeHandler::new(),
//             message_handler: MessageHandler::new(),
//             application: ApplicationLayerImpl::new(),
//         }
//     }

//     /// Step 1: Set device identification
//     pub fn set_device_id(&mut self, vendor_id: u16, device_id: u32, function_id: u16) {
//         let device_identification = DeviceIdentification {
//             vendor_id,
//             device_id,
//             function_id,
//             reserved: 0,
//         };
//         self.application.set_device_id(device_identification);
//     }

//     /// Step 2: Get current DL-Mode state
//     pub fn get_dl_mode_state(&self) -> DlModeState {
//         self.dl_mode.state()
//     }

//     /// Step 3: Basic polling function
//     pub fn poll(&mut self, hal: &mut MockHal) -> IoLinkResult<()> {
//         // Poll all state machines
//         self.dl_mode.poll(hal)?;
//         self.message_handler.poll()?;
//         self.application.poll()?;
//         Ok(())
//     }

//     /// Step 4: Request mode change
//     pub fn request_mode_change(&mut self, mode: IoLinkMode) -> IoLinkResult<()> {
//         self.dl_mode.request_mode_change(mode)
//     }
// }

// fn main() {
//     println!("=== IO-Link Device Stack - Step by Step Demo ===");
    
//     // Step 1: Create device and HAL
//     println!("Step 1: Creating device and HAL...");
//     let mut device = SimpleIoLinkDevice::new();
//     let mut hal = MockHal::new();
//     println!("✓ Device and HAL created successfully");

//     // Step 2: Configure device identification
//     println!("\nStep 2: Setting device identification...");
//     device.set_device_id(0x1234, 0x56789ABC, 0xDEF0);
//     println!("✓ Device ID set: Vendor=0x1234, Device=0x56789ABC, Function=0xDEF0");

//     // Step 3: Check initial state
//     println!("\nStep 3: Checking initial DL-Mode state...");
//     let initial_state = device.get_dl_mode_state();
//     println!("✓ Initial DL-Mode state: {:?}", initial_state);

//     // Step 4: Attempt mode change
//     println!("\nStep 4: Requesting mode change to Com1...");
//     match device.request_mode_change(IoLinkMode::Com1) {
//         Ok(()) => println!("✓ Mode change request successful"),
//         Err(e) => println!("⚠ Mode change request failed: {:?}", e),
//     }

//     // Step 5: Run a few polling cycles
//     println!("\nStep 5: Running polling cycles...");
//     for i in 0..5 {
//         match device.poll(&mut hal) {
//             Ok(()) => println!("✓ Polling cycle {} completed", i + 1),
//             Err(e) => {
//                 println!("⚠ Polling cycle {} failed: {:?}", i + 1, e);
//                 break;
//             }
//         }
//     }

//     // Step 6: Check final state
//     println!("\nStep 6: Checking final DL-Mode state...");
//     let final_state = device.get_dl_mode_state();
//     println!("✓ Final DL-Mode state: {:?}", final_state);

//     println!("\n=== Step by Step Demo Complete! ===");
// }

// use iolink_device_stack::{IoLinkError, IoLinkMode, IoLinkResult, PhysicalLayer, PhysicalLayerStatus};

// /// Mock HAL implementation for testing
// pub struct MockHal {
//     mode: IoLinkMode,
//     status: PhysicalLayerStatus,
//     cq_state: bool,
//     timer_expired: bool,
//     tx_buffer: heapless::Vec<u8, 256>,
//     rx_buffer: heapless::Vec<u8, 256>,
// }

// impl MockHal {
//     /// Create a new mock HAL instance
//     pub fn new() -> Self {
//         Self {
//             mode: IoLinkMode::Sio,
//             status: PhysicalLayerStatus::NoCommunication,
//             cq_state: false,
//             timer_expired: false,
//             tx_buffer: heapless::Vec::new(),
//             rx_buffer: heapless::Vec::new(),
//         }
//     }

//     /// Add data to the receive buffer for testing
//     pub fn add_rx_data(&mut self, data: &[u8]) -> IoLinkResult<()> {
//         for &byte in data {
//             self.rx_buffer.push(byte).map_err(|_| IoLinkError::BufferOverflow)?;
//         }
//         Ok(())
//     }

//     /// Get transmitted data for verification
//     pub fn get_tx_data(&self) -> &[u8] {
//         &self.tx_buffer
//     }
// }


// impl PhysicalLayer for MockHal {
//     fn pl_set_mode(&mut self, mode: IoLinkMode) -> IoLinkResult<()> {
//         self.mode = mode;
//         self.status = PhysicalLayerStatus::Communication;
//         Ok(())
//     }

//     fn pl_transfer(&mut self, tx_data: &[u8], rx_buffer: &mut [u8]) -> IoLinkResult<usize> {
//         // Store transmitted data
//         for &byte in tx_data {
//             self.tx_buffer.push(byte).map_err(|_| IoLinkError::BufferOverflow)?;
//         }

//         // Copy available receive data
//         let copy_len = core::cmp::min(rx_buffer.len(), self.rx_buffer.len());
//         rx_buffer[..copy_len].copy_from_slice(&self.rx_buffer[..copy_len]);
        
//         // Remove copied data from internal buffer
//         for _ in 0..copy_len {
//             self.rx_buffer.remove(0);
//         }

//         Ok(copy_len)
//     }

//     fn pl_wake_up(&mut self) -> IoLinkResult<()> {
//         self.status = PhysicalLayerStatus::Communication;
//         Ok(())
//     }

//     fn pl_status(&self) -> PhysicalLayerStatus {
//         self.status
//     }

//     fn data_available(&self) -> bool {
//         !self.rx_buffer.is_empty()
//     }

//     fn get_baud_rate(&self) -> u32 {
//         match self.mode {
//             IoLinkMode::Sio => 0,
//             IoLinkMode::Com1 => 4800,
//             IoLinkMode::Com2 => 38400,
//             IoLinkMode::Com3 => 230400,
//         }
//     }
// }