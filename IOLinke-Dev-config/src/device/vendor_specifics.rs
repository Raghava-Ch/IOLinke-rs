//! # Vendor Specifics Module
//!
//! This module provides vendor-specific constants and configuration parameters for IO-Link devices.
//! It defines protocol revision identifiers, vendor and device IDs, function IDs, and human-readable names
//! as required by the IO-Link specification. These constants are used to uniquely identify the device and
//! its capabilities during IO-Link communication.
//!
//! ## Specification References
//! - IO-Link v1.1.4 Annex B.1: Direct Parameter Page 1
//! - Section B.1.4: RevisionID parameter
//! - Section B.1.8: VendorID (VID)
//!
//! ## Configuration
//! Parameter storage configuration must be defined in the
//! `IOLinke-Derived-CONFIG/src/device/vendor_specifics.rs` file due to current Rust compiler limitations.
//! Once compiler support improves, storage configuration may be generalized and defined here.
//!
//! ## Components
//! - `REVISION_ID`: Protocol revision identifier (major and minor version).
//! - `VENDOR_ID`: Unique vendor identifier.
//! - `DEVICE_ID`: Unique device identifier.
//! - `FUNCTION_ID`: Device function identifier.
//! - `VENDOR_NAME`: Human-readable vendor name.
//! - `PRODUCT_NAME`: Human-readable product name.
//! - `revision_id!`: Macro to construct the revision identifier from major and minor values.
//!
//! ## Note
//! Values marked with `/*CONFIG:...*/` are intended to be configured per device or build.

/// Contains vendor-specific constants and configuration parameters for the IO-Link device.
///
/// This module defines protocol revision identifiers, vendor and device IDs, function IDs,
/// and human-readable names as required by the IO-Link specification. These constants are
/// used to uniquely identify the device and its capabilities during IO-Link communication.
///
/// # Specification References
/// - IO-Link v1.1.4 Annex B.1: Direct Parameter Page 1
/// - Section B.1.4: RevisionID parameter
/// - Section B.1.8: VendorID (VID)
///
/// # Configuration
/// The parameter storage configuration must be defined in the
/// `IOLinke-Derived-CONFIG/src/device/vendor_specifics.rs` file due to current Rust compiler
/// limitations. Once compiler support improves, storage configuration may be generalized
/// and defined here.
///
/// # Components
/// - `REVISION_ID`: Protocol revision identifier (major and minor version).
/// - `VENDOR_ID`: Unique vendor identifier.
/// - `DEVICE_ID`: Unique device identifier.
/// - `FUNCTION_ID`: Device function identifier.
/// - `VENDOR_NAME`: Human-readable vendor name.
/// - `PRODUCT_NAME`: Human-readable product name.
/// - `revision_id!`: Macro to construct the revision identifier from major and minor values.
///
/// # Note
/// Values marked with `/*CONFIG:...*/` are intended to be configured per device or build.

/// Protocol revision identifier as per Annex B.1.
///
/// This bitfield contains the major and minor revision numbers
/// of the IO-Link protocol version implemented by the device.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Annex B.1: Direct Parameter Page 1
/// - Section B.1.4: RevisionID parameter
pub const REVISION_ID: u8 = crate::revision_id!(
    /*CONFIG:MAJOR_REVISION_ID*/ 0x09 /*ENDCONFIG*/,
    /*CONFIG:MINOR_REVISION_ID*/ 0x04 /*ENDCONFIG*/
);

/// Vendor ID (VID) as specified in IO-Link v1.1.4 Annex B.1.8.
///
/// This 2-octet value uniquely identifies the vendor worldwide.
///
/// # Specification Reference
/// - IO-Link v1.1.4 Annex B.1.8: VendorID (VID)
///
/// Note:
/// - VendorIDs are assigned by the IO-Link community.
pub const VENDOR_ID: [u8; 2] = [
    /*CONFIG:VENDOR_ID_1*/ 0x01 /*ENDCONFIG*/,
    /*CONFIG:VENDOR_ID_2*/ 0x02 /*ENDCONFIG*/,
];

/// Device ID (DID) as specified in IO-Link v1.1.4 Annex B.1.9.
///
/// This 3-octet value uniquely identifies the device. A value of "0" is not permitted.
/// It is recommended to store the DeviceID in non-volatile memory after a compatibility switch,
/// until a reset to the initial value through SystemCommands such as "Restore factory settings"
/// or "Back-to-box". The value may be overwritten during StartUp (see section 10.6.2).
///
/// # Specification Reference
/// - IO-Link v1.1.4 Annex B.1.9: DeviceID (DID)
pub const DEVICE_ID: [u8; 3] = [
    /*CONFIG:DEVICE_ID_1*/ 0x01 /*ENDCONFIG*/,
    /*CONFIG:DEVICE_ID_2*/ 0x02 /*ENDCONFIG*/,
    /*CONFIG:DEVICE_ID_3*/ 0x03 /*ENDCONFIG*/,
];

/// Refer the specifications B.1.10 FunctionID (FID)
/// This 2-octet value indicates the function of the device.
pub const FUNCTION_ID: [u8; 2] = [
    /*CONFIG:FUNCTION_ID_1*/ 0x01 /*ENDCONFIG*/,
    /*CONFIG:FUNCTION_ID_2*/ 0x01 /*ENDCONFIG*/,
];

/// Vendor Name as specified in IO-Link v1.1.4 Annex B.2.8.
///
/// The `VENDOR_NAME` parameter contains one of the vendor names listed for the assigned `VENDOR_ID`.
/// This parameter is a mandatory, read-only data object with a maximum fixed length of 64 characters.
/// The list of vendor names associated with a given `VENDOR_ID` is maintained by the IO-Link community.
///
/// # Specification Reference
/// - IO-Link v1.1.4 Annex B.2.8: VendorName
///
/// # Note
/// - The value must be a valid vendor name registered for the assigned `VENDOR_ID`.
/// - The list of vendor names associated with a given `VendorID` is maintained by the IO-Link community.
pub const VENDOR_NAME: &str = /*CONFIG:VENDOR_NAME*/ "IOLinke" /*ENDCONFIG*/;

// B.2.10 ProductName
/// Product Name as specified in IO-Link v1.1.4 Annex B.2.10.
///
/// The `PRODUCT_NAME` parameter contains the complete product name of the device.
/// This parameter is mandatory and read-only, with a maximum fixed length of 64 characters.
/// The data type is `StringT`.
///
/// # Specification Reference
/// - IO-Link v1.1.4 Annex B.2.10: ProductName
///
/// # Note
/// - The corresponding entry in the IODD Device variant list is expected to match this parameter.
pub const PRODUCT_NAME: &str = /*CONFIG:PRODUCT_NAME*/ "IOLinke" /*ENDCONFIG*/;

// !Parameter storage configuration must be defined in the IOLinke-Derived-CONFIG/src/device/vendor_specifics.rs file
// !This is done because for current rust compiler limitations.
// As soon as rust compiler supports, storage configuration can be generalised and defined here.
// For now, this is the only way to define the parameter storage configuration.

/* Below these are non configurable components, helper components for the configuration */

/// Macro to construct the revision identifier from major and minor revision numbers.
#[macro_export]
macro_rules! revision_id {
    ($major_revision_id:expr, $minor_revision_id:expr) => {
        $major_revision_id << 4 | $minor_revision_id << 0
    };
}
