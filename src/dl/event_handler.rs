//! Event Handler
//!
//! This module implements the Event Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 8.4.4

use crate::types::{self, Event, EventType, IoLinkError, IoLinkResult};
use heapless::Deque;

/// See Table 60 – State transition tables of the Device Event handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventHandlerState {
    /// - Inactive_0: Waiting on activation
    Inactive,
    /// - Idle_1: Waiting on DL-Event service from 
    /// AL providing Event data and the DL_EventTrigger
    /// service to fire the "Event flag" bit (see A.1.5)
    Idle,
    /// - FreezeEventMemory_2: Waiting on readout of the Event memory and on Event memory readout
    /// confirmation through write access to the StatusCode
    FreezeEventMemory,
}

/// See Table 60 – State transition tables of the Device Event handler
#[derive(Debug, PartialEq, Eq)]
enum Transition {
    /// Tn: No transition
    Tn,
    /// T1: State: Inactive (0) -> Idle (1)
    /// Action: -
    T1,
    /// T2: State: Idle (1) -> Idle (1)
    /// Action: Change Event memory entries with new Event data (see Table 58)
    T2,
    /// T3: State: Idle (1) -> FreezeEventMemory (2)
    /// Action: Invoke service EventFlag.req (Flag = TRUE) to indicate Event activation to the Master via the "Event flag" bit. Mark all Event slots in memory as not changeable.
    T3,
    /// T4: State: FreezeEventMemory (2) -> FreezeEventMemory (2)
    /// Action: Master requests Event memory data via EventRead (= OD.ind). Send Event data by invoking OD.rsp with Event data of the requested Event memory address.
    T4,
    /// T5: State: FreezeEventMemory (2) -> Idle (1)
    /// Action: Invoke service EventFlag.req (Flag = FALSE) to indicate Event deactivation to the Master via the "Event flag" bit. Mark all Event slots in memory as invalid according to A.6.3.
    T5,
    /// T6: State: Idle (1) -> Idle (1)
    /// Action: Send contents of Event memory by invoking OD.rsp with Event data
    T6,
    /// T7: State: Idle (1) -> Inactive (0)
    /// Action: -
    T7,
    /// T8: State: FreezeEventMemory (2) -> Inactive (0)
    /// Action: Discard Event memory data
    T8,
}

/// Figure 56 – State machine of the Device Event handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventHandlerEvent {
    /// {EH_Conf_ACTIVE} See Table 60, Tiggers T1
    EhConfActive,
    /// {DL_Event} See Table 60, Tiggers T2
    DlEvent,
    /// {DL_EventTrigger} See Table 60, Tiggers T3
    DLEventTrigger,
    /// {EventRead} See Table 60, Tiggers T4, T6
    EventRead,
    /// {EventConf} See Table 60, Tiggers T5
    EventConf,
    /// {EH_Config_INACTIVE} See Table 60, Tiggers T7, T8
    EhConfInactive,
}
/// Event Handler implementation
pub struct EventHandler {
    /// Current state of the Event Handler
    state: EventHandlerState,
    exec_transition: Transition,
}

impl EventHandler {
    /// Create a new Event Handler
    pub fn new() -> Self {
        Self {
            state: EventHandlerState::Inactive,
            exec_transition: Transition::Tn,
        }
    }

    /// Process an event
    pub fn process_event(&mut self, event: EventHandlerEvent) -> IoLinkResult<()> {
        use EventHandlerEvent as Event;
        use EventHandlerState as State;

        let (new_transition, new_state) = match (self.state, event) {
            (State::Inactive, Event::EhConfActive) => {
                (Transition::T1, State::Idle)
            }
            (State::Idle, Event::DlEvent) => {
                (Transition::T2, State::Idle)
            }
            (State::Idle, Event::DLEventTrigger) => {
                (Transition::T3, State::FreezeEventMemory)
            }
            (State::FreezeEventMemory, Event::EventRead) => {
                (Transition::T4, State::FreezeEventMemory)
            }
            (State::FreezeEventMemory, Event::EventConf) => {
                (Transition::T5, State::Idle)
            }
            (State::Idle, Event::EventRead) => {
                (Transition::T6, State::Idle)
            }
            (State::Idle, Event::EhConfInactive) => {
                (Transition::T7, State::Inactive)
            }
            (State::FreezeEventMemory, Event::EhConfInactive) => {
                (Transition::T8, State::Inactive)
            }
            _ => return Err(IoLinkError::InvalidEvent),
        };
        // Update the state and transition
        self.state = new_state;
        self.exec_transition = new_transition;
        Ok(())
    }

    /// Poll the handler
    pub fn poll(&mut self) -> IoLinkResult<()> {
        // Process pending events
        match self.exec_transition {
            Transition::Tn => {
                // No transition to execute
            }
            Transition::T1 => {
                // State: Inactive (0) -> Idle (1)
                // Action: -
            }
            Transition::T2 => {
                // State: Idle (1) -> Idle (1)
                // Action: Change Event memory entries with new Event data (see Table 58)
            }
            Transition::T3 => {
                // State: Idle (1) -> FreezeEventMemory (2)
                // Action: Invoke service EventFlag.req (Flag = TRUE) to indicate Event activation to the Master via the "Event flag" bit. Mark all Event slots in memory as not changeable.
            }
            Transition::T4 => {
                // State: FreezeEventMemory (2) -> FreezeEventMemory (2)
                // Action: Master requests Event memory data via EventRead (= OD.ind). Send Event data by invoking OD.rsp with Event data of the requested Event memory address.
            }
            Transition::T5 => {
                // State: FreezeEventMemory (2) -> Idle (1)
                // Action: Invoke service EventFlag.req (Flag = FALSE) to indicate Event deactivation to the Master via the "Event flag" bit. Mark all Event slots in memory as invalid according to A.6.3.
            }
            Transition::T6 => {
                // State: Idle (1) -> Idle (1)
                // Action: Send contents of Event memory by invoking OD.rsp with Event data
            }
            Transition::T7 => {
                // State: Idle (1) -> Inactive (0)
                // Action: -
            }
            Transition::T8 => {
                // State: FreezeEventMemory (2) -> Inactive (0)
                // Action: Discard Event memory data
            }
        }
        Ok(())
    }

    /// Event handler conf for updating Active or Inactive
    /// See 7.3.8.4 State machine of the Device Event handler
    pub fn eh_conf(&mut self, state: types::EhConfState) -> IoLinkResult<()> {
        // Update the event handler configuration
        match state {
            types::EhConfState::Active => self.process_event(EventHandlerEvent::EhConfActive),
            types::EhConfState::Inactive => self.process_event(EventHandlerEvent::EhConfInactive),
        }?;

        Ok(())
    }

}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
