pub mod pre_operate {
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
    pub const fn od_length() -> u8 {
        // PREOPERATE OD length configuration accepted length values are 1, 2, 8, 32
        const PRE_OP_OD_LEN: u8 = /*CONFIG:PRE_OP_OD_LEN*/ 2 /*ENDCONFIG*/;

        match PRE_OP_OD_LEN {
            1 => 1u8,
            2 => 2u8,
            8 => 8u8,
            32 => 32u8,
            _ => panic!("Invalid PREOPERATE OD length configuration"),
        }
    }
}

pub mod operate {
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
    pub const fn od_length() -> u8 {
        const OP_OD_LEN: u8 = /*CONFIG:OP_OD_LEN*/ 32 /*ENDCONFIG*/;
        match OP_OD_LEN {
            1 => 1u8,
            2 => 2u8,
            8 => 8u8,
            32 => 32u8,
            _ => panic!("Invalid OPERATE M-sequence configuration"),
        }
    }
}

/// Returns the maximum on-request data length (in octets) for all operating modes.
pub const fn max_possible_od_length() -> u8 {
    const OPERATE_OD_LENGTH: u8 = operate::od_length();
    const PRE_OPERATE_OD_LENGTH: u8 = pre_operate::od_length();
    if OPERATE_OD_LENGTH > PRE_OPERATE_OD_LENGTH {
        OPERATE_OD_LENGTH
    } else {
        PRE_OPERATE_OD_LENGTH
    }
}
