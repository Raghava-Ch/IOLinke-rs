//! IOLinke-macros: Procedural macros for IO-Link Device Stack
//!
//! This crate provides procedural macros to simplify IO-Link device implementation
//! according to IO-Link Specification v1.1.4.
use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, ExprLit, ItemEnum, Lit, Variant};
use syn::{Token, Type};
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
};

/// Master Command values according to IO-Link Specification v1.1.4
#[proc_macro]
pub fn master_command(input: TokenStream) -> TokenStream {
    let command_ident = parse_macro_input!(input as syn::Ident);

    let hex_value = match command_ident.to_string().as_str() {
        "Fallback" => 0x5Au8,
        "MasterIdent" => 0x95u8,
        "DeviceIdent" => 0x96u8,
        "DeviceStartup" => 0x97u8,
        "ProcessDataOutputOperate" => 0x98u8,
        "DeviceOperate" => 0x99u8,
        "DevicePreoperate" => 0x9Au8,
        _ => panic!("Unknown master command: {}", command_ident),
    };

    let expanded = quote! {
        #hex_value
    };

    TokenStream::from(expanded)
}

/// Resolves IO-Link direct parameter identifiers to their corresponding addresses.
///
/// This function maps parameter names to their standardized IO-Link addresses according
/// to the IO-Link specification v1.1. Direct parameters are organized into two pages:
///
/// ## Direct Parameter Page 1 (0x00-0x0F) - Standard Parameters
///
/// | Address | Parameter | Access | Description |
/// |---------|-----------|--------|-------------|
/// | 0x00u8 | `MasterCommand` | W | Master command to switch to operating states |
/// | 0x01u8 | `MasterCycleTime` | R/W | Actual cycle duration used by Master |
/// | 0x02u8 | `MinCycleTime` | R | Minimum cycle duration supported by Device |
/// | 0x03u8 | `MSequenceCapability` | R | M-sequences and physical configuration options |
/// | 0x04u8 | `RevisionID` | R/W | Protocol version ID (shall be 0x11) |
/// | 0x05u8 | `ProcessDataIn` | R | Input data type and length (Device to Master) |
/// | 0x06u8 | `ProcessDataOut` | R | Output data type and length (Master to Device) |
/// | 0x07u8 | `VendorID1` | R | Vendor identification MSB |
/// | 0x08u8 | `VendorID2` | R | Vendor identification LSB |
/// | 0x09u8 | `DeviceID1` | R/W | Device identification Octet 2 (MSB) |
/// | 0x0Au8 | `DeviceID2` | R/W | Device identification Octet 1 |
/// | 0x0Bu8 | `DeviceID3` | R/W | Device identification Octet 0 (LSB) |
/// | 0x0Cu8 | `FunctionID1` | R | Reserved (MSB) |
/// | 0x0Du8 | `FunctionID2` | R | Reserved (LSB) |
/// | 0x0Eu8 | `Reserved0E` | R | Reserved |
/// | 0x0Fu8 | `SystemCommand` | W | Command interface for end user applications |
///
/// ## Direct Parameter Page 2 (0x10-0x1F) - Vendor Specific
///
/// Addresses 0x10-0x1F are reserved for vendor-specific parameters.
///
/// # Parameters
///
/// * `param_ident` - The parameter identifier as a string slice
///
/// # Returns
///
/// Returns the corresponding 8-bit address for the given parameter.
///
/// # Panics
///
/// Panics if the provided parameter identifier is not recognized.
///
/// # Examples
///
/// ```rust
/// let addr = match "VendorID1".to_string().as_str() { /* ... */ };
/// assert_eq!(addr, 0x07u8);
/// ```
#[proc_macro]
pub fn direct_parameter_address(input: TokenStream) -> TokenStream {
    let param_ident = parse_macro_input!(input as syn::Ident);
    let address = match param_ident.to_string().as_str() {
        // Direct Parameter Page 1 (0x00-0x0F)
        "MasterCommand" => 0x00u8, // W, Mandatory - Master command to switch to operating states
        "MasterCycleTime" => 0x01u8, // R/W, Mandatory - Actual cycle duration used by Master
        "MinCycleTime" => 0x02u8,  // R, Mandatory - Minimum cycle duration supported by Device
        "MSequenceCapability" => 0x03u8, // R, Mandatory - M-sequences and physical configuration options
        "RevisionID" => 0x04u8,          // R/W, Mandatory - Protocol version ID (shall be 0x11)
        "ProcessDataIn" => 0x05u8, // R, Mandatory - Input data type and length (Device to Master)
        "ProcessDataOut" => 0x06u8, // R, Mandatory - Output data type and length (Master to Device)
        "VendorID1" => 0x07u8,     // R, Mandatory - Vendor identification MSB
        "VendorID2" => 0x08u8,     // R, Mandatory - Vendor identification LSB
        "DeviceID1" => 0x09u8,     // R/W, Mandatory - Device identification Octet 2 (MSB)
        "DeviceID2" => 0x0Au8,     // R/W, Mandatory - Device identification Octet 1
        "DeviceID3" => 0x0Bu8,     // R/W, Mandatory - Device identification Octet 0 (LSB)
        "FunctionID1" => 0x0Cu8,   // R, Optional - Reserved (MSB)
        "FunctionID2" => 0x0Du8,   // R, Optional - Reserved (LSB)
        "Reserved0E" => 0x0Eu8,    // R, Reserved
        "SystemCommand" => 0x0Fu8, // W, Optional - Command interface for end user applications

        // Direct Parameter Page 2 (0x10-0x1F) - Vendor Specific
        "VendorSpecific10" => 0x10u8,
        "VendorSpecific11" => 0x11u8,
        "VendorSpecific12" => 0x12u8,
        "VendorSpecific13" => 0x13u8,
        "VendorSpecific14" => 0x14u8,
        "VendorSpecific15" => 0x15u8,
        "VendorSpecific16" => 0x16u8,
        "VendorSpecific17" => 0x17u8,
        "VendorSpecific18" => 0x18u8,
        "VendorSpecific19" => 0x19u8,
        "VendorSpecific1A" => 0x1Au8,
        "VendorSpecific1B" => 0x1Bu8,
        "VendorSpecific1C" => 0x1Cu8,
        "VendorSpecific1D" => 0x1Du8,
        "VendorSpecific1E" => 0x1Eu8,
        "VendorSpecific1F" => 0x1Fu8,

        _ => panic!("Unknown direct parameter: {}", param_ident),
    };

    let expanded = quote! {
        #address
    };

    TokenStream::from(expanded)
}

/// ISDU Flow Control values according to IO-Link Specification v1.1.4
///
/// Flow control values are used in ISDU (Index-based Service Data Unit) communication
/// to manage the transmission of service data between Master and Device.
///
/// ## Flow Control Definitions
///
/// | Value | Name | Description |
/// |-------|------|-------------|
/// | 0x00u8-0x0Fu8 | COUNT | M-sequence counter within an ISDU. Increments beginning with 1 after an ISDU START. Jumps back from 15 to 0 in the event of an overflow. |
/// | 0x10u8 | START | Start of an ISDU I-Service, i.e., start of a request or a response. For the start of a request, any previously incomplete services may be rejected. For a start request associated with a response, a Device shall send "No Service" until its application returns response data. |
/// | 0x11u8 | IDLE_1 | No request for ISDU transmission. |
/// | 0x12u8 | IDLE_2 | Reserved for future use. No request for ISDU transmission. |
/// | 0x13u8-0x1Eu8 | Reserved | Reserved for future use. |
/// | 0x1Fu8 | ABORT | Abort entire service. The Master responds by rejecting received response data. The Device responds by rejecting received request data and may generate an abort. |
///
/// # Examples
///
/// ```rust
/// use iolinke_macros::flow_ctrl;
///
/// let start_value = flow_ctrl!(START);
/// assert_eq!(start_value, 0x10u8);
///
/// let abort_value = flow_ctrl!(ABORT);
/// assert_eq!(abort_value, 0x1Fu8);
/// ```
#[proc_macro]
pub fn flow_ctrl(input: TokenStream) -> TokenStream {
    let flow_ident = parse_macro_input!(input as syn::Ident);

    let hex_value = match flow_ident.to_string().as_str() {
        "START" => 0x10u8,  // 0b10000
        "IDLE_1" => 0x11u8, // 0b10001
        "IDLE_2" => 0x12u8, // 0b10010
        "ABORT" => 0x1Fu8,  // 0b11111
        _ => panic!(
            "Unknown flow control value: {}. Valid values are: START, IDLE_1, IDLE_2, ABORT",
            flow_ident
        ),
    };

    let expanded = quote! {
        #hex_value
    };

    TokenStream::from(expanded)
}

