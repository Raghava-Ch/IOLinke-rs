//! Test-specific types and enums for IO-Link device testing

use std::vec::Vec;

/// Thread message types for communication between test threads.
///
/// Frames travel in both directions over two separate `mpsc` channels:
///
/// - **`RxData`** — test → poll thread: a raw master-frame byte sequence to
///   inject into the device via `pl_transfer_ind`.
/// - **`TxData`** — device → test: the raw device-response bytes that the
///   mock physical layer captured from `pl_transfer_req`.
///
/// Timer events are **not** carried over these channels.  Expired timers are
/// detected inside `IoLinkDevice::poll()` by `pl_collect_elapsed_timers()`
/// and delivered to the DL state machines within the same poll cycle.
#[derive(Debug, Clone)]
pub enum ThreadMessage {
    /// Raw master-frame bytes to inject into the device.
    RxData(Vec<u8>),
    /// Raw device-response bytes captured from `pl_transfer_req`.
    TxData(Vec<u8>),
}

/// Test device modes for different testing scenarios
#[derive(Debug, Clone, PartialEq)]
pub enum TestDeviceMode {
    Startup,
    Preoperate,
    Operate,
}
