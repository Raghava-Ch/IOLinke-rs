use iolinke_device as device;
use iolinke_device::test_utils::TestDeviceMode;
use iolinke_device::*;

use device::test_utils::{self, config};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_cycle_time() {
        let (device, poll_tx, poll_response_rx) = test_utils::setup_test_environment();
        let min_cycle_time = test_utils::page_params::read_min_cycle_time(
            &poll_tx,
            &poll_response_rx,
            TestDeviceMode::Startup,
        );
        const CONFIG_MIN_CYCLE_TIME: CycleTime =
            config::timings::min_cycle_time::min_cycle_time_parameter();
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
        let (device, poll_tx, poll_response_rx) = test_utils::setup_test_environment();
        let m_sequence_capability = test_utils::page_params::read_m_sequence_capability(
            &poll_tx,
            &poll_response_rx,
            TestDeviceMode::Startup,
        );
        const CONFIG_M_SEQUENCE_CAPABILITY: MsequenceCapability =
            config::m_seq_capability::m_sequence_capability_parameter();
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
        let (device, poll_tx, poll_response_rx) = test_utils::setup_test_environment();
        let revision_id = test_utils::page_params::read_revision_id(
            &poll_tx,
            &poll_response_rx,
            TestDeviceMode::Startup,
        );
        const CONFIG_REVISION_ID: RevisionId = config::vendor_specifics::revision_id_parameter();
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
        let (device, poll_tx, poll_response_rx) = test_utils::setup_test_environment();
        let process_data_in = test_utils::page_params::read_process_data_in(
            &poll_tx,
            &poll_response_rx,
            TestDeviceMode::Startup,
        );
        const CONFIG_PROCESS_DATA_IN: ProcessDataIn = config::process_data::pd_in::pd_in_parameter();
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
        let (device, poll_tx, poll_response_rx) = test_utils::setup_test_environment();
        let process_data_out = test_utils::page_params::read_process_data_out(
            &poll_tx,
            &poll_response_rx,
            TestDeviceMode::Startup,
        );
        const CONFIG_PROCESS_DATA_OUT: ProcessDataOut =
            config::process_data::pd_out::pd_out_parameter();
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
        let (device, poll_tx, poll_response_rx) = test_utils::setup_test_environment();
        let vendor_id_1 = test_utils::page_params::read_vendor_id_1(
            &poll_tx,
            &poll_response_rx,
            TestDeviceMode::Startup,
        );
        const CONFIG_VENDOR_ID_1: u8 = config::vendor_specifics::vendor_id_1();
        assert_eq!(
            CONFIG_VENDOR_ID_1,
            vendor_id_1,
            "VendorID1 is not matching"
        );
    }

    #[test]
    fn test_vendor_id_2() {
        let (device, poll_tx, poll_response_rx) = test_utils::setup_test_environment();
        let vendor_id_2 = test_utils::page_params::read_vendor_id_2(
            &poll_tx,
            &poll_response_rx,
            TestDeviceMode::Startup,
        );
        const CONFIG_VENDOR_ID_2: u8 = config::vendor_specifics::vendor_id_2();
        assert_eq!(
            CONFIG_VENDOR_ID_2,
            vendor_id_2,
            "VendorID2 is not matching"
        );
    }
}