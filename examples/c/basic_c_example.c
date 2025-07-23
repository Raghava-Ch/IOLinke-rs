/**
 * @file basic_c_example.c
 * @brief Basic IO-Link Device Stack C API Example
 * 
 * This example demonstrates how to use the IO-Link device stack
 * from C code using the FFI bindings.
 */

#include <stdio.h>
#include <stdint.h>
#include <string.h>

// Include the generated header (will be available after build)
// #include "iolink_device_stack.h"

// For demonstration, we'll define the types here
// In real usage, these would come from the generated header

typedef enum {
    IOLINK_OK = 0,
    IOLINK_INVALID_PARAMETER = 1,
    IOLINK_TIMEOUT = 2,
    IOLINK_CHECKSUM_ERROR = 3,
    IOLINK_INVALID_FRAME = 4,
    IOLINK_BUFFER_OVERFLOW = 5,
    IOLINK_DEVICE_NOT_READY = 6,
    IOLINK_HARDWARE_ERROR = 7,
    IOLINK_PROTOCOL_ERROR = 8,
    IOLINK_NULL_POINTER = 9,
} iolink_error_t;

typedef enum {
    IOLINK_MODE_SIO = 0,
    IOLINK_MODE_COM1 = 1,
    IOLINK_MODE_COM2 = 2,
    IOLINK_MODE_COM3 = 3,
} iolink_mode_t;

typedef struct {
    uint8_t* input_data;
    size_t input_length;
    const uint8_t* output_data;
    size_t output_length;
    bool valid;
} iolink_process_data_t;

typedef struct {
    uint16_t vendor_id;
    uint32_t device_id;
    uint16_t function_id;
    uint8_t reserved;
} iolink_device_id_t;

// Opaque handle type
typedef struct IoLinkDeviceHandle IoLinkDeviceHandle;

// Function declarations (normally from generated header)
extern IoLinkDeviceHandle* iolink_device_create(void);
extern void iolink_device_destroy(IoLinkDeviceHandle* handle);
extern iolink_error_t iolink_device_poll(IoLinkDeviceHandle* handle);
extern iolink_error_t iolink_get_input_data(IoLinkDeviceHandle* handle, iolink_process_data_t* process_data);
extern iolink_error_t iolink_set_output_data(IoLinkDeviceHandle* handle, const iolink_process_data_t* process_data);
extern iolink_error_t iolink_read_parameter(IoLinkDeviceHandle* handle, uint16_t index, uint8_t sub_index, uint8_t* data, size_t* data_length);
extern iolink_error_t iolink_write_parameter(IoLinkDeviceHandle* handle, uint16_t index, uint8_t sub_index, const uint8_t* data, size_t data_length);
extern iolink_error_t iolink_get_device_id(IoLinkDeviceHandle* handle, iolink_device_id_t* device_id);
extern iolink_error_t iolink_get_min_cycle_time(IoLinkDeviceHandle* handle, uint8_t* cycle_time);
extern const char* iolink_get_version(void);

/**
 * @brief Print error message for IO-Link error codes
 */
void print_iolink_error(iolink_error_t error) {
    switch (error) {
        case IOLINK_OK:
            printf("Success\n");
            break;
        case IOLINK_INVALID_PARAMETER:
            printf("Error: Invalid parameter\n");
            break;
        case IOLINK_TIMEOUT:
            printf("Error: Timeout\n");
            break;
        case IOLINK_CHECKSUM_ERROR:
            printf("Error: Checksum error\n");
            break;
        case IOLINK_INVALID_FRAME:
            printf("Error: Invalid frame\n");
            break;
        case IOLINK_BUFFER_OVERFLOW:
            printf("Error: Buffer overflow\n");
            break;
        case IOLINK_DEVICE_NOT_READY:
            printf("Error: Device not ready\n");
            break;
        case IOLINK_HARDWARE_ERROR:
            printf("Error: Hardware error\n");
            break;
        case IOLINK_PROTOCOL_ERROR:
            printf("Error: Protocol error\n");
            break;
        case IOLINK_NULL_POINTER:
            printf("Error: Null pointer\n");
            break;
        default:
            printf("Error: Unknown error code %d\n", error);
            break;
    }
}

/**
 * @brief Main example function
 */
