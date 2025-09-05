#![cfg_attr(not(feature = "std"), no_std)]
//! Utility modules for IO-Link Device Stack.
//!
//! This module provides utility functions and data structures used
//! throughout the IO-Link implementation.
//!
//! ## Components
//!
//! - **Bitwise Operations**: Bit manipulation and bitfield utilities
//! - **Frame Format**: IO-Link frame parsing and formatting
//! - **Event Handling**: Event processing and management utilities
//!
//! ## Specification Compliance
//!
//! - Section 5.4: Frame Formats and Structures
//! - Annex A: Protocol Details and Bit Definitions
//! - Section 8.3: Event Handling and Processing

pub mod event;
pub mod frame_fromat;
pub mod log_utils;