/// IO-Link Device Event Codes according to IO-Link Specification v1.1.4
///
/// Device Event Codes are used to report the current Device status and specific
/// events that occur during Device operation. Each event code has an associated
/// definition, recommended maintenance action, preferred DeviceStatus value, and type.
///
/// ## Event Code Categories
///
/// ## Event Code Reference Table
///
/// | EventCode ID | Definition and recommended maintenance action | Preferred DeviceStatus Value | Type |
/// |--------------|---------------------------------------------|------------------------------|------|
/// | 0x0000 | No malfunction | 0 | Notification |
/// | 0x0001 to 0x0FFF | Reserved | | |
/// | 0x1000 | General malfunction – unknown error | 4 | Error |
/// | 0x1001 to 0x17FF | Reserved | | |
/// | 0x1800 to 0x18FF | Vendor specific | | |
/// | 0x1900 to 0x3FFF | Reserved | | |
/// | 0x4000 | Temperature fault – Overload | 4 | Error |
/// | 0x4001 to 0x420F | Reserved | | |
/// | 0x4210 | Device temperature overrun – Clear source of heat | 2 | Warning |
/// | 0x4211 to 0x421F | Reserved | | |
/// | 0x4220 | Device temperature underrun – Insulate Device | 2 | Warning |
/// | 0x4221 to 0x4FFF | Reserved | | |
/// | 0x5000 | Device hardware fault – Device exchange | 4 | Error |
/// | 0x5001 to 0x500F | Reserved | | |
/// | 0x5010 | Component malfunction – Repair or exchange | 4 | Error |
/// | 0x5011 | Non volatile memory loss – Check batteries | 4 | Error |
/// | 0x5012 | Batteries low – Exchange batteries | 2 | Warning |
/// | 0x5013 to 0x50FF | Reserved | | |
/// | 0x5100 | General power supply fault – Check availability | 4 | Error |
/// | 0x5101 | Fuse blown/open – Exchange fuse | 4 | Error |
/// | 0x5102 to 0x510F | Reserved | | |
/// | 0x5110 | Primary supply voltage overrun – Check tolerance | 2 | Warning |
/// | 0x5111 | Primary supply voltage underrun – Check tolerance | 2 | Warning |
/// | 0x5112 | Secondary supply voltage fault (Port Class B) – Check tolerance | 2 | Warning |
/// | 0x5113 to 0x5FFF | Reserved | | |
/// | 0x6000 | Device software fault – Check firmware revision | 4 | Error |
/// | 0x6001 to 0x631F | Reserved | | |
/// | 0x6320 | Parameter error – Check data sheet and values | 4 | Error |
/// | 0x6321 | Parameter missing – Check data sheet | 4 | Error |
/// | 0x6322 to 0x634F | Reserved | | |
/// | 0x6350 | Reserved | | |
/// | 0x6351 to 0x76FF | Reserved | | |
/// | 0x7700 | Wire break of a subordinate device – Check installation | 4 | Error |
/// | 0x7701 to 0x770F | Wire break of subordinate device 1 …device 15 – Check installation | 4 | Error |
/// | 0x7710 | Short circuit – Check installation | 4 | Error |
/// | 0x7711 | Ground fault – Check installation | 4 | Error |
/// | 0x7712 to 0x8BFF | Reserved | | |
/// | 0x8C00 | Technology specific application fault – Reset Device | 4 | Error |
/// | 0x8C01 | Simulation active – Check operational mode | 3 | Warning |
/// | 0x8C02 to 0x8C0F | Reserved | | |
/// | 0x8C10 | Process variable range overrun – Process Data uncertain | 2 | Warning |
/// | 0x8C11 to 0x8C1F | Reserved | | |
/// | 0x8C20 | Measurement range exceeded – Check application | 4 | Error |
/// | 0x8C21 to 0x8C2F | Reserved | | |
/// | 0x8C30 | Process variable range underrun – Process Data uncertain | 2 | Warning |
/// | 0x8C31 to 0x8C3F | Reserved | | |
/// | 0x8C40 | Maintenance required – Cleaning | 1 | Warning |
/// | 0x8C41 | Maintenance required – Refill | 1 | Warning |
/// | 0x8C42 | Maintenance required – Exchange wear and tear parts | 1 | Warning |
/// | 0x8C43 to 0x8C9F | Reserved | | |
/// | 0x8CA0 to 0x8DFF | Vendor specific | | |
/// | 0x8E00 to 0xAFFF | Reserved | | |
/// | 0xB000 to 0xB0FF | Reserved for Safety extensions | See [10] in IO-Link spec 1.1.4 | See [10] in IO-Link spec 1.1.4 |
/// | 0xB100 to 0xBFFF | Reserved for profiles | | |
/// | 0xC000 to 0xFF90 | Reserved | | |
/// | 0xFF91 | Data Storage upload request ("DS_UPLOAD_REQ") – internal, not visible to user | 0 | Notification (single shot) |
/// | 0xFF92 to 0xFFAF | Reserved | | |
/// | 0xFFB0 to 0xFFB7 | Reserved for Wireless extensions | See [11] in IO-Link spec 1.1.4 | See [11] in IO-Link spec 1.1.4 |
/// | 0xFFB8 to 0xFFFF | Reserved | | |
///
/// # Examples
///
/// ```rust
/// use iolinke_macros::device_event_code;
///
/// let no_malfunction = device_event_code!(NO_MALFUNCTION);
/// assert_eq!(no_malfunction, 0x0000u16);
///
/// let temp_fault = device_event_code!(TEMPERATURE_FAULT_OVERLOAD);
/// assert_eq!(temp_fault, 0x4000u16);
/// ```
#[proc_macro]
pub fn device_event_code(input: TokenStream) -> TokenStream {
    let event_ident = parse_macro_input!(input as syn::Ident);

    let hex_value = match event_ident.to_string().as_str() {
        // 0x0000 - No malfunction
        "NO_MALFUNCTION" => 0x0000u16,

        // 0x1000-0x1FFF - General malfunctions
        "GENERAL_MALFUNCTION_UNKNOWN_ERROR" => 0x1000u16,

        // 0x4000-0x4FFF - Temperature faults
        "TEMPERATURE_FAULT_OVERLOAD" => 0x4000u16,
        "DEVICE_TEMPERATURE_OVERRUN" => 0x4210u16,
        "DEVICE_TEMPERATURE_UNDERRUN" => 0x4220u16,

        // 0x5000-0x5FFF - Hardware and power supply faults
        "DEVICE_HARDWARE_FAULT" => 0x5000u16,
        "COMPONENT_MALFUNCTION" => 0x5010u16,
        "NON_VOLATILE_MEMORY_LOSS" => 0x5011u16,
        "BATTERIES_LOW" => 0x5012u16,
        "GENERAL_POWER_SUPPLY_FAULT" => 0x5100u16,
        "FUSE_BLOWN_OPEN" => 0x5101u16,
        "PRIMARY_SUPPLY_VOLTAGE_OVERRUN" => 0x5110u16,
        "PRIMARY_SUPPLY_VOLTAGE_UNDERRUN" => 0x5111u16,
        "SECONDARY_SUPPLY_VOLTAGE_FAULT" => 0x5112u16,

        // 0x6000-0x6FFF - Software and parameter faults
        "DEVICE_SOFTWARE_FAULT" => 0x6000u16,
        "PARAMETER_ERROR" => 0x6320u16,
        "PARAMETER_MISSING" => 0x6321u16,

        // 0x7700-0x8BFF - Installation and wiring faults
        "WIRE_BREAK_SUBORDINATE_DEVICE" => 0x7700u16,
        "WIRE_BREAK_SUBORDINATE_DEVICE_1" => 0x7701u16,
        "WIRE_BREAK_SUBORDINATE_DEVICE_2" => 0x7702u16,
        "WIRE_BREAK_SUBORDINATE_DEVICE_3" => 0x7703u16,
        "WIRE_BREAK_SUBORDINATE_DEVICE_4" => 0x7704u16,
        "WIRE_BREAK_SUBORDINATE_DEVICE_5" => 0x7705u16,
        "WIRE_BREAK_SUBORDINATE_DEVICE_6" => 0x7706u16,
        "WIRE_BREAK_SUBORDINATE_DEVICE_7" => 0x7707u16,
        "WIRE_BREAK_SUBORDINATE_DEVICE_8" => 0x7708u16,
        "WIRE_BREAK_SUBORDINATE_DEVICE_9" => 0x7709u16,
        "WIRE_BREAK_SUBORDINATE_DEVICE_10" => 0x770Au16,
        "WIRE_BREAK_SUBORDINATE_DEVICE_11" => 0x770Bu16,
        "WIRE_BREAK_SUBORDINATE_DEVICE_12" => 0x770Cu16,
        "WIRE_BREAK_SUBORDINATE_DEVICE_13" => 0x770Du16,
        "WIRE_BREAK_SUBORDINATE_DEVICE_14" => 0x770Eu16,
        "WIRE_BREAK_SUBORDINATE_DEVICE_15" => 0x770Fu16,
        "SHORT_CIRCUIT" => 0x7710u16,
        "GROUND_FAULT" => 0x7711u16,

        // 0x8C00-0x8DFF - Application faults
        "TECHNOLOGY_SPECIFIC_APPLICATION_FAULT" => 0x8C00u16,
        "SIMULATION_ACTIVE" => 0x8C01u16,
        "PROCESS_VARIABLE_RANGE_OVERRUN" => 0x8C10u16,
        "MEASUREMENT_RANGE_EXCEEDED" => 0x8C20u16,
        "PROCESS_VARIABLE_RANGE_UNDERRUN" => 0x8C30u16,
        "MAINTENANCE_REQUIRED_CLEANING" => 0x8C40u16,
        "MAINTENANCE_REQUIRED_REFILL" => 0x8C41u16,
        "MAINTENANCE_REQUIRED_EXCHANGE_PARTS" => 0x8C42u16,

        // 0xFF91-0xFFFF - Internal and extensions
        "DS_UPLOAD_REQ" => 0xFF91u16,

        _ => panic!(
            "Unknown device event code: {}. Check IO-Link specification v1.1.4 for valid event codes.",
            event_ident
        ),
    };

    let expanded = quote! {
        #hex_value
    };

    TokenStream::from(expanded)
}

