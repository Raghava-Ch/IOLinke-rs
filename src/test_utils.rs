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
pub use crate::utils::frame_fromat::isdu::IsduService;
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
pub fn take_care_of_poll(
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
            for _ in 0..9 {
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
    test_utils::take_care_of_poll(io_link_device_clone_poll, poll_rx, poll_response_tx);

    // Perform startup sequence
    let io_link_device_clone = Arc::clone(&io_link_device);
    startup_routine(io_link_device_clone);

    (io_link_device, poll_tx, poll_response_rx)
}

/// Complete startup routine that combines configuration and startup sequence
pub fn startup_routine(io_link_device: Arc<Mutex<IoLinkDevice>>) {
    frame_format_utils::setup_device_configuration(&io_link_device);
    frame_format_utils::perform_startup_sequence(&io_link_device);
}

pub fn util_test_startup_sequence(
    poll_tx: &Sender<ThreadMessage>,
    poll_response_rx: &Receiver<ThreadMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read min cycle time
    let min_cycle_time = test_utils::page_params::read_min_cycle_time(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Startup,
    );
    const CONFIG_MIN_CYCLE_TIME: CycleTime =
        config::timings::min_cycle_time::min_cycle_time_parameter();
    assert!(
        CONFIG_MIN_CYCLE_TIME.time_base() == min_cycle_time.time_base(),
        "Min cycle time time_base not matching"
    );
    assert!(
        CONFIG_MIN_CYCLE_TIME.multiplier() == min_cycle_time.multiplier(),
        "Min cycle time multiplier not matching"
    );

    // Read m-sequence capability
    let m_sequence_capability = test_utils::page_params::read_m_sequence_capability(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Startup,
    );
    const CONFIG_M_SEQUENCE_CAPABILITY: MsequenceCapability =
        config::m_seq_capability::m_sequence_capability_parameter();
    assert!(
        CONFIG_M_SEQUENCE_CAPABILITY.preoperate_m_sequence()
            == m_sequence_capability.preoperate_m_sequence(),
        "M-sequenceCapability Pre operate msequnce is not matching"
    );
    assert!(
        CONFIG_M_SEQUENCE_CAPABILITY.operate_m_sequence()
            == m_sequence_capability.operate_m_sequence(),
        "M-sequenceCapability Operate msequnce is not matching"
    );
    assert!(
        CONFIG_M_SEQUENCE_CAPABILITY.isdu() == m_sequence_capability.isdu(),
        "M-sequenceCapability ISDU is not matching"
    );

    // Read revision id
    let revision_id = test_utils::page_params::read_revision_id(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Startup,
    );
    const CONFIG_REVISION_ID: RevisionId = config::vendor_specifics::revision_id_parameter();
    assert!(
        CONFIG_REVISION_ID.major_rev() == revision_id.major_rev(),
        "RevisionID major rev is not matching"
    );
    assert!(
        CONFIG_REVISION_ID.minor_rev() == revision_id.minor_rev(),
        "RevisionID minor rev is not matching"
    );

    // Read process data in
    let process_data_in = test_utils::page_params::read_process_data_in(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Startup,
    );
    const CONFIG_PROCESS_DATA_IN: ProcessDataIn = config::process_data::pd_in::pd_in_parameter();
    assert!(
        CONFIG_PROCESS_DATA_IN.byte() == process_data_in.byte(),
        "ProcessDataIn byte is not matching"
    );
    assert!(
        CONFIG_PROCESS_DATA_IN.sio() == process_data_in.sio(),
        "ProcessDataIn sio is not matching"
    );
    assert!(
        CONFIG_PROCESS_DATA_IN.length() == process_data_in.length(),
        "ProcessDataIn length is not matching"
    );

    // Read process data out
    let process_data_out = test_utils::page_params::read_process_data_out(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Startup,
    );
    const CONFIG_PROCESS_DATA_OUT: ProcessDataOut =
        config::process_data::pd_out::pd_out_parameter();
    assert!(
        CONFIG_PROCESS_DATA_OUT.byte() == process_data_out.byte(),
        "ProcessDataOut byte is not matching"
    );
    assert!(
        CONFIG_PROCESS_DATA_OUT.length() == process_data_out.length(),
        "ProcessDataOut length is not matching"
    );

    // Read vendor id
    let vendor_id_1 = test_utils::page_params::read_vendor_id_1(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Startup,
    );
    const CONFIG_VENDOR_ID_1: u8 = config::vendor_specifics::vendor_id_1();
    assert!(
        CONFIG_VENDOR_ID_1 == vendor_id_1,
        "VendorID1 is not matching"
    );

    // Read vendor id
    let vendor_id_2 = test_utils::page_params::read_vendor_id_2(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Startup,
    );
    const CONFIG_VENDOR_ID_2: u8 = config::vendor_specifics::vendor_id_2();
    assert!(
        CONFIG_VENDOR_ID_2 == vendor_id_2,
        "VendorID2 is not matching"
    );

    // Assuming startup sequence is successful, because all the configure
    // parameters page1 (0x02 to 0x06) are read and checked
    // Now we can command the device to Pre operate mode
    let is_master_ident_written = test_utils::page_params::write_master_command(
        poll_tx,
        poll_response_rx,
        TestDeviceMode::Startup,
        MasterCommand::MasterIdent,
    );
    assert!(is_master_ident_written, "Failed to write master ident");

    let is_master_pre_op_written = test_utils::page_params::write_master_command(
        poll_tx,
        poll_response_rx,
        TestDeviceMode::Startup,
        MasterCommand::DevicePreOperate,
    );
    assert!(is_master_pre_op_written, "Failed to write master pre op");

    Ok(())
}

pub fn util_test_preop_sequence(
    poll_tx: &Sender<ThreadMessage>,
    poll_response_rx: &Receiver<ThreadMessage>,
) -> Result<MsequenceCapability, Box<dyn std::error::Error>> {
    // Read m-sequence capability
    let m_sequence_capability = test_utils::page_params::read_m_sequence_capability(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Preoperate,
    );

    Ok(m_sequence_capability)
}

/// Test ISDU sequence
/// 
/// # Arguments
/// * `poll_tx` - The sender for the polling thread
/// * `poll_response_rx` - The receiver for the polling thread
/// * `index` - The index of the ISDU request
/// * `subindex` - The subindex of the ISDU request
/// * `expected_data` - The expected data of the ISDU request
/// * `expected_length` - The expected length of the ISDU request
///
/// # Returns
/// The ISDU service
///
pub fn util_test_isdu_sequence(
    poll_tx: &Sender<ThreadMessage>,
    poll_response_rx: &Receiver<ThreadMessage>,
    index: u16,
    subindex: Option<u8>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    const OD_LENGTH_PER_FRAME: usize = config::on_req_data::pre_operate::od_length() as usize;

    let isdu_read_request = test_utils::frame_format_utils::isdu_frame::create_isdu_read_request(
        index,
        subindex,
    );
    println!("isdu_read_request: {:?}", isdu_read_request);

    // Write ISDU request to device
    let mut isdu_frames = Vec::new();
    let mut offset = 0;
    while offset < isdu_read_request.len() {
        let end = core::cmp::min(offset + OD_LENGTH_PER_FRAME, isdu_read_request.len());
        let chunk = &isdu_read_request[offset..end];
        let frame =
            test_utils::frame_format_utils::create_preop_write_isdu_request(offset as u8, chunk);
        offset += OD_LENGTH_PER_FRAME;
        let mut isdu_write_response = test_utils::send_test_message_and_wait(
            &poll_tx,
            &poll_response_rx,
            frame,
            Duration::from_millis(1000),
        )
        .expect("ISDU read request failed");
        let is_checksum_valid = test_utils::frame_format_utils::validate_device_frame_checksum(
            &mut isdu_write_response,
        );
        assert!(is_checksum_valid, "Checksum not matching");
        isdu_frames.push(isdu_write_response);
    }
    // Writing ISDU reading details to device is compeleted
    let isdu_start_frame = test_utils::frame_format_utils::create_preop_read_start_isdu_request();
    let mut isdu_read_response = match test_utils::send_test_message_and_wait(
        &poll_tx,
        &poll_response_rx,
        isdu_start_frame,
        Duration::from_millis(4000),
    ) {
        Ok(response) => response,
        Err(e) => {
            println!("Error: {}", e);
            panic!("ISDU transfer failed");
        }
    };
    let is_checksum_valid =
        test_utils::frame_format_utils::validate_device_frame_checksum(&mut isdu_read_response);
    assert!(is_checksum_valid, "Checksum not matching");
    println!("isdu_read_response: {:?}", isdu_read_response[0]);

    // Start reading ISDU data from the device for the written request
    let mut retry_count = 0;
    let mut isdu_data_start = Vec::new();
    loop {
        let isdu_start_frame =
            test_utils::frame_format_utils::create_preop_read_start_isdu_request();
        let mut isdu_read_response = match test_utils::send_test_message_and_wait(
            &poll_tx,
            &poll_response_rx,
            isdu_start_frame,
            Duration::from_millis(4000),
        ) {
            Ok(response) => response,
            Err(e) => {
                println!("Error: {}", e);
                if retry_count > 4 {
                    break;
                }
                retry_count += 1;
                continue;
            }
        };
        let is_checksum_valid =
            test_utils::frame_format_utils::validate_device_frame_checksum(&mut isdu_read_response);
        assert!(is_checksum_valid, "Checksum not matching");
        println!("isdu_read_response: {:?}", isdu_read_response[0]);
        if 0x01 != isdu_read_response[0] || retry_count > 4 {
            isdu_data_start.extend_from_slice(&isdu_read_response[0..OD_LENGTH_PER_FRAME]);
            break;
        }
        retry_count += 1;
    }
    let mut isdu_data = Vec::new();
    let isdu_service: IsduService = IsduService::from_bits(isdu_data_start[0]);
    let isdu_data_rx_length = isdu_service.length() as f32;
    isdu_data.extend_from_slice(&isdu_data_start[1..OD_LENGTH_PER_FRAME]);
    let segments = (isdu_data_rx_length / OD_LENGTH_PER_FRAME as f32).ceil() as usize - 1;
    for segment in 0..segments {
        let frame =
            test_utils::frame_format_utils::create_preop_read_isdu_segment((segment + 1) as u8);
        let mut isdu_read_response = match test_utils::send_test_message_and_wait(
            &poll_tx,
            &poll_response_rx,
            frame,
            Duration::from_millis(4000),
        ) {
            Ok(response) => response,
            Err(e) => {
                println!("Error: {}", e);
                panic!("ISDU transfer failed");
            }
        };
        let is_checksum_valid =
            test_utils::frame_format_utils::validate_device_frame_checksum(&mut isdu_read_response);
        assert!(is_checksum_valid, "Checksum not matching");
        isdu_data.extend_from_slice(&isdu_read_response[0..(OD_LENGTH_PER_FRAME)]);
    }
    Ok(isdu_data)
}

/// Page parameters module

pub mod frame_format_utils {
    use std::sync::{Arc, Mutex};

    use crate::test_utils::{self, ChecksumMsequenceType, ChecksumStatus, ThreadMessage};
    use crate::*;

    /// Extracts and validates checksum from response
    pub fn validate_checksum(response: &[u8], expected_checksum: u8) -> bool {
        if response.len() < 2 {
            return false;
        }

        let received_checksum = test_utils::ChecksumStatus::from(response[1]);
        received_checksum.checksum() == expected_checksum
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
        let cks_calculated_checksum =
            test_utils::calculate_checksum_for_testing(data_len as u8, data);
        let rec_cks = cks.checksum();
        cks_calculated_checksum == rec_cks
    }

    /// Sets up the device with basic configuration for testing
    pub fn setup_device_configuration(io_link_device: &Arc<Mutex<IoLinkDevice>>) {
        let mseq_cap = config::m_seq_capability::m_sequence_capability_parameter();

        let _ = io_link_device
            .lock()
            .unwrap()
            .sm_set_device_mode_req(DeviceMode::Idle);
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

        let checksum =
            test_utils::calculate_checksum_for_testing(rx_buffer.len() as u8, &rx_buffer);
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
            .with_m_seq_type(
                config::m_seq_capability::pre_operate_m_sequence::m_sequence_base_type(),
            )
            .with_checksum(0)
            .build();

        let mc_bits = mc.into_bits();
        let ckt_bits = ckt.into_bits();

        let mut rx_buffer = Vec::new();
        rx_buffer.push(mc_bits);
        rx_buffer.push(ckt_bits);

        let checksum =
            test_utils::calculate_checksum_for_testing(rx_buffer.len() as u8, &rx_buffer);
        ckt.set_checksum(checksum);

        let tx_buffer_1 = rx_buffer.get_mut(1).unwrap();
        *tx_buffer_1 = ckt.into_bits();

        rx_buffer
    }

    /// Creates a read request message for testing
    pub fn create_preop_write_isdu_request(flow_control: u8, buffer: &[u8]) -> Vec<u8> {
        let flow_control = if flow_control == 0 {
            flow_ctrl!(START)
        } else {
            flow_control
        };
        let mc = test_utils::MsequenceControlBuilder::new()
            .with_read_write(RwDirection::Write)
            .with_comm_channel(ComChannel::Isdu)
            .with_address_fctrl(flow_control)
            .build();

        let mut ckt = test_utils::ChecksumMsequenceTypeBuilder::new()
            .with_m_seq_type(
                config::m_seq_capability::pre_operate_m_sequence::m_sequence_base_type(),
            )
            .with_checksum(0)
            .build();

        let mc_bits = mc.into_bits();
        let ckt_bits = ckt.into_bits();

        let mut rx_buffer = Vec::new();
        rx_buffer.push(mc_bits);
        rx_buffer.push(ckt_bits);
        rx_buffer.extend_from_slice(buffer);

        let checksum =
            test_utils::calculate_checksum_for_testing(rx_buffer.len() as u8, &rx_buffer);
        ckt.set_checksum(checksum);

        let tx_buffer_1 = rx_buffer.get_mut(1).unwrap();
        *tx_buffer_1 = ckt.into_bits();

        rx_buffer
    }

    pub fn create_preop_read_start_isdu_request() -> Vec<u8> {
        let flow_control = flow_ctrl!(START);
        let mc = test_utils::MsequenceControlBuilder::new()
            .with_read_write(RwDirection::Read)
            .with_comm_channel(ComChannel::Isdu)
            .with_address_fctrl(flow_control)
            .build();

        let mut ckt = test_utils::ChecksumMsequenceTypeBuilder::new()
            .with_m_seq_type(
                config::m_seq_capability::pre_operate_m_sequence::m_sequence_base_type(),
            )
            .with_checksum(0)
            .build();

        let mc_bits = mc.into_bits();
        let ckt_bits = ckt.into_bits();

        let mut rx_buffer = Vec::new();
        rx_buffer.push(mc_bits);
        rx_buffer.push(ckt_bits);

        let checksum =
            test_utils::calculate_checksum_for_testing(rx_buffer.len() as u8, &rx_buffer);
        ckt.set_checksum(checksum);

        let tx_buffer_1 = rx_buffer.get_mut(1).unwrap();
        *tx_buffer_1 = ckt.into_bits();

        rx_buffer
    }

    /// Creates a read request message for testing
    pub fn create_preop_read_isdu_segment(flow_control: u8) -> Vec<u8> {
        let mc = test_utils::MsequenceControlBuilder::new()
            .with_read_write(RwDirection::Read)
            .with_comm_channel(ComChannel::Isdu)
            .with_address_fctrl(flow_control)
            .build();

        let mut ckt = test_utils::ChecksumMsequenceTypeBuilder::new()
            .with_m_seq_type(
                config::m_seq_capability::pre_operate_m_sequence::m_sequence_base_type(),
            )
            .with_checksum(0)
            .build();

        let mc_bits = mc.into_bits();
        let ckt_bits = ckt.into_bits();

        let mut rx_buffer = Vec::new();
        rx_buffer.push(mc_bits);
        rx_buffer.push(ckt_bits);

        let checksum =
            test_utils::calculate_checksum_for_testing(rx_buffer.len() as u8, &rx_buffer);
        ckt.set_checksum(checksum);

        let tx_buffer_1 = rx_buffer.get_mut(1).unwrap();
        *tx_buffer_1 = ckt.into_bits();

        rx_buffer
    }
    /// Creates a read request message for testing
    pub fn create_preop_read_isdu_idle_request(buffer: &[u8]) -> Vec<u8> {
        let flow_control = flow_ctrl!(IDLE_1);
        let mc = test_utils::MsequenceControlBuilder::new()
            .with_read_write(RwDirection::Write)
            .with_comm_channel(ComChannel::Isdu)
            .with_address_fctrl(flow_control)
            .build();

        let mut ckt = test_utils::ChecksumMsequenceTypeBuilder::new()
            .with_m_seq_type(
                config::m_seq_capability::pre_operate_m_sequence::m_sequence_base_type(),
            )
            .with_checksum(0)
            .build();

        let mc_bits = mc.into_bits();
        let ckt_bits = ckt.into_bits();

        let mut rx_buffer = Vec::new();
        rx_buffer.push(mc_bits);
        rx_buffer.push(ckt_bits);
        rx_buffer.extend_from_slice(buffer);

        let checksum =
            test_utils::calculate_checksum_for_testing(rx_buffer.len() as u8, &rx_buffer);
        ckt.set_checksum(checksum);

        let tx_buffer_1 = rx_buffer.get_mut(1).unwrap();
        *tx_buffer_1 = ckt.into_bits();

        rx_buffer
    }

    /// Creates a read request message for testing
    pub fn create_preop_write_isdu_complete_request() -> Vec<u8> {
        let buffer = &[]; // No OD
        let flow_control = flow_ctrl!(START);
        let mc = test_utils::MsequenceControlBuilder::new()
            .with_read_write(RwDirection::Read)
            .with_comm_channel(ComChannel::Isdu)
            .with_address_fctrl(flow_control)
            .build();

        let mut ckt = test_utils::ChecksumMsequenceTypeBuilder::new()
            .with_m_seq_type(
                config::m_seq_capability::pre_operate_m_sequence::m_sequence_base_type(),
            )
            .with_checksum(0)
            .build();

        let mc_bits = mc.into_bits();
        let ckt_bits = ckt.into_bits();

        let mut rx_buffer = Vec::new();
        rx_buffer.push(mc_bits);
        rx_buffer.push(ckt_bits);
        rx_buffer.extend_from_slice(buffer);

        let checksum =
            test_utils::calculate_checksum_for_testing(rx_buffer.len() as u8, &rx_buffer);
        ckt.set_checksum(checksum);

        let tx_buffer_1 = rx_buffer.get_mut(1).unwrap();
        *tx_buffer_1 = ckt.into_bits();

        rx_buffer
    }

    pub fn create_op_read_request(address: u8) -> Vec<u8> {
        todo!()
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

        let checksum =
            test_utils::calculate_checksum_for_testing(rx_buffer.len() as u8, &rx_buffer);
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
            .with_m_seq_type(
                config::m_seq_capability::pre_operate_m_sequence::m_sequence_base_type(),
            )
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

        let checksum =
            test_utils::calculate_checksum_for_testing(rx_buffer.len() as u8, &rx_buffer);
        ckt.set_checksum(checksum);

        let tx_buffer_1 = rx_buffer.get_mut(1).unwrap();
        *tx_buffer_1 = ckt.into_bits();

        rx_buffer
    }

    pub fn create_op_write_request(address: u8, data: &[u8]) -> Vec<u8> {
        todo!()
    }

    pub mod isdu_frame {
        use crate::{
            isdu_read_request_index_code, isdu_read_request_index_index_subindex_code,
            isdu_read_request_index_subindex_code,
            utils::{self, frame_fromat::isdu::IsduService},
        };

        // 0x0010 0x00
        pub fn create_isdu_read_request(index: u16, sub_index: Option<u8>) -> Vec<u8> {
            // {I-Service(0x9), Length(0x3), Index, CHKPDU} ^
            // {I-Service(0xA), Length(0x4), Index, Subindex, CHKPDU} ^
            // {I-Service(0xB), Length(0x5), Index, Index, Subindex, CHKPDU}
            let index_1 = (index & 0xFF) as u8;
            let index_2 = (index >> 8) as u8;
            let isdu_request_buffer = if index <= 0xFF && sub_index.is_none() {
                let mut isdu_service = IsduService::from_bits(isdu_read_request_index_code!());
                isdu_service.set_length(0x03);
                let mut rx_buffer = Vec::new();
                rx_buffer.push(isdu_service.into_bits());
                rx_buffer.push(index_1);
                rx_buffer.push(0); // CHKPDU
                let checkpdu = utils::frame_fromat::isdu::calculate_checksum(
                    rx_buffer.len() as u8,
                    &rx_buffer,
                );
                rx_buffer.pop();
                rx_buffer.push(checkpdu);
                rx_buffer
            } else if index <= 0xFF && sub_index.is_some() {
                let mut isdu_service =
                    IsduService::from_bits(isdu_read_request_index_subindex_code!());
                isdu_service.set_length(0x04);
                let mut rx_buffer = Vec::new();
                rx_buffer.push(isdu_service.into_bits());
                rx_buffer.push(index_1);
                rx_buffer.push(sub_index.unwrap());
                rx_buffer.push(0); // CHKPDU
                let checkpdu = utils::frame_fromat::isdu::calculate_checksum(
                    rx_buffer.len() as u8,
                    &rx_buffer,
                );
                rx_buffer.pop();
                rx_buffer.push(checkpdu);
                rx_buffer
            } else if index > 0xFF && sub_index.is_some() {
                let mut isdu_service =
                    IsduService::from_bits(isdu_read_request_index_index_subindex_code!());
                isdu_service.set_length(0x05);
                let mut rx_buffer = Vec::new();
                rx_buffer.push(isdu_service.into_bits());
                rx_buffer.push(index_1);
                rx_buffer.push(index_2);
                rx_buffer.push(sub_index.unwrap());
                rx_buffer.push(0); // CHKPDU
                let checkpdu = utils::frame_fromat::isdu::calculate_checksum(
                    rx_buffer.len() as u8,
                    &rx_buffer,
                );
                rx_buffer.pop();
                rx_buffer.push(checkpdu);
                rx_buffer
            } else {
                panic!("Invalid index or subindex");
            };

            isdu_request_buffer
        }
    }
}

pub mod page_params {
    use super::TestDeviceMode;
    use crate::test_utils::{self, ThreadMessage, send_test_message_and_wait};
    use crate::*;
    use std::sync::mpsc::{Receiver, Sender};
    use std::time::Duration;

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
        let (rx_buffer, expected_bytes) = if device_mode == TestDeviceMode::Startup {
            let rx_buffer = super::frame_format_utils::create_startup_read_request(
                direct_parameter_address!(MinCycleTime),
            );
            const EXPECTED_BYTES: u8 = 2;
            (rx_buffer, EXPECTED_BYTES)
        } else {
            let rx_buffer = super::frame_format_utils::create_preop_read_request(
                direct_parameter_address!(MinCycleTime),
            );
            const EXPECTED_BYTES: u8 = config::on_req_data::pre_operate::od_length() + 1 /* CKS Byte */;
            (rx_buffer, EXPECTED_BYTES)
        }; // for read cycle startup and preop builds to the same frame

        // Wait for response from the MockPhysicalLayer
        let mut response_data =
            super::send_test_message_and_wait(poll_tx, poll_response_rx, rx_buffer, TIMEOUT)
                .expect("Failed to get response from device");

        let min_cycle_time = CycleTime::from_bits(*response_data.get(0).unwrap());
        // Validate checksum
        let is_checksum_valid =
            super::frame_format_utils::validate_device_frame_checksum(&mut response_data);
        assert!(
            response_data.len() == expected_bytes as usize,
            "Unexpected response length"
        );
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
            let rx_buffer = super::frame_format_utils::create_startup_read_request(
                direct_parameter_address!(MSequenceCapability),
            );
            const EXPECTED_BYTES: u8 = 2 /* OD + CKS Bytes */;
            (rx_buffer, EXPECTED_BYTES)
        } else {
            let rx_buffer = super::frame_format_utils::create_preop_read_request(
                direct_parameter_address!(MSequenceCapability),
            );
            const EXPECTED_BYTES: u8 = config::on_req_data::pre_operate::od_length() + 1 /* CKS Byte */;
            (rx_buffer, EXPECTED_BYTES)
        };

        let mut response_data =
            send_test_message_and_wait(poll_tx, poll_response_rx, rx_buffer, TIMEOUT)
                .expect("Failed to get response from device");
        let m_sequence_capability = MsequenceCapability::from_bits(*response_data.get(0).unwrap());
        println!("m_sequence_capability: {:?}", m_sequence_capability);
        // Validate checksum
        let is_checksum_valid =
            super::frame_format_utils::validate_device_frame_checksum(&mut response_data);
        assert!(
            response_data.len() == expected_rx_bytes as usize,
            "Unexpected response length"
        );
        assert!(is_checksum_valid, "Checksum not matching");
        m_sequence_capability
    }

    /// Reads the revision id from the device using the provided communication channels.
    pub fn read_revision_id(
        poll_tx: &Sender<ThreadMessage>,
        poll_response_rx: &Receiver<ThreadMessage>,
        device_mode: TestDeviceMode,
    ) -> RevisionId {
        // Create read request for MinCycleTime
        let (rx_buffer, expected_bytes) = if device_mode == TestDeviceMode::Startup {
            let rx_buffer = super::frame_format_utils::create_startup_read_request(
                direct_parameter_address!(RevisionID),
            );
            const EXPECTED_BYTES: u8 = 2 /* OD + CKS Bytes */;
            (rx_buffer, EXPECTED_BYTES)
        } else {
            let rx_buffer = super::frame_format_utils::create_preop_read_request(
                direct_parameter_address!(RevisionID),
            );
            const EXPECTED_BYTES: u8 = config::on_req_data::pre_operate::od_length() + 1 /* CKS Byte */;
            (rx_buffer, EXPECTED_BYTES)
        };

        let mut response_data =
            send_test_message_and_wait(poll_tx, poll_response_rx, rx_buffer, TIMEOUT)
                .expect("Failed to get response from device");
        let revision_id = RevisionId::from(response_data[0]);
        // Validate checksum
        let is_checksum_valid =
            super::frame_format_utils::validate_device_frame_checksum(&mut response_data);
        assert!(
            response_data.len() == expected_bytes as usize,
            "Unexpected response length"
        );
        assert!(is_checksum_valid, "Checksum not matching");
        revision_id
    }

    /// Reads the process data in from the device using the provided communication channels.
    pub fn read_process_data_in(
        poll_tx: &Sender<ThreadMessage>,
        poll_response_rx: &Receiver<ThreadMessage>,
        device_mode: TestDeviceMode,
    ) -> ProcessDataIn {
        // Create read request for MinCycleTime
        let (rx_buffer, expected_bytes) = if device_mode == TestDeviceMode::Startup {
            let rx_buffer = super::frame_format_utils::create_startup_read_request(
                direct_parameter_address!(ProcessDataIn),
            );
            const EXPECTED_BYTES: u8 = 2 /* OD + CKS Bytes */;
            (rx_buffer, EXPECTED_BYTES)
        } else {
            let rx_buffer = super::frame_format_utils::create_preop_read_request(
                direct_parameter_address!(ProcessDataIn),
            );
            const EXPECTED_BYTES: u8 = config::on_req_data::pre_operate::od_length() + 1 /* CKS Byte */;
            (rx_buffer, EXPECTED_BYTES)
        };

        let mut response_data =
            send_test_message_and_wait(poll_tx, poll_response_rx, rx_buffer, TIMEOUT)
                .expect("Failed to get response from device");

        // Extract the response data
        let process_data_in = ProcessDataIn::from(response_data[0]);
        // Validate checksum
        let is_checksum_valid =
            super::frame_format_utils::validate_device_frame_checksum(&mut response_data);
        assert!(
            response_data.len() == expected_bytes as usize,
            "Unexpected response length"
        );
        assert!(is_checksum_valid, "Checksum not matching");
        process_data_in
    }

    /// Reads the process data out from the device using the provided communication channels.
    pub fn read_process_data_out(
        poll_tx: &Sender<ThreadMessage>,
        poll_response_rx: &Receiver<ThreadMessage>,
        device_mode: TestDeviceMode,
    ) -> ProcessDataOut {
        // Create read request for MinCycleTime
        let (rx_buffer, expected_bytes) = if device_mode == TestDeviceMode::Startup {
            let rx_buffer = super::frame_format_utils::create_startup_read_request(
                direct_parameter_address!(ProcessDataOut),
            );
            const EXPECTED_BYTES: u8 = 2 /* OD + CKS Bytes */;
            (rx_buffer, EXPECTED_BYTES)
        } else {
            let rx_buffer = super::frame_format_utils::create_preop_read_request(
                direct_parameter_address!(ProcessDataOut),
            );
            const EXPECTED_BYTES: u8 = config::on_req_data::pre_operate::od_length() + 1 /* CKS Byte */;
            (rx_buffer, EXPECTED_BYTES)
        };

        let mut response_data =
            send_test_message_and_wait(poll_tx, poll_response_rx, rx_buffer, TIMEOUT)
                .expect("Failed to get response from device");

        // Extract the response data
        let process_data_out = ProcessDataOut::from(response_data[0]);
        // Validate checksum
        let is_checksum_valid =
            super::frame_format_utils::validate_device_frame_checksum(&mut response_data);
        assert!(
            response_data.len() == expected_bytes as usize,
            "Unexpected response length"
        );
        assert!(is_checksum_valid, "Checksum not matching");
        process_data_out
    }

    /// Reads the vendor id 1 from the device using the provided communication channels.
    pub fn read_vendor_id_1(
        poll_tx: &Sender<ThreadMessage>,
        poll_response_rx: &Receiver<ThreadMessage>,
        device_mode: TestDeviceMode,
    ) -> u8 {
        // Create read request for MinCycleTime
        let (rx_buffer, expected_bytes) = if device_mode == TestDeviceMode::Startup {
            let rx_buffer = super::frame_format_utils::create_startup_read_request(
                direct_parameter_address!(VendorID1),
            );
            const EXPECTED_BYTES: u8 = 2 /* OD + CKS Bytes */;
            (rx_buffer, EXPECTED_BYTES)
        } else {
            let rx_buffer = super::frame_format_utils::create_preop_read_request(
                direct_parameter_address!(VendorID1),
            );
            const EXPECTED_BYTES: u8 = config::on_req_data::pre_operate::od_length() + 1 /* CKS Byte */;
            (rx_buffer, EXPECTED_BYTES)
        };

        let mut response_data =
            send_test_message_and_wait(poll_tx, poll_response_rx, rx_buffer, TIMEOUT)
                .expect("Failed to get response from device");

        // Extract the response data
        let vendor_id_1 = response_data[0];
        // Validate checksum
        let is_checksum_valid =
            super::frame_format_utils::validate_device_frame_checksum(&mut response_data);
        assert!(
            response_data.len() == expected_bytes as usize,
            "Unexpected response length"
        );
        assert!(is_checksum_valid, "Checksum not matching");
        vendor_id_1
    }

    /// Reads the vendor id 2 from the device using the provided communication channels.
    pub fn read_vendor_id_2(
        poll_tx: &Sender<ThreadMessage>,
        poll_response_rx: &Receiver<ThreadMessage>,
        device_mode: TestDeviceMode,
    ) -> u8 {
        // Create read request for VendorID2
        let (rx_buffer, expected_bytes) = if device_mode == TestDeviceMode::Startup {
            let rx_buffer = super::frame_format_utils::create_startup_read_request(
                direct_parameter_address!(VendorID2),
            );
            const EXPECTED_BYTES: u8 = 2 /* OD + CKS Bytes */;
            (rx_buffer, EXPECTED_BYTES)
        } else {
            let rx_buffer = super::frame_format_utils::create_preop_read_request(
                direct_parameter_address!(VendorID2),
            );
            const EXPECTED_BYTES: u8 = config::on_req_data::pre_operate::od_length() + 1 /* CKS Byte */;
            (rx_buffer, EXPECTED_BYTES)
        };

        let mut response_data =
            send_test_message_and_wait(poll_tx, poll_response_rx, rx_buffer, TIMEOUT)
                .expect("Failed to get response from device");

        // Extract the response data
        let vendor_id_2 = response_data[0];
        // Validate checksum
        let is_checksum_valid =
            super::frame_format_utils::validate_device_frame_checksum(&mut response_data);
        assert!(
            response_data.len() == expected_bytes as usize,
            "Unexpected response length"
        );
        assert!(is_checksum_valid, "Checksum not matching");
        vendor_id_2
    }

    pub fn write_master_command(
        poll_tx: &Sender<ThreadMessage>,
        poll_response_rx: &Receiver<ThreadMessage>,
        device_mode: TestDeviceMode,
        master_command: MasterCommand,
    ) -> bool {
        let (rx_buffer, expected_bytes) = if device_mode == TestDeviceMode::Startup {
            let rx_buffer = super::frame_format_utils::create_startup_write_request(
                direct_parameter_address!(MasterCommand),
                master_command.into(),
            );
            const EXPECTED_BYTES: u8 = 1 /* OD + CKS Bytes */;
            (rx_buffer, EXPECTED_BYTES)
        } else {
            let rx_buffer = super::frame_format_utils::create_preop_write_request(
                direct_parameter_address!(MasterCommand),
                &[master_command.into()],
            );
            const EXPECTED_BYTES: u8 = 1 /* CKS Byte */;
            (rx_buffer, EXPECTED_BYTES)
        };

        let mut response_data =
            send_test_message_and_wait(poll_tx, poll_response_rx, rx_buffer, TIMEOUT)
                .expect("Failed to get response from device");

        // Validate checksum
        let is_checksum_valid =
            super::frame_format_utils::validate_device_frame_checksum(&mut response_data);
        assert!(
            response_data.len() == expected_bytes as usize,
            "Unexpected response length"
        );
        assert!(is_checksum_valid, "Checksum not matching");
        true
    }
}
