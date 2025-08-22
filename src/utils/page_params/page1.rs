use bitfields::bitfield;

/// Minimum cycle time configuration as per Annex B.1.
///
/// This bitfield configures the minimum cycle time that the device
/// supports, which is used by the master for timing coordination.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Annex B.1: Direct Parameter Page 1
/// - Table B.3: MinCycleTime parameter restrictions
#[bitfield(u8)]
#[derive(Clone)]
pub struct MinCycleTime {
    /// Bits 6 to 7: Time Base
    /// 
    /// These bits specify the time base for the calculation of MasterCycleTime
    /// and MinCycleTime. In the following cases, when:
    /// * the Device provides no MinCycleTime, which is indicated by a MinCycleTime
    ///   equal zero (binary code 0x00),
    /// * or the MinCycleTime is shorter than the calculated M-sequence time with
    ///   the M-sequence type used by the Device, with (t1, t2, tidle) equal zero
    ///   and tA equal one bit time (see A.3.4 to A.3.6)
    #[bits(2)]
    pub time_base: u8,
    
    /// Bits 0 to 5: Multiplier
    /// 
    /// These bits contain a 6-bit multiplier for the calculation of
    /// MasterCycleTime and MinCycleTime. Permissible values for the multiplier
    /// are 0 to 63, further restrictions see Table B.3.
    #[bits(6)]
    pub multiplier: u8,
}


///
/// Represents the M-sequenceCapability parameter as defined in IO-Link Specification v1.1.4, Section B.1.4 (see Figure B.3).
///
/// This parameter encodes the device's support for ISDU communication and the available M-sequence types
/// during the OPERATE and PREOPERATE states. The structure of the byte is as follows:
///
/// ```text
///  7   6   5   4   3   2   1   0
/// +---+---+---+---+---+---+---+---+
/// | R | R |PRE|PRE| O | O | O | I |
/// +---+---+---+---+---+---+---+---+
///   |   |   |   |   |   |   |   +-- Bit 0: ISDU (0 = not supported, 1 = supported)
///   |   |   |   |   |   +------+ Bits 1-3: OPERATE M-sequence code
///   |   +---+---+-------------- Bits 4-5: PREOPERATE M-sequence code
///   +---+---------------------- Bits 6-7: Reserved (must be 0)
/// ```
///
/// - **Bit 0 (ISDU):** Indicates whether the ISDU communication channel is supported.
///   - 0: ISDU not supported
///   - 1: ISDU supported
/// - **Bits 1-3 (OPERATE M-sequence code):** Codes the available M-sequence type during the OPERATE state.
/// - **Bits 4-5 (PREOPERATE M-sequence code):** Codes the available M-sequence type during the PREOPERATE state.
/// - **Bits 6-7 (Reserved):** Reserved, must be set to 0.
///
/// # Reference
/// - IO-Link Specification v1.1.4, Section B.1.4, Figure B.3
/// - Table B.4 – Values of ISDU

#[bitfield(u8)]
#[derive(Clone, Copy)]
pub struct MsequenceCapability {
    #[bits(1)]
    pub isdu: bool,
    #[bits(3)]
    pub operate_m_sequence: u8,
    #[bits(2)]
    pub preoperate_m_sequence: u8,
    #[bits(2)]
    __: u8, // Reserved bit, must be 0
}

/// Protocol revision identifier as per Annex B.1.
///
/// This bitfield contains the major and minor revision numbers
/// of the IO-Link protocol version implemented by the device.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Annex B.1: Direct Parameter Page 1
/// - Section B.1.4: RevisionID parameter
#[bitfield(u8)]
#[derive(Clone, Copy)]
pub struct RevisionId {
    /// Bits 0 to 3: Minor Revision
    /// 
    /// These bits contain the minor digit of the version number,
    /// for example 0 for the protocol version 1.0. Permissible
    /// values for MinorRev are 0x0 to 0xF.
    #[bits(4)]
    pub minor_rev: u8,
    
    /// Bits 4 to 7: Major Revision
    /// 
    /// These bits contain the major digit of the version number,
    /// for example 1 for the protocol version 1.0. Permissible
    /// values for MajorRev are 0x0 to 0xF.
    #[bits(4)]
    pub major_rev: u8,
}