/// IO-Link Service Error Codes according to IO-Link Specification v1.1.4
///
/// Service Error Codes are used in ISDU (Index-based Service Data Unit) communication
/// to indicate various error conditions during service execution. Each error code
/// consists of an Error Code and Additional Code pair.
///
/// ## Error Code Categories
///
/// | Error Code | Additional Code | Name | Definition |
/// |------------|-----------------|------|------------|
/// | 0x80       | 0x00      | APP_DEV | Device application error – no details |
/// | 0x80       | 0x11      | IDX_NOTAVAIL | Index not available |
/// | 0x80       | 0x12      | SUBIDX_NOTAVAIL | Subindex not available |
/// | 0x80       | 0x20      | SERV_NOTAVAIL | Service temporarily not available |
/// | 0x80       | 0x21      | SERV_NOTAVAIL_LOCCTRL | Service temporarily not available – local control |
/// | 0x80       | 0x22      | SERV_NOTAVAIL_DEVCTRL | Service temporarily not available – Device control |
/// | 0x80       | 0x23      | IDX_NOT_ACCESSIBLE | Access denied |
/// | 0x80       | 0x30      | PAR_VALUEUTOFRNG | Parameter value out of range |
/// | 0x80       | 0x31      | PAR_VALUGTLIM | Parameter value above limit |
/// | 0x80       | 0x32      | PAR_VALUGTLIM | Parameter value below limit |
/// | 0x80       | 0x33      | VAL_LENOVRRUN | Parameter length overrun |
/// | 0x80       | 0x34      | VAL_LENUNDRUN | Parameter length underrun |
/// | 0x80       | 0x35      | FUNC_NOTAVAIL | Function not available |
/// | 0x80       | 0x36      | FUNC_UNAVAILTEMP | Function temporarily unavailable |
/// | 0x80       | 0x40      | PAR_SETINVALID | Invalid parameter set |
/// | 0x80       | 0x41      | PAR_SETINCONSIST | Inconsistent parameter set |
/// | 0x80       | 0x82      | APP_DEVNOTRDY | Application not ready |
/// | 0x81       | 0x00      | UNSPECIFIC | Vendor specific |
/// | 0x81       | 0x01-0xFF | VENDOR_SPECIFIC | Vendor specific |
///
/// # Examples
///
/// ```rust
/// use iolinke_macros::service_error_code;
///
/// let (error_code, additional_code) = service_error_code!(APP_DEV);
/// assert_eq!(error_code, 0x80u8);
/// assert_eq!(additional_code, 0x00u8);
///
/// let (error_code, additional_code) = service_error_code!(IDX_NOTAVAIL);
/// assert_eq!(error_code, 0x80u8);
/// assert_eq!(additional_code, 0x11u8);
/// ```
#[proc_macro]
pub fn isdu_error_code(input: TokenStream) -> TokenStream {
    let error_ident = parse_macro_input!(input as syn::Ident);

    let (error_code, additional_code) = match error_ident.to_string().as_str() {
        "APP_DEV" => (0x80u8, 0x00u8),
        "IDX_NOTAVAIL" => (0x80u8, 0x11u8),
        "SUBIDX_NOTAVAIL" => (0x80u8, 0x12u8),
        "SERV_NOTAVAIL" => (0x80u8, 0x20u8),
        "SERV_NOTAVAIL_LOCCTRL" => (0x80u8, 0x21u8),
        "SERV_NOTAVAIL_DEVCTRL" => (0x80u8, 0x22u8),
        "IDX_NOT_ACCESSIBLE" => (0x80u8, 0x23u8),
        "PAR_VALUEUTOFRNG" => (0x80u8, 0x30u8),
        "PAR_VALUGTLIM" => (0x80u8, 0x31u8),
        "PAR_VALUBTLIM" => (0x80u8, 0x32u8),
        "VAL_LENOVRRUN" => (0x80u8, 0x33u8),
        "VAL_LENUNDRUN" => (0x80u8, 0x34u8),
        "FUNC_NOTAVAIL" => (0x80u8, 0x35u8),
        "FUNC_UNAVAILTEMP" => (0x80u8, 0x36u8),
        "PAR_SETINVALID" => (0x80u8, 0x40u8),
        "PAR_SETINCONSIST" => (0x80u8, 0x41u8),
        "APP_DEVNOTRDY" => (0x80u8, 0x82u8),
        "UNSPECIFIC" => (0x81u8, 0x00u8),
        "VENDOR_SPECIFIC" => (0x81u8, 0x01u8), // Representative value for vendor specific range
        // Configure the `VENDOR_SPECIFIC` until the additional code 0xFF
        _ => panic!(
            "Unknown service error code: {}. Check IO-Link specification v1.1.4 for valid error codes.",
            error_ident
        ),
    };

    let expanded = quote! {
        (#error_code, #additional_code)
    };

    TokenStream::from(expanded)
}

/// Generates a comprehensive `DeviceParameterIndex` enum based on the IO-Link specification v1.1.4.
///
/// This procedural macro creates a strongly-typed representation of all device parameter indices
/// as defined in the IO-Link specification. It covers the complete range from 0x0000 to 0xFFFF,
/// categorizing parameters into standard, profile-specific, and device-specific ranges.
///
/// ## Index Ranges and Categories
///
/// ### Standard Parameters (0x0000-0x0030)
/// These are mandatory or conditional parameters that all IO-Link devices must support:
///
/// | Index | Name | Access | Data Type | M/O/C | Description |
/// |-------|------|--------|-----------|-------|-------------|
/// | 0x0000 | Direct Parameter Page 1 | R | RecordT | M | Redirected to page communication channel |
/// | 0x0001 | Direct Parameter Page 2 | R/W | RecordT | M | Redirected to page communication channel |
/// | 0x0002 | System-Command | W | UIntegerT | C | Command code definition (1 octet) |
/// | 0x0003 | Data-Storage-Index | R/W | RecordT | M | Set of data objects for storage |
/// | 0x000C | Device-Access-Locks | R/W | RecordT | O | Standardized device locking functions (2 octets) |
/// | 0x000D | Profile-Characteristic | R | ArrayT of UIntegerT16 | C | Reserved for Common Profile |
/// | 0x000E | PDInput-Descriptor | R | ArrayT of OctetStringT3 | C | Reserved for Common Profile |
/// | 0x000F | PDOutput-Descriptor | R | ArrayT of OctetStringT3 | C | Reserved for Common Profile |
/// | 0x0010 | Vendor-Name | R | StringT | M | Vendor information (max 64 octets) |
/// | 0x0011 | Vendor-Text | R | StringT | O | Additional vendor information (max 64 octets) |
/// | 0x0012 | Product-Name | R | StringT | M | Detailed product or type name (max 64 octets) |
/// | 0x0013 | ProductID | R | StringT | O | Product or type identification (max 64 octets) |
/// | 0x0014 | Product-Text | R | StringT | O | Description of device function (max 64 octets) |
/// | 0x0015 | Serial-Number | R | StringT | O | Vendor specific serial number (max 16 octets) |
/// | 0x0016 | Hardware-Revision | R | StringT | O | Vendor specific format (max 64 octets) |
/// | 0x0017 | Firmware-Revision | R | StringT | O | Vendor specific format (max 64 octets) |
/// | 0x0018 | Application-Specific-Tag | R/W | StringT | O | Tag defined by user (16-32 octets) |
/// | 0x0019 | Function-Tag | R/W | StringT | C | Reserved for Common Profile (max 32 octets) |
/// | 0x001A | Location-Tag | R/W | StringT | C | Reserved for Common Profile (max 32 octets) |
/// | 0x001B | Product-URI | R | StringT | C | Reserved for Common Profile (max 100 octets) |
/// | 0x0020 | ErrorCount | R | UIntegerT | O | Errors since power-on or reset (2 octets) |
/// | 0x0024 | Device-Status | R | UIntegerT | O | Current status of the device (1 octet) |
/// | 0x0025 | Detailed-Device-Status | R | ArrayT of OctetStringT3 | O | Detailed device status information |
/// | 0x0028 | Process-DataInput | R | Device specific | O | Read last valid process data from PDin channel |
/// | 0x0029 | Process-DataOutput | R | Device specific | O | Read last valid process data from PDout channel |
/// | 0x0030 | Offset-Time | R/W | RecordT | O | Synchronization of device timing to M-sequence (1 octet) |
///
/// ### Reserved Ranges
/// The following ranges are reserved and should not be used:
/// - 0x0004-0x000B: Reserved for exceptional operations
/// - 0x001C-0x001F: Reserved
/// - 0x0021-0x0023: Reserved
/// - 0x0026-0x0027: Reserved
/// - 0x002A-0x002F: Reserved
/// - 0x00FF: Reserved
/// - 0x5100-0xFFFF: Reserved
///
/// ### Profile-Specific Parameters (0x0031-0x003F)
/// Reserved for device profiles and common profile extensions.
///
/// ### Preferred Device-Specific Parameters (0x0040-0x00FE)
/// 8-bit range for device-specific parameters that are commonly used.
///
/// ### Extended Device-Specific Parameters (0x0100-0x3FFF)
/// 16-bit range for extended device-specific functionality.
///
/// ### Device Profile Parameters (0x4000-0x41FF, 0x4300-0x4FFF)
/// Reserved ranges for device profile specifications.
///
/// ### Safety System Extensions (0x4200-0x42FF)
/// Reserved for safety system extensions as defined in [10].
///
/// ### Wireless System Extensions (0x5000-0x50FF)
/// Reserved for wireless system extensions as defined in [11].
///
/// ## Generated Methods
///
/// The macro generates the following methods for the enum:
///
/// - `from_index(index: u16) -> Option<Self>`: Creates a parameter index from a raw value
/// - `index() -> u16`: Returns the raw index value
/// - `name() -> &'static str`: Returns the human-readable parameter name
/// - `category() -> IndexCategory`: Returns the parameter category
///
/// ## Usage Example
///
/// ```rust
/// use iolinke_macros::DeviceParameterIndex;
///
/// #[derive(DeviceParameterIndex)]
/// enum MyDeviceParameters {}
///
/// // Access standard parameters
/// let vendor_name = MyDeviceParameters::VendorName;
/// assert_eq!(vendor_name.index(), 0x0010);
/// assert_eq!(vendor_name.name(), "Vendor-Name");
/// assert_eq!(vendor_name.category(), IndexCategory::Standard);
///
/// // Create from raw index
/// let param = MyDeviceParameters::from_index(0x0012).unwrap();
/// assert_eq!(param, MyDeviceParameters::ProductName);
///
/// // Handle device-specific parameters
/// let custom_param = MyDeviceParameters::PreferredIndex(0x50);
/// assert_eq!(custom_param.index(), 0x50);
/// assert_eq!(custom_param.category(), IndexCategory::PreferredIndex);
/// ```
///
/// ## References
///
/// - IO-Link Specification v1.1.4, Section B.8 - Index assignment of data objects
/// - Common Profile [7] for profile-specific parameters
/// - Safety system extensions [10]
/// - Wireless system extensions [11]
///
/// ## Notes
///
/// - All standard parameters with M (Mandatory) status must be implemented by IO-Link devices
/// - Parameters with O (Optional) status may be implemented based on device capabilities
/// - Parameters with C (Conditional) status are required when specific profiles are implemented
/// - Reserved ranges should not be used for custom parameters to avoid conflicts
/// - The macro automatically handles range validation and provides type-safe access to all valid indices
#[proc_macro]
pub fn device_parameter_index(input: TokenStream) -> TokenStream {
    let name = parse_macro_input!(input as syn::Ident);

    let expanded = quote! {
        /// Represents all possible device parameter indices as defined in the specification.
        ///
        /// This enum categorizes parameters into different ranges with specific purposes:
        /// - Standard parameters (0x0000-0x0030)
        /// - Profile-specific parameters (0x0031-0x003F)
        /// - Preferred device-specific parameters (0x0040-0x00FE)
        /// - Extended device-specific parameters (0x0100-0x3FFF)
        /// - Various profile-specific ranges (0x4000-0x4FFF)
        /// - Safety and wireless extensions (0x4200-0x42FF, 0x5000-0x50FF)
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(u16)]
        pub enum #name {
            // Standard parameters (0x0000-0x0030)
            DirectParameterPage1 = 0x0000,
            DirectParameterPage2 = 0x0001,
            SystemCommand = 0x0002,
            DataStorageIndex = 0x0003,
            DeviceAccessLocks = 0x000C,
            ProfileCharacteristic = 0x000D,
            PDInputDescriptor = 0x000E,
            PDOutputDescriptor = 0x000F,
            VendorName = 0x0010,
            VendorText = 0x0011,
            ProductName = 0x0012,
            ProductID = 0x0013,
            ProductText = 0x0014,
            SerialNumber = 0x0015,
            HardwareRevision = 0x0016,
            FirmwareRevision = 0x0017,
            ApplicationSpecificTag = 0x0018,
            FunctionTag = 0x0019,
            LocationTag = 0x001A,
            ProductURI = 0x001B,
            ErrorCount = 0x0020,
            DeviceStatus = 0x0024,
            DetailedDeviceStatus = 0x0025,
            ProcessDataInput = 0x0028,
            ProcessDataOutput = 0x0029,
            OffsetTime = 0x0030,

            /// Profile-specific parameters range (0x0031-0x003F)
            ProfileSpecific(u16),
            /// Preferred device-specific parameters (8-bit) range (0x0040-0x00FE)
            PreferredIndex(u16),
            /// Extended device-specific parameters (16-bit) range (0x0100-0x3FFF)
            ExtendedIndex(u16),
            /// Device profile-specific parameters range (0x4000-0x41FF)
            DeviceProfileIndex(u16),
            /// Safety system extensions parameters range (0x4200-0x42FF)
            SafetySpecificIndex(u16),
            /// Secondary device profile-specific parameters range (0x4300-0x4FFF)
            SecondaryDeviceProfileIndex(u16),
            /// Wireless system extensions parameters range (0x5000-0x50FF)
            WirelessSpecificIndex(u16),
        }

        /// Device parameter index as defined by the IO-Link Specification.
        ///
        /// This enum represents the standard and extended parameter indices used for accessing
        /// device parameters in IO-Link devices. The variants cover:
        /// - Standard parameters (0x0000-0x0030), such as direct parameter pages, system commands, device identification, and process data descriptors.
        /// - Profile-specific and device-specific parameter ranges, including:
        ///   - ProfileSpecific: 0x0031-0x003F
        ///   - PreferredIndex: 0x0040-0x00FE (8-bit preferred device-specific)
        ///   - ExtendedIndex: 0x0100-0x3FFF (16-bit extended device-specific)
        ///   - DeviceProfileIndex: 0x4000-0x41FF (device profile-specific)
        ///   - SafetySpecificIndex: 0x4200-0x42FF (safety system extensions)
        ///   - SecondaryDeviceProfileIndex: 0x4300-0x4FFF (secondary device profile-specific)
        ///   - WirelessSpecificIndex: 0x5000-0x50FF (wireless system extensions)
        ///
        /// Use this enum to match or construct parameter indices for device parameter access,
        /// including vendor-specific and profile-specific extensions.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(u8)]
        pub enum DirectParameterPage1SubIndex {
            // Direct Parameter Page 1 (0x00-0x0F)
            MasterCommand = 0x00u8, // W, Mandatory - Master command to switch to operating states
            MasterCycleTime = 0x01u8, // R/W, Mandatory - Actual cycle duration used by Master
            MinCycleTime = 0x02u8,  // R, Mandatory - Minimum cycle duration supported by Device
            MSequenceCapability = 0x03u8, // R, Mandatory - M-sequences and physical configuration options
            RevisionID = 0x04u8,          // R/W, Mandatory - Protocol version ID (shall be 0x11)
            ProcessDataIn = 0x05u8, // R, Mandatory - Input data type and length (Device to Master)
            ProcessDataOut = 0x06u8, // R, Mandatory - Output data type and length (Master to Device)
            VendorID1 = 0x07u8,     // R, Mandatory - Vendor identification MSB
            VendorID2 = 0x08u8,     // R, Mandatory - Vendor identification LSB
            DeviceID1 = 0x09u8,     // R/W, Mandatory - Device identification Octet 2 (MSB)
            DeviceID2 = 0x0Au8,     // R/W, Mandatory - Device identification Octet 1
            DeviceID3 = 0x0Bu8,     // R/W, Mandatory - Device identification Octet 0 (LSB)
            FunctionID1 = 0x0Cu8,   // R, Optional - Reserved (MSB)
            FunctionID2 = 0x0Du8,   // R, Optional - Reserved (LSB)
            Reserved0E = 0x0Eu8,    // R, Reserved
            SystemCommand = 0x0Fu8, // W, Optional - Command interface for end user applications
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(u8)]
        pub enum DirectParameterPage2SubIndex {
            // Direct Parameter Page 2 (0x10-0x1F) - Vendor Specific
            VendorSpecific10 = 0x10u8,
            VendorSpecific11 = 0x11u8,
            VendorSpecific12 = 0x12u8,
            VendorSpecific13 = 0x13u8,
            VendorSpecific14 = 0x14u8,
            VendorSpecific15 = 0x15u8,
            VendorSpecific16 = 0x16u8,
            VendorSpecific17 = 0x17u8,
            VendorSpecific18 = 0x18u8,
            VendorSpecific19 = 0x19u8,
            VendorSpecific1A = 0x1Au8,
            VendorSpecific1B = 0x1Bu8,
            VendorSpecific1C = 0x1Cu8,
            VendorSpecific1D = 0x1Du8,
            VendorSpecific1E = 0x1Eu8,
            VendorSpecific1F = 0x1Fu8,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(u8)]
        pub enum DataStorageIndexSubIndex {
            DsCommand = 0x01u8,
            StateProperty = 0x02u8,
            DataStorageSize = 0x03u8,
            ParameterChecksum = 0x04u8,
            IndexList = 0x05u8,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(u8)]
        pub enum SubIndex {
            DpPage1(DirectParameterPage1SubIndex),
            DpPage2(DirectParameterPage2SubIndex),
            DataStorageIndex(DataStorageIndexSubIndex),
            VendorName,
        }
        impl #name {
            /// Creates a DeviceParameterIndex from a raw index value
            ///
            /// # Arguments
            /// * `index` - The raw index value to convert
            ///
            /// # Returns
            /// `Some(DeviceParameterIndex)` if the index is valid, `None` otherwise
            ///
            /// # Examples
            /// ```
            /// let param = DeviceParameterIndex::from_index(0x0010).unwrap();
            /// assert_eq!(param, DeviceParameterIndex::VendorName);
            /// ```
            pub fn from_index(index: u16) -> Option<Self> {
                match index {
                    // Standard parameters
                    0x0000 => Some(Self::DirectParameterPage1),
                    0x0001 => Some(Self::DirectParameterPage2),
                    0x0002 => Some(Self::SystemCommand),
                    0x0003 => Some(Self::DataStorageIndex),
                    0x000C => Some(Self::DeviceAccessLocks),
                    0x000D => Some(Self::ProfileCharacteristic),
                    0x000E => Some(Self::PDInputDescriptor),
                    0x000F => Some(Self::PDOutputDescriptor),
                    0x0010 => Some(Self::VendorName),
                    0x0011 => Some(Self::VendorText),
                    0x0012 => Some(Self::ProductName),
                    0x0013 => Some(Self::ProductID),
                    0x0014 => Some(Self::ProductText),
                    0x0015 => Some(Self::SerialNumber),
                    0x0016 => Some(Self::HardwareRevision),
                    0x0017 => Some(Self::FirmwareRevision),
                    0x0018 => Some(Self::ApplicationSpecificTag),
                    0x0019 => Some(Self::FunctionTag),
                    0x001A => Some(Self::LocationTag),
                    0x001B => Some(Self::ProductURI),
                    0x0020 => Some(Self::ErrorCount),
                    0x0024 => Some(Self::DeviceStatus),
                    0x0025 => Some(Self::DetailedDeviceStatus),
                    0x0028 => Some(Self::ProcessDataInput),
                    0x0029 => Some(Self::ProcessDataOutput),
                    0x0030 => Some(Self::OffsetTime),

                    // Profile-specific range (0x0031-0x003F)
                    x @ 0x0031..=0x003F => Some(Self::ProfileSpecific(x as u16)),

                    // Preferred index range (0x0040-0x00FE)
                    x @ 0x0040..=0x00FE => Some(Self::PreferredIndex(x as u16)),

                    // Extended index range (0x0100-0x3FFF)
                    x @ 0x0100..=0x3FFF => Some(Self::ExtendedIndex(x)),

                    // Device profile range (0x4000-0x41FF)
                    x @ 0x4000..=0x41FF => Some(Self::DeviceProfileIndex(x)),

                    // Safety specific range (0x4200-0x42FF)
                    x @ 0x4200..=0x42FF => Some(Self::SafetySpecificIndex(x)),

                    // Secondary device profile range (0x4300-0x4FFF)
                    x @ 0x4300..=0x4FFF => Some(Self::SecondaryDeviceProfileIndex(x)),

                    // Wireless specific range (0x5000-0x50FF)
                    x @ 0x5000..=0x50FF => Some(Self::WirelessSpecificIndex(x)),

                    // Reserved ranges (0x0004-0x000B, 0x001C-0x001F, 0x0021-0x0023, 0x0026-0x0027,
                    // 0x002A-0x002F, 0x00FF, 0x5100-0xFFFF)
                    _ => None,
                }
            }

            /// Returns the raw index value for this parameter
            ///
            /// # Examples
            /// ```
            /// let param = DeviceParameterIndex::VendorName;
            /// assert_eq!(param.index(), 0x0010);
            /// ```
            pub const fn index(&self) -> u16 {
                match *self {
                    // Standard parameters
                    Self::DirectParameterPage1 => 0x0000,
                    Self::DirectParameterPage2 => 0x0001,
                    Self::SystemCommand => 0x0002,
                    Self::DataStorageIndex => 0x0003,
                    Self::DeviceAccessLocks => 0x000C,
                    Self::ProfileCharacteristic => 0x000D,
                    Self::PDInputDescriptor => 0x000E,
                    Self::PDOutputDescriptor => 0x000F,
                    Self::VendorName => 0x0010,
                    Self::VendorText => 0x0011,
                    Self::ProductName => 0x0012,
                    Self::ProductID => 0x0013,
                    Self::ProductText => 0x0014,
                    Self::SerialNumber => 0x0015,
                    Self::HardwareRevision => 0x0016,
                    Self::FirmwareRevision => 0x0017,
                    Self::ApplicationSpecificTag => 0x0018,
                    Self::FunctionTag => 0x0019,
                    Self::LocationTag => 0x001A,
                    Self::ProductURI => 0x001B,
                    Self::ErrorCount => 0x0020,
                    Self::DeviceStatus => 0x0024,
                    Self::DetailedDeviceStatus => 0x0025,
                    Self::ProcessDataInput => 0x0028,
                    Self::ProcessDataOutput => 0x0029,
                    Self::OffsetTime => 0x0030,

                    // Parameter ranges
                    Self::ProfileSpecific(x) => x as u16,
                    Self::PreferredIndex(x) => x as u16,
                    Self::ExtendedIndex(x) => x,
                    Self::DeviceProfileIndex(x) => x,
                    Self::SafetySpecificIndex(x) => x,
                    Self::SecondaryDeviceProfileIndex(x) => x,
                    Self::WirelessSpecificIndex(x) => x,
                }
            }

            /// Returns the subindex for this parameter, if applicable.
            ///
            /// For most parameters, the subindex is 0. For parameter variants that
            /// encode a subindex, this method should be updated accordingly.
            ///
            /// # Examples
            /// ```
            /// let param = DeviceParameterIndex::VendorName;
            /// assert_eq!(param.subindex(), 0);
            /// ```
            pub const fn subindex(&self, subindex: SubIndex) -> u8 {
                // By default, subindex is 0. If you add variants with subindex, match them here.
                match (*self, subindex) {
                    (Self::DirectParameterPage1, SubIndex::DpPage1(DirectParameterPage1SubIndex::MasterCommand)) => 0x00u8,
                    (Self::DirectParameterPage1, SubIndex::DpPage1(DirectParameterPage1SubIndex::MasterCycleTime)) => 0x01u8,
                    (Self::DirectParameterPage1, SubIndex::DpPage1(DirectParameterPage1SubIndex::MinCycleTime)) => 0x02u8,
                    (Self::DirectParameterPage1, SubIndex::DpPage1(DirectParameterPage1SubIndex::MSequenceCapability)) => 0x03u8,
                    (Self::DirectParameterPage1, SubIndex::DpPage1(DirectParameterPage1SubIndex::RevisionID)) => 0x04u8,
                    (Self::DirectParameterPage1, SubIndex::DpPage1(DirectParameterPage1SubIndex::ProcessDataIn)) => 0x05u8,
                    (Self::DirectParameterPage1, SubIndex::DpPage1(DirectParameterPage1SubIndex::ProcessDataOut)) => 0x06u8,
                    (Self::DirectParameterPage1, SubIndex::DpPage1(DirectParameterPage1SubIndex::VendorID1)) => 0x07u8,
                    (Self::DirectParameterPage1, SubIndex::DpPage1(DirectParameterPage1SubIndex::VendorID2)) => 0x08u8,
                    (Self::DirectParameterPage1, SubIndex::DpPage1(DirectParameterPage1SubIndex::DeviceID1)) => 0x09u8,
                    (Self::DirectParameterPage1, SubIndex::DpPage1(DirectParameterPage1SubIndex::DeviceID2)) => 0x0Au8,
                    (Self::DirectParameterPage1, SubIndex::DpPage1(DirectParameterPage1SubIndex::DeviceID3)) => 0x0Bu8,
                    (Self::DirectParameterPage1, SubIndex::DpPage1(DirectParameterPage1SubIndex::FunctionID1)) => 0x0Cu8,
                    (Self::DirectParameterPage1, SubIndex::DpPage1(DirectParameterPage1SubIndex::FunctionID2)) => 0x0Du8,
                    (Self::DirectParameterPage1, SubIndex::DpPage1(DirectParameterPage1SubIndex::Reserved0E)) => 0x0Eu8,
                    (Self::DirectParameterPage1, SubIndex::DpPage1(DirectParameterPage1SubIndex::SystemCommand)) => 0x0Fu8,

                    (Self::DirectParameterPage2, SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific10)) => 0x10u8,
                    (Self::DirectParameterPage2, SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific11)) => 0x11u8,
                    (Self::DirectParameterPage2, SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific12)) => 0x12u8,
                    (Self::DirectParameterPage2, SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific13)) => 0x13u8,
                    (Self::DirectParameterPage2, SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific14)) => 0x14u8,
                    (Self::DirectParameterPage2, SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific15)) => 0x15u8,
                    (Self::DirectParameterPage2, SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific16)) => 0x16u8,
                    (Self::DirectParameterPage2, SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific17)) => 0x17u8,
                    (Self::DirectParameterPage2, SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific18)) => 0x18u8,
                    (Self::DirectParameterPage2, SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific19)) => 0x19u8,
                    (Self::DirectParameterPage2, SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific1A)) => 0x1Au8,
                    (Self::DirectParameterPage2, SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific1B)) => 0x1Bu8,
                    (Self::DirectParameterPage2, SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific1C)) => 0x1Cu8,
                    (Self::DirectParameterPage2, SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific1D)) => 0x1Du8,
                    (Self::DirectParameterPage2, SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific1E)) => 0x1Eu8,
                    (Self::DirectParameterPage2, SubIndex::DpPage2(DirectParameterPage2SubIndex::VendorSpecific1F)) => 0x1Fu8,

                    (Self::DataStorageIndex, SubIndex::DataStorageIndex(DataStorageIndexSubIndex::DsCommand)) => 0x01u8,
                    (Self::DataStorageIndex, SubIndex::DataStorageIndex(DataStorageIndexSubIndex::StateProperty)) => 0x02u8,
                    (Self::DataStorageIndex, SubIndex::DataStorageIndex(DataStorageIndexSubIndex::DataStorageSize)) => 0x03u8,
                    (Self::DataStorageIndex, SubIndex::DataStorageIndex(DataStorageIndexSubIndex::ParameterChecksum)) => 0x04u8,
                    (Self::DataStorageIndex, SubIndex::DataStorageIndex(DataStorageIndexSubIndex::IndexList)) => 0x05u8,

                    (Self::VendorName, SubIndex::VendorName) => 0x00u8,

                    _ => panic!("Invalid subindex for parameter"),
                }
            }

            /// Returns the human-readable name of the parameter
            ///
            /// # Examples
            /// ```
            /// let param = DeviceParameterIndex::VendorName;
            /// assert_eq!(param.name(), "Vendor-Name");
            /// ```
            pub fn name(&self) -> &'static str {
                match self {
                    // Standard parameters
                    Self::DirectParameterPage1 => "Direct Parameter Page 1",
                    Self::DirectParameterPage2 => "Direct Parameter Page 2",
                    Self::SystemCommand => "System-Command",
                    Self::DataStorageIndex => "Data-Storage-Index",
                    Self::DeviceAccessLocks => "Device-Access-Locks",
                    Self::ProfileCharacteristic => "Profile-Characteristic",
                    Self::PDInputDescriptor => "PDInput-Descriptor",
                    Self::PDOutputDescriptor => "PDOutput-Descriptor",
                    Self::VendorName => "Vendor-Name",
                    Self::VendorText => "Vendor-Text",
                    Self::ProductName => "Product-Name",
                    Self::ProductID => "ProductID",
                    Self::ProductText => "Product-Text",
                    Self::SerialNumber => "Serial-Number",
                    Self::HardwareRevision => "Hardware-Revision",
                    Self::FirmwareRevision => "Firmware-Revision",
                    Self::ApplicationSpecificTag => "Application-Specific-Tag",
                    Self::FunctionTag => "Function-Tag",
                    Self::LocationTag => "Location-Tag",
                    Self::ProductURI => "Product-URI",
                    Self::ErrorCount => "ErrorCount",
                    Self::DeviceStatus => "Device-Status",
                    Self::DetailedDeviceStatus => "Detailed-Device-Status",
                    Self::ProcessDataInput => "Process-DataInput",
                    Self::ProcessDataOutput => "Process-DataOutput",
                    Self::OffsetTime => "Offset-Time",

                    // Parameter ranges
                    Self::ProfileSpecific(_) => "Profile-Specific",
                    Self::PreferredIndex(_) => "Preferred-Index",
                    Self::ExtendedIndex(_) => "Extended-Index",
                    Self::DeviceProfileIndex(_) => "Device-Profile-Index",
                    Self::SafetySpecificIndex(_) => "Safety-Specific-Index",
                    Self::SecondaryDeviceProfileIndex(_) => "Secondary-Device-Profile-Index",
                    Self::WirelessSpecificIndex(_) => "Wireless-Specific-Index",
                }
            }

            /// Returns the category of the parameter index
            ///
            /// # Examples
            /// ```
            /// let param = DeviceParameterIndex::VendorName;
            /// assert_eq!(param.category(), IndexCategory::Standard);
            /// ```
            pub fn category(&self) -> IndexCategory {
                match self {
                    // Standard parameters
                    Self::DirectParameterPage1
                    | Self::DirectParameterPage2
                    | Self::SystemCommand
                    | Self::DataStorageIndex
                    | Self::DeviceAccessLocks
                    | Self::ProfileCharacteristic
                    | Self::PDInputDescriptor
                    | Self::PDOutputDescriptor
                    | Self::VendorName
                    | Self::VendorText
                    | Self::ProductName
                    | Self::ProductID
                    | Self::ProductText
                    | Self::SerialNumber
                    | Self::HardwareRevision
                    | Self::FirmwareRevision
                    | Self::ApplicationSpecificTag
                    | Self::FunctionTag
                    | Self::LocationTag
                    | Self::ProductURI
                    | Self::ErrorCount
                    | Self::DeviceStatus
                    | Self::DetailedDeviceStatus
                    | Self::ProcessDataInput
                    | Self::ProcessDataOutput
                    | Self::OffsetTime => IndexCategory::Standard,

                    Self::ProfileSpecific(_) => IndexCategory::ProfileSpecific,
                    Self::PreferredIndex(_) => IndexCategory::PreferredIndex,
                    Self::ExtendedIndex(_) => IndexCategory::ExtendedIndex,
                    Self::DeviceProfileIndex(_) => IndexCategory::DeviceProfile,
                    Self::SafetySpecificIndex(_) => IndexCategory::SafetySpecific,
                    Self::SecondaryDeviceProfileIndex(_) => IndexCategory::DeviceProfile,
                    Self::WirelessSpecificIndex(_) => IndexCategory::WirelessSpecific,
                }
            }
        }

        /// Categorizes device parameter indices based on their range
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum IndexCategory {
            /// Standard parameters (0x0000-0x0030)
            Standard,
            /// Profile-specific parameters (0x0031-0x003F)
            ProfileSpecific,
            /// Preferred device-specific parameters (8-bit) (0x0040-0x00FE)
            PreferredIndex,
            /// Extended device-specific parameters (16-bit) (0x0100-0x3FFF)
            ExtendedIndex,
            /// Device profile-specific parameters (0x4000-0x41FF, 0x4300-0x4FFF)
            DeviceProfile,
            /// Safety system extensions parameters (0x4200-0x42FF)
            SafetySpecific,
            /// Wireless system extensions parameters (0x5000-0x50FF)
            WirelessSpecific,
        }
    };

    TokenStream::from(expanded)
}

