// //! Test utilities for IO-Link device stack testing
// //!
// //! This module provides utilities and mock implementations for testing
// //! the IO-Link device stack components.

pub use crate::SystemManagementReq;
pub use crate::config;
pub use crate::config::m_seq_capability;
pub use crate::types::ComChannel;
pub use crate::types::MsequenceBaseType;
pub use crate::types::MsequenceType;
pub use crate::types::RwDirection;
pub use crate::utils::frame_fromat::message::ChecksumMsequenceType;
pub use crate::utils::frame_fromat::message::ChecksumMsequenceTypeBuilder;
pub use crate::utils::frame_fromat::message::ChecksumStatus;
pub use crate::utils::frame_fromat::message::ChecksumStatusBuilder;
pub use crate::utils::frame_fromat::message::calculate_checksum_for_testing;
pub use crate::utils::frame_fromat::message::{MsequenceControl, MsequenceControlBuilder};
use crate::*;
use crate::{IoLinkDevice, IoLinkError, IoLinkMode, IoLinkResult, PhysicalLayerReq, Timer};

use crate::pl::physical_layer::PhysicalLayerInd;
use core::time::Duration;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum ThreadMessage {
    TimerExpired(Timer),
    RxData(Vec<u8>),
    TxData(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestDeviceMode {
    Startup,
    Preoperate,
    Operate,
}

pub struct MockPhysicalLayer {
    mode: IoLinkMode,
    tx_data: Vec<u8>,
    rx_data: Vec<u8>,
    timers: Arc<Mutex<Vec<MockTimerState>>>,
    mock_to_usr_tx: Sender<ThreadMessage>,
}

pub struct MockTimerState {
    timer_id: Timer,
    start_time: Instant,
    duration_us: u32,
    active: bool,
}

impl MockTimerState {
    fn new(timer_id: Timer, duration_us: u32) -> Self {
        Self {
            timer_id,
            start_time: Instant::now(),
            duration_us,
            active: true,
        }
    }

    fn is_expired(&self) -> bool {
        if !self.active {
            return false;
        }
        let elapsed = self.start_time.elapsed();
        let elapsed_us = elapsed.as_micros() as u32;
        elapsed_us >= self.duration_us
    }

    fn restart(&mut self, duration_us: u32) {
        self.start_time = Instant::now();
        self.duration_us = duration_us;
        self.active = true;
    }

    fn stop(&mut self) {
        self.active = false;
    }
}

impl MockPhysicalLayer {
    /// Create a new MockPhysicalLayer
    ///
    /// # Arguments
    ///
    /// * `usr_to_mock_rx` - A receiver for messages from the user thread
    /// * `mock_to_usr_tx` - A sender for messages to the user thread
    ///
    /// # Returns
    ///
    /// A new MockPhysicalLayer
    ///
    pub fn new(mock_to_usr_tx: Sender<ThreadMessage>) -> Self {
        Self {
            mode: IoLinkMode::Inactive,
            tx_data: Vec::new(),
            rx_data: Vec::new(),
            timers: Arc::new(Mutex::new(Vec::new())),
            mock_to_usr_tx,
        }
    }

    /// Transfer the received data to the IO-Link device
    pub fn transfer_ind(
        &mut self,
        rx_buffer: &[u8],
        io_link_device: Arc<Mutex<IoLinkDevice>>,
    ) -> IoLinkResult<()> {
        self.rx_data.clear();
        self.rx_data.extend_from_slice(rx_buffer);
        let mut io_link_device_lock = io_link_device.lock().unwrap();
        for rx_buffer_byte in rx_buffer {
            let _ = io_link_device_lock.pl_transfer_ind(self, *rx_buffer_byte);
        }
        Ok(())
    }

    pub fn timer_expired(&mut self, timer: Timer) {
        println!("Timer expired: {:?}", timer);
        // Here you can add any specific logic needed when a timer expires
        // For example, triggering state transitions or error handling
    }

    /// Check if any timers have expired and call timer_expired for them
    pub fn check_timers(&mut self) {
        let expired_timers = {
            let mut timers = self.timers.lock().unwrap();
            let mut expired_timers = Vec::new();
            let mut i = 0;

            // Find expired timers
            while i < timers.len() {
                if timers[i].is_expired() {
                    expired_timers.push(timers[i].timer_id);
                    timers.remove(i);
                } else {
                    i += 1;
                }
            }
            expired_timers
        };

        // Call timer_expired for each expired timer (after releasing the lock)
        for timer_id in expired_timers {
            self.timer_expired(timer_id);
        }
    }
}

impl PhysicalLayerReq for MockPhysicalLayer {
    fn pl_set_mode_req(&mut self, mode: IoLinkMode) -> IoLinkResult<()> {
        self.mode = mode;
        Ok(())
    }

    fn pl_transfer_req(&mut self, tx_data: &[u8]) -> IoLinkResult<usize> {
        self.tx_data.clear();
        self.tx_data.extend_from_slice(tx_data);
        self.mock_to_usr_tx
            .send(ThreadMessage::TxData(self.tx_data.clone()))
            .unwrap();
        let tx_data_len = tx_data.len();
        Ok(tx_data_len)
    }

    fn stop_timer(&self, timer: Timer) -> IoLinkResult<()> {
        let mut timers = self.timers.lock().unwrap();
        for timer_state in timers.iter_mut() {
            if timer_state.timer_id == timer {
                timer_state.stop();
                break;
            }
        }
        Ok(())
    }

    fn start_timer(&self, timer: Timer, duration_us: u32) -> IoLinkResult<()> {
        let mut timers = self.timers.lock().unwrap();
        let timer_state = MockTimerState::new(timer, duration_us);
        timers.push(timer_state);
        Ok(())
    }

    fn restart_timer(&self, timer: Timer, duration_us: u32) -> IoLinkResult<()> {
        let mut timers = self.timers.lock().unwrap();
        for timer_state in timers.iter_mut() {
            if timer_state.timer_id == timer {
                timer_state.restart(duration_us);
                return Ok(());
            }
        }
        // If timer doesn't exist, create it
        let timer_state = MockTimerState::new(timer, duration_us);
        timers.push(timer_state);
        Ok(())
    }
}

/// This function is used to poll the device and check for expired timers.
pub fn take_care_of_poll_nb(
    io_link_device: Arc<Mutex<IoLinkDevice>>,
    usr_to_mock_rx: Receiver<ThreadMessage>,
    mock_to_usr_tx: Sender<ThreadMessage>,
) {
    // Main device loop
    std::thread::spawn(move || {
        // Check for expired timers before polling
        let mut physical_layer: MockPhysicalLayer = MockPhysicalLayer::new(mock_to_usr_tx);
        // Poll all protocol layers
        loop {
            for _ in 0..9
            {
                let mut io_link_device_lock = io_link_device.lock().unwrap();
                match io_link_device_lock.poll(&mut physical_layer) {
                    Ok(()) => {
                        // Device operating normally
                        // In a real implementation, you might add some delay here
                        // sleep(Duration::from_millis(10));
                    }
                    Err(IoLinkError::NoImplFound) => {
                        // Feature not implemented yet, continue operation
                        // This is expected in the basic example
                    }
                    Err(e) => {
                        // Handle other errors
                        eprintln!("Device error: {:?}", e);
                    }
                }
            }
            match usr_to_mock_rx.recv_timeout(Duration::from_micros(5)) {
                Ok(msg) => {
                    match msg {
                        ThreadMessage::RxData(data) => {
                            let io_link_device_clone = Arc::clone(&io_link_device);
                            let _ = physical_layer.transfer_ind(&data, io_link_device_clone);
                        }
                        _ => {
                            // Do nothing
                        }
                    }
                }
                Err(_e) => {
                    // eprintln!("Error receiving message: {:?}", e);
                    // No message received, continue polling
                }
            }
        }
    });
}

/// Validate the checksum of the device frame
///
/// # Arguments
///
/// * `data` - The data to validate
///
/// # Returns
///
/// True if the checksum is valid, false otherwise
///
pub fn validate_device_frame_checksum(data: &mut Vec<u8>) -> bool {
    let data_len = data.len();
    let cks = ChecksumStatus::from(data[data_len - 1]);
    let mut cleared_checksum_cks = cks.clone();
    cleared_checksum_cks.set_checksum(0);

    let data_last_index = data.get_mut(data_len - 1).unwrap();
    let cleared_checksum_cks_bits = cleared_checksum_cks.into_bits();
    *data_last_index = cleared_checksum_cks_bits;
    let cks_calculated_checksum = calculate_checksum_for_testing(data_len as u8, data);
    let rec_cks = cks.checksum();
    cks_calculated_checksum == rec_cks
}

/// Creates a new IO-Link device instance with communication channels
pub fn create_test_device() -> (
    Arc<Mutex<IoLinkDevice>>,
    Sender<ThreadMessage>,
    Receiver<ThreadMessage>,
) {
    let io_link_device: Arc<Mutex<IoLinkDevice>> = Arc::new(Mutex::new(IoLinkDevice::new()));
    let (usr_to_mock_tx, usr_to_mock_rx) = mpsc::channel();
    let (mock_to_usr_tx, mock_to_usr_rx): (Sender<ThreadMessage>, Receiver<ThreadMessage>) =
        mpsc::channel();

    (io_link_device, usr_to_mock_tx, mock_to_usr_rx)
}

/// Sets up the device with basic configuration for testing
pub fn setup_device_configuration(io_link_device: &Arc<Mutex<IoLinkDevice>>) {
    let mseq_cap = m_seq_capability::m_sequence_capability_parameter();

    let _ = io_link_device.lock().unwrap().sm_set_device_mode_req(DeviceMode::Idle);
    // Set device identification parameters
    let _ = io_link_device
        .lock()
        .unwrap()
        .sm_set_device_com_req(&DeviceCom {
            suppported_sio_mode: SioMode::default(),
            transmission_rate: TransmissionRate::Com3,
            min_cycle_time: CycleTimeBuilder::new()
                .with_time_base(0b10)
                .with_multiplier(0b000001)
                .build(),
            msequence_capability: MsequenceCapability::from(mseq_cap),
            revision_id: RevisionIdBuilder::new()
                .with_major_rev(1)
                .with_minor_rev(1)
                .build(),
            process_data_in: ProcessDataInBuilder::new()
                .with_length(9)
                .with_sio(true)
                .with_byte(true)
                .build(),
            process_data_out: ProcessDataOutBuilder::new()
                .with_length(9)
                .with_byte(true)
                .build(),
        });

    let _ = io_link_device
        .lock()
        .unwrap()
        .sm_set_device_ident_req(&DeviceIdent {
            vendor_id: [0x12, 0x34],
            device_id: [0x56, 0x78, 0x9A],
            function_id: [0xBC, 0xDE],
        });

    let _ = io_link_device
        .lock()
        .unwrap()
        .sm_set_device_mode_req(DeviceMode::Sio);
}

/// Performs the startup sequence for the device
pub fn perform_startup_sequence(io_link_device: &Arc<Mutex<IoLinkDevice>>) {
    let _ = io_link_device.lock().unwrap().pl_wake_up_ind();
    std::thread::sleep(std::time::Duration::from_micros(100));
    let _ = io_link_device
        .lock()
        .unwrap()
        .successful_com(TransmissionRate::Com3);
}

/// Creates a read request message for testing
pub fn create_startup_read_request(address: u8) -> Vec<u8> {
    let mc = test_utils::MsequenceControlBuilder::new()
        .with_read_write(RwDirection::Read)
        .with_comm_channel(ComChannel::Page)
        .with_address_fctrl(address)
        .build();

    let mut ckt: ChecksumMsequenceType = test_utils::ChecksumMsequenceTypeBuilder::new()
        .with_m_seq_type(MsequenceBaseType::Type0)
        .with_checksum(0)
        .build();

    let mc_bits = mc.into_bits();
    let ckt_bits = ckt.into_bits();

    let mut rx_buffer = Vec::new();
    rx_buffer.push(mc_bits);
    rx_buffer.push(ckt_bits);

    let checksum = test_utils::calculate_checksum_for_testing(rx_buffer.len() as u8, &rx_buffer);
    ckt.set_checksum(checksum);

    let tx_buffer_1 = rx_buffer.get_mut(1).unwrap();
    *tx_buffer_1 = ckt.into_bits();

    rx_buffer
}

/// Creates a read request message for testing
pub fn create_preop_read_request(address: u8) -> Vec<u8> {
    let mc = test_utils::MsequenceControlBuilder::new()
    .with_read_write(RwDirection::Read)
    .with_comm_channel(ComChannel::Page)
    .with_address_fctrl(address)
    .build();

    let mut ckt = test_utils::ChecksumMsequenceTypeBuilder::new()
        .with_m_seq_type(config::m_seq_capability::pre_operate_m_sequence::m_sequence_base_type())
        .with_checksum(0)
        .build();

    let mc_bits = mc.into_bits();
    let ckt_bits = ckt.into_bits();

    let mut rx_buffer = Vec::new();
    rx_buffer.push(mc_bits);
    rx_buffer.push(ckt_bits);

    let checksum = test_utils::calculate_checksum_for_testing(rx_buffer.len() as u8, &rx_buffer);
    ckt.set_checksum(checksum);

    let tx_buffer_1 = rx_buffer.get_mut(1).unwrap();
    *tx_buffer_1 = ckt.into_bits();

    rx_buffer
}

/// Creates a write request message for testing
pub fn create_startup_write_request(address: u8, data: u8) -> Vec<u8> {
    let mc = test_utils::MsequenceControlBuilder::new()
        .with_read_write(RwDirection::Write)
        .with_comm_channel(ComChannel::Page)
        .with_address_fctrl(address)
        .build();

    let mut ckt = test_utils::ChecksumMsequenceTypeBuilder::new()
        .with_m_seq_type(MsequenceBaseType::Type0)
        .with_checksum(0)
        .build();

    let mc_bits = mc.into_bits();
    let ckt_bits = ckt.into_bits();

    let mut rx_buffer = Vec::new();
    rx_buffer.push(mc_bits);
    rx_buffer.push(ckt_bits);
    rx_buffer.push(data); // Add the data to write

    let checksum = test_utils::calculate_checksum_for_testing(rx_buffer.len() as u8, &rx_buffer);
    ckt.set_checksum(checksum);

    let tx_buffer_1 = rx_buffer.get_mut(1).unwrap();
    *tx_buffer_1 = ckt.into_bits();

    rx_buffer
}

/// Creates a write request message for testing
pub fn create_preop_write_request(address: u8, data: &[u8]) -> Vec<u8> {
    const OD_LENGTH_BYTES: u8 = config::on_req_data::pre_operate::od_length();
    let mc = test_utils::MsequenceControlBuilder::new()
        .with_read_write(RwDirection::Write)
        .with_comm_channel(ComChannel::Page)
        .with_address_fctrl(address)
        .build();

    let mut ckt = test_utils::ChecksumMsequenceTypeBuilder::new()
        .with_m_seq_type(config::m_seq_capability::pre_operate_m_sequence::m_sequence_base_type())
        .with_checksum(0)
        .build();

    let mc_bits = mc.into_bits();
    let ckt_bits = ckt.into_bits();

    let mut rx_buffer = Vec::new();
    rx_buffer.push(mc_bits);
    rx_buffer.push(ckt_bits);
    for index in 0..OD_LENGTH_BYTES as usize {
        if index < data.len() {
            rx_buffer.push(data[index]); // Add the data to write
        } else {
            rx_buffer.push(0);
        }
    }

    let checksum = test_utils::calculate_checksum_for_testing(rx_buffer.len() as u8, &rx_buffer);
    ckt.set_checksum(checksum);

    let tx_buffer_1 = rx_buffer.get_mut(1).unwrap();
    *tx_buffer_1 = ckt.into_bits();

    rx_buffer
}

/// Sends a test message and waits for response
pub fn send_test_message_and_wait(
    tx_sender: &Sender<ThreadMessage>,
    rx_receiver: &Receiver<ThreadMessage>,
    message: Vec<u8>,
    timeout: Duration,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let _ = tx_sender.send(ThreadMessage::RxData(message));

    let rx_data = rx_receiver.recv_timeout(timeout)?;
    match rx_data {
        ThreadMessage::TxData(data) => Ok(data),
        _ => Err("Expected TxData response".into()),
    }
}

/// Extracts and validates checksum from response
pub fn validate_checksum(response: &[u8], expected_checksum: u8) -> bool {
    if response.len() < 2 {
        return false;
    }

    let received_checksum = test_utils::ChecksumStatus::from(response[1]);
    received_checksum.checksum() == expected_checksum
}

/// Sets up a test environment with device and polling thread
pub fn setup_test_environment() -> (
    Arc<Mutex<IoLinkDevice>>,
    Sender<ThreadMessage>,
    Receiver<ThreadMessage>,
) {
    // Create test device and channels
    let (io_link_device, _usr_to_mock_tx, _mock_to_usr_rx) = create_test_device();

    // Create separate channels for the polling thread
    let (poll_tx, poll_rx) = mpsc::channel();
    let (poll_response_tx, poll_response_rx) = mpsc::channel();

    // Start the polling thread
    let io_link_device_clone_poll = Arc::clone(&io_link_device);
    test_utils::take_care_of_poll_nb(io_link_device_clone_poll, poll_rx, poll_response_tx);

    // Perform startup sequence
    let io_link_device_clone = Arc::clone(&io_link_device);
    startup_routine(io_link_device_clone);

    (io_link_device, poll_tx, poll_response_rx)
}

/// Complete startup routine that combines configuration and startup sequence
pub fn startup_routine(io_link_device: Arc<Mutex<IoLinkDevice>>) {
    setup_device_configuration(&io_link_device);
    perform_startup_sequence(&io_link_device);
}/// Page parameters module
pub mod page_params {
    use std::sync::mpsc::{Receiver, Sender};
    use std::time::Duration;
    use crate::*;
    use crate::test_utils::{self, ThreadMessage};
    use super::TestDeviceMode;

const TIMEOUT: Duration = Duration::from_secs(40);
/// Reads the min cycle time from the device using the provided communication channels.
/// 
/// # Arguments
/// * `poll_tx` - Sender to send requests to the device
/// * `poll_response_rx` - Receiver to receive responses from the device
/// 
/// # Returns
/// * `CycleTime` - The min cycle time read from the device
pub fn read_min_cycle_time(
    poll_tx: &Sender<ThreadMessage>,
    poll_response_rx: &Receiver<ThreadMessage>,
    device_mode: TestDeviceMode,
) -> CycleTime {
    // Create read request for MinCycleTime
    let rx_buffer = if device_mode == TestDeviceMode::Startup { 
        test_utils::create_startup_read_request(direct_parameter_address!(MinCycleTime))
    } else {
        test_utils::create_preop_read_request(direct_parameter_address!(MinCycleTime))
    }; // for read cycle startup and preop builds to the same frame
    
    // Send message to the device through the polling thread
    let _ = poll_tx.send(ThreadMessage::RxData(rx_buffer));
    
    // Wait for response from the MockPhysicalLayer
    let response = poll_response_rx.recv_timeout(TIMEOUT)
        .expect("Failed to get response from device");
    
    // Extract the response data
    let mut response_data = match response {
        ThreadMessage::TxData(data) => data,
        _ => panic!("Expected TxData response, got: {:?}", response),
    };
    let min_cycle_time = CycleTime::from_bits(*response_data.get(0).unwrap());
    // Validate checksum
    let is_checksum_valid = crate::test_utils::validate_device_frame_checksum(&mut response_data);
    assert!(response_data.len() == 2, "Response too short");
    assert!(is_checksum_valid, "Checksum not matching");
    min_cycle_time
}

/// Reads the m-sequence capability from the device using the provided communication channels.
pub fn read_m_sequence_capability(
    poll_tx: &Sender<ThreadMessage>,
    poll_response_rx: &Receiver<ThreadMessage>,
    device_mode: TestDeviceMode,
) -> MsequenceCapability {
    // Create read request for MinCycleTime
    let (rx_buffer, expected_rx_bytes) = if device_mode == TestDeviceMode::Startup { 
        (test_utils::create_startup_read_request(direct_parameter_address!(MSequenceCapability)), 2)
    } else {
        (test_utils::create_preop_read_request(direct_parameter_address!(MSequenceCapability)), 3)
    };
    
    // Send message to the device through the polling thread
    let _ = poll_tx.send(ThreadMessage::RxData(rx_buffer));
    
    // Wait for response from the MockPhysicalLayer
    let response = poll_response_rx.recv_timeout(TIMEOUT)
        .expect("Failed to get response from device");
    
    // Extract the response data
    let mut response_data: Vec<u8> = match response {
        ThreadMessage::TxData(data) => data,
        _ => panic!("Expected TxData response, got: {:?}", response),
    };
    // for n in response_data.iter() {
    //     print!(" -{}", n);
    // }
    let m_sequence_capability = MsequenceCapability::from_bits(*response_data.get(0).unwrap());
    println!("m_sequence_capability: {:?}", m_sequence_capability);
    // Validate checksum
    let is_checksum_valid = test_utils::validate_device_frame_checksum(&mut response_data);
    assert!(response_data.len() == expected_rx_bytes as usize, "Response too short");
    assert!(is_checksum_valid, "Checksum not matching");
    m_sequence_capability
}

/// Reads the revision id from the device using the provided communication channels.
pub fn read_revision_id(
    poll_tx: &Sender<ThreadMessage>,
    poll_response_rx: &Receiver<ThreadMessage>,
) -> RevisionId {
    // Create read request for MinCycleTime
    let rx_buffer = crate::test_utils::create_startup_read_request(crate::direct_parameter_address!(RevisionID));
    
    // Send message to the device through the polling thread
    let _ = poll_tx.send(ThreadMessage::RxData(rx_buffer));
    
    // Wait for response from the MockPhysicalLayer
    let response = poll_response_rx.recv_timeout(TIMEOUT)
        .expect("Failed to get response from device");
    
    // Extract the response data
    let mut response_data = match response {
        ThreadMessage::TxData(data) => data,
        _ => panic!("Expected TxData response, got: {:?}", response),
    };
    let revision_id = RevisionId::from(response_data[0]);
    // Validate checksum
    let is_checksum_valid = test_utils::validate_device_frame_checksum(&mut response_data);
    assert!(response_data.len() == 2, "Response too short");
    assert!(is_checksum_valid, "Checksum not matching");
    revision_id
}

/// Reads the process data in from the device using the provided communication channels.
pub fn read_process_data_in(
    poll_tx: &Sender<ThreadMessage>,
    poll_response_rx: &Receiver<ThreadMessage>,
) -> ProcessDataIn {
    // Create read request for MinCycleTime
    let rx_buffer = crate::test_utils::create_startup_read_request(crate::direct_parameter_address!(ProcessDataIn));
    
    // Send message to the device through the polling thread
    let _ = poll_tx.send(ThreadMessage::RxData(rx_buffer));
    
    // Wait for response from the MockPhysicalLayer
    let response = poll_response_rx.recv_timeout(TIMEOUT)
        .expect("Failed to get response from device");
    
    // Extract the response data
    let mut response_data = match response {
        ThreadMessage::TxData(data) => data,
        _ => panic!("Expected TxData response, got: {:?}", response),
    };
    let process_data_in = ProcessDataIn::from(response_data[0]);
    // Validate checksum
    let is_checksum_valid = test_utils::validate_device_frame_checksum(&mut response_data);
    assert!(response_data.len() == 2, "Response too short");
    assert!(is_checksum_valid, "Checksum not matching");
    process_data_in
}

/// Reads the process data out from the device using the provided communication channels.
pub fn read_process_data_out(
    poll_tx: &Sender<ThreadMessage>,
    poll_response_rx: &Receiver<ThreadMessage>,
) -> ProcessDataOut {
    // Create read request for MinCycleTime
    let rx_buffer = crate::test_utils::create_startup_read_request(crate::direct_parameter_address!(ProcessDataOut));
    
    // Send message to the device through the polling thread
    let _ = poll_tx.send(ThreadMessage::RxData(rx_buffer));
    
    // Wait for response from the MockPhysicalLayer
    let response = poll_response_rx.recv_timeout(TIMEOUT)
        .expect("Failed to get response from device");
    
    // Extract the response data
    let mut response_data = match response {
        ThreadMessage::TxData(data) => data,
        _ => panic!("Expected TxData response, got: {:?}", response),
    };
    let process_data_out = ProcessDataOut::from(response_data[0]);
    // Validate checksum
    let is_checksum_valid = test_utils::validate_device_frame_checksum(&mut response_data);
    assert!(response_data.len() == 2, "Response too short");
    assert!(is_checksum_valid, "Checksum not matching");
    process_data_out
}

/// Reads the vendor id 1 from the device using the provided communication channels.
pub fn read_vendor_id_1(
    poll_tx: &Sender<ThreadMessage>,
    poll_response_rx: &Receiver<ThreadMessage>,
) -> u8 {
    // Create read request for MinCycleTime
    let rx_buffer = crate::test_utils::create_startup_read_request(crate::direct_parameter_address!(VendorID1));
    
    // Send message to the device through the polling thread
    let _ = poll_tx.send(ThreadMessage::RxData(rx_buffer));
    
    // Wait for response from the MockPhysicalLayer
    let response = poll_response_rx.recv_timeout(TIMEOUT)
        .expect("Failed to get response from device");
    
    // Extract the response data
    let mut response_data = match response {
        ThreadMessage::TxData(data) => data,
        _ => panic!("Expected TxData response, got: {:?}", response),
    };
    let vendor_id_1 = response_data[0];
    // Validate checksum
    let is_checksum_valid = test_utils::validate_device_frame_checksum(&mut response_data);
    assert!(response_data.len() == 2, "Response too short");
    assert!(is_checksum_valid, "Checksum not matching");
    vendor_id_1
}

/// Reads the vendor id 2 from the device using the provided communication channels.
pub fn read_vendor_id_2(
    poll_tx: &Sender<ThreadMessage>,
    poll_response_rx: &Receiver<ThreadMessage>,
) -> u8 {
    // Create read request for VendorID2
    let rx_buffer = crate::test_utils::create_startup_read_request(crate::direct_parameter_address!(VendorID2));
    
    // Send message to the device through the polling thread
    let _ = poll_tx.send(ThreadMessage::RxData(rx_buffer));
    
    // Wait for response from the MockPhysicalLayer
    let response = poll_response_rx.recv_timeout(TIMEOUT)
        .expect("Failed to get response from device");
    
    // Extract the response data
    let mut response_data = match response {
        ThreadMessage::TxData(data) => data,
        _ => panic!("Expected TxData response, got: {:?}", response),
    };
    let vendor_id_2 = response_data[0];
    // Validate checksum
    let is_checksum_valid = test_utils::validate_device_frame_checksum(&mut response_data);
    assert!(response_data.len() == 2, "Response too short");
    assert!(is_checksum_valid, "Checksum not matching");
    vendor_id_2
}

}

