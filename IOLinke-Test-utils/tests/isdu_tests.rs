//! # Integration tests — ISDU parameter transfers
//!
//! ## Philosophy
//!
//! Same three-line pattern as all other integration tests:
//!
//! ```text
//! let harness = TestHarness::new();               // 1. own harness, no globals
//! harness.enter_preoperate()?;                    // 2. advance to required mode
//! let data = harness.isdu_read_preoperate(…)?;   // 3. exercise ISDU path
//! assert_eq!(data, EXPECTED);                     // 4. assert against static config
//! ```
//!
//! ## Coverage
//!
//! | Scenario | PreOperate | Operate |
//! |---|---|---|
//! | Single-byte read (OD-frame fits 1 ISDU byte) | ✔ | ✔ |
//! | Multi-byte read (spans several PreOp OD frames) | ✔ | ✔ |
//! | Write (device acknowledges with `WriteSuccess`) | ✔ | ✔ |
//!
//! Test naming convention: `<noun>_<scenario>_<expected_outcome>`

use iolinke_dev_config::device as dev_config;
use iolinke_test_utils::TestHarness;

// ─── Parameter constants used across tests ───────────────────────────────────

/// Index 0x0000, sub-index 0x09: DeviceID byte 1 (ReadWrite, 1 byte).
/// Single-byte parameter — fits inside one 2-byte PreOp OD frame.
const DEVICE_ID1_INDEX: u16 = 0x0000;
const DEVICE_ID1_SUBINDEX: u8 = 0x09;

/// Index 0x0010, no sub-index: VendorName (ReadOnly, 7 bytes = "IOLinke").
/// Multi-byte parameter — spans several 2-byte PreOp OD frames.
const VENDOR_NAME_INDEX: u16 = 0x0010;

/// Index 0x0012, no sub-index: ProductName (ReadOnly, 13 bytes = "IOLinke Stack").
const PRODUCT_NAME_INDEX: u16 = 0x0012;

// ─── PreOperate ───────────────────────────────────────────────────────────────

/// Verifies that a single-byte ISDU read in PreOperate mode returns the expected value.
///
/// Reads `DeviceID1` (index 0x0000, sub-index 0x09) — the data fits within one
/// 2-byte PreOp OD frame, exercising the single-segment path of the ISDU handler.
#[test]
fn preoperate_isdu_single_byte_read_returns_correct_data() {
    let harness = TestHarness::new();
    harness.enter_preoperate().expect("failed to enter Preoperate");

    let data = harness
        .isdu_read_preoperate(DEVICE_ID1_INDEX, Some(DEVICE_ID1_SUBINDEX))
        .expect("ISDU single-byte read failed in Preoperate");

    const EXPECTED: u8 = dev_config::vendor_specifics::DEVICE_ID[0];
    assert_eq!(data.len(), 1, "Expected exactly 1 data byte");
    assert_eq!(data[0], EXPECTED, "DeviceID1 value mismatch in Preoperate");
}

/// Verifies that a multi-byte ISDU read in PreOperate mode returns the expected value.
///
/// Reads `VendorName` (index 0x0010) — 7 bytes spread across multiple 2-byte PreOp
/// OD frames, exercising the multi-segment reassembly path of the ISDU handler.
#[test]
fn preoperate_isdu_multi_byte_read_returns_correct_data() {
    let harness = TestHarness::new();
    harness.enter_preoperate().expect("failed to enter Preoperate");

    let data = harness
        .isdu_read_preoperate(VENDOR_NAME_INDEX, None)
        .expect("ISDU multi-byte read failed in Preoperate");

    const EXPECTED: &[u8] = b"IOLinke";
    assert_eq!(data.as_slice(), EXPECTED, "VendorName mismatch in Preoperate");
}

/// Verifies that an ISDU write in PreOperate mode is acknowledged with `WriteSuccess`.
///
/// Writes a new value to `DeviceID1` (index 0x0000, sub-index 0x09).
/// The assertion is implicit: `isdu_write_preoperate` internally checks the device
/// response for `IsduIServiceCode::WriteSuccess` and returns `Err` on failure.
#[test]
fn preoperate_isdu_write_succeeds() {
    let harness = TestHarness::new();
    harness.enter_preoperate().expect("failed to enter Preoperate");

    harness
        .isdu_write_preoperate(DEVICE_ID1_INDEX, Some(DEVICE_ID1_SUBINDEX), &[0x42])
        .expect("ISDU write failed in Preoperate");
}

// ─── Operate ─────────────────────────────────────────────────────────────────

/// Verifies that a single-byte ISDU read in Operate mode returns the expected value.
///
/// Reads `DeviceID1` (index 0x0000, sub-index 0x09).
/// With the 32-byte Operate OD length the entire response fits in one OD frame.
#[test]
fn operate_isdu_single_byte_read_returns_correct_data() {
    let harness = TestHarness::new();
    harness.enter_operate().expect("failed to enter Operate");

    let data = harness
        .isdu_read_operate(DEVICE_ID1_INDEX, Some(DEVICE_ID1_SUBINDEX))
        .expect("ISDU single-byte read failed in Operate");

    const EXPECTED: u8 = dev_config::vendor_specifics::DEVICE_ID[0];
    assert_eq!(data.len(), 1, "Expected exactly 1 data byte");
    assert_eq!(data[0], EXPECTED, "DeviceID1 value mismatch in Operate");
}

/// Verifies that a multi-byte ISDU read in Operate mode returns the expected value.
///
/// Reads `ProductName` (index 0x0012) — 13 bytes.
/// With the 32-byte Operate OD length the complete response fits in one OD frame,
/// but the ISDU handler still correctly strips the service header and checksum.
#[test]
fn operate_isdu_multi_byte_read_returns_correct_data() {
    let harness = TestHarness::new();
    harness.enter_operate().expect("failed to enter Operate");

    let data = harness
        .isdu_read_operate(PRODUCT_NAME_INDEX, None)
        .expect("ISDU multi-byte read failed in Operate");

    const EXPECTED: &[u8] = b"IOLinke Stack";
    assert_eq!(data.as_slice(), EXPECTED, "ProductName mismatch in Operate");
}

/// Verifies that an ISDU write in Operate mode is acknowledged with `WriteSuccess`.
///
/// Writes a new value to `DeviceID1` (index 0x0000, sub-index 0x09).
#[test]
fn operate_isdu_write_succeeds() {
    let harness = TestHarness::new();
    harness.enter_operate().expect("failed to enter Operate");

    harness
        .isdu_write_operate(DEVICE_ID1_INDEX, Some(DEVICE_ID1_SUBINDEX), &[0x42])
        .expect("ISDU write failed in Operate");
}
