//! [`TestHarness`] — the single, typed entry-point for integration tests.
//!
//! ## Architecture
//!
//! ```text
//!  Integration test
//!       │  TestHarness::new()
//!       │  harness.enter_startup()
//!       │  harness.read_min_cycle_time(…)
//!       ▼
//!  TestHarness   (this file)
//!       │  hides mpsc channels, mock device, and polling thread
//!       ▼
//!  test_environment / test_sequences / page_params
//!       │  raw wire-protocol encoding / decoding
//!       ▼
//!  MockPhysicalLayer  ↔  IoLinkDevice  ↔  MockApplicationLayer
//! ```
//!
//! ## Rules enforced here
//! - `TestHarness` does **not** assert anything — it is a clean interface.
//! - Each test owns its own `TestHarness`; there is no shared global state.
//! - Test bodies contain zero mentions of `mpsc`, `poll_tx`, or frame bytes.

use std::sync::mpsc::{Receiver, Sender};

use iolinke_device::CycleTime;

use crate::{
    page_params,
    test_environment::setup_test_environment,
    test_sequences::{
        util_test_change_operation_mode, util_test_preop_sequence, util_test_startup_sequence,
    },
    types::{TestDeviceMode, ThreadMessage},
};

/// A typed test environment that wraps the raw `mpsc` channels.
///
/// Call [`TestHarness::new`] at the start of every integration test.  The
/// harness wires up the mock device, applies device configuration, and starts
/// the background polling thread.  From that point forward, the test only
/// sees the high-level methods below.
pub struct TestHarness {
    poll_tx: Sender<ThreadMessage>,
    poll_response_rx: Receiver<ThreadMessage>,
}

impl TestHarness {
    /// Initializes the mock device, applies device configuration, and starts
    /// the background polling thread.
    pub fn new() -> Self {
        let (poll_tx, poll_response_rx) = setup_test_environment();
        Self {
            poll_tx,
            poll_response_rx,
        }
    }

    /// Verifies that the device responds correctly to all Startup-mode
    /// direct-parameter reads.  Does **not** perform a state transition.
    pub fn enter_startup(&self) -> Result<(), Box<dyn std::error::Error>> {
        util_test_startup_sequence(&self.poll_tx, &self.poll_response_rx)
    }

    /// Drives the device from Startup into PreOperate and verifies the
    /// PreOperate parameter set.
    pub fn enter_preoperate(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.enter_startup()?;
        util_test_change_operation_mode(
            &self.poll_tx,
            &self.poll_response_rx,
            TestDeviceMode::Startup,
            TestDeviceMode::Preoperate,
        )?;
        util_test_preop_sequence(&self.poll_tx, &self.poll_response_rx).map(|_| ())
    }

    /// Drives the device from Startup through PreOperate and into Operate.
    pub fn enter_operate(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.enter_preoperate()?;
        std::thread::sleep(std::time::Duration::from_millis(699));
        util_test_change_operation_mode(
            &self.poll_tx,
            &self.poll_response_rx,
            TestDeviceMode::Preoperate,
            TestDeviceMode::Operate,
        )
    }

    /// Reads the `MinCycleTime` direct parameter in the given device mode.
    pub fn read_min_cycle_time(&self, mode: TestDeviceMode) -> CycleTime {
        page_params::read_min_cycle_time(&self.poll_tx, &self.poll_response_rx, mode)
    }
}
