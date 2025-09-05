//! Test sequences for IO-Link device testing
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use iolinke_derived_config::device as derived_config;
use iolinke_device::{CycleTime, MsequenceCapability, ProcessDataIn, ProcessDataOut, RevisionId};
use iolinke_types::frame::isdu::{IsduFlowCtrl, IsduIServiceCode, IsduService};
use iolinke_types::page::page1::MasterCommand;

use crate::{TestDeviceMode, ThreadMessage, frame_utils, page_params, send_test_message_and_wait};

pub fn util_test_startup_sequence(
    poll_tx: &Sender<ThreadMessage>,
    poll_response_rx: &Receiver<ThreadMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read min cycle time
    let min_cycle_time =
        page_params::read_min_cycle_time(&poll_tx, &poll_response_rx, TestDeviceMode::Startup);
    const CONFIG_MIN_CYCLE_TIME: CycleTime =
        derived_config::timings::min_cycle_time::min_cycle_time_parameter();
    assert!(
        CONFIG_MIN_CYCLE_TIME.time_base() == min_cycle_time.time_base(),
        "Min cycle time time_base not matching"
    );
    assert!(
        CONFIG_MIN_CYCLE_TIME.multiplier() == min_cycle_time.multiplier(),
        "Min cycle time multiplier not matching"
    );

    // Read m-sequence capability
    let m_sequence_capability = page_params::read_m_sequence_capability(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Startup,
    );
    const CONFIG_M_SEQUENCE_CAPABILITY: MsequenceCapability =
        derived_config::m_seq_capability::m_sequence_capability_parameter();
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
    let revision_id =
        page_params::read_revision_id(&poll_tx, &poll_response_rx, TestDeviceMode::Startup);
    const CONFIG_REVISION_ID: RevisionId =
        derived_config::vendor_specifics::revision_id_parameter();
    assert!(
        CONFIG_REVISION_ID.major_rev() == revision_id.major_rev(),
        "RevisionID major rev is not matching"
    );
    assert!(
        CONFIG_REVISION_ID.minor_rev() == revision_id.minor_rev(),
        "RevisionID minor rev is not matching"
    );

    // Read process data in
    let process_data_in =
        page_params::read_process_data_in(&poll_tx, &poll_response_rx, TestDeviceMode::Startup);
    const CONFIG_PROCESS_DATA_IN: ProcessDataIn =
        derived_config::process_data::pd_in::pd_in_parameter();
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
    let process_data_out =
        page_params::read_process_data_out(&poll_tx, &poll_response_rx, TestDeviceMode::Startup);
    const CONFIG_PROCESS_DATA_OUT: ProcessDataOut =
        derived_config::process_data::pd_out::pd_out_parameter();
    assert!(
        CONFIG_PROCESS_DATA_OUT.byte() == process_data_out.byte(),
        "ProcessDataOut byte is not matching"
    );
    assert!(
        CONFIG_PROCESS_DATA_OUT.length() == process_data_out.length(),
        "ProcessDataOut length is not matching"
    );

    // Read vendor id
    let vendor_id_1 =
        page_params::read_vendor_id_1(&poll_tx, &poll_response_rx, TestDeviceMode::Startup);
    const CONFIG_VENDOR_ID_1: u8 = derived_config::vendor_specifics::vendor_id_1();
    assert!(
        CONFIG_VENDOR_ID_1 == vendor_id_1,
        "VendorID1 is not matching"
    );

    // Read vendor id
    let vendor_id_2 =
        page_params::read_vendor_id_2(&poll_tx, &poll_response_rx, TestDeviceMode::Startup);
    const CONFIG_VENDOR_ID_2: u8 = derived_config::vendor_specifics::vendor_id_2();
    assert!(
        CONFIG_VENDOR_ID_2 == vendor_id_2,
        "VendorID2 is not matching"
    );

    // Assuming startup sequence is successful, because all the configure
    // parameters page1 (0x02 to 0x06) are read and checked
    // Now we can command the device to Pre operate mode
    let is_master_ident_written = page_params::write_master_command(
        poll_tx,
        poll_response_rx,
        TestDeviceMode::Startup,
        MasterCommand::MasterIdent,
    );
    assert!(is_master_ident_written, "Failed to write master ident");

    Ok(())
}