/// Represents all possible system commands as defined in IO-Link Specification Table B.9.
///
/// This enum categorizes commands into different groups:
/// - Parameter-related commands (0x01-0x06)
/// - Reset and restore commands (0x80-0x83)
/// - Vendor-specific commands (0xA0-0xFF)
///
/// # Variants
/// | Command (hex) | Command name             | Description                                                                 |
/// |---------------|-------------------------|-----------------------------------------------------------------------------|
/// | 0x00          | Reserved                | Reserved for future use                                                     |
/// | 0x01          | ParamUploadStart        | Start parameter upload                                                      |
/// | 0x02          | ParamUploadEnd          | Stop parameter upload                                                       |
/// | 0x03          | ParamDownloadStart      | Start parameter download                                                    |
/// | 0x04          | ParamDownloadEnd        | Stop parameter download                                                     |
/// | 0x05          | ParamDownloadStore      | Finalize parameterization and start Data Storage                            |
/// | 0x06          | ParamBreak              | Cancel all Param commands                                                   |
/// | 0x07..=0x3F   | Reserved                | Reserved for future use                                                     |
/// | 0x40..=0x7F   | Reserved for profiles   | Reserved for profile-specific extensions                                    |
/// | 0x80          | DeviceReset             | Device reset (see section 10.7.2 of the specification)                      |
/// | 0x81          | ApplicationReset        | Application reset (see section 10.7.3 of the specification)                 |
/// | 0x82          | RestoreFactorySettings  | Restore factory settings (see section 10.7.4 of the specification)          |
/// | 0x83          | BackToBox               | Back-to-box (see section 10.7.5 of the specification)                       |
/// | 0x84..=0x9F   | Reserved                | Reserved for future use                                                     |
/// | 0xA0..=0xFF   | VendorSpecific          | Vendor-specific commands                                                    |
///
/// # Notes
/// - H = highly recommended
/// - O = optional
/// - C = conditional (see full description of command for condition)
///
/// # Example
/// ```
/// let cmd = SystemCommand::from_u8(0x01).unwrap();
/// assert_eq!(cmd, SystemCommand::ParamUploadStart);
/// ```
#[proc_macro]
pub fn system_commands(input: TokenStream) -> TokenStream {
    let enum_name = parse_macro_input!(input as syn::Ident);

    let expanded = quote! {
        /// Represents all possible system commands as defined in the specification.
        ///
        /// This enum categorizes commands into different groups:
        /// - Parameter-related commands (0x01-0x06)
        /// - Reset commands (0x80-0x83)
        /// - Vendor-specific commands (0xA0-0xFF)
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(u8)]
        enum #enum_name {
        /// 0x01: ParamUploadStart - Start parameter upload
        ParamUploadStart = 0x01,
        /// 0x02: ParamUploadEnd - Stop parameter upload
        ParamUploadEnd = 0x02,
        /// 0x03: ParamDownloadStart - Start parameter download
        ParamDownloadStart = 0x03,
        /// 0x04: ParamDownloadEnd - Stop parameter download
        ParamDownloadEnd = 0x04,
        /// 0x05: ParamDownloadStore - Finalize parameterization and start Data Storage
        ParamDownloadStore = 0x05,
        /// 0x06: ParamBreak - Cancel all Param commands
        ParamBreak = 0x06,
        /// 0x80: DeviceReset - Device Reset
        DeviceReset = 0x80,
        /// 0x81: ApplicationReset - Application Reset
        ApplicationReset = 0x81,
        /// 0x82: RestoreFactorySettings - Restore Factory Settings
        RestoreFactorySettings = 0x82,
        /// 0x83: BackToBox - Back to Box
        BackToBox = 0x83,
        /// 0xA0..=0xFF: VendorSpecific - Vendor-specific commands
        VendorSpecific(u8),
        }

        impl #enum_name {
            /// Creates a SystemCommand from a raw command value
            ///
            /// # Arguments
            /// * `command` - The raw command value to convert
            ///
            /// # Returns
            /// `Some(SystemCommand)` if the command is valid, `None` otherwise
            ///
            /// # Examples
            /// ```
            /// let cmd = SystemCommand::from_u8(0x01).unwrap();
            /// assert_eq!(cmd, SystemCommand::ParamUploadStart);
            /// ```
            pub fn from_u8(command: u8) -> Option<Self> {
                match command {
                    0x01 => Some(Self::ParamUploadStart),
                    0x02 => Some(Self::ParamUploadEnd),
                    0x03 => Some(Self::ParamDownloadStart),
                    0x04 => Some(Self::ParamDownloadEnd),
                    0x05 => Some(Self::ParamDownloadStore),
                    0x06 => Some(Self::ParamBreak),
                    0x80 => Some(Self::DeviceReset),
                    0x81 => Some(Self::ApplicationReset),
                    0x82 => Some(Self::RestoreFactorySettings),
                    0x83 => Some(Self::BackToBox),
                    x if (0xA0..=0xFF).contains(&x) => Some(Self::VendorSpecific(x)),
                    _ => None,
                }
            }

            /// Returns the raw command value
            ///
            /// # Examples
            /// ```
            /// let cmd = SystemCommand::ParamUploadStart;
            /// assert_eq!(cmd.as_u8(), 0x01);
            /// ```
            pub fn as_u8(&self) -> u8 {
                match self {
                    Self::ParamUploadStart => 0x01,
                    Self::ParamUploadEnd => 0x02,
                    Self::ParamDownloadStart => 0x03,
                    Self::ParamDownloadEnd => 0x04,
                    Self::ParamDownloadStore => 0x05,
                    Self::ParamBreak => 0x06,
                    Self::DeviceReset => 0x80,
                    Self::ApplicationReset => 0x81,
                    Self::RestoreFactorySettings => 0x82,
                    Self::BackToBox => 0x83,
                    Self::VendorSpecific(x) => *x,
                }
            }

            /// Returns the human-readable name of the command
            ///
            /// # Examples
            /// ```
            /// let cmd = SystemCommand::ParamUploadStart;
            /// assert_eq!(cmd.name(), "ParamUploadStart");
            /// ```
            pub fn name(&self) -> &'static str {
                match self {
                    Self::ParamUploadStart => "ParamUploadStart",
                    Self::ParamUploadEnd => "ParamUploadEnd",
                    Self::ParamDownloadStart => "ParamDownloadStart",
                    Self::ParamDownloadEnd => "ParamDownloadEnd",
                    Self::ParamDownloadStore => "ParamDownloadStore",
                    Self::ParamBreak => "ParamBreak",
                    Self::DeviceReset => "DeviceReset",
                    Self::ApplicationReset => "ApplicationReset",
                    Self::RestoreFactorySettings => "RestoreFactorySettings",
                    Self::BackToBox => "BackToBox",
                    Self::VendorSpecific(_) => "VendorSpecific",
                }
            }

            /// Returns the command's requirement level (Highly Recommended, Optional, or Conditional)
            ///
            /// # Examples
            /// ```
            /// let cmd = SystemCommand::ApplicationReset;
            /// assert_eq!(cmd.requirement(), CommandRequirement::HighlyRecommended);
            /// ```
            pub fn requirement(&self) -> CommandRequirement {
                match self {
                    Self::ParamUploadStart
                    | Self::ParamUploadEnd
                    | Self::ParamDownloadStart
                    | Self::ParamDownloadEnd
                    | Self::ParamDownloadStore
                    | Self::ParamBreak
                    | Self::BackToBox => CommandRequirement::Conditional,
                    Self::DeviceReset | Self::RestoreFactorySettings => CommandRequirement::Optional,
                    Self::ApplicationReset => CommandRequirement::HighlyRecommended,
                    Self::VendorSpecific(_) => CommandRequirement::VendorSpecific,
                }
            }
        }

        /// Indicates the requirement level for system commands
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum CommandRequirement {
            /// Highly recommended command (H)
            HighlyRecommended,
            /// Optional command (O)
            Optional,
            /// Conditional command (C)
            Conditional,
            /// Vendor-specific command
            VendorSpecific,
        }
    };

    TokenStream::from(expanded)
}

