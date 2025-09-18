//! Test environment setup and device management utilities
use iolinke_device::IoLinkDevice;
use iolinke_types::custom::IoLinkError;
use std::io;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::mock_physical_layer::{self, MockPhysicalLayer};

use super::mock_app_layer::MockApplicationLayer;
use super::types::ThreadMessage;

/// This function is used to poll the device and check for expired timers.
pub fn take_care_of_poll(
    io_link_device: Arc<Mutex<IoLinkDevice<MockPhysicalLayer, MockApplicationLayer>>>,
    usr_to_mock_rx: Receiver<ThreadMessage>,
    mock_to_usr_tx: Sender<ThreadMessage>,
) {
    // Main device loop
    std::thread::spawn(move || {
        // Check for expired timers before polling
        // let mut physical_layer: MockPhysicalLayer = MockPhysicalLayer::new(mock_to_usr_tx);
        // Poll all protocol layers
        loop {
            for _ in 0..9 {
                let mut io_link_device_lock = io_link_device.lock().unwrap();
                match io_link_device_lock.poll() {
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
                            let mut io_link_device_clone_lock =
                                io_link_device_clone.lock().unwrap();
                            let _ = mock_physical_layer::transfer_ind(
                                &data,
                                &mut io_link_device_clone_lock,
                            );
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
    Arc<Mutex<IoLinkDevice<MockPhysicalLayer, MockApplicationLayer>>>,
    Sender<ThreadMessage>,
    Receiver<ThreadMessage>,
) {
    let (usr_to_mock_tx, _usr_to_mock_rx) = mpsc::channel();
    let (_mock_to_usr_tx, mock_to_usr_rx): (Sender<ThreadMessage>, Receiver<ThreadMessage>) =
        mpsc::channel();
    let io_link_device: Arc<Mutex<IoLinkDevice<MockPhysicalLayer, MockApplicationLayer>>> =
        Arc::new(Mutex::new(IoLinkDevice::new(
            MockPhysicalLayer::new(usr_to_mock_tx.clone()),
            MockApplicationLayer::new(),
        )));

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
pub fn setup_test_environment() -> (Sender<ThreadMessage>, Receiver<ThreadMessage>) {
    // Create test device and channels
    // let (io_link_device, _usr_to_mock_tx, _mock_to_usr_rx) = create_test_device();

    // Create separate channels for the polling thread
    let (poll_tx, poll_rx) = mpsc::channel();
    let (poll_response_tx, poll_response_rx) = mpsc::channel();
    let io_link_device: Arc<Mutex<IoLinkDevice<MockPhysicalLayer, MockApplicationLayer>>> =
        Arc::new(Mutex::new(IoLinkDevice::new(
            MockPhysicalLayer::new(poll_response_tx.clone()),
            MockApplicationLayer::new(),
        )));

    let _ = io_link_device
        .lock()
        .unwrap()
        .al_pd_input_update_req(3, &[0x01, 0x02, 0x03]);
    // Start the polling thread
    let io_link_device_clone_poll = Arc::clone(&io_link_device);
    take_care_of_poll(io_link_device_clone_poll, poll_rx, poll_response_tx);

    // Perform startup sequence
    let io_link_device_clone = Arc::clone(&io_link_device);
    startup_routine(io_link_device_clone);

    (poll_tx, poll_response_rx)
}

/// Complete startup routine that combines configuration and startup sequence
pub fn startup_routine(
    io_link_device: Arc<Mutex<IoLinkDevice<MockPhysicalLayer, MockApplicationLayer>>>,
) {
    super::frame_utils::setup_device_configuration(&io_link_device);
    super::frame_utils::perform_startup_sequence(&io_link_device);
}
