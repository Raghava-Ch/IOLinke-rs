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
use iolinke_device::CycleTime;
use iolinke_test_utils::{TestHarness, TestDeviceMode};

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
