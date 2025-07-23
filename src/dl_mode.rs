//! Data Link Mode Handler
//!
//! This module implements the DL-Mode Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 6.2

use crate::types::{IoLinkError, IoLinkMode, IoLinkResult};
use crate::hal::{PhysicalLayer, Timer};

/// DL-Mode Handler states
/// See IO-Link v1.1.4 Section 7.3.2.5
/// See Table 45 – State transition tables of the Device DL-mode handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DlModeState {
    /// Inactive state
    Idle,
    /// Establish Communication
    EstablishCom,
    /// Startup state
    Startup,
    /// Preoperate state
    Preoperate,
    /// Operate state
    Operate,
}

/// DL-Mode Handler events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DlModeEvent {
    /// Table 45 – T1
    PlWakeUp,
    /// Mode change request
    ComChange(IoLinkMode),
    /// Initialize
    Initialize,
    /// Startup complete
    StartupComplete,
    /// Error occurred
    Error,
    /// Fallback request
    Fallback,
    /// Table 45 – T10
    TimerExpired(Timer),
}

/// DL-Mode Handler state machine
pub struct DlModeHandler {
    state: DlModeState,
    current_mode: IoLinkMode,
    target_mode: IoLinkMode,
    error_count: u32,
}

impl DlModeHandler {
    /// Create a new DL-Mode Handler
    pub fn new() -> Self {
        Self {
            state: DlModeState::Idle,
            current_mode: IoLinkMode::Sio,
            target_mode: IoLinkMode::Sio,
            error_count: 0,
        }
    }

    /// Process an event
    pub fn process_event(&mut self, event: DlModeEvent) -> IoLinkResult<()> {
        use DlModeEvent::*;
        use DlModeState::*;

        let new_state = match (self.state, event) {
            (Idle, Initialize) => Startup,
            (Startup, StartupComplete) => Preoperate,
            (Startup, Error) => Idle,
            (Preoperate, ComChange(mode)) => {
                self.target_mode = mode;
                Operate
            }
            (Operate, ComChange(mode)) => {
                self.target_mode = mode;
                Operate
            }
            (Operate, Fallback) => Preoperate,
            (Operate, Error) => Preoperate,
            (Preoperate, Error) => Idle,
            (Preoperate, Fallback) => Idle,
            _ => return Err(IoLinkError::InvalidParameter),
        };

        self.state = new_state;
        Ok(())
    }

    /// Poll the state machine
    /// See IO-Link v1.1.4 Section 7.3.2.5
    pub fn poll<P: PhysicalLayer>(&mut self, phy: &mut P) -> IoLinkResult<()> {
        match self.state {
            DlModeState::Idle => {
                // Initialize physical layer
                self.process_event(DlModeEvent::Initialize)?;
            }
            DlModeState::EstablishCom => {
                // Handle Establish Communication state (not implemented in detail)
                // See IO-Link v1.1.4 Section 7.3.2.5 Table 45
                // For now, transition to Startup or Preoperate as appropriate
                // You may want to implement actual communication establishment here
                // For now, just transition to Startup for demonstration
                self.state = DlModeState::Startup;
            }
            DlModeState::Startup => {
                // Perform startup sequence
                self.handle_startup(phy)?;
            }
            DlModeState::Preoperate => {
                // Wait for mode change request
                self.handle_preoperate(phy)?;
            }
            DlModeState::Operate => {
                // Normal operation
                self.handle_operate(phy)?;
            }
        }
        Ok(())
    }

    /// Handle startup state
    fn handle_startup<P: PhysicalLayer>(&mut self, phy: &mut P) -> IoLinkResult<()> {
        // Initialize physical layer
        phy.pl_wake_up()?;
        
        // Set initial mode to SIO
        phy.pl_set_mode(IoLinkMode::Sio)?;
        self.current_mode = IoLinkMode::Sio;
        
        // Signal startup complete
        self.process_event(DlModeEvent::StartupComplete)?;
        
        Ok(())
    }

    /// Handle preoperate state
    fn handle_preoperate<P: PhysicalLayer>(&mut self, _phy: &mut P) -> IoLinkResult<()> {
        // Wait for mode change request from master
        // This would typically involve listening for wake-up pulses
        // and master commands
        Ok(())
    }

    /// Handle operate state
    fn handle_operate<P: PhysicalLayer>(&mut self, phy: &mut P) -> IoLinkResult<()> {
        // Check if mode change is needed
        if self.current_mode != self.target_mode {
            phy.pl_set_mode(self.target_mode)?;
            self.current_mode = self.target_mode;
        }

        // Monitor communication status
        match phy.pl_status() {
            crate::types::PhysicalLayerStatus::Error => {
                self.error_count += 1;
                if self.error_count > 3 {
                    self.process_event(DlModeEvent::Error)?;
                    self.error_count = 0;
                }
            }
            crate::types::PhysicalLayerStatus::Communication => {
                self.error_count = 0;
            }
            _ => {}
        }

        Ok(())
    }

    /// Get current state
    pub fn state(&self) -> DlModeState {
        self.state
    }

    /// Get current mode
    pub fn current_mode(&self) -> IoLinkMode {
        self.current_mode
    }

    /// Get target mode
    pub fn target_mode(&self) -> IoLinkMode {
        self.target_mode
    }

    /// Request mode change
    pub fn request_mode_change(&mut self, mode: IoLinkMode) -> IoLinkResult<()> {
        self.process_event(DlModeEvent::ComChange(mode))
    }

    /// Request fallback
    pub fn request_fallback(&mut self) -> IoLinkResult<()> {
        self.process_event(DlModeEvent::Fallback)
    }
}

impl Default for DlModeHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::MockHal;

    #[test]
    fn test_dl_mode_initialization() {
        let mut handler = DlModeHandler::new();
        let mut hal = MockHal::new();

        assert_eq!(handler.state(), DlModeState::Idle);
        
        // Poll should trigger initialization
        handler.poll(&mut hal).unwrap();
        assert_eq!(handler.state(), DlModeState::Startup);
    }

    #[test]
    fn test_mode_change() {
        let mut handler = DlModeHandler::new();
        
        // First need to initialize and get to preoperate state
        handler.process_event(DlModeEvent::Initialize).unwrap();
        handler.process_event(DlModeEvent::StartupComplete).unwrap();
        
        // Now request mode change should work
        handler.request_mode_change(IoLinkMode::Com2).unwrap();
        assert_eq!(handler.target_mode(), IoLinkMode::Com2);
    }

    #[test]
    fn test_error_handling() {
        let mut handler = DlModeHandler::new();
        
        handler.process_event(DlModeEvent::Initialize).unwrap();
        assert_eq!(handler.state(), DlModeState::Startup);
        
        handler.process_event(DlModeEvent::Error).unwrap();
        assert_eq!(handler.state(), DlModeState::Idle);
    }
}
