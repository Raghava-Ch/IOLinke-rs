// //! C Foreign Function Interface (FFI) bindings
// //!
// //! This module provides C-compatible functions for integration with C/C++ applications.
// //! All functions use `#[no_mangle] extern "C"` for C compatibility.

// use crate::application::{ApplicationLayer, ApplicationLayerImpl};
// use crate::types::{IoLinkError, IoLinkMode, ProcessData, DeviceIdentification};
// use core::ptr;
// use cstr_core::{CStr, CString};

// extern crate alloc;
// use alloc::boxed::Box;

// /// Opaque handle for the IO-Link device stack
// pub struct IoLinkDeviceHandle {
//     application_layer: ApplicationLayerImpl,
// }

// /// C-compatible error codes
// #[repr(C)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum CIoLinkError {
//     /// Success
//     Ok = 0,
//     /// Invalid parameter
//     InvalidParameter = 1,
//     /// Communication timeout
//     Timeout = 2,
//     /// Checksum error
//     ChecksumError = 3,
//     /// Invalid frame format
//     InvalidFrame = 4,
//     /// Buffer overflow
//     BufferOverflow = 5,
//     /// Device not ready
//     DeviceNotReady = 6,
//     /// Hardware error
//     HardwareError = 7,
//     /// Protocol error
//     ProtocolError = 8,
//     /// Null pointer error
//     NullPointer = 9,
// }

// /// C-compatible IO-Link mode
// #[repr(C)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum CIoLinkMode {
//     /// SIO mode
//     Sio = 0,
//     /// COM1 mode (4.8 kbaud)
//     Com1 = 1,
//     /// COM2 mode (38.4 kbaud)
//     Com2 = 2,
//     /// COM3 mode (230.4 kbaud)
//     Com3 = 3,
// }

// /// C-compatible process data structure
// #[repr(C)]
// pub struct CProcessData {
//     /// Input data buffer
//     pub input_data: *mut u8,
//     /// Input data length
//     pub input_length: usize,
//     /// Output data buffer
//     pub output_data: *const u8,
//     /// Output data length
//     pub output_length: usize,
//     /// Data validity flag
//     pub valid: bool,
// }

// /// C-compatible device identification
// #[repr(C)]
// #[derive(Debug, Clone, Copy)]
// pub struct CDeviceIdentification {
//     /// Vendor ID
//     pub vendor_id: u16,
//     /// Device ID
//     pub device_id: u32,
//     /// Function ID
//     pub function_id: u16,
//     /// Reserved field
//     pub reserved: u8,
// }

// /// C-compatible event structure
// #[repr(C)]
// pub struct CEvent {
//     /// Event type
//     pub event_type: u16,
//     /// Event qualifier
//     pub qualifier: u8,
//     /// Event mode
//     pub mode: u8,
//     /// Event data buffer
//     pub data: *mut u8,
//     /// Event data length
//     pub data_length: usize,
// }

// /// Convert Rust error to C error
// impl From<IoLinkError> for CIoLinkError {
//     fn from(error: IoLinkError) -> Self {
//         match error {
//             IoLinkError::InvalidParameter => CIoLinkError::InvalidParameter,
//             IoLinkError::Timeout => CIoLinkError::Timeout,
//             IoLinkError::ChecksumError => CIoLinkError::ChecksumError,
//             IoLinkError::InvalidFrame => CIoLinkError::InvalidFrame,
//             IoLinkError::BufferOverflow => CIoLinkError::BufferOverflow,
//             IoLinkError::DeviceNotReady => CIoLinkError::DeviceNotReady,
//             IoLinkError::HardwareError => CIoLinkError::HardwareError,
//             IoLinkError::ProtocolError => CIoLinkError::ProtocolError,
//         }
//     }
// }

// /// Convert C mode to Rust mode
// impl From<CIoLinkMode> for IoLinkMode {
//     fn from(mode: CIoLinkMode) -> Self {
//         match mode {
//             CIoLinkMode::Sio => IoLinkMode::Sio,
//             CIoLinkMode::Com1 => IoLinkMode::Com1,
//             CIoLinkMode::Com2 => IoLinkMode::Com2,
//             CIoLinkMode::Com3 => IoLinkMode::Com3,
//         }
//     }
// }

