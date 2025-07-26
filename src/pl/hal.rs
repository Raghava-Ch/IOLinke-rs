//! Hardware Abstraction Layer (HAL) traits
//!
//! This module defines the hardware abstraction traits that must be implemented
//! for different target platforms. Based on IO-Link Specification v1.1.4 Section 5.2.

use crate::types::IoLinkResult;
use crate::pl::physical_layer::{PhysicalLayerInd, IoLinkGpio, IoLinkTimer, IoLinkUart};

/// Complete HAL implementation combining all required traits
pub trait IoLinkHal: PhysicalLayerInd + IoLinkGpio + IoLinkTimer + IoLinkUart {
    /// Initialize the hardware
    fn init(&mut self) -> IoLinkResult<()>;

    /// Perform a hardware reset
    fn reset(&mut self) -> IoLinkResult<()>;

    /// Enter low power mode
    fn enter_low_power(&mut self) -> IoLinkResult<()>;

    /// Exit low power mode
    fn exit_low_power(&mut self) -> IoLinkResult<()>;
}