/// Parse a list of parameter declarations
fn parse_parameter_list(input: TokenStream) -> Vec<ParameterDecl> {
    let mut params = Vec::new();
    let input_tokens = proc_macro2::TokenStream::from(input);
    let mut stream = input_tokens.clone().into_iter();

    eprintln!("Input tokens: {:?}", input_tokens);

    while let Some(token) = stream.next() {
        eprintln!("Processing token: {:?}", token);

        if let proc_macro2::TokenTree::Group(group) = token {
            eprintln!("Found group with delimiter: {:?}", group.delimiter());
            if group.delimiter() == proc_macro2::Delimiter::Parenthesis {
                let content = proc_macro2::TokenStream::from(group.stream());
                eprintln!("Group content: {:?}", content);
                match syn::parse2::<ParameterDecl>(content.clone()) {
                    Ok(param) => {
                        eprintln!("Successfully parsed parameter: {:?}", param);
                        params.push(param);
                    }
                    Err(_) => {
                        eprintln!("Failed to parse parameter from content: {:?}", content);
                    }
                }
            } else {
                eprintln!("Group is not parentheses, it's: {:?}", group.delimiter());
            }
        }

        // Check for comma separator
        if let Some(proc_macro2::TokenTree::Punct(punct)) = stream.next() {
            eprintln!("Found punct: {:?}", punct);
            if punct.as_char() != ',' {
                eprintln!("Not a comma, breaking");
                // Not a comma, we're done
                break;
            }
        } else {
            eprintln!("No more tokens");
            // No more tokens
            break;
        }
    }

    eprintln!("Final params: {:?}", params);
    params
}

