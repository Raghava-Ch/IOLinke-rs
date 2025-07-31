//! M-sequence capability configuration for IO-Link devices.
//!
//! This module provides macros for configuring M-sequence capabilities in IO-Link devices
//! according to the IO-Link Specification v1.1.4. It includes configuration for:
//!
//! - PREOPERATE mode M-sequence types (Table A.8)
//! - OPERATE mode M-sequence types for legacy devices (Table A.9)
//! - OPERATE mode M-sequence types for standard devices (Table A.10)
//! - ISDU (Index Sequential Data Unit) support
//! - Complete M-sequence capability parameter construction
//!
//! ## M-sequence Types
//!
//! M-sequences define the communication timing and data exchange patterns between
//! IO-Link masters and devices. Different M-sequence types are used depending on:
//! - Device operation mode (PREOPERATE vs OPERATE)
//! - On-request data length requirements
//! - Process data (PDin/PDout) configurations
//! - Legacy vs standard device implementations
//!
//! ## Usage
//!
//! The macros in this module should be configured according to your device's specific
//! requirements and then used to construct the M-sequence capability parameter that
//! gets reported to the IO-Link master during device identification.
//!
//! ## Compliance Notes
//!
//! - TYPE_0 M-sequences are discouraged for better error detection
//! - Interleaved modes (TYPE_1_1/1_2) must not be implemented in devices
//! - Minimum recovery times must be observed by masters in PREOPERATE mode
//! - All configurations must comply with IO-Link Specification requirements
//! Configuration macros for IO-Link device setup.

/// M-sequence types for the PREOPERATE mode as per IO-Link Specification (v1.1.4, Table A.8).
///
/// The following table describes valid M-sequence types, expected on-request data length (in octets),
/// and the required minimum recovery time (`T_initcyc`) the master must observe in PREOPERATE mode:
///
/// | PREOPERATE M-sequence code | On-request Data (Octets) | M-sequence Type | Minimum Recovery Time (`T_BIT`) |
/// |----------------------------|---------------------------|------------------|----------------------------------|
/// | 0                          | 1                         | TYPE_0           | 100                              |
/// | 1                          | 2                         | TYPE_1_2         | 100                              |
/// | 2                          | 8                         | TYPE_1_V         | 210                              |
/// | 3                          | 32                        | TYPE_1_V         | 550                              |
///
/// ---
///
/// - ⚠️ **Note a**: The minimum recovery time in PREOPERATE mode is a requirement for the Master.
/// - ⚠️ **Note b**: It is highly recommended for Devices **not to use TYPE_0**, as it improves error discovery
///   when the Master restarts communication.
pub const fn preoperate_m_sequence() -> u8 {
    0u8 // Accepted values are 0, 1, 2, or 3
}

/// Returns the on-request data length (in octets) for PREOPERATE mode M-sequences.
///
/// This function provides the expected on-request data length that corresponds to the
/// M-sequence type configured in `preoperate_m_sequence()`. The mapping follows
/// IO-Link Specification v1.1.4, Table A.8.
///
/// ## Dependency Mapping
///
/// | `preoperate_m_sequence()` | M-sequence Type | On-request Data Length | Return Value |
/// |---------------------------|------------------|------------------------|--------------|
/// | 0                         | TYPE_0           | 1 octet                | 1            |
/// | 1                         | TYPE_1_2         | 2 octets               | 2            |
/// | 2                         | TYPE_1_V         | 8 octets               | 8            |
/// | 3                         | TYPE_1_V         | 32 octets              | 32           |
///
/// ## Returns
///
/// The on-request data length in octets (1, 2, 8, or 32) corresponding to the
/// configured PREOPERATE M-sequence type.
///
/// ## Specification Reference
///
/// - IO-Link Specification v1.1.4, Table A.8: PREOPERATE mode M-sequence types
/// - Section A.1.3: M-sequence definitions and data length requirements
pub const fn preoperate_m_sequence_od_len() -> u8 {
    match preoperate_m_sequence() {
        0 => 1u8,                                                   // TYPE_0: 1 octet
        1 => 2u8,                                                   // TYPE_1_2: 2 octets
        2 => 8u8,                                                   // TYPE_1_V: 8 octets
        3 => 32u8,                                                  // TYPE_1_V: 32 octets
        _ => panic!("Invalid PREOPERATE M-sequence configuration"), // Default fallback (should not occur with valid config)
    }
}

