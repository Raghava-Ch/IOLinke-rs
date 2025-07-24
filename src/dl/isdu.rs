//! ISDU (Index Service Data Unit) Handler
//!
//! This module implements the ISDU Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 8.4.3

use crate::types::{IoLinkError, IoLinkResult, Isdu};
use heapless::Vec;

/// ISDU Handler states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsduState {
    /// Idle state
    Idle,
    /// Processing request
    Processing,
    /// Sending response
    Responding,
    /// Error state
    Error,
}

/// ISDU Handler implementation
pub struct IsduHandler {
    state: IsduState,
    current_request: Option<Isdu>,
    response_data: Vec<u8, 32>,
}

impl IsduHandler {
    /// Create a new ISDU Handler
    pub fn new() -> Self {
        Self {
            state: IsduState::Idle,
            current_request: None,
            response_data: Vec::new(),
        }
    }

    /// Poll the ISDU handler
    /// See IO-Link v1.1.4 Section 8.4.3
    pub fn poll(&mut self) -> IoLinkResult<()> {
        match self.state {
            IsduState::Idle => {
                // Wait for ISDU requests
            }
            IsduState::Processing => {
                // Process current request
                if let Some(request) = self.current_request.take() {
                    self.process_request(&request)?;
                    self.state = IsduState::Responding;
                }
            }
            IsduState::Responding => {
                // Send response
                self.state = IsduState::Idle;
            }
            IsduState::Error => {
                // Handle error recovery
                self.current_request = None;
                self.response_data.clear();
                self.state = IsduState::Idle;
            }
        }
        Ok(())
    }

    /// Process an ISDU request
    fn process_request(&mut self, request: &Isdu) -> IoLinkResult<()> {
        if request.is_write {
            // Handle write request
            self.handle_write(request)?;
        } else {
            // Handle read request
            self.handle_read(request)?;
        }
        Ok(())
    }

    /// Handle ISDU read request
    fn handle_read(&mut self, request: &Isdu) -> IoLinkResult<()> {
        // Implementation would read from parameter storage
        // For now, return dummy data based on index
        self.response_data.clear();
        match request.index {
            0x0000 => {
                // Vendor ID
                self.response_data.push(0x00).ok();
                self.response_data.push(0x01).ok();
            }
            0x0001 => {
                // Device ID
                self.response_data.push(0x00).ok();
                self.response_data.push(0x00).ok();
                self.response_data.push(0x00).ok();
                self.response_data.push(0x01).ok();
            }
            _ => {
                return Err(IoLinkError::InvalidParameter);
            }
        }
        Ok(())
    }

    /// Handle ISDU write request
    fn handle_write(&mut self, _request: &Isdu) -> IoLinkResult<()> {
        // Implementation would write to parameter storage
        Ok(())
    }

    /// Add ISDU request
    pub fn add_request(&mut self, request: Isdu) -> IoLinkResult<()> {
        if self.state != IsduState::Idle {
            return Err(IoLinkError::DeviceNotReady);
        }
        self.current_request = Some(request);
        self.state = IsduState::Processing;
        Ok(())
    }

    /// Get response data
    pub fn get_response(&mut self) -> Vec<u8, 32> {
        let data = self.response_data.clone();
        self.response_data.clear();
        data
    }

    /// Get current state
    pub fn state(&self) -> IsduState {
        self.state
    }
}

impl Default for IsduHandler {
    fn default() -> Self {
        Self::new()
    }
}
