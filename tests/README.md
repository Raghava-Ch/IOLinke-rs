# IOLinke-Device Tests

This directory contains comprehensive tests for the `iolinke_device` crate.

## Test Structure

- **`startup_tests.rs`** - Modular tests for device startup and parameter reading
- **`mod.rs`** - Test module organization

## Test Features

The tests verify:
- ✅ Device creation and configuration
- ✅ Startup sequence execution
- ✅ Parameter reading functionality
- ✅ Checksum validation
- ✅ Communication between test and device
- ✅ Multiple parameter types (read/write operations)

## Modular Test Functions

The test file provides reusable functions for common testing scenarios:

### Core Functions
- `create_test_device()` - Creates a new device instance with communication channels
- `setup_device_configuration()` - Configures device with basic parameters
- `perform_startup_sequence()` - Executes device startup sequence
- `startup_routine()` - Complete startup routine combining configuration and startup
- `setup_test_environment()` - Sets up complete test environment with device and polling thread

### Test Utilities
- `create_read_request(address)` - Creates a read request message for testing
- `create_write_request(address, data)` - Creates a write request message for testing
- `send_test_message_and_wait()` - Sends a message and waits for response
- `validate_checksum()` - Validates response checksums

### Test Cases
- `read_min_cycle_time()` - Tests reading minimum cycle time parameter
- `read_vendor_id()` - Tests reading vendor identification
- `read_device_id()` - Tests reading device identification
- `read_msequence_capability()` - Tests reading M-sequence capability
- `read_process_data_in()` - Tests reading process data input configuration

## Running Tests

### Run all tests
```bash
cd IOLinke-Device
cargo test
```

### Run specific test file
```bash
cargo test --test startup_tests
```

### Run tests with output
```bash
cargo test -- --nocapture
```

### Run tests in release mode
```bash
cargo test --release
```

## Adding New Tests

1. **Use the existing modular functions** for common operations
2. **Create new test cases** following the established pattern
3. **Add new utility functions** as needed for specific functionality
4. **Follow the existing test structure** and naming conventions

### Example: Adding a New Read Test
```rust
#[test]
fn read_new_parameter() {
    // Set up test environment
    let (_io_link_device, poll_tx, poll_response_rx) = setup_test_environment();

    // Create read request for new parameter
    let (rx_buffer, expected_checksum) = create_read_request(direct_parameter_address!(NewParameter));
    
    // Send message and wait for response
    let _ = poll_tx.send(ThreadMessage::RxData(rx_buffer));
    let response = poll_response_rx.recv_timeout(Duration::from_secs(4))
        .expect("Failed to get response from device");
    
    // Validate and process response
    let response_data = match response {
        ThreadMessage::TxData(data) => data,
        _ => panic!("Expected TxData response"),
    };
    
    // Add your specific assertions here
    assert!(response_data.len() >= 2, "Response too short");
}
```

## Test Dependencies

Tests use the `iolinke_device` crate directly, ensuring that:
- The crate can be properly imported
- All public APIs are accessible
- The crate compiles and links correctly
- Basic functionality works as expected

## Test Architecture

The tests use a mock-based approach:
- **Device Instance**: Created with `Arc<Mutex<IoLinkDevice>>`
- **Communication Channels**: Separate channels for polling thread and test communication
- **Polling Thread**: Runs device polling in background using `take_care_of_poll_nb`
- **Test Communication**: Direct communication with device through mock physical layer
- **Modular Setup**: `setup_test_environment()` handles all initialization

## Communication Flow

1. **Test Setup**: `setup_test_environment()` creates device and starts polling thread
2. **Message Creation**: `create_read_request()` or `create_write_request()` creates test messages
3. **Message Sending**: Messages sent to polling thread via `poll_tx`
4. **Device Processing**: Polling thread processes messages and calls device methods
5. **Response Reception**: Device responses received via `poll_response_rx`
6. **Validation**: Response data validated using utility functions

## Notes

- Tests are designed to work with both `std` and `no_std` features
- Tests verify the crate's public API surface
- Integration tests ensure the crate works end-to-end
- All tests should pass before releasing the crate
- The modular structure makes it easy to add new test cases
- Common setup code is centralized in utility functions
- Tests can be easily extended for write operations and other parameter types
