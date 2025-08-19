// //! Minimal working example

// use iolink_device_stack::{IoLinkMode, PhysicalLayer};
// fn main() {
//     let mut hal = MockHal::new();
//     let _ = hal.pl_set_mode(IoLinkMode::Com1);
//     println!("Example ran successfully!");
// }

// use iolink_device_stack::{IoLinkError, IoLinkResult, PhysicalLayerStatus};

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