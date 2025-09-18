use iolinke_derived_config::device as derived_config;
use iolinke_test_utils::TestDeviceMode;
use iolinke_types::{
    handlers::pm::{DataStorageIndexSubIndex, DeviceParametersIndex, SubIndex},
    page::page1::MasterCommand,
};

#[test]
fn test_write_master_ident() {
    let (poll_tx, poll_response_rx) = iolinke_test_utils::setup_test_environment();
    let result = iolinke_test_utils::util_test_startup_sequence(&poll_tx, &poll_response_rx);
    assert!(result.is_ok(), "Test startup sequence failed");
    let result = iolinke_test_utils::util_test_change_operation_mode(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Startup,
        TestDeviceMode::Preoperate,
    );
    assert!(result.is_ok(), "Test change operation mode failed");

    let is_master_ident_written = iolinke_test_utils::page_params::write_master_command(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Preoperate,
        MasterCommand::MasterIdent,
    );
    assert!(is_master_ident_written, "Failed to write master ident");
}

#[test]
fn test_write_master_pre_operate() {
    let (poll_tx, poll_response_rx) = iolinke_test_utils::setup_test_environment();
    let result = iolinke_test_utils::util_test_startup_sequence(&poll_tx, &poll_response_rx);
    assert!(result.is_ok(), "Test startup sequence failed");

    let is_master_pre_op_written = iolinke_test_utils::page_params::write_master_command(
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

    // Startup is successfull now command device to preop mode
    let result = iolinke_test_utils::util_test_preop_sequence(&poll_tx, &poll_response_rx);
    assert!(result.is_ok(), "Test preop sequence failed");
    let m_sequence_capability = result.unwrap();

    if m_sequence_capability.isdu() {
        // Vendor Name index 0x10, subindex 0x00
        const VENDOR_NAME: &'static str = derived_config::vendor_specifics::vendor_name();
        const VENDOR_NAME_LENGTH: u8 = VENDOR_NAME.len() as u8;
        let vendor_name = read_vendor_name(&poll_tx, &poll_response_rx);
        assert!(vendor_name.as_ref().is_ok(), "Test isdu sequence failed");
        assert!(
            VENDOR_NAME_LENGTH == vendor_name.as_ref().unwrap().len() as u8,
            "ISDU data length not matching"
        );
        assert!(
            VENDOR_NAME.as_bytes() == vendor_name.as_ref().unwrap(),
            "ISDU data not matching"
        );
    }
}

fn read_vendor_name(
    poll_tx: &std::sync::mpsc::Sender<iolinke_test_utils::ThreadMessage>,
    poll_response_rx: &std::sync::mpsc::Receiver<iolinke_test_utils::ThreadMessage>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let vendor_name_index = DeviceParametersIndex::VendorName.index();
    let vendor_name_subindex = DeviceParametersIndex::VendorName.subindex(SubIndex::VendorName);
    let vendor_name = iolinke_test_utils::util_pre_op_test_isdu_sequence_read(
        poll_tx,
        poll_response_rx,
        vendor_name_index,
        Some(vendor_name_subindex),
    );
    vendor_name
}

#[test]
fn test_write_data_storage_index_and_read_back() {
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

    // Startup is successfull now command device to preop mode
    let result = iolinke_test_utils::util_test_preop_sequence(&poll_tx, &poll_response_rx);
    assert!(result.is_ok(), "Test preop sequence failed");
    let m_sequence_capability = result.unwrap();

    if m_sequence_capability.isdu() {
        // Vendor Name index 0x10, subindex 0x00
        const VENDOR_NAME: &'static str = derived_config::vendor_specifics::vendor_name();
        const VENDOR_NAME_LENGTH: u8 = VENDOR_NAME.len() as u8;
        let vendor_name = read_vendor_name(&poll_tx, &poll_response_rx);
        assert!(vendor_name.as_ref().is_ok(), "Test isdu sequence failed");
        assert!(
            VENDOR_NAME_LENGTH == vendor_name.as_ref().unwrap().len() as u8,
            "ISDU data length not matching"
        );
        assert!(
            VENDOR_NAME.as_bytes() == vendor_name.as_ref().unwrap(),
            "ISDU data not matching"
        );

        loop_test(&poll_tx, &poll_response_rx);
    } else {
        println!("⚠️ Device does not configured to support ISDU in PreOperate mode ⚠️");
    }

    let result = iolinke_test_utils::util_test_change_operation_mode(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Preoperate,
        TestDeviceMode::Operate,
    );
    assert!(result.is_ok(), "Test change operation mode failed");

    if m_sequence_capability.isdu() {
        // Vendor Name index 0x10, subindex 0x00
        const VENDOR_NAME: &'static str = derived_config::vendor_specifics::vendor_name();
        const VENDOR_NAME_LENGTH: u8 = VENDOR_NAME.len() as u8;
        let vendor_name = read_vendor_name(&poll_tx, &poll_response_rx);
        assert!(vendor_name.as_ref().is_ok(), "Test isdu sequence failed");
        assert!(
            VENDOR_NAME_LENGTH == vendor_name.as_ref().unwrap().len() as u8,
            "ISDU data length not matching"
        );
        assert!(
            VENDOR_NAME.as_bytes() == vendor_name.as_ref().unwrap(),
            "ISDU data not matching"
        );

        loop_test(&poll_tx, &poll_response_rx);
    } else {
        println!("⚠️ Device does not configured to support ISDU in PreOperate mode ⚠️");
    }
}

fn loop_test(
    poll_tx: &std::sync::mpsc::Sender<iolinke_test_utils::ThreadMessage>,
    poll_response_rx: &std::sync::mpsc::Receiver<iolinke_test_utils::ThreadMessage>,
) {
    // Write DATA_STORAGE_INDEX_INDEX , INDEX_LIST_SUBINDEX, 0x0003, 0x05
    let data_storage_index_index = DeviceParametersIndex::DataStorageIndex.index();
    let index_list_subindex = DeviceParametersIndex::DataStorageIndex.subindex(
        SubIndex::DataStorageIndex(DataStorageIndexSubIndex::IndexList),
    );
    let index_list_data = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
        0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D,
    ];
    let result = iolinke_test_utils::util_pre_op_test_isdu_sequence_write(
        &poll_tx,
        &poll_response_rx,
        data_storage_index_index,
        Some(index_list_subindex),
        &index_list_data,
    );
    assert!(result.is_ok(), "Test isdu sequence failed");

    let result = iolinke_test_utils::util_pre_op_test_isdu_sequence_read(
        &poll_tx,
        &poll_response_rx,
        data_storage_index_index,
        Some(index_list_subindex),
    );
    assert!(result.as_ref().is_ok(), "Test isdu sequence failed");
    assert!(
        index_list_data.len() as u8 == result.as_ref().unwrap().len() as u8,
        "ISDU data length not matching"
    );
    assert!(
        index_list_data == result.as_ref().unwrap().as_slice(),
        "ISDU data not matching"
    );
}
