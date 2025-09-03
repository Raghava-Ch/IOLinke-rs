use iolinke_device::{
    MasterCommand,
    test_utils::{self, TestDeviceMode},
};

use iolinke_device::{config, DataStorageIndexSubIndex, DeviceParametersIndex, SubIndex};

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

#[test]
fn test_write_data_storage_index_and_read_back() {
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
        let vendor_name_subindex = DeviceParametersIndex::VendorName.subindex(SubIndex::VendorName);
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

        // Product Name index 0x12, subindex 0x00
        let product_name_index = DeviceParametersIndex::ProductName.index();
        let product_name_subindex =
            DeviceParametersIndex::ProductName.subindex(SubIndex::ProductName);
        const PRODUCT_NAME: &'static str = config::vendor_specifics::product_name();
        const PRODUCT_NAME_LENGTH: u8 = config::vendor_specifics::product_name_length();
        let result = test_utils::util_test_isdu_sequence_read(
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

        // Write DATA_STORAGE_INDEX_INDEX , INDEX_LIST_SUBINDEX, 0x0003, 0x05
        let data_storage_index_index = DeviceParametersIndex::DataStorageIndex.index();
        let index_list_subindex = DeviceParametersIndex::DataStorageIndex.subindex(
            SubIndex::DataStorageIndex(DataStorageIndexSubIndex::IndexList),
        );
        let index_list_data = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
            0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B,
            0x1C, 0x1D
        ];
        let result = test_utils::util_test_isdu_sequence_write(
            &poll_tx,
            &poll_response_rx,
            data_storage_index_index,
            Some(index_list_subindex),
            &index_list_data,
        );
        assert!(result.is_ok(), "Test isdu sequence failed");

        let result = test_utils::util_test_isdu_sequence_read(
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
}