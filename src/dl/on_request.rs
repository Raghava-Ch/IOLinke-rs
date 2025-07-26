//! On request Data Handler
//!
//! This module implements the On request Data Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 7.3.5.3

use crate::types::{self, IoLinkError, IoLinkResult};

/// On request Data Handler states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OnRequestHandlerState {
    /// {Inactive_0}
    Inactive,
    /// {Idle_1}
    Idle,
}

#[derive(Debug, PartialEq, Eq)]
enum Transition {
    /// Tn: No transition
    Tn,
    /// T1: State: Inactive (0) -> Idle (1)
    /// Action: -
    T1,
    /// T2: State: Idle (1) -> Idle (1)
    /// Action: Provide data content of requested parameter or perform appropriate write action
    T2,
    /// T3: State: Idle (1) -> Idle (1)
    /// Action: Redirect to command handler
    T3,
    /// T4: State: Idle (1) -> Idle (1)
    /// Action: Redirect to ISDU handler
    T4,
    /// T5: State: Idle (1) -> Idle (1)
    /// Action: Redirect to Event handler
    T5,
    /// T6: State: Idle (1) -> Inactive (0)
    /// Action: -
    T6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OnRequestHandlerEvent {
    /// {OD_ind_Command}
    OdIndCommand,
    /// {OD_ind_Param}
    OdIndParam,
    /// {OH_Conf_ACTIVE}
    OhConfActive,
    /// {OH_Conf_INACTIVE}
    OhConfInactive,
    /// {OD_ind_ISDU}
    OdIndIsdu,
    /// {OD_ind_Event}
    OdIndEvent,
}

/// Process Data Handler implementation
pub struct OnRequestHandler {
    state: OnRequestHandlerState,
    exec_transition: Transition,
}

impl OnRequestHandler {
    /// Create a new Process Data Handler
    pub fn new() -> Self {
        Self {
            state: OnRequestHandlerState::Inactive,
            exec_transition: Transition::Tn,
        }
    }

     /// Process an event
    pub fn process_event(&mut self, event: OnRequestHandlerEvent) -> IoLinkResult<()> {
        use OnRequestHandlerEvent as Event;
        use OnRequestHandlerState as State;

        let (new_transition, new_state) = match (self.state, event) {
            (State::Inactive, Event::OhConfInactive) => (Transition::T1, State::Idle),
            (State::Idle, Event::OdIndParam) => (Transition::T2, State::Idle),
            (State::Idle, Event::OdIndCommand) => (Transition::T3, State::Idle),
            (State::Idle, Event::OdIndIsdu) => (Transition::T4, State::Idle),
            (State::Idle, Event::OdIndEvent) => (Transition::T5, State::Idle),
            (State::Idle, Event::OhConfInactive) => (Transition::T6, State::Inactive),
            _ => return Err(IoLinkError::InvalidEvent),
        };
        self.exec_transition = new_transition;
        self.state = new_state;
        
        Ok(())
    }

    /// Poll the process data handler
    /// See IO-Link v1.1.4 Section 7.2
    pub fn poll(&mut self) -> IoLinkResult<()> {
        match self.exec_transition {
            Transition::Tn => {
                // No transition to execute
            }
            Transition::T1 => {
                // Transition from Inactive to Idle
                self.state = OnRequestHandlerState::Idle;
            }
            Transition::T2 => {
                // Provide data content of requested parameter or perform appropriate write action
                // This is a placeholder for actual implementation
            }
            Transition::T3 => {
                // Redirect to command handler
                // This is a placeholder for actual implementation
            }
            Transition::T4 => {
                // Redirect to ISDU handler
                // This is a placeholder for actual implementation
            }
            Transition::T5 => {
                // Redirect to Event handler
                // This is a placeholder for actual implementation
            }
            Transition::T6 => {
                // Transition from Idle to Inactive
                self.state = OnRequestHandlerState::Inactive;
            }
        }
        Ok(())
    }

    /// Handle On request configuration changes
    /// See 7.3.5.3 State machine of the Device On-request Data handler
    pub fn oh_conf(&mut self, state: types::ChConfState) -> IoLinkResult<()> {
        match state {
            types::ChConfState::Active => self.process_event(OnRequestHandlerEvent::OhConfActive),
            types::ChConfState::Inactive => self.process_event(OnRequestHandlerEvent::OhConfInactive),
        }
    }

}

impl Default for OnRequestHandler {
    fn default() -> Self {
        Self::new()
    }
}
