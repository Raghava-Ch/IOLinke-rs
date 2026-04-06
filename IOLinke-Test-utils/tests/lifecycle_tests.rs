//! # Integration tests — device lifecycle
//!
//! ## Philosophy
//!
//! Every test here follows the same three-line pattern:
//!
//! ```text
//! let harness = TestHarness::new();          // 1. own harness, no globals
//! let value   = harness.read_…(…);           // 2. exercise the device
//! assert_eq!(value, EXPECTED_FROM_CONFIG);   // 3. assert against config, not magic numbers
//! ```
//!
//! Test naming convention: `<noun>_<scenario>_<expected_outcome>`

use iolinke_derived_config::device as derived_config;
use iolinke_dev_config::device as dev_config;
use iolinke_device::{CycleTime, MsequenceCapability, ProcessDataIn, ProcessDataOut, RevisionId};
use iolinke_test_utils::{TestHarness, TestDeviceMode};

// ─── Startup phase ────────────────────────────────────────────────────────────

/// Verifies that the `MinCycleTime` direct parameter read in Startup mode
/// matches the compile-time device configuration.
#[test]
fn startup_reads_correct_min_cycle_time() {
    let harness = TestHarness::new();
    let min_cycle_time = harness.read_min_cycle_time(TestDeviceMode::Startup);
    const EXPECTED: CycleTime =
        derived_config::timings::min_cycle_time::min_cycle_time_parameter();
    assert_eq!(
        EXPECTED.time_base(),
        min_cycle_time.time_base(),
        "MinCycleTime time_base mismatch"
    );
    assert_eq!(
        EXPECTED.multiplier(),
        min_cycle_time.multiplier(),
        "MinCycleTime multiplier mismatch"
    );
}

/// Verifies that the full Startup → PreOperate state transition completes
/// without error.
#[test]
fn startup_to_preoperate_transition_succeeds() {
    let harness = TestHarness::new();
    let result = harness.enter_preoperate();
    assert!(
        result.is_ok(),
        "Startup → PreOperate transition failed: {result:?}"
    );
}

// ─── Preoperate phase ─────────────────────────────────────────────────────────

/// Verifies that `MSequenceCapability` read in Preoperate mode matches the
/// compile-time device configuration.
#[test]
fn preoperate_reads_correct_m_sequence_capability() {
    let harness = TestHarness::new();
    harness.enter_preoperate().expect("failed to enter Preoperate");
    let mseq = harness.read_m_sequence_capability(TestDeviceMode::Preoperate);
    const EXPECTED: MsequenceCapability =
        derived_config::m_seq_capability::m_sequence_capability_parameter();
    assert_eq!(
        EXPECTED.preoperate_m_sequence(),
        mseq.preoperate_m_sequence(),
        "MseqCap preoperate_m_sequence mismatch"
    );
    assert_eq!(
        EXPECTED.operate_m_sequence(),
        mseq.operate_m_sequence(),
        "MseqCap operate_m_sequence mismatch"
    );
    assert_eq!(EXPECTED.isdu(), mseq.isdu(), "MseqCap ISDU flag mismatch");
}

/// Verifies that `RevisionID` read in Preoperate mode matches the
/// compile-time device configuration.
#[test]
fn preoperate_reads_correct_revision_id() {
    let harness = TestHarness::new();
    harness.enter_preoperate().expect("failed to enter Preoperate");
    let revision_id = harness.read_revision_id(TestDeviceMode::Preoperate);
    const EXPECTED: RevisionId =
        RevisionId::from_bits(dev_config::vendor_specifics::REVISION_ID);
    assert_eq!(
        EXPECTED.major_rev(),
        revision_id.major_rev(),
        "RevisionID major_rev mismatch"
    );
    assert_eq!(
        EXPECTED.minor_rev(),
        revision_id.minor_rev(),
        "RevisionID minor_rev mismatch"
    );
}

