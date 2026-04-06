//! Test environment setup and device management utilities
use iolinke_device::IoLinkDevice;
use iolinke_types::custom::IoLinkError;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use core::result::{Result, Result::Err, Result::Ok};
use std::boxed::Box;
use std::vec::Vec;

use crate::mock_physical_layer::{self, MockPhysicalLayer};

use super::mock_app_layer::MockApplicationLayer;
use super::types::ThreadMessage;

/// Spawns the background polling thread that drives the IO-Link device state
/// machines and delivers expired timers.
///
/// ## Loop structure
///
/// Each iteration of the loop:
/// 1. Runs [`IoLinkDevice::poll`] several times so that multi-step
///    state-machine transitions (e.g. GetMessage → CheckMessage →
///    CreateMessage → Idle) and timer-expiry events (delivered inside
///    `poll`) have time to complete before the next inbound master frame.
/// 2. Waits up to 5 µs for an inbound master frame on `usr_to_mock_rx`.
///    If one arrives it is injected via [`mock_physical_layer::transfer_ind`].
///
/// Timer management is **entirely inside `poll()`** — the physical layer's
/// `pl_collect_elapsed_timers()` is called at the start of every poll cycle
/// and the results are routed directly to the DL state machines.  No extra
/// channel plumbing is needed for timers.
pub fn take_care_of_poll(
    io_link_device: Arc<Mutex<IoLinkDevice<MockPhysicalLayer, MockApplicationLayer>>>,
    usr_to_mock_rx: Receiver<ThreadMessage>,
) {
    std::thread::spawn(move || {
        loop {
            // Run several poll cycles so that multi-step state-machine
            // transitions complete before we block waiting for new data.
            // Timer expiry is handled inside poll() itself.
            for _ in 0..9 {
                let mut dev = io_link_device.lock().unwrap();
                match dev.poll() {
                    Ok(()) => {}
                    Err(IoLinkError::NoImplFound) => {}
                    Err(e) => eprintln!("Device error: {:?}", e),
                }
            }
            // Check for an inbound master frame; inject it if one arrived.
            match usr_to_mock_rx.recv_timeout(Duration::from_micros(5)) {
                Ok(ThreadMessage::RxData(data)) => {
                    let mut dev = io_link_device.lock().unwrap();
                    let _ = mock_physical_layer::transfer_ind(&data, &mut dev);
                }
                _ => {}
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
        .al_set_input_req(3, &[0x01, 0x02, 0x03]);

    // Start the polling thread — timer delivery is handled inside poll().
    let io_link_device_clone_poll = Arc::clone(&io_link_device);
    take_care_of_poll(io_link_device_clone_poll, poll_rx);

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