/// **This is a prerequisite for the `operate_m_sequence_legacy` config parameter check its documentation**.
/// Returns the on-request data length (in octets) for legacy OPERATE mode M-sequences.
/// ## Returns
/// The on-request data length in octets for all possible M-sequences.
pub const fn operate_m_sequence_legacy_od_len() -> u8 {
    2u8
}

/// **This is a prerequisite for the `operate_m_sequence_legacy` config parameter check its documentation**.
/// Returns the process data length (PDin) for legacy OPERATE mode M-sequences.
/// ## Returns
/// The process data length (PDin) for all possible M-sequences.
/// This is `must be configured as bits`, not octets, so the return value is in bits.
pub const fn operate_m_sequence_legacy_pd_in_len() -> u8 {
    0u8
}

/// **This is a prerequisite for the `operate_m_sequence_legacy` config parameter check its documentation**.
/// Returns the process data length (PDout) for legacy OPERATE mode  M-sequences.
/// ## Returns
/// The process data length (PDout) for all possible M-sequences.
/// This is `must be configured as bits`, not octets, so the return value is in bits.
pub const fn operate_m_sequence_legacy_pd_out_len() -> u8 {
    0u8
}

/// M-sequence types for the OPERATE mode (standard protocol) as per IO-Link Specification (Table A.9).
pub const fn operate_m_sequence_code_legacy() -> u8 {
    1u8 // Accepted values are 0, 1,
}

/// M-sequence types for the OPERATE mode (legacy protocol) as per IO-Link Specification (Table A.9).
///
/// This table describes valid M-sequence types in legacy devices for the OPERATE mode, depending on the
/// on-request data size and process data direction (PDin, PDout):
///
/// | OPERATE M-sequence code | On-request Data (Octets) | PDin (Process Data In)  | PDout (Process Data Out) | M-sequence Type        |
/// |-------------------------|--------------------------|-------------------------|--------------------------|-------------------------|
/// | 0                       | 1                         | 0                      | 0                        | TYPE_0 (⚠️ see NOTE)    |
/// | 1                       | 2                         | 0                      | 0                        | TYPE_1_2                |
/// | don't care              | 2                         | PDin + PDout > 2 octets|                          | TYPE_1_1/1_2 (interleaved) |
/// | don't care              | 1                         | 1…8 bit                | 0                        | TYPE_2_1                |
/// | don't care              | 1                         | 9…16 bit               | 0                        | TYPE_2_2                |
/// | don't care              | 1                         | 0                      | 1…8 bit                  | TYPE_2_3                |
/// | don't care              | 1                         | 0                      | 9…16 bit                 | TYPE_2_4                |
/// | don't care              | 1                         | 1…8 bit                | 1…8 bit                  | TYPE_2_5                |
///
/// ---
///
/// ⚠️ **NOTE**: It is highly recommended for Devices **not to use TYPE_0**, as this improves error discovery
/// when the Master restarts communication.
///
/// The minimum cycle time for Master in OPERATE mode is defined by the device's `MinCycleTime` parameter.
// #[cfg(feature = "legacy_device")]
pub const fn operate_m_sequence_legacy() -> crate::types::MsequenceType {
    match (
        operate_m_sequence_code_legacy(),
        operate_m_sequence_legacy_od_len(),
        operate_m_sequence_legacy_pd_in_len(),
        operate_m_sequence_legacy_pd_out_len(),
    ) {
        (0, 1, 0, 0) => crate::types::MsequenceType::Type0, // TYPE_0
        (1, 2, 0, 0) => crate::types::MsequenceType::Type12, // TYPE_1_2
        (_, 2, pd_in, pdout) if (pd_in + pdout) > 16 /* 2 Octects in bits */ => crate::types::MsequenceType::Type11, // TYPE_1_1/1_2 interleaved
        (_, 1, 1..=8, 0) => crate::types::MsequenceType::Type21, // TYPE_2_1
        (_, 1, 9..=16, 0) => crate::types::MsequenceType::Type22, // TYPE_2_2
        (_, 1, 0, 1..=8) => crate::types::MsequenceType::Type23, // TYPE_2_3
        (_, 1, 0, 9..=16) => crate::types::MsequenceType::Type24, // TYPE_2_4
        (_, 1, 1..=8, 1..=8) => crate::types::MsequenceType::Type25, // TYPE_2_5
        _ => panic!("Invalid OPERATE legacy M-sequence configuration"), // Default fallback (should not occur with valid config)
    }
}