/// Verifies that `ProcessDataIn` read in Preoperate mode matches the
/// compile-time device configuration.
#[test]
fn preoperate_reads_correct_process_data_in() {
    let harness = TestHarness::new();
    harness.enter_preoperate().expect("failed to enter Preoperate");
    let pd_in = harness.read_process_data_in(TestDeviceMode::Preoperate);
    const EXPECTED: ProcessDataIn = derived_config::process_data::pd_in::pd_in_parameter();
    assert_eq!(EXPECTED.length(), pd_in.length(), "ProcessDataIn length mismatch");
    assert_eq!(EXPECTED.byte(), pd_in.byte(), "ProcessDataIn byte flag mismatch");
    assert_eq!(EXPECTED.sio(), pd_in.sio(), "ProcessDataIn sio flag mismatch");
}

/// Verifies that `ProcessDataOut` read in Preoperate mode matches the
/// compile-time device configuration.
#[test]
fn preoperate_reads_correct_process_data_out() {
    let harness = TestHarness::new();
    harness.enter_preoperate().expect("failed to enter Preoperate");
    let pd_out = harness.read_process_data_out(TestDeviceMode::Preoperate);
    const EXPECTED: ProcessDataOut = derived_config::process_data::pd_out::pd_out_parameter();
    assert_eq!(EXPECTED.length(), pd_out.length(), "ProcessDataOut length mismatch");
    assert_eq!(EXPECTED.byte(), pd_out.byte(), "ProcessDataOut byte flag mismatch");
}

// ─── Operate phase ────────────────────────────────────────────────────────────

/// Verifies that the full Preoperate → Operate state transition completes
/// without error.
#[test]
fn preoperate_to_operate_transition_succeeds() {
    let harness = TestHarness::new();
    let result = harness.enter_operate();
    assert!(
        result.is_ok(),
        "PreOperate → Operate transition failed: {result:?}"
    );
}

/// Verifies that `MinCycleTime` read in Operate mode matches the
/// compile-time device configuration.
#[test]
fn operate_reads_correct_min_cycle_time() {
    let harness = TestHarness::new();
    harness.enter_operate().expect("failed to enter Operate");
    let min_cycle_time = harness.read_min_cycle_time(TestDeviceMode::Operate);
    const EXPECTED: CycleTime =
        derived_config::timings::min_cycle_time::min_cycle_time_parameter();
    assert_eq!(
        EXPECTED.time_base(),
        min_cycle_time.time_base(),
        "MinCycleTime time_base mismatch in Operate"
    );
    assert_eq!(
        EXPECTED.multiplier(),
        min_cycle_time.multiplier(),
        "MinCycleTime multiplier mismatch in Operate"
    );
}

/// Verifies that `MSequenceCapability` read in Operate mode matches the
/// compile-time device configuration.
#[test]
fn operate_reads_correct_m_sequence_capability() {
    let harness = TestHarness::new();
    harness.enter_operate().expect("failed to enter Operate");
    let mseq = harness.read_m_sequence_capability(TestDeviceMode::Operate);
    const EXPECTED: MsequenceCapability =
        derived_config::m_seq_capability::m_sequence_capability_parameter();
    assert_eq!(
        EXPECTED.preoperate_m_sequence(),
        mseq.preoperate_m_sequence(),
        "MseqCap preoperate_m_sequence mismatch in Operate"
    );
    assert_eq!(
        EXPECTED.operate_m_sequence(),
        mseq.operate_m_sequence(),
        "MseqCap operate_m_sequence mismatch in Operate"
    );
}

/// Verifies that `VendorID` bytes 1 and 2 read in Operate mode match the
/// compile-time device configuration.
#[test]
fn operate_reads_correct_vendor_id() {
    let harness = TestHarness::new();
    harness.enter_operate().expect("failed to enter Operate");
    let vid1 = harness.read_vendor_id_1(TestDeviceMode::Operate);
    let vid2 = harness.read_vendor_id_2(TestDeviceMode::Operate);
    const EXPECTED_VID1: u8 = dev_config::vendor_specifics::VENDOR_ID[0];
    const EXPECTED_VID2: u8 = dev_config::vendor_specifics::VENDOR_ID[1];
    assert_eq!(EXPECTED_VID1, vid1, "VendorID1 mismatch in Operate");
    assert_eq!(EXPECTED_VID2, vid2, "VendorID2 mismatch in Operate");
}
