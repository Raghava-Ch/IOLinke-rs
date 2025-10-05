use iolinke_derived_config::device as derived_config;
use iolinke_test_utils::TestDeviceMode;
use iolinke_types::{
    handlers::pm::{DataStorageIndexSubIndex, DeviceParametersIndex, SubIndex},
    page::page1::MasterCommand,
};

/// Test ISDU read operations for vendor name
#[test]
fn test_isdu_read_vendor_name() {
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

    // Startup is successful now command device to preop mode
    let result = iolinke_test_utils::util_test_preop_sequence(&poll_tx, &poll_response_rx);
    assert!(result.is_ok(), "Test preop sequence failed");
    let m_sequence_capability = result.unwrap();

    if m_sequence_capability.isdu() {
        // Vendor Name index 0x10, subindex 0x00
        let vendor_name_index = DeviceParametersIndex::VendorName.index();
        let vendor_name_subindex = DeviceParametersIndex::VendorName.subindex(SubIndex::VendorName);
        const VENDOR_NAME: &'static str = derived_config::vendor_specifics::VENDOR_NAME;
        const VENDOR_NAME_LENGTH: u8 = VENDOR_NAME.len() as u8;

        let result = iolinke_test_utils::util_pre_op_test_isdu_sequence_read(
            &poll_tx,
            &poll_response_rx,
            vendor_name_index,
            Some(vendor_name_subindex),
        );

        assert!(result.as_ref().is_ok(), "Test isdu sequence failed");
        assert!(
            VENDOR_NAME_LENGTH == result.as_ref().unwrap().len() as u8,
            "ISDU data length not matching"
        );
        assert!(
            VENDOR_NAME.as_bytes() == result.as_ref().unwrap(),
            "ISDU data not matching"
        );
    }
}

/// Test ISDU read operations for product name
#[test]
fn test_isdu_read_product_name() {
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

    // Startup is successful now command device to preop mode
    let result = iolinke_test_utils::util_test_preop_sequence(&poll_tx, &poll_response_rx);
    assert!(result.is_ok(), "Test preop sequence failed");
    let m_sequence_capability = result.unwrap();

    if m_sequence_capability.isdu() {
        // Product Name index 0x12, subindex 0x00
        let product_name_index = DeviceParametersIndex::ProductName.index();
        let product_name_subindex =
            DeviceParametersIndex::ProductName.subindex(SubIndex::ProductName);
        const PRODUCT_NAME: &'static str = derived_config::vendor_specifics::PRODUCT_NAME;
        const PRODUCT_NAME_LENGTH: u8 = PRODUCT_NAME.len() as u8;

        let result = iolinke_test_utils::util_pre_op_test_isdu_sequence_read(
            &poll_tx,
            &poll_response_rx,
            product_name_index,
            Some(product_name_subindex),
        );

        assert!(result.as_ref().is_ok(), "Test isdu sequence failed");
        assert!(
            PRODUCT_NAME_LENGTH == result.as_ref().unwrap().len() as u8,
            "ISDU data length not matching"
        );
        assert!(
            PRODUCT_NAME.as_bytes() == result.as_ref().unwrap(),
            "ISDU data not matching"
        );
    }
}

/// Test ISDU write and read back operations for data storage index
#[test]
fn test_isdu_write_and_read_data_storage_index() {
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

    // Startup is successful now command device to preop mode
    let result = iolinke_test_utils::util_test_preop_sequence(&poll_tx, &poll_response_rx);
    assert!(result.is_ok(), "Test preop sequence failed");
    let m_sequence_capability = result.unwrap();

    if m_sequence_capability.isdu() {
        // Write DATA_STORAGE_INDEX_INDEX, INDEX_LIST_SUBINDEX
        let data_storage_index_index = DeviceParametersIndex::DataStorageIndex.index();
        let index_list_subindex = DeviceParametersIndex::DataStorageIndex.subindex(
            SubIndex::DataStorageIndex(DataStorageIndexSubIndex::IndexList),
        );

        let test_data = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
            0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B,
            0x1C, 0x1D,
        ];

        // Write the data
        let result = iolinke_test_utils::util_pre_op_test_isdu_sequence_write(
            &poll_tx,
            &poll_response_rx,
            data_storage_index_index,
            Some(index_list_subindex),
            &test_data,
        );
        assert!(result.is_ok(), "Test isdu write sequence failed");

        // Read back the data to verify
        let result = iolinke_test_utils::util_pre_op_test_isdu_sequence_read(
            &poll_tx,
            &poll_response_rx,
            data_storage_index_index,
            Some(index_list_subindex),
        );

        assert!(result.as_ref().is_ok(), "Test isdu read sequence failed");
        assert!(
            test_data.len() as u8 == result.as_ref().unwrap().len() as u8,
            "ISDU data length not matching"
        );
        assert!(
            test_data == result.as_ref().unwrap().as_slice(),
            "ISDU data not matching"
        );
    }
}

#[test]
fn test_isdu_write_and_read_data_storage_index_operate_mode() {
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

    if m_sequence_capability.isdu() {
        // Write DATA_STORAGE_INDEX_INDEX, INDEX_LIST_SUBINDEX
        let data_storage_index_index = DeviceParametersIndex::DataStorageIndex.index();
        let index_list_subindex = DeviceParametersIndex::DataStorageIndex.subindex(
            SubIndex::DataStorageIndex(DataStorageIndexSubIndex::IndexList),
        );

        let test_data = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
            0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B,
            0x1C, 0x1D,
        ];

        // Write the data
        let result = iolinke_test_utils::util_op_test_isdu_sequence_write(
            &poll_tx,
            &poll_response_rx,
            data_storage_index_index,
            Some(index_list_subindex),
            &test_data,
        );
        assert!(result.is_ok(), "Test isdu write sequence failed");

        // Read back the data to verify
        let result = iolinke_test_utils::util_op_test_isdu_sequence_read(
            &poll_tx,
            &poll_response_rx,
            data_storage_index_index,
            Some(index_list_subindex),
        );

        assert!(result.as_ref().is_ok(), "Test isdu read sequence failed");
        assert!(
            test_data.len() as u8 == result.as_ref().unwrap().len() as u8,
            "ISDU data length not matching"
        );
        assert!(
            test_data == result.as_ref().unwrap().as_slice(),
            "ISDU data not matching"
        );
    }
}
