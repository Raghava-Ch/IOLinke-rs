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
use std::vec::Vec;

use iolinke_device::{CycleTime, MsequenceCapability, ProcessDataIn, ProcessDataOut, RevisionId};

use crate::{
    page_params,
    test_environment::setup_test_environment,
    test_sequences::{
        util_op_test_isdu_sequence_read, util_op_test_isdu_sequence_write,
        util_pre_op_test_isdu_sequence_read, util_pre_op_test_isdu_sequence_write,
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

    /// Reads the `MSequenceCapability` direct parameter in the given device mode.
    pub fn read_m_sequence_capability(&self, mode: TestDeviceMode) -> MsequenceCapability {
        page_params::read_m_sequence_capability(&self.poll_tx, &self.poll_response_rx, mode)
    }

    /// Reads the `RevisionID` direct parameter in the given device mode.
    pub fn read_revision_id(&self, mode: TestDeviceMode) -> RevisionId {
        page_params::read_revision_id(&self.poll_tx, &self.poll_response_rx, mode)
    }

    /// Reads the `ProcessDataIn` direct parameter in the given device mode.
    pub fn read_process_data_in(&self, mode: TestDeviceMode) -> ProcessDataIn {
        page_params::read_process_data_in(&self.poll_tx, &self.poll_response_rx, mode)
    }

    /// Reads the `ProcessDataOut` direct parameter in the given device mode.
    pub fn read_process_data_out(&self, mode: TestDeviceMode) -> ProcessDataOut {
        page_params::read_process_data_out(&self.poll_tx, &self.poll_response_rx, mode)
    }

    /// Reads the first byte of the `VendorID` direct parameter in the given device mode.
    pub fn read_vendor_id_1(&self, mode: TestDeviceMode) -> u8 {
        page_params::read_vendor_id_1(&self.poll_tx, &self.poll_response_rx, mode)
    }

    /// Reads the second byte of the `VendorID` direct parameter in the given device mode.
    pub fn read_vendor_id_2(&self, mode: TestDeviceMode) -> u8 {
        page_params::read_vendor_id_2(&self.poll_tx, &self.poll_response_rx, mode)
    }

    // ── ISDU ─────────────────────────────────────────────────────────────────

    /// Reads an indexed parameter via ISDU in **PreOperate** mode.
    ///
    /// The device must already be in the PreOperate state (call [`enter_preoperate`] first).
    ///
    /// Returns the raw application-data bytes (ISDU service header stripped).
    ///
    /// [`enter_preoperate`]: TestHarness::enter_preoperate
    pub fn isdu_read_preoperate(
        &self,
        index: u16,
        subindex: Option<u8>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        util_pre_op_test_isdu_sequence_read(&self.poll_tx, &self.poll_response_rx, index, subindex)
    }

    /// Writes an indexed parameter via ISDU in **PreOperate** mode.
    ///
    /// The device must already be in the PreOperate state (call [`enter_preoperate`] first).
    ///
    /// The write is considered successful when the device acknowledges with `WriteSuccess`.
    ///
    /// [`enter_preoperate`]: TestHarness::enter_preoperate
    pub fn isdu_write_preoperate(
        &self,
        index: u16,
        subindex: Option<u8>,
        data: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        util_pre_op_test_isdu_sequence_write(
            &self.poll_tx,
            &self.poll_response_rx,
            index,
            subindex,
            data,
        )
    }

    /// Reads an indexed parameter via ISDU in **Operate** mode.
    ///
    /// The device must already be in the Operate state (call [`enter_operate`] first).
    ///
    /// Returns the raw application-data bytes (ISDU service header stripped).
    ///
    /// [`enter_operate`]: TestHarness::enter_operate
    pub fn isdu_read_operate(
        &self,
        index: u16,
        subindex: Option<u8>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        util_op_test_isdu_sequence_read(&self.poll_tx, &self.poll_response_rx, index, subindex)
    }

    /// Writes an indexed parameter via ISDU in **Operate** mode.
    ///
    /// The device must already be in the Operate state (call [`enter_operate`] first).
    ///
    /// The write is considered successful when the device acknowledges with `WriteSuccess`.
    ///
    /// [`enter_operate`]: TestHarness::enter_operate
    pub fn isdu_write_operate(
        &self,
        index: u16,
        subindex: Option<u8>,
        data: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        util_op_test_isdu_sequence_write(
            &self.poll_tx,
            &self.poll_response_rx,
            index,
            subindex,
            data,
        )
    }
}