int main(void) {
    printf("IO-Link Device Stack C Example\n");
    printf("==============================\n\n");

    // Get library version
    const char* version = iolink_get_version();
    printf("Library version: %s\n\n", version);

    // Create device instance
    IoLinkDeviceHandle* device = iolink_device_create();
    if (device == NULL) {
        printf("Failed to create IO-Link device\n");
        return 1;
    }

    printf("Device created successfully\n");

    // Get device identification
    iolink_device_id_t device_id;
    iolink_error_t error = iolink_get_device_id(device, &device_id);
    if (error == IOLINK_OK) {
        printf("Device ID: Vendor=0x%04X, Device=0x%08X, Function=0x%04X\n",
               device_id.vendor_id, device_id.device_id, device_id.function_id);
    } else {
        printf("Failed to get device ID: ");
        print_iolink_error(error);
    }

    // Get minimum cycle time
    uint8_t cycle_time;
    error = iolink_get_min_cycle_time(device, &cycle_time);
    if (error == IOLINK_OK) {
        printf("Minimum cycle time: %d x 100µs = %d ms\n", cycle_time, cycle_time / 10);
    } else {
        printf("Failed to get cycle time: ");
        print_iolink_error(error);
    }

    // Simulate some process data exchange
    printf("\nSimulating process data exchange:\n");
    
    uint8_t input_buffer[32];
    uint8_t output_data[] = {0xAA, 0xBB, 0xCC, 0xDD};
    
    for (int i = 0; i < 5; i++) {
        printf("Cycle %d:\n", i + 1);
        
        // Poll the device
        error = iolink_device_poll(device);
        if (error != IOLINK_OK) {
            printf("  Poll failed: ");
            print_iolink_error(error);
            continue;
        }
        
        // Set output data
        iolink_process_data_t output_process_data = {
            .input_data = NULL,
            .input_length = 0,
            .output_data = output_data,
            .output_length = sizeof(output_data),
            .valid = true
        };
        
        error = iolink_set_output_data(device, &output_process_data);
        if (error == IOLINK_OK) {
            printf("  Output data set successfully\n");
        } else {
            printf("  Failed to set output data: ");
            print_iolink_error(error);
        }
        
        // Get input data
        iolink_process_data_t input_process_data = {
            .input_data = input_buffer,
            .input_length = sizeof(input_buffer),
            .output_data = NULL,
            .output_length = 0,
            .valid = false
        };
        
        error = iolink_get_input_data(device, &input_process_data);
        if (error == IOLINK_OK) {
            printf("  Input data received: %zu bytes, valid=%s\n", 
                   input_process_data.input_length, 
                   input_process_data.valid ? "true" : "false");
            
            if (input_process_data.input_length > 0) {
                printf("  Data: ");
                for (size_t j = 0; j < input_process_data.input_length; j++) {
                    printf("0x%02X ", input_buffer[j]);
                }
                printf("\n");
            }
        } else {
            printf("  Failed to get input data: ");
            print_iolink_error(error);
        }
        
        printf("\n");
    }

    // Test parameter access
    printf("Testing parameter access:\n");
    
    // Try to read vendor ID parameter
    uint8_t param_data[32];
    size_t param_length = sizeof(param_data);
    
    error = iolink_read_parameter(device, 0x0000, 0, param_data, &param_length);
    if (error == IOLINK_OK) {
        printf("Read parameter 0x0000: %zu bytes\n", param_length);
        if (param_length > 0) {
            printf("Data: ");
            for (size_t i = 0; i < param_length; i++) {
                printf("0x%02X ", param_data[i]);
            }
            printf("\n");
        }
    } else {
        printf("Failed to read parameter: ");
        print_iolink_error(error);
    }

    // Try to write a parameter
    uint8_t write_data[] = {0x12, 0x34};
    error = iolink_write_parameter(device, 0x1000, 0, write_data, sizeof(write_data));
    if (error == IOLINK_OK) {
        printf("Parameter write successful\n");
    } else {
        printf("Failed to write parameter: ");
        print_iolink_error(error);
    }

    // Cleanup
    iolink_device_destroy(device);
    printf("\nDevice destroyed, example complete\n");

    return 0;
}

/**
 * @brief Compile and run instructions:
 * 
 * 1. First build the Rust library:
 *    cargo build --release
 * 
 * 2. Compile this C example:
 *    gcc -o basic_c_example basic_c_example.c -L../target/release -liolink_device_stack
 * 
 * 3. Run the example:
 *    ./basic_c_example
 * 
 * Note: You may need to adjust library paths and linking depending on your system.
 * For embedded targets, use your target-specific compiler and linker settings.
 */
