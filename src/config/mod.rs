//! Configuration module for IO-Link Device Stack.
//!
//! This module provides configuration structures and parameters
//! for device behavior and communication settings.
//!
//! ## Components
//!
//! - **M-Sequence Capability**: M-sequence type and timing configuration
//! - **Vendor Specifics**: Vendor-specific configuration parameters
//! - **Process Data**: Process data configuration and settings
//! - **Timings**: Protocol timing and cycle time configuration
//!
//! ## Specification Compliance
//!
//! - Annex A: M-Sequence Types and Timing
//! - Annex B: Device Configuration Parameters
//! - Section 8.2: Process Data Configuration

pub mod m_seq_capability;
pub mod vendor_specifics;
pub mod process_data;
pub mod timings;