// /// Convert Rust identification to C identification
// impl From<DeviceIdentification> for CDeviceIdentification {
//     fn from(id: DeviceIdentification) -> Self {
//         Self {
//             vendor_id: id.vendor_id,
//             device_id: id.device_id,
//             function_id: id.function_id,
//             reserved: id.reserved,
//         }
//     }
// }

// /// Create a new IO-Link device instance
// /// 
// /// # Safety
// /// This function returns a raw pointer that must be properly managed.
// /// The caller is responsible for calling `iolink_device_destroy` when done.
// #[no_mangle]
// pub extern "C" fn iolink_device_create() -> *mut IoLinkDeviceHandle {
//     let device = IoLinkDeviceHandle {
//         application_layer: ApplicationLayerImpl::new(),
//     };
    
//     Box::into_raw(Box::new(device))
// }

// /// Destroy an IO-Link device instance
// /// 
// /// # Safety
// /// The handle must be a valid pointer returned by `iolink_device_create`.
// /// After calling this function, the handle becomes invalid.
// #[no_mangle]
// pub extern "C" fn iolink_device_destroy(handle: *mut IoLinkDeviceHandle) {
//     if !handle.is_null() {
//         unsafe {
//             let _ = Box::from_raw(handle);
//         }
//     }
// }

// /// Poll the IO-Link device
// /// 
// /// # Safety
// /// The handle must be a valid pointer returned by `iolink_device_create`.
// #[no_mangle]
// pub extern "C" fn iolink_device_poll(handle: *mut IoLinkDeviceHandle) -> CIoLinkError {
//     if handle.is_null() {
//         return CIoLinkError::NullPointer;
//     }
    
//     let device = unsafe { &mut *handle };
    
//     match device.application_layer.poll() {
//         Ok(()) => CIoLinkError::Ok,
//         Err(error) => error.into(),
//     }
// }

// /// Get input data from the device
// /// 
// /// # Safety
// /// The handle must be a valid pointer returned by `iolink_device_create`.
// /// The process_data pointer must be valid and properly initialized.
// #[no_mangle]
// pub extern "C" fn iolink_get_input_data(
//     handle: *mut IoLinkDeviceHandle,
//     process_data: *mut CProcessData,
// ) -> CIoLinkError {
//     if handle.is_null() || process_data.is_null() {
//         return CIoLinkError::NullPointer;
//     }
    
//     let device = unsafe { &mut *handle };
//     let c_data = unsafe { &mut *process_data };
    
//     match device.application_layer.al_get_input_req() {
//         Ok(data) => {
//             // Copy input data to C buffer
//             if !c_data.input_data.is_null() {
//                 let input_len = core::cmp::min(data.input.len(), c_data.input_length);
//                 unsafe {
//                     ptr::copy_nonoverlapping(
//                         data.input.as_ptr(),
//                         c_data.input_data,
//                         input_len,
//                     );
//                 }
//                 c_data.input_length = input_len;
//             }
//             c_data.valid = data.valid;
//             CIoLinkError::Ok
//         }
//         Err(error) => error.into(),
//     }
// }

// /// Set output data to the device
// /// 
// /// # Safety
// /// The handle must be a valid pointer returned by `iolink_device_create`.
// /// The process_data pointer must be valid.
// #[no_mangle]
// pub extern "C" fn iolink_set_output_data(
//     handle: *mut IoLinkDeviceHandle,
//     process_data: *const CProcessData,
// ) -> CIoLinkError {
//     if handle.is_null() || process_data.is_null() {
//         return CIoLinkError::NullPointer;
//     }
    
//     let device = unsafe { &mut *handle };
//     let c_data = unsafe { &*process_data };
    
//     // Convert C data to Rust data
//     let mut rust_data = ProcessData::default();
    
//     if !c_data.output_data.is_null() && c_data.output_length > 0 {
//         let output_slice = unsafe {
//             core::slice::from_raw_parts(c_data.output_data, c_data.output_length)
//         };
        
//         for &byte in output_slice {
//             if rust_data.output.push(byte).is_err() {
//                 return CIoLinkError::BufferOverflow;
//             }
//         }
//     }
    
//     rust_data.valid = c_data.valid;
    
