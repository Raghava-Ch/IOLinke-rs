//! Parameter Manager
//!
//! This module implements the Parameter Manager as defined in
//! IO-Link Specification v1.1.4

use crate::types::{IoLinkError, IoLinkResult};
use heapless::FnvIndexMap;

/// Parameter Manager implementation
pub struct ParameterManager {
    parameters: FnvIndexMap<u16, heapless::Vec<u8, 32>, 64>,
}

impl ParameterManager {
    /// Create a new Parameter Manager
    pub fn new() -> Self {
        Self {
            parameters: FnvIndexMap::new(),
        }
    }

    /// Read a parameter
    pub fn read_parameter(&self, index: u16) -> IoLinkResult<&heapless::Vec<u8, 32>> {
        self.parameters.get(&index)
            .ok_or(IoLinkError::InvalidParameter)
    }

    /// Write a parameter
    pub fn write_parameter(&mut self, index: u16, data: &[u8]) -> IoLinkResult<()> {
        let mut param_data = heapless::Vec::new();
        for &byte in data {
            param_data.push(byte)
                .map_err(|_| IoLinkError::BufferOverflow)?;
        }
        self.parameters.insert(index, param_data)
            .map_err(|_| IoLinkError::BufferOverflow)?;
        Ok(())
    }

    /// Poll the parameter manager
    pub fn poll(&mut self) -> IoLinkResult<()> {
        Ok(())
    }
}

impl Default for ParameterManager {
    fn default() -> Self {
        Self::new()
    }
}