/// Helper struct for parameter declaration parsing
#[derive(Debug)]
struct ParameterDecl {
    index: syn::Expr,
    subindex: syn::Expr,
    length: syn::Expr,
    range: syn::Expr,
    access: syn::Ident,
    data_type: Type,
    default_value: syn::Expr,
}

impl Parse for ParameterDecl {
    fn parse(input: ParseStream) -> Result<Self> {
        eprintln!("Starting to parse ParameterDecl");

        let index: syn::Expr = input.parse()?;
        eprintln!("Parsed index: {:?}", index);
        input.parse::<Token![,]>()?;
        eprintln!("Parsed first comma");

        let subindex: syn::Expr = input.parse()?;
        eprintln!("Parsed subindex: {:?}", subindex);
        input.parse::<Token![,]>()?;
        eprintln!("Parsed second comma");

        let length: syn::Expr = input.parse()?;
        eprintln!("Parsed length: {:?}", length);
        input.parse::<Token![,]>()?;
        eprintln!("Parsed third comma");

        let range: syn::Expr = input.parse()?;
        eprintln!("Parsed range: {:?}", range);
        input.parse::<Token![,]>()?;
        eprintln!("Parsed fourth comma");

        let access: syn::Ident = input.parse()?;
        eprintln!("Parsed access: {:?}", access);
        input.parse::<Token![,]>()?;
        eprintln!("Parsed fifth comma");

        let data_type: Type = input.parse()?;
        eprintln!("Parsed data_type: {:?}", data_type);
        input.parse::<Token![,]>()?;
        eprintln!("Parsed sixth comma");

        let default_value: syn::Expr = input.parse()?;
        eprintln!("Parsed default_value: {:?}", default_value);

        eprintln!("Successfully parsed all fields");
        Ok(Self {
            index,
            subindex,
            length,
            range,
            access,
            data_type,
            default_value,
        })
    }
}

/// Helper function to extract integer value from syn::Expr
/// Supports both literal integers and const expressions
fn extract_int_value(expr: &syn::Expr, expected_type: &str) -> Result<u64> {
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(lit_int),
            ..
        }) => {
            // Handle literal integers
            let value = lit_int.base10_parse::<u64>()?;
            eprintln!("Parsed literal: {} -> {}", lit_int, value);
            Ok(value)
        }
        syn::Expr::Block(syn::ExprBlock { block, .. }) => {
            // Handle block expressions like { 2 + 2 }
            if block.stmts.len() != 1 {
                return Err(syn::Error::new_spanned(
                    expr,
                    "Block expressions must contain exactly one statement",
                ));
            }

            match &block.stmts[0] {
                syn::Stmt::Expr(expr_stmt, _) => extract_int_value(expr_stmt, expected_type),
                _ => Err(syn::Error::new_spanned(
                    expr,
                    "Block expressions must contain an expression statement",
                )),
            }
        }
        syn::Expr::Path(syn::ExprPath { path, .. }) => {
            // Handle const variables and other identifiers
            // For now, we'll return an error suggesting to use literal values
            // In a more advanced implementation, you could try to resolve const values
            Err(syn::Error::new_spanned(
                expr,
                format!(
                    "Const expressions like '{}' are not yet supported. Please use literal values for now.",
                    path.segments.last().unwrap().ident
                ),
            ))
        }
        syn::Expr::Binary(syn::ExprBinary {
            left, op, right, ..
        }) => {
            // Handle binary expressions like 2 + 3, 0x10 << 4, etc.
            let left_val = extract_int_value(left, expected_type)?;
            let right_val = extract_int_value(right, expected_type)?;

            eprintln!("Binary operation: {} {:?} {} = ?", left_val, op, right_val);

            let result = match op {
                syn::BinOp::Add(_) => left_val + right_val,
                syn::BinOp::Sub(_) => left_val.saturating_sub(right_val),
                syn::BinOp::Mul(_) => left_val * right_val,
                syn::BinOp::Div(_) => {
                    if right_val == 0 {
                        return Err(syn::Error::new_spanned(expr, "Division by zero"));
                    } else {
                        left_val / right_val
                    }
                }
                syn::BinOp::Shl(_) => left_val << right_val,
                syn::BinOp::Shr(_) => left_val >> right_val,
                syn::BinOp::BitOr(_) => left_val | right_val,
                syn::BinOp::BitAnd(_) => left_val & right_val,
                syn::BinOp::BitXor(_) => left_val ^ right_val,
                _ => {
                    return Err(syn::Error::new_spanned(
                        expr,
                        format!(
                            "Binary operator '{:?}' not supported in const expressions",
                            op
                        ),
                    ));
                }
            };

            eprintln!("Binary operation result: {}", result);
            Ok(result)
        }
        syn::Expr::Unary(syn::ExprUnary {
            op,
            expr: unary_expr,
            ..
        }) => {
            // Handle unary expressions like -5, !0, etc.
            let val = extract_int_value(unary_expr, expected_type)?;
            match op {
                syn::UnOp::Neg(_) => {
                    if val > i64::MAX as u64 {
                        Err(syn::Error::new_spanned(expr, "Negation overflow"))
                    } else {
                        Ok((-(val as i64)) as u64)
                    }
                }
                syn::UnOp::Not(_) => Ok(!val),
                _ => Err(syn::Error::new_spanned(
                    expr,
                    format!(
                        "Unary operator '{:?}' not supported in const expressions",
                        op
                    ),
                )),
            }
        }
        _ => Err(syn::Error::new_spanned(
            expr,
            format!(
                "Expected a literal integer, const expression, or simple arithmetic expression, got: {:?}",
                expr
            ),
        )),
    }
}

