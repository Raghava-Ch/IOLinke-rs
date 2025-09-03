# IOLinke-Device Tests

This directory contains comprehensive tests for the `iolinke_device` crate, organized into logical modules for better maintainability and clarity.

## Test Structure

The tests are organized into three main modules:

- **`startup_tests.rs`** - Tests for device startup sequence and parameter reading
- **`preop_tests.rs`** - Tests for preoperational mode functionality and master commands
- **`isdu_tests.rs`** - Tests for ISDU (IO-Link Service Data Unit) operations
- **`mod.rs`** - Test module organization and imports

## Test Categories

### Startup Tests (`startup_tests.rs`)

Tests the device startup sequence and basic parameter reading functionality:

- ✅ **Device Creation and Configuration** - Verifies device can be created and configured
- ✅ **Startup Sequence Execution** - Tests the complete startup sequence
- ✅ **Parameter Reading** - Tests reading various device parameters
- ✅ **Checksum Validation** - Validates response checksums
- ✅ **Communication Testing** - Tests communication between test and device

**Test Functions:**
- `test_min_cycle_time()` - Tests reading minimum cycle time parameter
- `test_m_sequence_capability()` - Tests reading M-sequence capability
- `test_revision_id()` - Tests reading revision identification
- `test_process_data_in()` - Tests reading process data input configuration
- `test_process_data_out()` - Tests reading process data output configuration
- `test_vendor_id_1()` - Tests reading vendor ID part 1
- `test_vendor_id_2()` - Tests reading vendor ID part 2

### Preoperational Tests (`preop_tests.rs`)

Tests the preoperational mode functionality and master command handling:

- ✅ **Mode Transitions** - Tests transitions between startup and preoperational modes
- ✅ **Master Commands** - Tests master command handling
- ✅ **ISDU Operations** - Tests ISDU read/write operations in preop mode
- ✅ **Data Storage** - Tests data storage index operations

**Test Functions:**
- `test_write_master_ident()` - Tests writing master identification command
- `test_write_master_pre_operate()` - Tests writing master preoperate command
- `test_read_isdu_read_vendor_name()` - Tests ISDU read of vendor name
- `test_write_data_storage_index_and_read_back()` - Tests data storage write/read operations

### ISDU Tests (`isdu_tests.rs`)

Dedicated tests for ISDU (IO-Link Service Data Unit) operations:

- ✅ **ISDU Read Operations** - Tests reading device parameters via ISDU
- ✅ **ISDU Write Operations** - Tests writing data via ISDU
- ✅ **Data Validation** - Validates ISDU data integrity
- ✅ **Error Handling** - Tests ISDU error conditions
- ✅ **Edge Cases** - Tests boundary conditions and edge cases

**Test Functions:**
- `test_isdu_read_vendor_name()` - Tests ISDU read of vendor name
- `test_isdu_read_product_name()` - Tests ISDU read of product name
- `test_isdu_write_and_read_data_storage_index()` - Tests ISDU write/read of data storage
- `test_isdu_invalid_index_error()` - Tests ISDU error handling for invalid indices
- `test_isdu_write_empty_data()` - Tests ISDU write with empty data

## Test Features

The tests verify:
- ✅ Device creation and configuration
- ✅ Startup sequence execution
- ✅ Parameter reading functionality
- ✅ Checksum validation
- ✅ Communication between test and device
- ✅ Multiple parameter types (read/write operations)
- ✅ Mode transitions (startup → preoperational)
- ✅ Master command handling
- ✅ ISDU read/write operations
- ✅ Error handling and edge cases

## Test Architecture

The tests use a mock-based approach:
- **Device Instance**: Created with `Arc<Mutex<IoLinkDevice>>`
- **Communication Channels**: Separate channels for polling thread and test communication
- **Polling Thread**: Runs device polling in background using `take_care_of_poll_nb`
- **Test Communication**: Direct communication with device through mock physical layer
- **Modular Setup**: `setup_test_environment()` handles all initialization

## Communication Flow

1. **Test Setup**: `setup_test_environment()` creates device and starts polling thread
2. **Message Creation**: Test utility functions create appropriate test messages
3. **Message Sending**: Messages sent to polling thread via communication channels
4. **Device Processing**: Polling thread processes messages and calls device methods
5. **Response Reception**: Device responses received via response channels
6. **Validation**: Response data validated using utility functions

## Running Tests

### Run all tests
```bash
cd IOLinke-Device
cargo test
```

### Run specific test module
```bash
# Run startup tests only
cargo test --test startup_tests

# Run preoperational tests only
cargo test --test preop_tests

# Run ISDU tests only
cargo test --test isdu_tests
```

### Run tests with output
```bash
cargo test -- --nocapture
```

### Run tests in release mode
```bash
cargo test --release
```

### Run specific test function
```bash
cargo test test_min_cycle_time
```

## Adding New Tests

### For Startup Tests
1. Add new test functions to `startup_tests.rs`
2. Use `test_utils::setup_test_environment()` for setup
3. Use `test_utils::page_params::read_*()` functions for parameter reading
4. Follow the existing naming convention: `test_<parameter_name>()`

### For Preoperational Tests
1. Add new test functions to `preop_tests.rs`
2. Ensure device is in preoperational mode using `util_test_change_operation_mode()`
3. Use appropriate master commands or ISDU operations
4. Test both read and write operations where applicable

### For ISDU Tests
1. Add new test functions to `isdu_tests.rs`
2. Ensure device supports ISDU using `m_sequence_capability.isdu()`
3. Use `util_test_isdu_sequence_read()` and `util_test_isdu_sequence_write()`
4. Test error conditions and edge cases

### General Guidelines
1. **Use existing utility functions** for common operations
2. **Follow established patterns** and naming conventions
3. **Add proper assertions** with descriptive error messages
4. **Test both success and failure cases** where applicable
5. **Document complex test logic** with comments

## Test Dependencies

Tests use the `iolinke_device` crate directly, ensuring that:
- The crate can be properly imported
- All public APIs are accessible
- The crate compiles and links correctly
- Basic functionality works as expected

## Notes

- Tests are designed to work with both `std` and `no_std` features
- Tests verify the crate's public API surface
- Integration tests ensure the crate works end-to-end
- All tests should pass before releasing the crate
- The modular structure makes it easy to add new test cases
- Common setup code is centralized in utility functions
- Tests can be easily extended for additional parameter types and operations