// //! Basic IO-Link Device Example
// //!
// //! This example demonstrates the basic usage of the IO-Link Device Stack.
// //! It creates a device instance and runs a simple polling loop.
// //!
// //! ## Features Demonstrated
// //!
// //! - Device initialization and configuration
// //! - Basic polling loop operation
// //! - Error handling and recovery
// //!
// //! ## Usage
// //!
// //! ```bash
// //! cargo run --example basic
// //! ```
// //!
// //! ## Expected Behavior
// //!
// //! The device will start in the Idle state and continuously poll all
// //! protocol layers. Since this is a basic example without hardware
// //! implementation, most operations will return `NoImplFound` errors,
// //! which are handled gracefully by continuing the loop.

// use iolinke_device::{DeviceCom, PhysicalLayerInd, SystemManagementReq, *};
// use std::time::Instant;
// use std::sync::{Arc, Mutex};

// struct PhysicalLayer {
//     mode: IoLinkMode,
//     tx_data: Vec<u8>,
//     rx_data: Vec<u8>,
//     timers: Arc<Mutex<Vec<TimerState>>>,
// }

// struct TimerState {
//     timer_id: Timer,
//     start_time: Instant,
//     duration_us: u32,
//     active: bool,
// }

// impl TimerState {
//     fn new(timer_id: Timer, duration_us: u32) -> Self {
//         Self {
//             timer_id,
//             start_time: Instant::now(),
//             duration_us,
//             active: true,
//         }
//     }

//     fn is_expired(&self) -> bool {
//         if !self.active {
//             return false;
//         }
//         let elapsed = self.start_time.elapsed();
//         let elapsed_us = elapsed.as_micros() as u32;
//         elapsed_us >= self.duration_us
//     }

//     fn restart(&mut self, duration_us: u32) {
//         self.start_time = Instant::now();
//         self.duration_us = duration_us;
//         self.active = true;
//     }

//     fn stop(&mut self) {
//         self.active = false;
//     }
// }

// impl PhysicalLayer {
//     pub fn new() -> Self {
//         Self {
//             mode: IoLinkMode::Inactive,
//             tx_data: Vec::new(),
//             rx_data: Vec::new(),
//             timers: Arc::new(Mutex::new(Vec::new())),
//         }
//     }

//     pub fn transfer_ind(&mut self, rx_buffer: &[u8]) -> IoLinkResult<()> {
//         self.rx_data.extend_from_slice(rx_buffer);
//         Ok(())
//     }

//     pub fn timer_expired(&mut self, timer: Timer) {
//         println!("Timer expired: {:?}", timer);
//         // Here you can add any specific logic needed when a timer expires
//         // For example, triggering state transitions or error handling
//     }

//     /// Check if any timers have expired and call timer_expired for them
//     pub fn check_timers(&mut self) {
//         let expired_timers = {
//             let mut timers = self.timers.lock().unwrap();
//             let mut expired_timers = Vec::new();
//             let mut i = 0;
            
//             // Find expired timers
//             while i < timers.len() {
//                 if timers[i].is_expired() {
//                     expired_timers.push(timers[i].timer_id);
//                     timers.remove(i);
//                 } else {
//                     i += 1;
//                 }
//             }
//             expired_timers
//         };

//         // Call timer_expired for each expired timer (after releasing the lock)
//         for timer_id in expired_timers {
//             self.timer_expired(timer_id);
//         }
//     }
// }

// impl PhysicalLayerReq for PhysicalLayer {
//     fn pl_set_mode_req(&mut self, mode: IoLinkMode) -> IoLinkResult<()> {
//         self.mode = mode;
//         Ok(())
//     }

//     fn pl_transfer_req(&mut self, tx_data: &[u8]) -> IoLinkResult<usize> {
//         self.tx_data.extend_from_slice(tx_data);
//         Ok(tx_data.len())
//     }

//     fn stop_timer(&mut self, timer: Timer) -> IoLinkResult<()> {
//         let mut timers = self.timers.lock().unwrap();
//         for timer_state in timers.iter_mut() {
//             if timer_state.timer_id == timer {
//                 timer_state.stop();
//                 break;
//             }
//         }
//         Ok(())
//     }

//     fn start_timer(&mut self, timer: Timer, duration_us: u32) -> IoLinkResult<()> {
//         let mut timers = self.timers.lock().unwrap();
//         let timer_state = TimerState::new(timer, duration_us);
//         timers.push(timer_state);
//         Ok(())
//     }

//     fn restart_timer(&mut self, timer: Timer, duration_us: u32) -> IoLinkResult<()> {
//         let mut timers = self.timers.lock().unwrap();
//         for timer_state in timers.iter_mut() {
//             if timer_state.timer_id == timer {
//                 timer_state.restart(duration_us);
//                 return Ok(());
//             }
//         }
//         // If timer doesn't exist, create it
//         let timer_state = TimerState::new(timer, duration_us);
//         timers.push(timer_state);
//         Ok(())
//     }

//     fn read_direct_param_page(
//         &mut self,
//         address: u8,
//         length: u8,
//         buffer: &mut [u8],
//     ) -> PageResult<usize> {
//         let _ = address;
//         let _ = length;
//         let _ = buffer;

