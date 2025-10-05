//! # IO-Link Protocol Layer (PL) Handler Module
//!
//! This module defines types for handling protocol timer identifiers in IO-Link devices.
//! It enumerates timers used for implementing timing requirements such as SIO mode timing,
//! maximum cycle time, and UART frame timing.
//!
//! ## Key Types
//! - [`Timer`]: Enumerates protocol timers as per IO-Link specification.
//!
//! ## Specification Reference
//! - IO-Link v1.1.4, Table 42 (Wake-up procedure and retry characteristics)
//! - Annex A.3.7 (Cycle time)
//! - Table 47 (Internal items)
//!
//! This module is intended for use in IO-Link device implementations to manage protocol timing
//! and ensure compliance with timing requirements.

/// Protocol timer identifiers as defined in the IO-Link specification.
///
/// These timers are used to implement the various timing requirements
/// specified in the IO-Link protocol.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Table 42: Wake-up procedure and retry characteristics
/// - Annex A.3.7: Cycle time
/// - Table 47: Internal items
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timer {
    /// Tdsio - SIO mode timing (see Table 42)
    Tdsio,
    /// MaxCycleTime - Maximum cycle time (see Annex A.3.7)
    MaxCycleTime,
    /// MaxUARTFrameTime - Maximum UART frame time (see Table 47)
    MaxUARTFrameTime,
    /// MaxUARTframeTime - Alternative naming for maximum UART frame time
    MaxUARTframeTime,
}
