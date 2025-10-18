# IOLinke-Device Tests

This directory contains tests for the `iolinke_device` crate, organized into modules for maintainability.

## Test Modules

- **`startup_tests.rs`**: Tests for device startup and parameter reading.
- **`preop_tests.rs`**: Tests for preoperational mode and master commands.
- **`isdu_tests.rs`**: Tests for ISDU operations.
- **`mod.rs`**: Organizes and imports test modules.

## Running Tests

### Run all tests
```bash
cargo test
```

### Run specific test module
```bash
cargo test --test <module_name>
```

### Run specific test function
```bash
cargo test <function_name>
```

## Adding Tests

1. Add test functions to the appropriate module.
2. Use `setup_test_environment()` for initialization.
3. Follow existing naming conventions and patterns.
4. Test both success and failure cases.

## Notes

- Tests verify the crate's public API and functionality.
- Ensure all tests pass before releasing the crate.
- Use utility functions for common operations.