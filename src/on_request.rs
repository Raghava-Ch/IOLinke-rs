//! On-request Data Handler
//!
//! This module implements the On-request Data Handler state machine as defined in
//! IO-Link Specification v1.1.4

use crate::types::{IoLinkError, IoLinkResult};

/// On-request Data Handler implementation
pub struct OnRequestHandler {
    // Placeholder implementation
}

impl OnRequestHandler {
    /// Create a new On-request Data Handler
    pub fn new() -> Self {
        Self {}
    }

    /// Poll the handler
    pub fn poll(&mut self) -> IoLinkResult<()> {
        Ok(())
    }
}

impl Default for OnRequestHandler {
    fn default() -> Self {
        Self::new()
    }
}
