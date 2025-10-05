//! This module provides functionality related to ISDU (Indexed Service Data Unit) frame formatting.
//!
//! It re-exports all items from the `isdu` submodule for convenient access.
//!
//! The ISDU frame format is typically used in IO-Link communication for structured data exchange.
//!
//! # Modules
//! - `isdu`: Contains the core logic and types for ISDU frame formatting.
mod isdu;
pub use isdu::*;
