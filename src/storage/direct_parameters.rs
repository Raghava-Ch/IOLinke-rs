/// Direct Parameter Page implementation for IO-Link device communication.
///
/// This module provides access to the Direct Parameter pages as defined in the IO-Link
/// specification v1.1.4, Annex B.1. Direct parameters are fundamental device parameters
/// that can be accessed without requiring higher-layer protocol overhead.
///
/// ## Parameter Categories
///
/// The Direct Parameter page 1 and page 2 contain the following categories of parameters:
///
/// ### Read-Only Parameters
/// - `MinCycleTime`: Minimum cycle time supported by the device
/// - `MSequenceCapability`: M-sequence capability of the device
/// - `ProcessDataIn`: Process data input configuration
/// - `ProcessDataOut`: Process data output configuration
/// - `VendorID1`: First byte of vendor identification
/// - `VendorID2`: Second byte of vendor identification
/// - `FunctionID1`: First byte of function identification
/// - `FunctionID2`: Second byte of function identification
///
/// ### Write-Only Parameters
/// - `MasterCommand`: Command from master to device
/// - `SystemCommand`: System-level command interface
///
/// ### Reserved Parameters
/// - `Reserved0E`: Reserved for future use
///
/// ## Error Handling
///
/// The module enforces access restrictions and returns appropriate errors:
/// - `InvalidAddrOrLen`: Address + length exceeds valid range
/// - `ReadOnly`: Attempt to write to read-only parameter
/// - `WriteOnly`: Attempt to read from write-only parameter
/// - `Reserved`: Access to reserved parameter address
///
/// ## Specification
///
/// Implementation follows IO-Link specification v1.1.4, Annex B.1 for Direct Parameter
/// page access patterns and restrictions.
/// Direct Parameter page module for IO-Link device parameter access.
///
/// This module implements access control for Direct Parameter pages 1 and 2 as defined
/// in IO-Link specification v1.1.4, Annex B.1. Direct parameters provide access to
/// device identification, configuration, and control information.
///
/// The Direct Parameter page contains three types of parameters:
/// - **Read-only parameters**: Device identification and capability information
/// - **Write-only parameters**: Master and system commands
/// - **Reserved parameters**: Future use or vendor-specific implementations
///
/// ## Address Space
/// - Page 1: Addresses 0x00-0x0F (Specifications defined parameters)
/// - Page 2: Addresses 0x10-0x1F (Vendor-specific parameters)
///
/// ## Parameter Categories
///
use crate::pl;

/// ### Read-only Parameters address [0x02, 0x03, 0x05, 0x06, 0x07, 0x08, 0x0C, 0x0D]
/// - MinCycleTime: Minimum cycle time supported by the device
/// - MSequenceCapability: M-sequence capability information
/// - ProcessDataIn: Input process data length
/// - ProcessDataOut: Output process data length
/// - VendorID1/VendorID2: Device vendor identification
/// - FunctionID1/FunctionID2: Device function identification
const _READ_ONLY: [u8; 8] = [
    iolinke_macros::direct_parameter_address!(MinCycleTime),
    iolinke_macros::direct_parameter_address!(MSequenceCapability),
    iolinke_macros::direct_parameter_address!(ProcessDataIn),
    iolinke_macros::direct_parameter_address!(ProcessDataOut),
    iolinke_macros::direct_parameter_address!(VendorID1),
    iolinke_macros::direct_parameter_address!(VendorID2),
    iolinke_macros::direct_parameter_address!(FunctionID1),
    iolinke_macros::direct_parameter_address!(FunctionID2),
];
/// ### Write-only Parameters address [0x00, 0x0F]
/// - MasterCommand: Commands from master to device
/// - SystemCommand: System-level commands
const _WRITE_ONLY: [u8; 2] = [
    iolinke_macros::direct_parameter_address!(MasterCommand),
    iolinke_macros::direct_parameter_address!(SystemCommand),
];
/// ### Reserved Parameters address [0x0E]
/// - Reserved for future specification extensions
const _RESERVED: [u8; 1] = [iolinke_macros::direct_parameter_address!(Reserved0E)];

