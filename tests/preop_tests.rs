use iolinke_device::{
    MasterCommand,
    test_utils::{self, TestDeviceMode, ThreadMessage},
};
use std::sync::mpsc::{Receiver, Sender};

mod preop_tests {
    use std::{result, time::Duration};

    use iolinke_device::{DeviceParametersIndex, SubIndex, config, test_utils::IsduService};

    use super::*;

    #[test]
    fn test_write_master_ident() {
        let (_io_link_device, poll_tx, poll_response_rx) = test_utils::setup_test_environment();
        let result = test_utils::util_test_startup_sequence(&poll_tx, &poll_response_rx);
        assert!(result.is_ok(), "Test startup sequence failed");
        let result = test_utils::util_test_change_operation_mode(
            &poll_tx,
            &poll_response_rx,
            TestDeviceMode::Startup,
            TestDeviceMode::Preoperate,
        );
        assert!(result.is_ok(), "Test change operation mode failed");

        let is_master_ident_written = test_utils::page_params::write_master_command(
            &poll_tx,
            &poll_response_rx,
            TestDeviceMode::Preoperate,
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
        let result = test_utils::util_test_change_operation_mode(
            &poll_tx,
            &poll_response_rx,
            TestDeviceMode::Startup,
            TestDeviceMode::Preoperate,
        );
        assert!(result.is_ok(), "Test change operation mode failed");

        // Startup is successfull now command device to preop mode
        let result = test_utils::util_test_preop_sequence(&poll_tx, &poll_response_rx);
        assert!(result.is_ok(), "Test preop sequence failed");
        let m_sequence_capability = result.unwrap();

        if m_sequence_capability.isdu() {
            // Vendor Name index 0x10, subindex 0x00
            let venodor_name_index = DeviceParametersIndex::VendorName.index();
            let vendor_name_subindex =
                DeviceParametersIndex::VendorName.subindex(SubIndex::VendorName);
            const VENODOR_NAME: &'static str = config::vendor_specifics::vendor_name();
            const VENODOR_NAME_LENGTH: u8 = config::vendor_specifics::vendor_name_length();
            let result = test_utils::util_test_isdu_sequence_read(
                &poll_tx,
                &poll_response_rx,
                venodor_name_index,
                Some(vendor_name_subindex),
            );
            assert!(result.as_ref().is_ok(), "Test isdu sequence failed");
            assert!(
                VENODOR_NAME_LENGTH == result.as_ref().unwrap().len() as u8,
                "ISDU data length not matching"
            );
            assert!(
                VENODOR_NAME.as_bytes() == result.as_ref().unwrap(),
                "ISDU data not matching"
            );
        }
    }
}
