//! Event State Machine
//!
//! This module implements the Event State Machine as defined in
//! IO-Link Specification v1.1.4

use crate::types::{IoLinkError, IoLinkResult};

/// See 8.3.3.2 Event state machine of the Device AL
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventStateMachineState {
    /// {Event_inactive_0}
    EventInactive,
    /// {Event_idle_1}
    EventIdle,
    /// {Await_Event_response_2}
    AwaitEventResponse,
}

// See Table 77 – State and transitions of the Event state machine of the Device AL
#[derive(Debug, PartialEq, Eq)]
enum Transition {
    /// Tn: No transition
    Tn,
    /// T1: State: EventInactive (0) -> EventIdle (1)
    /// Action: -
    T1,
    /// T2: State: EventIdle (1) -> EventInactive (0)
    /// Action: -
    T2,
    /// T3: State: EventIdle (1) -> AwaitEventResponse (2)
    /// Action: An AL_Event request triggers a DL_Event and the corresponding
    /// DL_EventTrigger service. The DL_Event carries the diagnosis information
    /// from AL to DL. The DL_EventTrigger sets the Event flag within the cyclic
    /// data exchange (see A.1.5).
    T3,
    /// T4: State: AwaitEventResponse (2) -> EventIdle (1)
    /// Action: A DL_EventTrigger confirmation triggers an AL_Event confirmation.
    T4,
}

/// See 8.3.3.2 Event state machine of the Device AL
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventStateMachineEvent {
    /// {Activate} See 8.3.3.2 , Triggers T1
    Activate,
    /// {Deactivate} See 8.3.3.2, Triggers T2
    Deactivate,
    /// {AL_Event_request} See 8.3.3.2, Triggers T3
    AlEventRequest,
    /// {DL_EventTrigger_conf} See 8.3.3.2, Triggers T4
    DlEventTriggerConf,
}

/// Event State Machine implementation
pub struct EventStateMachine {
    state: EventStateMachineState,
    exec_transition: Transition,
}

impl EventStateMachine {
    /// Create a new Event State Machine
    pub fn new() -> Self {
        Self {
            state: EventStateMachineState::EventInactive,
            exec_transition: Transition::Tn,
        }
    }

    /// Process an event
    pub fn process_event(&mut self, event: EventStateMachineEvent) -> IoLinkResult<()> {
        use EventStateMachineEvent as Event;
        use EventStateMachineState as State;

        let (new_transition, new_state) = match (self.state, event) {
            // Valid transitions according to Table 77
            (State::EventInactive, Event::Activate) => {
                (Transition::T1, State::EventIdle)
            }
            (State::EventIdle, Event::Deactivate) => {
                (Transition::T2, State::EventInactive)
            }
            (State::EventIdle, Event::AlEventRequest) => {
                (Transition::T3, State::AwaitEventResponse)
            }
            (State::AwaitEventResponse, Event::DlEventTriggerConf) => {
                (Transition::T4, State::EventIdle)
            }
            // Invalid transitions - no state change
            _ => return Err(IoLinkError::InvalidEvent),
        };

        self.exec_transition = new_transition;
        self.state = new_state;

        Ok(())
    }

    /// Poll the state machine
    pub fn poll(&mut self) -> IoLinkResult<()> {
        // Process pending transitions
        match self.exec_transition {
            Transition::Tn => {
                // No transition to process
            }
            Transition::T1 => {
                // Transition T1: Activate -> EventIdle
                // No action needed, state already updated
            }
            Transition::T2 => {
                // Transition T2: Deactivate -> EventInactive
                // No action needed, state already updated
            }
            Transition::T3 => {
                // Transition T3: AlEventRequest -> AwaitEventResponse
                // This is where we would trigger the DL_Event and DL_EventTrigger service
            }
            Transition::T4 => {
                // Transition T4: DlEventTriggerConf -> EventIdle
                // This is where we would trigger the AL_Event confirmation
            }
        }
        Ok(())
    }
}

impl Default for EventStateMachine {
    fn default() -> Self {
        Self::new()
    }
}