/// Helper function to extract u16 value from syn::Expr
fn extract_u16_value(expr: &syn::Expr) -> Result<u16> {
    let value = extract_int_value(expr, "u16")?;
    if value > u16::MAX as u64 {
        Err(syn::Error::new_spanned(
            expr,
            format!("Value {} is too large for u16 (max: {})", value, u16::MAX),
        ))
    } else {
        Ok(value as u16)
    }
}

/// Helper function to extract u8 value from syn::Expr
fn extract_u8_value(expr: &syn::Expr) -> Result<u8> {
    let value = extract_int_value(expr, "u8")?;
    if value > u8::MAX as u64 {
        Err(syn::Error::new_spanned(
            expr,
            format!("Value {} is too large for u8 (max: {})", value, u8::MAX),
        ))
    } else {
        Ok(value as u8)
    }
}

/// Macro for declaring parameter storage with validation and access control
///
/// Creates memory space and helper functions for parameter access with:
/// - Index range: 0-65535
/// - Subindex range: 0-255
/// - Configurable value length, range, access rights and type
/// - Support for literal integers and simple arithmetic expressions
///
/// Panics if:
/// - Default value is not a slice
/// - Default value is not the same length as the length
/// - Default value is not the same type as the data type
/// - Default value is not the same range as the range
/// - Default value is not the same access as the access
/// - Default value is not the same type as the data type
///
/// # Example
/// ```
/// declare_parameter_storage! {
///     // Index, Subindex, Length, Range, Access, Type, DefaultValue
///     (0x0000, 0x00, 16, 0..0x0f, ReadOnly, u8, &[16]),  // Direct Parameter Page 1
///     (0x0001, 0x00, 1, 0..0x01, ReadWrite, u8, &[0]),  // Direct Parameter Page 2
///     (0x0002, 0x00, 1, 0..0x01, WriteOnly, u8, &[0]),  // System-Command
/// }
///
/// // You can also use arithmetic expressions:
/// declare_parameter_storage! {
///     (0x0010, 0x00, 2 + 2, 0..3, ReadWrite, u8, &[4]),  // Length as expression
///     (0x0020, 0x00, 1, 0..1, ReadOnly, u8, &[1]),       // Normal usage
/// }
/// ```
///
/// // Example usage:
/// let mut storage = ParameterStorage::new();
///
/// // Get a parameter (read)
/// let value = storage.get_parameter(0x0000, 0x00);
///
/// // Set a parameter (write)
/// let result = storage.set_parameter(0x0001, 0x00, &[123]);
///
/// // Get parameter info
/// let info = storage.get_parameter_info(0x0001, 0x00);
///
/// // Read all parameters for an index
/// let all = storage.read_index_memory(0x0001);
///
/// // Write all parameters for an index
/// let write_result = storage.write_index_memory(0x0001, &[1]);
///
/// // Validate constraints
/// let valid = storage.validate_constraints();
/// ```
#[proc_macro]
pub fn declare_parameter_storage(input: TokenStream) -> TokenStream {
    let params = parse_parameter_list(input);

    // Debug output
    eprintln!("Parsed {} parameters", params.len());
    for (i, param) in params.iter().enumerate() {
        eprintln!(
            "Parameter {}: index={:?}, subindex={:?}, length={:?}, access={:?}, data_type={:?}",
            i, param.index, param.subindex, param.length, param.access, param.data_type
        );
    }

    let mut storage_fields = Vec::new();
    let mut new_fields = Vec::new();
    let mut parameter_map = Vec::new();
    let mut get_match_arms = Vec::new();
    let mut set_match_arms = Vec::new();

    let max_parameter_length = params
        .iter()
        .map(|p| extract_u8_value(&p.length).unwrap() as usize)
        .max()
        .unwrap();
    eprintln!("Max parameter length: {}", max_parameter_length);

    for param in params {
        let index = &param.index;
        let subindex = &param.subindex;
        let length = &param.length;
        let range = &param.range;
        let access = &param.access;
        let data_type = &param.data_type;
        let default_value = &param.default_value;

        // Compile-time validation: Index 0x0000 and 0x0001 must have subindexes 0x00-0x0F
        let index_val = match extract_u16_value(index) {
            Ok(val) => val,
            Err(e) => return e.to_compile_error().into(),
        };
        let subindex_val = match extract_u8_value(subindex) {
            Ok(val) => val,
            Err(e) => return e.to_compile_error().into(),
        };

        if (index_val == 0x0000 || index_val == 0x0001) && subindex_val > 0x0F {
            let error_msg = format!(
                "Index 0x{:04X} can only have subindexes 0x00-0x0F, but got 0x{:02X}",
                index_val, subindex_val
            );
            return quote! {
                compile_error!(#error_msg);
            }
            .into();
        }

        // Compile-time validation: Length vs Range proportionality
        let length_val = match extract_u8_value(length) {
            Ok(val) => val,
            Err(e) => return e.to_compile_error().into(),
        };

        // Parse range to get start and end values
        let (range_start, range_end) = match range {
            syn::Expr::Range(range_expr) => {
                let start = match &range_expr.start {
                    Some(start_expr) => {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Int(start_lit),
                            ..
                        }) = start_expr.as_ref()
                        {
                            start_lit.base10_parse::<u8>().unwrap()
                        } else {
                            return quote! {
                                compile_error!("Range start must be a literal integer");
                            }
                            .into();
                        }
                    }
                    None => 0u8,
                };

                let end = match &range_expr.end {
                    Some(end_expr) => {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Int(end_lit),
                            ..
                        }) = end_expr.as_ref()
                        {
                            end_lit.base10_parse::<u8>().unwrap()
                        } else {
                            return quote! {
                                compile_error!("Range end must be a literal integer");
                            }
                            .into();
                        }
                    }
                    None => {
                        return quote! {
                            compile_error!("Range end must be specified");
                        }
                        .into();
                    }
                };

                (start, end)
            }
            _ => {
                return quote! {
                    compile_error!("Range must be specified as start..=end");
                }
                .into();
            }
        };

        // Validate length vs range at declaration time
        // For half-open ranges (0..31), max value is 30, so we need 31 bytes
        // Since we're using half-open ranges by default, we calculate accordingly
        let min_bytes_needed = range_end - range_start + 1;

        if length_val < min_bytes_needed {
            let error_msg = format!(
                "Length {} bytes is too small for range {}..{}. Minimum required length is {} bytes",
                length_val, range_start, range_end, min_bytes_needed
            );
            return quote! {
                compile_error!(#error_msg);
            }
            .into();
        } else if length_val > min_bytes_needed {
            let error_msg = format! {
                "Length {} bytes is too large for range {}..{}. Maximum allowed length is {} bytes",
                length_val, range_start, range_end, min_bytes_needed
            };
            return quote! {
                compile_error!(#error_msg);
            }
            .into();
        }

        // Note: Length can be larger than minimum required bytes for storage purposes
        // We only validate that it's not too small

        // Generate field identifier
        let field_ident = syn::Ident::new(
            &format!("index_{:04X}_sub_{:02x}", index_val, subindex_val),
            proc_macro2::Span::call_site(),
        );

        eprintln!("Generated field: {}", field_ident);

        // Generate storage field
        storage_fields.push(quote! {
            #[doc = "Parameter storage field"]
            pub #field_ident: heapless::Vec<u8, { #length_val as usize }>,
        });

        // Generate default value for new()
        new_fields.push(quote! {
            #field_ident: heapless::Vec::<u8, { #length_val as usize }>::from_slice(#default_value).unwrap(),
        });

        // Generate parameter map entry
        let access_right = match access.to_string().as_str() {
            "ReadOnly" => quote! { AccessRight::ReadOnly },
            "WriteOnly" => quote! { AccessRight::WriteOnly },
            "ReadWrite" => quote! { AccessRight::ReadWrite },
            _ => panic!("Invalid access right: {}", access),
        };

        parameter_map.push(quote! {
            ParameterInfo {
                index: #index_val,
                subindex: #subindex_val,
                length: #length_val as usize,
                range: Some(#range),
                access: #access_right,
                data_type: stringify!(#data_type),
            }
        });

        // Generate get match arm
        get_match_arms.push(quote! {
            (#index, #subindex) => {
                (self.#field_ident.len() as u8, self.#field_ident.as_slice())
            },
        });

        // Generate set match arm
        set_match_arms.push(quote! {
            (#index, #subindex) => {
                self.#field_ident.copy_from_slice(data);
                Ok(())
            },
        });
    }

    eprintln!(
        "Generated {} storage fields, {} parameter map entries",
        storage_fields.len(),
        parameter_map.len()
    );

    let expanded = quote! {
        /// Access rights for a parameter in the IO-Link parameter storage.
        ///
        /// This enum defines the allowed access modes for a parameter:
        /// - `ReadOnly`: The parameter can only be read.
        /// - `WriteOnly`: The parameter can only be written.
        /// - `ReadWrite`: The parameter can be both read and written.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum AccessRight {
            /// The parameter can only be read.
            ReadOnly,
            /// The parameter can only be written.
            WriteOnly,
            /// The parameter can be both read and written.
            ReadWrite,
        }

        /// Error types for parameter access and manipulation.
        ///
        /// This enum represents all possible error conditions that may occur
        /// when accessing or modifying parameters in the storage, such as
        /// unavailable indices, access violations, value range errors, and
        /// protocol-specific service errors.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum ParameterError {
            /// The specified index is not available in the parameter storage.
            IndexNotAvailable,
            /// The specified subindex is not available for the given index.
            SubindexNotAvailable,
            /// The requested service is not available.
            ServiceNotAvailable,
            /// The requested service is not available due to local control restrictions.
            ServiceNotAvailableLocalControl,
            /// The requested service is not available due to device control restrictions.
            ServiceNotAvailableDeviceControl,
            /// Access to the parameter is denied due to insufficient permissions.
            AccessDenied,
            /// The parameter value is out of the allowed range.
            ValueOutOfRange,
            /// The parameter value exceeds the upper limit.
            ValueAboveLimit,
            /// The parameter value is below the lower limit.
            ValueBelowLimit,
            /// The provided data length exceeds the parameter's capacity.
            LengthOverrun,
            /// The provided data length is insufficient for the parameter.
            LengthUnderrun,
            /// The requested function is not available.
            FunctionNotAvailable,
            /// The requested function is temporarily unavailable.
            FunctionTemporarilyUnavailable,
            /// The parameter set is invalid.
            InvalidParameterSet,
            /// The parameter set is inconsistent.
            InconsistentParameterSet,
            /// The application is not ready to process the request.
            ApplicationNotReady,
        }

        /// Metadata describing a parameter in the storage.
        ///
        /// This structure provides information about a parameter, including:
        /// - `index` and `subindex`: The parameter's address.
        /// - `length`: The length in bytes of the parameter value.
        /// - `range`: An optional valid value range for the parameter.
        /// - `access`: The access rights for the parameter.
        /// - `data_type`: The name of the parameter's data type.
        #[derive(Debug, Clone)]
        pub struct ParameterInfo {
            /// The parameter's index address.
            pub index: u16,
            /// The parameter's subindex address.
            pub subindex: u8,
            /// The length in bytes of the parameter value.
            pub length: usize,
            /// An optional valid value range for the parameter.
            pub range: Option<core::ops::Range<u8>>,
            /// The access rights for the parameter.
            pub access: AccessRight,
            /// The name of the parameter's data type.
            pub data_type: &'static str,
        }

        /// Storage structure for all parameters.
        ///
        /// This struct contains a field for each parameter, as generated by the macro.
        /// Each field holds the value of a parameter, typically as a fixed-size buffer.
        #[allow(missing_docs)]
        pub struct ParameterStorage {
            #(#storage_fields)*
        }

        impl ParameterStorage {
            /// Creates a new parameter storage instance with all parameters set to their default values.
            pub fn new() -> Self {
                Self {
                    #(#new_fields)*
                }
            }

            /// Resets all parameters to their default values.
            pub fn clear(&mut self) {
                *self = Self::new();
            }

            /// Retrieves the metadata information for a parameter by index and subindex.
            ///
            /// Returns `Ok(ParameterInfo)` if the parameter exists, or an appropriate `ParameterError`.
            pub fn get_parameter_info(&self, index: u16, subindex: u8) -> Result<ParameterInfo, ParameterError> {
                let param_infos = [
                    #(#parameter_map),*
                ];

                for info in &param_infos {
                    if info.index == index && info.subindex == subindex {
                        return Ok(info.clone());
                    }
                }

                Err(ParameterError::IndexNotAvailable)
            }

            /// Reads the value of a parameter as a byte slice.
            ///
            /// Returns a reference to the parameter's value if it exists and is readable,
            /// or an appropriate `ParameterError`.
            pub fn get_parameter<'a>(&'a self, index: u16, subindex: u8) -> Result<(u8, &'a [u8]), ParameterError> {
                let info = self.get_parameter_info(index, subindex)?;

                if !matches!(info.access, AccessRight::ReadOnly | AccessRight::ReadWrite) {
                    return Err(ParameterError::AccessDenied);
                }

                // Get the data based on index and subindex
                let (length, field_data): (u8, &[u8]) = match (index, subindex) {
                    #(#get_match_arms)*
                    _ => return Err(ParameterError::IndexNotAvailable),
                };
                // Return the field data as bytes
                Ok((length, field_data))
            }

            /// Writes a value to a parameter.
            ///
            /// The provided data must match the parameter's length and access rights.
            /// Returns `Ok(())` on success, or an appropriate `ParameterError`.
            pub fn set_parameter(&mut self, index: u16, subindex: u8, data: &[u8]) -> Result<(), ParameterError> {
                let info = self.get_parameter_info(index, subindex)?;

                if !matches!(info.access, AccessRight::WriteOnly | AccessRight::ReadWrite) {
                    return Err(ParameterError::AccessDenied);
                }

                if data.len() > info.length {
                    return Err(ParameterError::LengthOverrun);
                } else if data.len() < info.length {
                    return Err(ParameterError::LengthUnderrun);
                }

                // Set the data based on index and subindex
                match (index, subindex) {
                    #(#set_match_arms)*
                    _ => return Err(ParameterError::IndexNotAvailable),
                }
            }

            /// Reads the concatenated values of all subindexes for a given index.
            ///
            /// Returns a buffer containing the values of all readable subindexes,
            /// or an appropriate `ParameterError`.
            pub fn read_index_memory(&self, index: u16) -> Result<heapless::Vec<u8, #max_parameter_length>, ParameterError> {
                let mut memory = heapless::Vec::new();

                // Find all subindexes for this index
                let param_infos = [
                    #(#parameter_map),*
                ];

                let mut found = false;
                for info in &param_infos {
                    if info.index == index {
                        found = true;
                        if !matches!(info.access, AccessRight::ReadOnly | AccessRight::ReadWrite) {
                            return Err(ParameterError::AccessDenied);
                        }

                        // Get the parameter data
                        let (_, data) = self.get_parameter(index, info.subindex)?;
                        memory.extend_from_slice(&data).map_err(|_| ParameterError::LengthOverrun)?;
                    }
                }

                if !found {
                    return Err(ParameterError::IndexNotAvailable);
                }

                Ok(memory)
            }

            /// Writes values to all subindexes of a given index in one operation.
            ///
            /// The provided data must match the total length of all writable subindexes.
            /// Returns `Ok(())` on success, or an appropriate `ParameterError`.
            pub fn write_index_memory(&mut self, index: u16, data: &[u8]) -> Result<(), ParameterError> {
                let param_infos = [
                    #(#parameter_map),*
                ];

                let mut found = false;
                let mut total_length = 0;

                // Calculate total length needed
                for info in &param_infos {
                    if info.index == index {
                        found = true;
                        if !matches!(info.access, AccessRight::WriteOnly | AccessRight::ReadWrite) {
                            return Err(ParameterError::AccessDenied);
                        }
                        total_length += info.length;
                    }
                }

                if !found {
                    return Err(ParameterError::IndexNotAvailable);
                }

                if data.len() != total_length {
                    return Err(ParameterError::LengthOverrun);
                }

                // Write data to each subindex
                let mut offset = 0;
                for info in &param_infos {
                    if info.index == index {
                        let subindex_data = &data[offset..offset + info.length];
                        self.set_parameter(index, info.subindex, subindex_data)?;
                        offset += info.length;
                    }
                }

                Ok(())
            }

            /// Returns a static slice of all parameter metadata.
            ///
            /// This can be used for introspection, diagnostics, or documentation.
            pub fn get_all_parameters(&self) -> &'static [ParameterInfo] {
                static PARAM_INFOS: &[ParameterInfo] = &[
                    #(#parameter_map),*
                ];
                PARAM_INFOS
            }

            /// Validates parameter constraints for the storage.
            ///
            /// Checks for protocol-specific constraints, such as valid subindex ranges
            /// for certain indices. Returns `Ok(())` if all constraints are satisfied,
            /// or an appropriate `ParameterError`.
            pub fn validate_constraints(&self) -> Result<(), ParameterError> {
                let param_infos = self.get_all_parameters();

                for info in param_infos {
                    // Index 0x0000 and 0x0001 can only have subindexes 0x00-0x0F
                    if (info.index == 0x0000 || info.index == 0x0001) && info.subindex > 0x0F {
                        return Err(ParameterError::InvalidParameterSet);
                    }
                }

                Ok(())
            }

            // Get parameters by index
            // pub fn get_parameters_by_index(&self, index: u16) -> heapless::Vec<&ParameterInfo> {
            //     let param_infos = self.get_all_parameters();

            //     param_infos.iter()
            //         .filter(|info| info.index == index)
            //         .collect()
            // }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn bitfield_support(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);

    let enum_ident = &input.ident;
    let vis = &input.vis;
    let generics = &input.generics;
    let variants: Vec<&Variant> = input.variants.iter().collect();
    let attrs = &input.attrs;

    // Collect (variant_ident, discriminant_value)
    let mut variant_arms = Vec::new();

    for (index, variant) in variants.iter().enumerate() {
        let ident = &variant.ident;

        let value = if let Some((
            _,
            Expr::Lit(ExprLit {
                lit: Lit::Int(lit_int),
                ..
            }),
        )) = &variant.discriminant
        {
            // Parse as u8 since the enum is repr(u8)
            lit_int.base10_parse::<u8>().unwrap()
        } else {
            // If no explicit value, fallback to index in the enum
            index as u8
        };
        variant_arms.push((ident, value));
    }

    let from_bits_arms = variant_arms.iter().map(|(ident, value)| {
        quote! { #value => Self::#ident, }
    });

    let is_valid_bits_arms = variant_arms.iter().map(|(_, value)| {
        quote! { #value => true, }
    });

    let expanded = quote! {
        #(#attrs)*
        #vis enum #enum_ident #generics {
            #(#variants),*
        }

        impl #enum_ident {
            /// Create an enum variant from bits
            ///
            /// # Examples
            ///
            /// ```rust
            /// let variant = MyEnum::from_bits(0);
            /// assert_eq!(variant, MyEnum::Variant1);
            /// ```
            pub const fn from_bits(bits: u8) -> Self {
                match bits {
                    #(#from_bits_arms)*
                    _ => Self::new(),
                }
            }

            /// Convert an enum variant to bits
            ///
            /// # Examples
            ///
            /// ```rust
            /// let bits = MyEnum::Variant1.into_bits();
            /// assert_eq!(bits, 0);
            /// ```
            pub const fn into_bits(self) -> u8 {
                self as u8
            }

            /// Check if the given bits value is within the valid range for this enum
            ///
            /// Returns `true` if the bits value corresponds to a valid enum variant,
            /// `false` otherwise.
            ///
            /// # Examples
            ///
            /// ```rust
            /// let valid = MyEnum::is_valid_bits(0);
            /// assert!(valid);
            ///
            /// let invalid = MyEnum::is_valid_bits(255);
            /// assert!(!invalid);
            /// ```
            pub const fn is_valid_bits(bits: u8) -> bool {
                match bits {
                    #(#is_valid_bits_arms)*
                    _ => false,
                }
            }

            /// Try to create an enum variant from bits, returning `None` if invalid
            ///
            /// This is a safe alternative to `from_bits` that doesn't panic on invalid values.
            ///
            /// # Examples
            ///
            /// ```rust
            /// let valid = MyEnum::try_from_bits(0);
            /// assert_eq!(valid, Some(MyEnum::Variant1));
            ///
            /// let invalid = MyEnum::try_from_bits(255);
            /// assert_eq!(invalid, None);
            /// ```
            pub const fn try_from_bits(bits: u8) -> Option<Self> {
                if Self::is_valid_bits(bits) {
                    Some(Self::from_bits(bits))
                } else {
                    None
                }
            }
        }
    };

    TokenStream::from(expanded)
}
