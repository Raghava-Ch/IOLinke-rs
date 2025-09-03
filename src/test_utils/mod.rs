//! Test utilities for IO-Link device stack testing
//!
//! This module provides utilities and mock implementations for testing
//! the IO-Link device stack components.

// Re-export all submodules
pub mod types;
pub mod mock_physical_layer;
pub mod test_environment;
pub mod frame_utils;
pub mod page_params;
pub mod test_sequences;

// Re-export commonly used types and functions to maintain backward compatibility
pub use types::{ThreadMessage, TestDeviceMode};
pub use mock_physical_layer::{MockPhysicalLayer, MockTimerState};
pub use test_environment::{
    take_care_of_poll,
    create_test_device,
    send_test_message_and_wait,
    setup_test_environment,
    startup_routine,
};
pub use frame_utils::{
    validate_checksum,
    validate_device_frame_checksum,
    setup_device_configuration,
    perform_startup_sequence,
    create_startup_read_request,
    create_preop_read_request,
    create_preop_write_isdu_request,
    create_preop_read_start_isdu_request,
    create_preop_read_isdu_segment,
    create_preop_isdu_idle_request,
    create_preop_write_isdu_complete_request,
    create_op_read_request,
    create_startup_write_request,
    create_preop_write_request,
    create_op_write_request,
};
pub use page_params::{
    read_min_cycle_time,
    read_m_sequence_capability,
    read_revision_id,
    read_process_data_in,
    read_process_data_out,
    read_vendor_id_1,
    read_vendor_id_2,
    write_master_command,
};
pub use test_sequences::{
    util_test_startup_sequence,
    util_test_preop_sequence,
    util_test_isdu_sequence_read,
    util_test_isdu_sequence_write,
    util_test_change_operation_mode,
};

// Re-export the frame_utils submodule for backward compatibility
pub use frame_utils as frame_format_utils;


// Re-export commonly used crate types and functions
pub use crate::SystemManagementReq;
pub use crate::config;
pub use crate::config::m_seq_capability;
pub use crate::types::{ComChannel, MsequenceBaseType, MsequenceType, RwDirection};
pub use crate::utils::frame_fromat::isdu::IsduService;
pub use crate::utils::frame_fromat::message::{
    ChecksumMsequenceType,
    ChecksumMsequenceTypeBuilder,
    ChecksumStatus,
    ChecksumStatusBuilder,
    calculate_checksum_for_testing,
    MsequenceControl,
    MsequenceControlBuilder,
};