//     match device.application_layer.al_set_output_req(&rust_data) {
//         Ok(()) => CIoLinkError::Ok,
//         Err(error) => error.into(),
//     }
// }

// /// Read parameter via ISDU
// /// 
// /// # Safety
// /// The handle must be a valid pointer returned by `iolink_device_create`.
// /// The data buffer must be valid and large enough for the expected data.
// #[no_mangle]
// pub extern "C" fn iolink_read_parameter(
//     handle: *mut IoLinkDeviceHandle,
//     index: u16,
//     sub_index: u8,
//     data: *mut u8,
//     data_length: *mut usize,
// ) -> CIoLinkError {
//     if handle.is_null() || data.is_null() || data_length.is_null() {
//         return CIoLinkError::NullPointer;
//     }
    
//     let device = unsafe { &mut *handle };
//     let max_length = unsafe { *data_length };
    
//     match device.application_layer.al_read_req(index, sub_index) {
//         Ok(result_data) => {
//             let copy_length = core::cmp::min(result_data.len(), max_length);
//             unsafe {
//                 ptr::copy_nonoverlapping(
//                     result_data.as_ptr(),
//                     data,
//                     copy_length,
//                 );
//                 *data_length = copy_length;
//             }
//             CIoLinkError::Ok
//         }
//         Err(error) => error.into(),
//     }
// }

// /// Write parameter via ISDU
// /// 
// /// # Safety
// /// The handle must be a valid pointer returned by `iolink_device_create`.
// /// The data buffer must be valid and contain the data to write.
// #[no_mangle]
// pub extern "C" fn iolink_write_parameter(
//     handle: *mut IoLinkDeviceHandle,
//     index: u16,
//     sub_index: u8,
//     data: *const u8,
//     data_length: usize,
// ) -> CIoLinkError {
//     if handle.is_null() || (data.is_null() && data_length > 0) {
//         return CIoLinkError::NullPointer;
//     }
    
//     let device = unsafe { &mut *handle };
//     let data_slice = if data_length > 0 {
//         unsafe { core::slice::from_raw_parts(data, data_length) }
//     } else {
//         &[]
//     };
    
//     match device.application_layer.al_write_req(index, sub_index, data_slice) {
//         Ok(()) => CIoLinkError::Ok,
//         Err(error) => error.into(),
//     }
// }

// /// Get device identification
// /// 
// /// # Safety
// /// The handle must be a valid pointer returned by `iolink_device_create`.
// /// The device_id pointer must be valid.
// #[no_mangle]
// pub extern "C" fn iolink_get_device_id(
//     handle: *mut IoLinkDeviceHandle,
//     device_id: *mut CDeviceIdentification,
// ) -> CIoLinkError {
//     if handle.is_null() || device_id.is_null() {
//         return CIoLinkError::NullPointer;
//     }
    
//     let device = unsafe { &mut *handle };
    
//     match device.application_layer.al_get_device_id_req() {
//         Ok(id) => {
//             unsafe {
//                 *device_id = id.into();
//             }
//             CIoLinkError::Ok
//         }
//         Err(error) => error.into(),
//     }
// }

// /// Get minimum cycle time
// /// 
// /// # Safety
// /// The handle must be a valid pointer returned by `iolink_device_create`.
// /// The cycle_time pointer must be valid.
// #[no_mangle]
// pub extern "C" fn iolink_get_min_cycle_time(
//     handle: *mut IoLinkDeviceHandle,
//     cycle_time: *mut u8,
// ) -> CIoLinkError {
//     if handle.is_null() || cycle_time.is_null() {
//         return CIoLinkError::NullPointer;
//     }
    
//     let device = unsafe { &mut *handle };
    
//     match device.application_layer.al_get_min_cycle_time_req() {
//         Ok(time) => {
//             unsafe {
//                 *cycle_time = time;
//             }
//             CIoLinkError::Ok
//         }
//         Err(error) => error.into(),
//     }
// }

// /// Get library version string
// /// 
// /// # Safety
// /// Returns a static string that is valid for the lifetime of the program.
// #[no_mangle]
// pub extern "C" fn iolink_get_version() -> *const core::ffi::c_char {
//     const VERSION: &[u8] = b"iolink_device_stack v0.1.0\0";
//     VERSION.as_ptr() as *const core::ffi::c_char
// }