/// Returns whether the device is configured to use interleaved mode for M-sequences, when device is in legacy mode.
#[cfg(feature = "legacy_device")]
pub const fn interleaved_mode() -> bool {
    const OPERATE_M_SEQUENCE_LEGACY: crate::types::MsequenceType = operate_m_sequence_legacy();
    (OPERATE_M_SEQUENCE_LEGACY as u8) == (crate::types::MsequenceType::Type11 as u8)
}

/// Returns whether the device is configured to use interleaved mode for M-sequences.
#[cfg(not(feature = "legacy_device"))]
pub const fn interleaved_mode() -> bool {
    const OPERATE_M_SEQUENCE: crate::types::MsequenceType = operate_m_sequence();
    (OPERATE_M_SEQUENCE as u8) == (crate::types::MsequenceType::Type11 as u8)
}

/// **This is a prerequisite for the `operate_m_sequence` config parameter check its documentation**.
/// Returns the on-request data length (in octets) for legacy OPERATE mode M-sequences.
/// ## Returns
/// The on-request data length in octets for all possible M-sequences.
pub const fn operate_m_sequence_od_len() -> u8 {
    2u8
}

/// Returns the process data input length (PDin) for standard OPERATE mode M-sequences.
///
/// This function provides the PDin configuration that corresponds to the M-sequence type
/// configured in `operate_m_sequence()`. The return value is a tuple containing:
/// - The data length value
/// - A boolean indicating if the length is specified `in octets (true) or bits (false`
/// ## Returns
///
/// A tuple `(length, is_octets)` where:
/// - `length`: The process data input length (0-32 for octets, 0-255 for bits)
/// - `is_octets`: `true` if length is in octets, `false` if in bits
///
/// ## Configuration Note
///
/// This function must be configured according to your device's specific PDin requirements.
/// The default implementation returns `(0, false` indicating no process data input.
/// Modify the return value based on your device's actual PDin specification.
///
/// ## Specification Reference
///
/// - IO-Link Specification v1.1.4, Table A.10: OPERATE mode M-sequence types
/// - Section 7.3.4: Process data definitions and bit/octet specifications
pub const fn operate_m_sequence_pd_in_len() -> (u8, bool) {
    (3u8, true) // Default: No process data input

    // Example configurations:
    // (8, false   // 8 bits of process data input
    // (16, false  // 16 bits of process data input
    // (2, true)    // 2 octets of process data input
    // (32, true)   // 32 octets of process data input
}

/// Returns the process data input length (PDin) for standard OPERATE mode M-sequences.
///
/// This function provides the PDin configuration that corresponds to the M-sequence type
/// configured in `operate_m_sequence()`. The return value is a tuple containing:
/// - The data length value
/// - A boolean indicating if the length is specified `in octets (true) or bits (false`
/// ## Returns
///
/// A tuple `(length, is_octets)` where:
/// - `length`: The process data input length (0-32 for octets, 0-255 for bits)
/// - `is_octets`: `true` if length is in octets, `false` if in bits
///
/// ## Configuration Note
///
/// This function must be configured according to your device's specific PDin requirements.
/// The default implementation returns `(0, false` indicating no process data input.
/// Modify the return value based on your device's actual PDin specification.
///
/// ## Specification Reference
/// - IO-Link Specification v1.1.4, Table A.10: OPERATE mode M-sequence types
/// - Section 7.3.4: Process data definitions and bit/octet specifications
pub const fn operate_m_sequence_pd_out_len() -> (u8, bool) {
    (3u8, true)

    // Example configurations:
    // (8, false   // 8 bits of process data input
    // (16, false  // 16 bits of process data input
    // (2, true)    // 2 octets of process data input
    // (32, true)   // 32 octets of process data input
}

/// M-sequence types for the OPERATE mode (standard protocol) as per IO-Link Specification (Table A.10).
pub const fn operate_m_sequence_code() -> u8 {
    0u8 // Accepted values are 0, 1, 4, 5, 6, or 7
}

