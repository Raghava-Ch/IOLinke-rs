// Test module for iolinke_device crate
// This file organizes all test modules

use std::time::Duration;

use iolinke_device::{
    DeviceParametersIndex, MsequenceCapability, SubIndex, config,
    test_utils::{self, IsduService, TestDeviceMode},
};

pub mod isdu_tests;
pub mod preop_tests;
pub mod startup_tests;

#[test]
fn mock_test_device_operations() {
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
        // Vendor Name index 0x10, subindex 0x00
        let venodor_name_index = DeviceParametersIndex::VendorName.index();
        let vendor_name_subindex = DeviceParametersIndex::VendorName.subindex(SubIndex::VendorName);
        const VENODOR_NAME: &'static str = config::vendor_specifics::vendor_name();
        const VENODOR_NAME_LENGTH: u8 = config::vendor_specifics::vendor_name_length();
        let result = test_utils::util_test_isdu_sequence(
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