//         todo!("Implement read page logic");
//     }

//     fn write_direct_param_page(&mut self, address: u8, length: u8, _data: &[u8]) -> PageResult<()> {
//         let _ = address;
//         let _ = length;

//         todo!("Implement write page logic");
//     }
// }

// fn poll(io_link_device: &mut IoLinkDevice, physical_layer: &mut PhysicalLayer) {
//     // Main device loop
//     for _i in 0..=99 {
//         // Check for expired timers before polling
//         physical_layer.check_timers();
        
//         // Poll all protocol layers
//         match io_link_device.poll(physical_layer) {
//             Ok(()) => {
//                 // Device operating normally
//                 // In a real implementation, you might add some delay here
//                 // sleep(Duration::from_millis(10));
//             }
//             Err(IoLinkError::NoImplFound) => {
//                 // Feature not implemented yet, continue operation
//                 // This is expected in the basic example
//                 continue;
//             }
//             Err(e) => {
//                 // Handle other errors
//                 eprintln!("Device error: {:?}", e);
//             }
//         }
        
//         // Add a small delay to make timer demonstration visible
//         std::thread::sleep(std::time::Duration::from_millis(1));
//     }
// }

// fn main() {
//     // Create a new IO-Link device instance
//     let mut io_link_device = IoLinkDevice::new();
//     let mut physical_layer = PhysicalLayer::new();
//     println!("IO-Link Device started. Press Ctrl+C to stop.");

//     // Set device identification parameters
//     // These are required by the IO-Link specification
//     let _ = io_link_device.sm_set_device_com_req(&DeviceCom {
//         suppported_sio_mode: SioMode::default(),
//         transmission_rate: TransmissionRate::Com3,
//         min_cycle_time: MinCycleTime::new().with_time_base(3).with_multiplier(60),
//         msequence_capability: MsequenceCapability::new()
//             .with_isdu(1)
//             .with_op_m_seq_code(4)
//             .with_pre_op_m_seq_code(1),
//         revision_id: RevisionId::new().with_major_rev(1).with_minor_rev(1),
//         process_data_in: ProcessDataIn::new().with_length(9).with_sio(1).with_byte(1),
//         process_data_out: ProcessDataOut::new().with_length(9).with_byte(1),
//     });

//     let _ = io_link_device.sm_set_device_ident_req(&DeviceIdent {
//         vendor_id: [0x12, 0x34],
//         device_id: [0x56, 0x78, 0x9A],
//         function_id: [0xBC, 0xDE],
//     });

//     let _ = io_link_device.sm_set_device_mode_req(DeviceMode::Sio);

//     let _ = io_link_device.pl_wake_up_ind();

//     // Poll the device to trigger timer checks
//     poll(&mut io_link_device, &mut physical_layer);

//     let mc = test_utils::MsequenceControl::new()
//         .with_read_write(test_utils::RwDirection::Read)
//         .with_comm_channel(ComChannel::Page)
//         .with_address_fctrl(0);
//     physical_layer.rx_data = mc.into_bytes().into();
//     let _ = io_link_device
//         .pl_transfer_ind(&physical_layer.rx_data);

//     poll(&mut io_link_device, &mut physical_layer);

//     println!("Physical layer: {:?}", physical_layer.rx_data);

// }

use bitfields::bitfield;

const CONST_VAR: u8 = 0x2;

const fn provide_val() -> u8 {
    0x1
}

#[bitfield(u32)]
struct Bitfield {
    #[bits(default = 0xFF)]
    a: u8,
    #[bits(default = -127)]
    b: i8,
    /// Sign-extended by the most significant bit of 4 bits. Also treated as 2's
    /// complement, meaning this field with 4 bits has the value range of
    /// `-8` to `7`. You can add more bits to increase this range!
    #[bits(4, default = 9)]
    c_sign_extended: i8,
    #[bits(2, default = CONST_VAR)] // No compile time checks for const variables.
    const_var_default: u8,
    #[bits(2, default = provide_val())] // No compile time checks for const functions.
    const_fn_default: u8, // No compile time checks for const functions.
    #[bits(8, default = CustomType::C)]
    custom_type: CustomType
}

#[derive(Debug, PartialEq)]
enum CustomType {
    A = 0,
    B = 1,
    C = 2,
}

impl CustomType {
    const fn from_bits(bits: u8) -> Self {
        match bits {
            0 => Self::A,
            1 => Self::B,
            2 => Self::C,
            _ => unreachable!(),
        }
    }

    const fn into_bits(self) -> u8 {
        self as u8
    }
}

fn main() {
    let bitfield = Bitfield::new();
    assert_eq!(bitfield.a(), 0xFF);
    assert_eq!(bitfield.b(), -127);
    assert_eq!(bitfield.c_sign_extended(), -7);
    assert_eq!(bitfield.const_var_default(), 0x2);
    assert_eq!(bitfield.const_fn_default(), 0x1);
    assert_eq!(bitfield.custom_type(), CustomType::C);
}