/// M-sequence types for the OPERATE mode (per IO-Link Specification, Table A.10).
///
/// This table outlines valid M-sequence types in the OPERATE mode for **Devices**, depending on the
/// on-request data and process data (PDin and PDout). These combinations determine the M-sequence
/// type that the Master and Device must use.
///
/// | OPERATE M-sequence code | On-request Data (Octets)   | PDin                     | PDout                    | M-sequence Type          |
/// |--------------------------|---------------------------|--------------------------|--------------------------|--------------------------|
/// | 0                        | 1                         | 0                        | 0                        | TYPE_0 (⚠️ see NOTE 1)   |
/// | 1                        | 2                         | 0                        | 0                        | TYPE_1_2                 |
/// | 6                        | 8                         | 0                        | 0                        | TYPE_1_V                 |
/// | 7                        | 32                        | 0                        | 0                        | TYPE_1_V                 |
/// | 0                        | 2                         | 3…32 octets              | 0…32 octets              | TYPE_1_1 / 1_2 interleaved (⚠️ see NOTE 3) |
/// | 0                        | 2                         | 0…32 octets              | 3…32 octets              | TYPE_1_1 / 1_2 interleaved (⚠️ see NOTE 3) |
/// | 0                        | 1                         | 1…8 bit                  | 0                        | TYPE_2_1                 |
/// | 0                        | 1                         | 9…16 bit                 | 0                        | TYPE_2_2                 |
/// | 0                        | 1                         | 0                        | 1…8 bit                  | TYPE_2_3                 |
/// | 0                        | 1                         | 0                        | 9…16 bit                 | TYPE_2_4                 |
/// | 0                        | 1                         | 1…8 bit                  | 1…8 bit                  | TYPE_2_5                 |
/// | 0                        | 1                         | 9…16 bit                 | 1…16 bit                 | TYPE_2_V (⚠️ see NOTE 2) |
/// | 0                        | 1                         | 1…16 bit                 | 9…16 bit                 | TYPE_2_V (⚠️ see NOTE 2) |
/// | 4                        | 1                         | 0…32 octets              | 3…32 octets              | TYPE_2_V                 |
/// | 4                        | 1                         | 3…32 octets              | 0…32 octets              | TYPE_2_V                 |
/// | 5                        | 2                         | >0 bit, octets           | ≥0 bit, octets           | TYPE_2_V                 |
/// | 5                        | 2                         | ≥0 bit, octets           | >0 bit, octets           | TYPE_2_V                 |
/// | 6                        | 8                         | >0 bit, octets           | ≥0 bit, octets           | TYPE_2_V                 |
/// | 6                        | 8                         | ≥0 bit, octets           | >0 bit, octets           | TYPE_2_V                 |
/// | 7                        | 32                        | >0 bit, octets           | ≥0 bit, octets           | TYPE_2_V                 |
/// | 7                        | 32                        | ≥0 bit, octets           | >0 bit, octets           | TYPE_2_V                 |
///
/// ---
///
/// ⚠️ **NOTE 1**: It is highly recommended for Devices **not to use TYPE_0**, to improve error detection when the Master restarts communication.
///
/// ⚠️ **NOTE 2**: Former `TYPE_2_6` has been deprecated in favor of `TYPE_2_V` due to inefficiency.
///
/// ⚠️ **NOTE 3**: Interleaved mode (`TYPE_1_1/1_2`) **must not** be implemented in Devices, but should be supported by Masters.
pub const fn operate_m_sequence() -> crate::types::MsequenceType {
    const PD_IN_LEN: (u8, bool) = operate_m_sequence_pd_in_len();
    const PD_OUT_LEN: (u8, bool) = operate_m_sequence_pd_out_len();
    match (
        operate_m_sequence_code(),
        operate_m_sequence_od_len(),
        PD_IN_LEN.0,
        PD_IN_LEN.1,
        PD_OUT_LEN.0,
        PD_OUT_LEN.1,
    ) {
        (0, 1, 0, false, 0, false) => crate::types::MsequenceType::Type0, // TYPE_0
        (1, 2, 0, false, 0, false) => crate::types::MsequenceType::Type12, // TYPE_1_2
        (6, 8, 0, false, 0, false) => crate::types::MsequenceType::Type1V, // TYPE_1_V
        (7, 32,0, false, 0, false) => crate::types::MsequenceType::Type1V, // TYPE_1_V
        (0, 2, 3..=32, true, 0..=32, true) => crate::types::MsequenceType::Type11, // TYPE_1_1/1_2 interleaved
        (0, 2, 0..=32, true, 3..=32, true) => crate::types::MsequenceType::Type11, // TYPE_1_1/1_2 interleaved
        (0, 1, 1..=8, false, 0, false) => crate::types::MsequenceType::Type21,     // TYPE_2_1
        (0, 1, 9..=16, false, 0, false) => crate::types::MsequenceType::Type22,    // TYPE_2_2
        (0, 1, 0, false, 1..=8, false) => crate::types::MsequenceType::Type23,     // TYPE_2_3
        (0, 1, 0, false, 9..=16, false) => crate::types::MsequenceType::Type24,    // TYPE_2_4
        (0, 1, 1..=8, false, 1..=8, false) => crate::types::MsequenceType::Type25, // TYPE_2_5
        (0, 1, 9..=16, false, 1..=16, false) => crate::types::MsequenceType::Type2V, // TYPE_2_V
        (0, 1, 1..=16, false, 9..=16, false) => crate::types::MsequenceType::Type2V, // TYPE_2_V
        (4, 1, 0..=32, true, 3..=32, true) => crate::types::MsequenceType::Type2V, // TYPE_2_V
        (4, 1, 3..=32, true, 0..=32, true) => crate::types::MsequenceType::Type2V, // TYPE_2_V
        (5 | 6 | 7, 2 | 8 | 32, _, _, _, _) => crate::types::MsequenceType::Type2V,          // TYPE_2_V
        _ => panic!("Invalid OPERATE M-sequence configuration"), // Default fallback (should not occur with valid config)
    }
}

