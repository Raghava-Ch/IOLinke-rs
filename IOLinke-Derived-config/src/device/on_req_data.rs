//! Re-exports functions related to device request data operations from the `iolinke_dev_config` crate.
//!
//! This module provides access to the following functions:
//! - `max_possible_od_length`: Determines the maximum possible length of the object dictionary.
//! - `operate`: Handles device operation requests.
//! - `pre_operate`: Prepares the device for operation requests.
//!
//! These utilities are essential for managing and processing device request data in the IO-Link configuration workflow.
pub use iolinke_dev_config::device::on_req_data::{max_possible_od_length, operate, pre_operate};