pub fn util_test_change_operation_mode(
    poll_tx: &Sender<ThreadMessage>,
    poll_response_rx: &Receiver<ThreadMessage>,
    current_mode: TestDeviceMode,
    target_mode: TestDeviceMode,
) -> Result<(), Box<dyn std::error::Error>> {
    let master_command = match target_mode {
        TestDeviceMode::Preoperate => MasterCommand::DevicePreOperate,
        TestDeviceMode::Operate => MasterCommand::DeviceOperate,
        _ => panic!("Invalid target mode"),
    };
    let _ =
        page_params::write_master_command(poll_tx, poll_response_rx, current_mode, master_command);

    Ok(())
}

pub fn util_test_preop_sequence(
    poll_tx: &Sender<ThreadMessage>,
    poll_response_rx: &Receiver<ThreadMessage>,
) -> Result<MsequenceCapability, Box<dyn std::error::Error>> {
    // Read m-sequence capability
    let m_sequence_capability = page_params::read_m_sequence_capability(
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
pub fn util_test_isdu_sequence_read(
    poll_tx: &Sender<ThreadMessage>,
    poll_response_rx: &Receiver<ThreadMessage>,
    index: u16,
    subindex: Option<u8>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    const OD_LENGTH_PER_FRAME: usize =
        derived_config::on_req_data::pre_operate::od_length() as usize;

    let isdu_read_request = frame_utils::isdu_frame::create_isdu_read_request(index, subindex);
    println!("isdu_read_request: {:?}", isdu_read_request);

    // Write ISDU request to device
    let mut isdu_frames = Vec::new();
    let mut offset = 0;
    let mut circular_offset = IsduFlowCtrl::Start;
    while offset < isdu_read_request.len() {
        let end = core::cmp::min(offset + OD_LENGTH_PER_FRAME, isdu_read_request.len());
        let mut chunk_buf = [0u8; OD_LENGTH_PER_FRAME];
        let chunk_len = end - offset;
        chunk_buf[..chunk_len].copy_from_slice(&isdu_read_request[offset..end]);
        let chunk = &chunk_buf[..];
        let frame =
            frame_utils::create_preop_write_isdu_request(circular_offset.into_bits(), chunk);
        offset += OD_LENGTH_PER_FRAME;
        circular_offset = IsduFlowCtrl::Count(circular_offset.into_bits() + 1);
        if circular_offset.into_bits() > 15 {
            circular_offset = IsduFlowCtrl::Count(0);
        }
        let mut isdu_write_response = match send_test_message_and_wait(
            &poll_tx,
            &poll_response_rx,
            frame,
            Duration::from_millis(1000),
        ) {
            Ok(response) => response,
            Err(e) => {
                println!("Error: {}", e);
                panic!("ISDU transfer failed");
            }
        };
        let is_checksum_valid =
            frame_utils::validate_device_frame_checksum(&mut isdu_write_response);
        assert!(is_checksum_valid, "Checksum not matching");
        isdu_frames.push(isdu_write_response);
    }
    // Writing ISDU reading details to device is compeleted

    // Start reading ISDU data from the device for the written request
    let mut retry_count = 0;
    let mut isdu_data_start = Vec::new();
    loop {
        let isdu_start_frame = frame_utils::create_preop_read_start_isdu_request();
        let mut isdu_read_response = match send_test_message_and_wait(
            &poll_tx,
            &poll_response_rx,
            isdu_start_frame,
            Duration::from_millis(4000),
        ) {
            Ok(response) => response,
            Err(e) => {
                println!("Error: {}", e);
                if retry_count > 4 {
                    panic!("ISDU transfer failed, Could not get ISDU start frame");
                }
                retry_count += 1;
                continue;
            }
        };
        let is_checksum_valid =
            frame_utils::validate_device_frame_checksum(&mut isdu_read_response);
        assert!(is_checksum_valid, "Checksum not matching");
        println!("isdu_read_response: {:?}", isdu_read_response[0]);
        if 0x00 == isdu_read_response[0] {
            panic!("ISDU transfer failed, ISDU Aborted by device");
        }
        if retry_count > 4 {
            panic!("ISDU transfer failed, Could not get ISDU start frame response, out of retries");
        }
        if 0x01 != isdu_read_response[0] && 0x00 != isdu_read_response[0] {
            isdu_data_start.extend_from_slice(&isdu_read_response[0..OD_LENGTH_PER_FRAME]);
            break;
        }
        if 0x01 == isdu_read_response[0] {
            std::thread::sleep(std::time::Duration::from_millis(500));
            retry_count += 1;
        }
    }
    let mut isdu_data = Vec::new();
    let isdu_service: IsduService = IsduService::from_bits(isdu_data_start[0]);
    let od_len;
    let mut isdu_data_rx_length = isdu_service.length() as f32;
    isdu_data_rx_length = if isdu_data_rx_length == 1.0 {
        od_len = isdu_data_start[1] - 3; // i_service(1) + extended length(1) + checksum(1) = 3
        isdu_data_start[1] as f32
    } else {
        od_len = (isdu_data_rx_length - 2.0) as u8; // i_service(1) + checksum(1) = 2
        isdu_data.extend_from_slice(&isdu_data_start[1..OD_LENGTH_PER_FRAME]);
        isdu_data_rx_length
    };
    let segments = (isdu_data_rx_length / OD_LENGTH_PER_FRAME as f32).ceil() as usize;
    for segment in 1..segments {
        // Revolve segment from 0 to 15, wrapping around if segment > 15
        let segment_num = (segment % 16) as u8;
        let frame = frame_utils::create_preop_read_isdu_segment(segment_num);
        let mut isdu_read_response = match send_test_message_and_wait(
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
            frame_utils::validate_device_frame_checksum(&mut isdu_read_response);
        assert!(is_checksum_valid, "Checksum not matching");
        isdu_data.extend_from_slice(&isdu_read_response[0..(OD_LENGTH_PER_FRAME)]);
    }
    let isdu_idle_frame = frame_utils::create_preop_isdu_idle_request();
    let mut isdu_idle_response = match send_test_message_and_wait(
        &poll_tx,
        &poll_response_rx,
        isdu_idle_frame,
        Duration::from_millis(4000),
    ) {
        Ok(response) => response,
        Err(e) => {
            println!("Error: {}", e);
            panic!("ISDU transfer failed");
        }
    };
    let is_checksum_valid = frame_utils::validate_device_frame_checksum(&mut isdu_idle_response);
    assert!(is_checksum_valid, "Checksum not matching");
    println!("isdu_idle_response: {:?}", isdu_idle_response[0]);
    Ok(isdu_data[0..od_len as usize].to_vec())
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
pub fn util_test_isdu_sequence_write(
    poll_tx: &Sender<ThreadMessage>,
    poll_response_rx: &Receiver<ThreadMessage>,
    index: u16,
    subindex: Option<u8>,
    data: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    const OD_LENGTH_PER_FRAME: usize =
        derived_config::on_req_data::pre_operate::od_length() as usize;

    let isdu_write_request =
        frame_utils::isdu_frame::create_isdu_write_request(index, subindex, data);
    println!("isdu_write_request: {:?}", isdu_write_request);

    // Write ISDU request to device
    let mut isdu_frames = Vec::new();
    let mut offset = 0;
    let mut circular_offset = IsduFlowCtrl::Start;
    while offset < isdu_write_request.len() {
        let end = core::cmp::min(offset + OD_LENGTH_PER_FRAME, isdu_write_request.len());
        let mut chunk_buf = [0u8; OD_LENGTH_PER_FRAME];
        let chunk_len = end - offset;
        chunk_buf[..chunk_len].copy_from_slice(&isdu_write_request[offset..end]);
        let chunk = &chunk_buf[..];
        let frame =
            frame_utils::create_preop_write_isdu_request(circular_offset.into_bits(), chunk);
        offset += OD_LENGTH_PER_FRAME;
        // circular_offset wraps around 0..=15
        circular_offset = IsduFlowCtrl::Count(circular_offset.into_bits() + 1);
        if circular_offset.into_bits() > 15 {
            circular_offset = IsduFlowCtrl::Count(0);
        }
        let mut isdu_write_response = match send_test_message_and_wait(
            &poll_tx,
            &poll_response_rx,
            frame,
            Duration::from_millis(1000),
        ) {
            Ok(response) => response,
            Err(e) => {
                println!("Error: {}", e);
                panic!("ISDU transfer failed");
            }
        };
        let is_checksum_valid =
            frame_utils::validate_device_frame_checksum(&mut isdu_write_response);
        assert!(is_checksum_valid, "Checksum not matching");
        isdu_frames.push(isdu_write_response);
    }

    // Start reading ISDU data from the device for the written request
    let mut retry_count = 0;
    let mut isdu_data_start = Vec::new();
    loop {
        let isdu_start_frame = frame_utils::create_preop_read_start_isdu_request();
        let mut isdu_read_response = match send_test_message_and_wait(
            &poll_tx,
            &poll_response_rx,
            isdu_start_frame,
            Duration::from_millis(4000),
        ) {
            Ok(response) => response,
            Err(e) => {
                println!("Error: {}", e);
                retry_count += 1;
                if retry_count > 4 {
                    panic!(
                        "ISDU transfer failed, Could not get ISDU start frame response, out of retries"
                    );
                }
                continue;
            }
        };
        let is_checksum_valid =
            frame_utils::validate_device_frame_checksum(&mut isdu_read_response);
        assert!(is_checksum_valid, "Checksum not matching");
        println!("isdu_read_response: {:?}", isdu_read_response[0]);
        if 0x00 == isdu_read_response[0] {
            panic!("ISDU transfer failed, ISDU Aborted by device");
        }
        if 0x01 != isdu_read_response[0] && 0x00 != isdu_read_response[0] {
            isdu_data_start.extend_from_slice(&isdu_read_response[0..OD_LENGTH_PER_FRAME]);
            break;
        }
        retry_count += 1;
    }
    let mut isdu_data = Vec::new();
    let isdu_service: IsduService = IsduService::from_bits(isdu_data_start[0]);
    let isdu_data_rx_length = isdu_service.length();
    isdu_data.extend_from_slice(&isdu_data_start[1..OD_LENGTH_PER_FRAME]);
    assert!(isdu_data_rx_length == 2, "ISDU data write failed");
    assert!(
        isdu_service.i_service() == IsduIServiceCode::WriteSuccess,
        "ISDU data write failed"
    );

    // Send ISDU idle request to device to complete the write operation
    let isdu_idle_frame = frame_utils::create_preop_isdu_idle_request();
    let mut isdu_idle_response = match send_test_message_and_wait(
        &poll_tx,
        &poll_response_rx,
        isdu_idle_frame,
        Duration::from_millis(4000),
    ) {
        Ok(response) => response,
        Err(e) => {
            println!("Error: {}", e);
            panic!("ISDU transfer failed");
        }
    };
    let is_checksum_valid = frame_utils::validate_device_frame_checksum(&mut isdu_idle_response);
    assert!(is_checksum_valid, "Checksum not matching");
    println!("isdu_idle_response: {:?}", isdu_idle_response[0]);
    Ok(())
}