/// Represents the M-sequence type used in IO-Link communication, based on bits 6 and 7 of the
/// Checksum / M-sequence Type (CKT) field. These bits define how the Master structures messages
/// within an M-sequence, as specified in Table A.3 of the IO-Link specification (see Section A.1.3).
///
/// This macro depends on the `operate_m_sequence` or `operate_m_sequence_legacy` macros. The selected
/// M-sequence type in `operate_m_sequence` or `operate_m_sequence_legacy` determines the value of the
/// `operate_m_sequence_base_type` macro.
///
/// The mapping is as follows:
/// - `TYPE_0`   → `0`
/// - `TYPE_1_x` → `1`
/// - `TYPE_2_x` → `2`
/// - `3` is reserved and should not be used
///
/// Ensure consistency with the selected M-sequence type when defining dependent macros.
pub const fn operate_m_sequence_base_type() -> u8 {
    1u8 // Accepted values are 0, 1, 2 and 3 but reserved.
}

/// ## B.1.4 M-sequenceCapability
/// Bit 0: ISDU
// This bit indicates whether or not the ISDU communication channel is supported. Permissible
// values for ISDU are listed in Table B.4.
/// - `0` = ISDU not supported
/// - `1` = ISDU supported
pub const fn isdu_supported() -> u8 {
    1u8 // Accepted values 1 for ISDU support, 0 for no support
}

/// ## M-sequenceCapability (B.1.4)
///
/// The structure of the `M-sequenceCapability` parameter is defined as follows:
///
/// ```text
/// Bit 7   | Bit 6   | Bit 5 - Bit 4  | Bit 3 - Bit 1  | Bit 0
/// ------- | ------- | -------------- | -------------- | -------
/// Reserved| Reserved| PREOP.         | OPERATE        | ISDU
///         |         | M-seq code     | M-seq code     |
/// ```
///
/// ### Bit 0: ISDU
/// - `0` = ISDU not supported  
/// - `1` = ISDU supported
///
/// ### Bits 1 to 3: OPERATE M-sequence type
/// Encodes the M-sequence type for the OPERATE state.  
/// - See Table A.9 for legacy Devices  
/// - See Table A.10 for standard Devices
///
/// ### Bits 4 to 5: PREOPERATE M-sequence type
/// Encodes the M-sequence type for the PREOPERATE state.  
/// - See Table A.8
///
/// ### Bits 6 to 7: Reserved
/// - These bits are reserved and shall be set to `0`
pub const fn config_m_sequence_capability() -> u8 {
    (preoperate_m_sequence() << 4) | ((operate_m_sequence() as u8) << 1) | isdu_supported()
}