/// Represents the ProcessDataIn parameter as defined in IO-Link Specification v1.1.4 Section B.1.6.
///
/// The ProcessDataIn parameter is a single byte (u8) structured as follows:
///
/// ```text
///  7   6   5   4   3   2   1   0
/// +---+---+---+---+---+---+---+---+
/// | B | S | R |      Length      |
/// +---+---+---+---+---+---+---+---+
///   |   |   |         |
///   |   |   |         +-- Bits 0-4: Length (length of input data)
///   |   |   +------------ Bit 5: Reserved (must be 0)
///   |   +---------------- Bit 6: SIO (0 = SIO mode not supported, 1 = SIO mode supported)
///   +-------------------- Bit 7: BYTE (0 = bit Process Data, 1 = byte Process Data)
/// ```
///
/// - **Bits 0-4 (Length):** Length of the input data (Process Data from Device to Master).
///   - Permissible values depend on the BYTE bit (see Table B.6 below).
/// - **Bit 5 (Reserved):** Reserved, must be set to 0.
/// - **Bit 6 (SIO):** Indicates if SIO (Switching Signal) mode is supported.
///   - 0: SIO mode not supported
///   - 1: SIO mode supported
/// - **Bit 7 (BYTE):** Indicates the unit for Length.
///   - 0: Length is in bits (bit Process Data)
///   - 1: Length is in bytes (octets, byte Process Data)
///
/// # Table B.6 – Permitted combinations of BYTE and Length
///
/// | BYTE | Length      | Definition                                         |
/// |------|-------------|----------------------------------------------------|
/// | 0    | 0           | no Process Data                                    |
/// | 0    | 1           | 1 bit Process Data, structured in bits              |
/// | 0    | n (2–15)    | n bit Process Data, structured in bits              |
/// | 0    | 16          | 16 bit Process Data, structured in bits             |
/// | 0    | 17–31       | Reserved                                           |
/// | 1    | 0, 1        | Reserved                                           |
/// | 1    | 2           | 3 octets Process Data, structured in octets         |
/// | 1    | n (3–30)    | n+1 octets Process Data, structured in octets       |
/// | 1    | 31          | 32 octets Process Data, structured in octets        |
///
/// # Reference
/// - IO-Link Specification v1.1.4, Section B.1.6, Figure B.5
/// - Table B.6 – Permitted combinations of BYTE and Length
#[bitfield(u8)]
#[derive(Clone, Copy)]
pub struct ProcessDataIn {
    #[bits(1)]
    pub byte: bool,
    #[bits(1)]
    __: u8, // Reserved bit, must be 0
    #[bits(1)]
    pub sio: bool,
    #[bits(5)]
    pub length: u8,
}

/// Represents the ProcessDataIn parameter as defined in IO-Link Specification v1.1.4 Section B.1.6.
///
/// The ProcessDataIn parameter is a single byte (u8) structured as follows:
///
/// ```text
///  7   6   5   4   3   2   1   0
/// +---+---+---+---+---+---+---+---+
/// | B | S | R |      Length      |
/// +---+---+---+---+---+---+---+---+
///   |   |   |         |
///   |   |   |         +-- Bits 0-4: Length (length of input data)
///   |   |   +------------ Bit 5: Reserved (must be 0)
///   |   +---------------- Bit 6: SIO (0 = SIO mode not supported, 1 = SIO mode supported)
///   +-------------------- Bit 7: BYTE (0 = bit Process Data, 1 = byte Process Data)
/// ```
///
/// - **Bits 0-4 (Length):** Length of the input data (Process Data from Device to Master).
///   - Permissible values depend on the BYTE bit (see Table B.6 below).
/// - **Bit 5 (Reserved):** Reserved, must be set to 0.
/// - **Bit 6 (SIO):** Indicates if SIO (Switching Signal) mode is supported.
///   - 0: SIO mode not supported
///   - 1: SIO mode supported
/// - **Bit 7 (BYTE):** Indicates the unit for Length.
///   - 0: Length is in bits (bit Process Data)
///   - 1: Length is in bytes (octets, byte Process Data)
///
/// # Table B.6 – Permitted combinations of BYTE and Length
///
/// | BYTE | Length      | Definition                                         |
/// |------|-------------|----------------------------------------------------|
/// | 0    | 0           | no Process Data                                    |
/// | 0    | 1           | 1 bit Process Data, structured in bits              |
/// | 0    | n (2–15)    | n bit Process Data, structured in bits              |
/// | 0    | 16          | 16 bit Process Data, structured in bits             |
/// | 0    | 17–31       | Reserved                                           |
/// | 1    | 0, 1        | Reserved                                           |
/// | 1    | 2           | 3 octets Process Data, structured in octets         |
/// | 1    | n (3–30)    | n+1 octets Process Data, structured in octets       |
/// | 1    | 31          | 32 octets Process Data, structured in octets        |
///
/// # Reference
/// - IO-Link Specification v1.1.4, Section B.1.6, Figure B.5
/// - Table B.6 – Permitted combinations of BYTE and Length
#[bitfield(u8)]
#[derive(Clone, Copy)]
pub struct ProcessDataOut {
    #[bits(1)]
    pub byte: bool,
    #[bits(2)]
    __: u8, // Reserved bit, must be 0
    #[bits(5)]
    pub length: u8,
}

/// Device identification parameters.
///
/// This struct contains the device identification information
/// including vendor ID, device ID, and function ID.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Section 7.3.4.1: Device Identification
/// - Annex B.1: Direct Parameter Page 1 (VendorID1, VendorID2, DeviceID1-3)
#[derive(Clone, Debug, Default)]
pub struct DeviceIdent {
    /// Vendor ID (VID) - 16-bit vendor identification
    pub vendor_id: [u8; 2],
    /// Device ID (DID) - 24-bit device identification
    pub device_id: [u8; 3],
    /// Function ID (FID) - 16-bit function identification (reserved)
    pub function_id: [u8; 2],
}