use iolinke_device::{
    MasterCommand,
    test_utils::{self, TestDeviceMode, ThreadMessage},
};
use std::sync::mpsc::{Receiver, Sender};

mod preop_tests {
    use std::time::Duration;

    use iolinke_device::{config, test_utils::IsduService, DeviceParametersIndex, SubIndex};

    use super::*;

    #[test]
    fn test_write_master_ident() {
        let (_io_link_device, poll_tx, poll_response_rx) = test_utils::setup_test_environment();
        let result = test_utils::util_test_startup_sequence(&poll_tx, &poll_response_rx);
        assert!(result.is_ok(), "Test startup sequence failed");

        let is_master_ident_written = test_utils::page_params::write_master_command(
            &poll_tx,
            &poll_response_rx,
            TestDeviceMode::Startup,
            MasterCommand::MasterIdent,
        );
        assert!(is_master_ident_written, "Failed to write master ident");
    }

    #[test]
    fn test_write_master_pre_operate() {
        let (_io_link_device, poll_tx, poll_response_rx) = test_utils::setup_test_environment();
        let result = test_utils::util_test_startup_sequence(&poll_tx, &poll_response_rx);
        assert!(result.is_ok(), "Test startup sequence failed");

        let is_master_pre_op_written = test_utils::page_params::write_master_command(
            &poll_tx,
            &poll_response_rx,
            TestDeviceMode::Startup,
            MasterCommand::DevicePreOperate,
        );
        assert!(is_master_pre_op_written, "Failed to write master pre op");
    }

    #[test]
    fn test_read_isdu_read_vendor_name() {
        // Set up test environment
        let (_io_link_device, poll_tx, poll_response_rx) = test_utils::setup_test_environment();

        // Test startup sequence is successful and device is in startup mode
        let result = test_utils::util_test_startup_sequence(&poll_tx, &poll_response_rx);
        assert!(result.is_ok(), "Test startup sequence failed");

        // Startup is successfull now command device to preop mode
        let result = test_utils::util_test_preop_sequence(&poll_tx, &poll_response_rx);
        assert!(result.is_ok(), "Test preop sequence failed");
        let m_sequence_capability = result.unwrap();

        if m_sequence_capability.isdu() {
            const OD_LENGTH_PER_FRAME: usize =
                config::on_req_data::pre_operate::od_length() as usize;
            // Vendor Name index 0x10, subindex 0x00
            let venodor_name_index = DeviceParametersIndex::VendorName.index();
            let vendor_name_subindex =
                DeviceParametersIndex::VendorName.subindex(SubIndex::VendorName);
            let venodor_name = config::vendor_specifics::vendor_name();
            let venodor_name_length = config::vendor_specifics::vendor_name_length();

            let isdu_read_request =
                test_utils::frame_format_utils::isdu_frame::create_isdu_read_request(
                    venodor_name_index,
                    Some(vendor_name_subindex),
                );
            println!("isdu_read_request: {:?}", isdu_read_request);

            // Write ISDU request to device
            let mut isdu_frames = Vec::new();
            let mut offset = 0;
            while offset < isdu_read_request.len() {
                let end = core::cmp::min(offset + OD_LENGTH_PER_FRAME, isdu_read_request.len());
                let chunk = &isdu_read_request[offset..end];
                let frame = test_utils::frame_format_utils::create_preop_write_isdu_request(
                    offset as u8,
                    chunk,
                );
                offset += OD_LENGTH_PER_FRAME;
                let mut isdu_write_response = test_utils::send_test_message_and_wait(
                    &poll_tx,
                    &poll_response_rx,
                    frame,
                    Duration::from_millis(1000),
                )
                .expect("ISDU read request failed");
                let is_checksum_valid =
                    test_utils::frame_format_utils::validate_device_frame_checksum(
                        &mut isdu_write_response,
                    );
                assert!(is_checksum_valid, "Checksum not matching");
                isdu_frames.push(isdu_write_response);
            }
            // Writing ISDU reading details to device is compeleted
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
                    panic!("ISDU transfer failed");
                }
            };
            let is_checksum_valid = test_utils::frame_format_utils::validate_device_frame_checksum(
                &mut isdu_read_response,
            );
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
                    test_utils::frame_format_utils::validate_device_frame_checksum(
                        &mut isdu_read_response,
                    );
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
                let frame = test_utils::frame_format_utils::create_preop_read_isdu_segment(
                    (segment + 1) as u8,
                );
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
                    test_utils::frame_format_utils::validate_device_frame_checksum(
                        &mut isdu_read_response,
                    );
                assert!(is_checksum_valid, "Checksum not matching");
                isdu_data.extend_from_slice(&isdu_read_response[0..(OD_LENGTH_PER_FRAME)]);
            }
            assert!(
                venodor_name_length == (isdu_data_rx_length - 1.0) as u8,
                "ISDU data length not matching"
            );
            assert!(venodor_name.as_bytes() == isdu_data, "ISDU data not matching");
        }
    }
}
