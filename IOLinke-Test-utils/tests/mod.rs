// Test module for iolinke_device crate
// This file organizes all test modules

use iolinke_test_utils::{self, TestDeviceMode};
use iolinke_types::page::page1::MasterCommand;

pub mod isdu_tests;
pub mod preop_tests;
pub mod startup_tests;

#[test]
fn mock_test_device_operations() {
    // Set up test environment
    let (poll_tx, poll_response_rx) = iolinke_test_utils::setup_test_environment();

    // Test startup sequence is successful and device is in startup mode
    let result = iolinke_test_utils::util_test_startup_sequence(&poll_tx, &poll_response_rx);
    assert!(result.is_ok(), "Test startup sequence failed");

    let result = iolinke_test_utils::util_test_change_operation_mode(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Startup,
        TestDeviceMode::Preoperate,
    );
    assert!(result.is_ok(), "Test change operation mode failed");

    // Startup is successful now command device to operate mode
    let result = iolinke_test_utils::util_test_preop_sequence(&poll_tx, &poll_response_rx);
    assert!(result.is_ok(), "Test preop sequence failed");
    let _m_sequence_capability = result.unwrap();
    std::thread::sleep(std::time::Duration::from_millis(699));
    let result = iolinke_test_utils::util_test_change_operation_mode(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Preoperate,
        TestDeviceMode::Operate,
    );
    assert!(result.is_ok(), "Test change operation mode failed");

    let is_master_pre_op_written = iolinke_test_utils::page_params::write_master_command(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Operate,
        MasterCommand::ProcessDataOutputOperate,
    );
    assert!(
        is_master_pre_op_written,
        "Failed to write master process data output operate"
    );

    let m_sequence_capability = iolinke_test_utils::read_m_sequence_capability(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Operate,
    );
    assert!(
        m_sequence_capability.isdu(),
        "Test m sequence capability failed"
    );
}
