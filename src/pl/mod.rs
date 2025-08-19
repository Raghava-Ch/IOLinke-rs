//! Physical Layer module for IO-Link Device Stack.
//!
//! This module provides the physical layer abstractions for UART
//! communication, GPIO control, and timing as required by the
//! IO-Link protocol.
//!
//! ## Components
//!
//! - **Physical Layer**: Core physical layer operations and state management
//! - **Hardware Abstraction Layer**: Platform-independent hardware interfaces
//!
//! ## Specification Compliance
//!
//! - Section 5.2: Physical Layer and Communication Modes
//! - Section 5.3: C/Q Line Control and Timing
//! - Annex A: Protocol Timing and Sequences

pub mod physical_layer;
pub mod hal;