use iolinke_derived_config::device as derived_config;
use iolinke_device::*;
use iolinke_test_utils;
use iolinke_test_utils::TestDeviceMode;

#[test]
fn test_min_cycle_time() {
    let (_device, poll_tx, poll_response_rx) = iolinke_test_utils::setup_test_environment();
    let min_cycle_time = iolinke_test_utils::page_params::read_min_cycle_time(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Startup,
    );
    const CONFIG_MIN_CYCLE_TIME: CycleTime =
        derived_config::timings::min_cycle_time::min_cycle_time_parameter();
    assert_eq!(
        CONFIG_MIN_CYCLE_TIME.time_base(),
        min_cycle_time.time_base(),
        "Min cycle time time_base not matching"
    );
    assert_eq!(
        CONFIG_MIN_CYCLE_TIME.multiplier(),
        min_cycle_time.multiplier(),
        "Min cycle time multiplier not matching"
    );
}

#[test]
fn test_m_sequence_capability() {
    let (_device, poll_tx, poll_response_rx) = iolinke_test_utils::setup_test_environment();
    let m_sequence_capability = iolinke_test_utils::page_params::read_m_sequence_capability(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Startup,
    );
    const CONFIG_M_SEQUENCE_CAPABILITY: MsequenceCapability =
        derived_config::m_seq_capability::m_sequence_capability_parameter();
    assert_eq!(
        CONFIG_M_SEQUENCE_CAPABILITY.preoperate_m_sequence(),
        m_sequence_capability.preoperate_m_sequence(),
        "M-sequenceCapability Pre operate msequnce is not matching"
    );
    assert_eq!(
        CONFIG_M_SEQUENCE_CAPABILITY.operate_m_sequence(),
        m_sequence_capability.operate_m_sequence(),
        "M-sequenceCapability Operate msequnce is not matching"
    );
    assert_eq!(
        CONFIG_M_SEQUENCE_CAPABILITY.isdu(),
        m_sequence_capability.isdu(),
        "M-sequenceCapability ISDU is not matching"
    );
}

#[test]
fn test_revision_id() {
    let (_device, poll_tx, poll_response_rx) = iolinke_test_utils::setup_test_environment();
    let revision_id = iolinke_test_utils::page_params::read_revision_id(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Startup,
    );
    const CONFIG_REVISION_ID: RevisionId =
        derived_config::vendor_specifics::revision_id_parameter();
    assert_eq!(
        CONFIG_REVISION_ID.major_rev(),
        revision_id.major_rev(),
        "RevisionID major rev is not matching"
    );
    assert_eq!(
        CONFIG_REVISION_ID.minor_rev(),
        revision_id.minor_rev(),
        "RevisionID minor rev is not matching"
    );
}

#[test]
fn test_process_data_in() {
    let (_device, poll_tx, poll_response_rx) = iolinke_test_utils::setup_test_environment();
    let process_data_in = iolinke_test_utils::page_params::read_process_data_in(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Startup,
    );
    const CONFIG_PROCESS_DATA_IN: ProcessDataIn =
        derived_config::process_data::pd_in::pd_in_parameter();
    assert_eq!(
        CONFIG_PROCESS_DATA_IN.byte(),
        process_data_in.byte(),
        "ProcessDataIn byte is not matching"
    );
    assert_eq!(
        CONFIG_PROCESS_DATA_IN.sio(),
        process_data_in.sio(),
        "ProcessDataIn sio is not matching"
    );
    assert_eq!(
        CONFIG_PROCESS_DATA_IN.length(),
        process_data_in.length(),
        "ProcessDataIn length is not matching"
    );
}

#[test]
fn test_process_data_out() {
    let (_device, poll_tx, poll_response_rx) = iolinke_test_utils::setup_test_environment();
    let process_data_out = iolinke_test_utils::page_params::read_process_data_out(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Startup,
    );
    const CONFIG_PROCESS_DATA_OUT: ProcessDataOut =
        derived_config::process_data::pd_out::pd_out_parameter();
    assert_eq!(
        CONFIG_PROCESS_DATA_OUT.byte(),
        process_data_out.byte(),
        "ProcessDataOut byte is not matching"
    );
    assert_eq!(
        CONFIG_PROCESS_DATA_OUT.length(),
        process_data_out.length(),
        "ProcessDataOut length is not matching"
    );
}

#[test]
fn test_vendor_id_1() {
    let (_device, poll_tx, poll_response_rx) = iolinke_test_utils::setup_test_environment();
    let vendor_id_1 = iolinke_test_utils::page_params::read_vendor_id_1(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Startup,
    );
    const CONFIG_VENDOR_ID_1: u8 = derived_config::vendor_specifics::vendor_id_1();
    assert_eq!(CONFIG_VENDOR_ID_1, vendor_id_1, "VendorID1 is not matching");
}

#[test]
fn test_vendor_id_2() {
    let (_device, poll_tx, poll_response_rx) = iolinke_test_utils::setup_test_environment();
    let vendor_id_2 = iolinke_test_utils::page_params::read_vendor_id_2(
        &poll_tx,
        &poll_response_rx,
        TestDeviceMode::Startup,
    );
    const CONFIG_VENDOR_ID_2: u8 = derived_config::vendor_specifics::vendor_id_2();
    assert_eq!(CONFIG_VENDOR_ID_2, vendor_id_2, "VendorID2 is not matching");
}
