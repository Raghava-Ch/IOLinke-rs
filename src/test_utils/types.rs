//! Test-specific types and enums for IO-Link device testing

use crate::Timer;

/// Thread message types for communication between test threads
#[derive(Debug, Clone)]
pub enum ThreadMessage {
    TimerExpired(Timer),
    RxData(Vec<u8>),
    TxData(Vec<u8>),
}

/// Test device modes for different testing scenarios
#[derive(Debug, Clone, PartialEq)]
pub enum TestDeviceMode {
    Startup,
    Preoperate,
    Operate,
}
