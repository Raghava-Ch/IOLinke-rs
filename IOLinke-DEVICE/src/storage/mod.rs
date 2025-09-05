//! Storage module for IO-Link Device Stack.
//!
//! This module provides storage abstractions for device parameters,
//! events, and ISDU data as required by IO-Link Specification v1.1.4.
//!
//! ## Components
//!
//! - **Event Memory**: Stores and manages device events
//! - **Direct Parameters**: Handles direct parameter page access
//! - **ISDU Memory**: Manages Index-based Service Data Unit storage
//! - **Parameters Memory**: Handles device parameter storage
//!
//! ## Specification Compliance
//!
//! - Section 8.3: Event Handling and Storage
//! - Annex B: Parameter Storage and Access
//! - Section 8.1: ISDU Data Storage

pub mod event_memory;
pub mod isdu_memory;