/// Reads data from the Direct Parameter page with access control validation.
///
/// This function performs read operations on Direct Parameter pages 1 and 2,
/// enforcing access restrictions defined in the IO-Link specification.
///
/// # Arguments
/// * `pl` - Mutable reference to the physical layer interface
/// * `address` - Starting address within the Direct Parameter page (0x00-0x1F)
/// * `length` - Number of bytes to read
/// * `buffer` - Mutable buffer to store the read data
///
/// # Returns
/// * `Ok(())` - Read operation completed successfully
/// * `Err(PageError::InvalidAddrOrLen)` - Address + length exceeds page boundary (0x1F)
/// * `Err(PageError::WriteOnly(addr))` - Attempted to read from write-only parameter
/// * `Err(PageError::Reserved(addr))` - Attempted to read from reserved parameter
///
/// # Access Control
/// - Write-only parameters address [0x00, 0x0F] cannot be read
/// - Reserved parameters address (0x0E) cannot be accessed
/// - Valid address range: 0x00-0x1F
/// - Page 2 (0x10-0x1F) is Vendor-specific
pub fn read(
    pl: &mut pl::physical_layer::PhysicalLayer,
    address: u8,
    length: u8,
    buffer: &mut [u8],
) -> pl::physical_layer::PageResult<()> {
    if address + length > 0x1F {
        return Err(pl::physical_layer::PageError::InvalidAddrOrLen);
    };
    for addr in address..(address + length) {
        if _WRITE_ONLY.contains(&addr) {
            return Err(pl::physical_layer::PageError::WriteOnly(addr));
        }
        if _RESERVED.contains(&addr) {
            return Err(pl::physical_layer::PageError::Reserved(addr));
        }
    }

    pl.read_direct_param_page(address, length, buffer)?;
    Ok(())
}

/// Writes data to the Direct Parameter page with access control validation.
///
/// This function performs write operations on Direct Parameter page 1 only,
/// enforcing access restrictions defined in the IO-Link specification.
///
/// # Arguments
/// * `pl` - Mutable reference to the physical layer interface
/// * `address` - Starting address within the Direct Parameter page (0x00-0x0F)
/// * `length` - Number of bytes to write
/// * `buffer` - Buffer containing data to write
///
/// # Returns
/// * `Ok(())` - Write operation completed successfully
/// * `Err(PageError::InvalidAddrOrLen)` - Address + length exceeds writable boundary (0x0F)
/// * `Err(PageError::ReadOnly(addr))` - Attempted to write to read-only parameter
/// * `Err(PageError::Reserved(addr))` - Attempted to write to reserved parameter
///
/// # Access Control
/// - Read-only parameters address [0x02, 0x03, 0x05, 0x06, 0x07, 0x08, 0x0C, 0x0D] cannot be written
/// - Reserved parameters address [0x0E] cannot be accessed
/// - Valid address range for writes: 0x00-0x0F (page 1 only)
/// - Page 2 [0x10-0x1F] is Vendor-specific
pub fn write(
    pl: &mut pl::physical_layer::PhysicalLayer,
    address: u8,
    length: u8,
    buffer: &[u8],
) -> pl::physical_layer::PageResult<()> {
    if address + length > 0x0F {
        return Err(pl::physical_layer::PageError::InvalidAddrOrLen);
    }
    for addr in address..(address + length) {
        if _READ_ONLY.contains(&addr) {
            return Err(pl::physical_layer::PageError::ReadOnly(addr));
        }
        if _RESERVED.contains(&addr) {
            return Err(pl::physical_layer::PageError::Reserved(addr));
        }
    }

    pl.write_direct_param_page(address, length, buffer)?;
    Ok(())
}
