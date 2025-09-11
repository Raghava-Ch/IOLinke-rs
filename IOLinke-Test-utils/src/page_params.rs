//! Page parameter reading and writing utilities for testing

use super::TestDeviceMode;
use super::frame_utils;
use super::test_environment::send_test_message_and_wait;
use super::types::ThreadMessage;
use iolinke_derived_config::device as derived_config;
use iolinke_device::{
    CycleTime, MsequenceCapability, ProcessDataIn, ProcessDataOut, RevisionId,
    direct_parameter_address,
};
use iolinke_types::page::page1::MasterCommand;
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
        let rx_buffer =
            frame_utils::create_startup_read_request(direct_parameter_address!(MinCycleTime));
        const EXPECTED_BYTES: u8 = 2;
        (rx_buffer, EXPECTED_BYTES)
    } else {
        let rx_buffer =
            frame_utils::create_preop_read_request(direct_parameter_address!(MinCycleTime));
        const EXPECTED_BYTES: u8 = derived_config::on_req_data::pre_operate::od_length() + 1 /* CKS Byte */;
        (rx_buffer, EXPECTED_BYTES)
    }; // for read cycle startup and preop builds to the same frame

    // Wait for response from the MockPhysicalLayer
    let mut response_data =
        send_test_message_and_wait(poll_tx, poll_response_rx, rx_buffer, TIMEOUT)
            .expect("Failed to get response from device");

    let min_cycle_time = CycleTime::from_bits(*response_data.get(0).unwrap());
    // Validate checksum
    let is_checksum_valid = frame_utils::validate_device_frame_checksum(&mut response_data);
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
        let rx_buffer = frame_utils::create_startup_read_request(direct_parameter_address!(
            MSequenceCapability
        ));
        const EXPECTED_BYTES: u8 = 1 + 1 /* OD + CKS Bytes */;
        (rx_buffer, EXPECTED_BYTES)
    } else if device_mode == TestDeviceMode::Preoperate {
        let rx_buffer =
            frame_utils::create_preop_read_request(direct_parameter_address!(MSequenceCapability));
        const EXPECTED_BYTES: u8 = derived_config::on_req_data::pre_operate::od_length() + 1 /* CKS Byte */;
        (rx_buffer, EXPECTED_BYTES)
    } else {
        let rx_buffer =
            frame_utils::create_op_read_request(direct_parameter_address!(MSequenceCapability));
        const EXPECTED_BYTES: u8 = derived_config::on_req_data::operate::od_length() + derived_config::process_data::pd_in::config_length_in_bytes() + 1 /* CKS Byte */;
        (rx_buffer, EXPECTED_BYTES)
    };

    let mut response_data =
        send_test_message_and_wait(poll_tx, poll_response_rx, rx_buffer, TIMEOUT)
            .expect("Failed to get response from device");
    let m_sequence_capability = MsequenceCapability::from_bits(*response_data.get(0).unwrap());
    println!("m_sequence_capability: {:?}", m_sequence_capability);
    // Validate checksum
    let is_checksum_valid = frame_utils::validate_device_frame_checksum(&mut response_data);
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
        let rx_buffer =
            frame_utils::create_startup_read_request(direct_parameter_address!(RevisionID));
        const EXPECTED_BYTES: u8 = 2 /* OD + CKS Bytes */;
        (rx_buffer, EXPECTED_BYTES)
    } else {
        let rx_buffer =
            frame_utils::create_preop_read_request(direct_parameter_address!(RevisionID));
        const EXPECTED_BYTES: u8 = derived_config::on_req_data::pre_operate::od_length() + 1 /* CKS Byte */;
        (rx_buffer, EXPECTED_BYTES)
    };

    let mut response_data =
        send_test_message_and_wait(poll_tx, poll_response_rx, rx_buffer, TIMEOUT)
            .expect("Failed to get response from device");
    let revision_id = RevisionId::from(response_data[0]);
    // Validate checksum
    let is_checksum_valid = frame_utils::validate_device_frame_checksum(&mut response_data);
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
        let rx_buffer =
            frame_utils::create_startup_read_request(direct_parameter_address!(ProcessDataIn));
        const EXPECTED_BYTES: u8 = 2 /* OD + CKS Bytes */;
        (rx_buffer, EXPECTED_BYTES)
    } else {
        let rx_buffer =
            frame_utils::create_preop_read_request(direct_parameter_address!(ProcessDataIn));
        const EXPECTED_BYTES: u8 = derived_config::on_req_data::pre_operate::od_length() + 1 /* CKS Byte */;
        (rx_buffer, EXPECTED_BYTES)
    };

    let mut response_data =
        send_test_message_and_wait(poll_tx, poll_response_rx, rx_buffer, TIMEOUT)
            .expect("Failed to get response from device");

    // Extract the response data
    let process_data_in = ProcessDataIn::from(response_data[0]);
    // Validate checksum
    let is_checksum_valid = frame_utils::validate_device_frame_checksum(&mut response_data);
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
        let rx_buffer =
            frame_utils::create_startup_read_request(direct_parameter_address!(ProcessDataOut));
        const EXPECTED_BYTES: u8 = 2 /* OD + CKS Bytes */;
        (rx_buffer, EXPECTED_BYTES)
    } else {
        let rx_buffer =
            frame_utils::create_preop_read_request(direct_parameter_address!(ProcessDataOut));
        const EXPECTED_BYTES: u8 = derived_config::on_req_data::pre_operate::od_length() + 1 /* CKS Byte */;
        (rx_buffer, EXPECTED_BYTES)
    };

    let mut response_data =
        send_test_message_and_wait(poll_tx, poll_response_rx, rx_buffer, TIMEOUT)
            .expect("Failed to get response from device");

    // Extract the response data
    let process_data_out = ProcessDataOut::from(response_data[0]);
    // Validate checksum
    let is_checksum_valid = frame_utils::validate_device_frame_checksum(&mut response_data);
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
        let rx_buffer =
            frame_utils::create_startup_read_request(direct_parameter_address!(VendorID1));
        const EXPECTED_BYTES: u8 = 2 /* OD + CKS Bytes */;
        (rx_buffer, EXPECTED_BYTES)
    } else {
        let rx_buffer =
            frame_utils::create_preop_read_request(direct_parameter_address!(VendorID1));
        const EXPECTED_BYTES: u8 = derived_config::on_req_data::pre_operate::od_length() + 1 /* CKS Byte */;
        (rx_buffer, EXPECTED_BYTES)
    };

    let mut response_data =
        send_test_message_and_wait(poll_tx, poll_response_rx, rx_buffer, TIMEOUT)
            .expect("Failed to get response from device");

    // Extract the response data
    let vendor_id_1 = response_data[0];
    // Validate checksum
    let is_checksum_valid = frame_utils::validate_device_frame_checksum(&mut response_data);
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
        let rx_buffer =
            frame_utils::create_startup_read_request(direct_parameter_address!(VendorID2));
        const EXPECTED_BYTES: u8 = 2 /* OD + CKS Bytes */;
        (rx_buffer, EXPECTED_BYTES)
    } else {
        let rx_buffer =
            frame_utils::create_preop_read_request(direct_parameter_address!(VendorID2));
        const EXPECTED_BYTES: u8 = derived_config::on_req_data::pre_operate::od_length() + 1 /* CKS Byte */;
        (rx_buffer, EXPECTED_BYTES)
    };

    let mut response_data =
        send_test_message_and_wait(poll_tx, poll_response_rx, rx_buffer, TIMEOUT)
            .expect("Failed to get response from device");

    // Extract the response data
    let vendor_id_2 = response_data[0];
    // Validate checksum
    let is_checksum_valid = frame_utils::validate_device_frame_checksum(&mut response_data);
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
        let rx_buffer = frame_utils::create_startup_write_request(
            direct_parameter_address!(MasterCommand),
            master_command.into(),
        );
        const EXPECTED_BYTES: u8 = 1 /* CKS Bytes */;
        (rx_buffer, EXPECTED_BYTES)
    } else if device_mode == TestDeviceMode::Preoperate {
        let rx_buffer = frame_utils::create_preop_write_request(
            direct_parameter_address!(MasterCommand),
            &[master_command.into()],
        );
        const EXPECTED_BYTES: u8 = 1 /* CKS Byte */;
        (rx_buffer, EXPECTED_BYTES)
    } else {
        const PD_LENGTH_BYTES: u8 = derived_config::process_data::pd_in::config_length_in_bytes();
        let rx_buffer = frame_utils::create_op_write_request(
            direct_parameter_address!(MasterCommand),
            &[master_command.into()],
        );
        const EXPECTED_BYTES: u8 = 1 + PD_LENGTH_BYTES; /* CKS Byte + PD Length Bytes */
        (rx_buffer, EXPECTED_BYTES)
    };

    let mut response_data =
        send_test_message_and_wait(poll_tx, poll_response_rx, rx_buffer, TIMEOUT)
            .expect("Failed to get response from device");

    // Validate checksum
    let is_checksum_valid = frame_utils::validate_device_frame_checksum(&mut response_data);
    assert!(
        response_data.len() == expected_bytes as usize,
        "Unexpected response length"
    );
    assert!(is_checksum_valid, "Checksum not matching");
    true
}
