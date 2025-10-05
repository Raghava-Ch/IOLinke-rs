//! Event Handler Module for IO-Link Device Application Layer
//!
//! This module implements the Event State Machine as specified in IO-Link standard section 8.3.3.2,
//! handling event activation, deactivation, AL_Event requests, and DL_EventTrigger confirmations.
//!
//! # Overview
//!
//! The event state machine manages the lifecycle of events within the IO-Link Device Application Layer (AL).
//! It transitions between inactive, idle, and awaiting response states based on incoming events and service requests.
//!
//! ## State Machine
//!
//! - **EventInactive**: The initial state where no events are processed.
//! - **EventIdle**: Ready to process AL_Event requests.
//! - **AwaitEventResponse**: Waiting for confirmation from the Data Link Layer (DL).
//!
//! ## Transitions
//!
//! - **T1**: Activation (EventInactive → EventIdle)
//! - **T2**: Deactivation (EventIdle → EventInactive)
//! - **T3**: AL_Event request (EventIdle → AwaitEventResponse)
//! - **T4**: DL_EventTrigger confirmation (AwaitEventResponse → EventIdle)
//!
//! ## Integration
//!
//! The module provides an `EventHandler` struct implementing the state machine logic, and integrates with
//! AL and DL services via trait implementations. It uses heapless vectors for event entry storage to support
//! embedded environments.
//!
//! # Usage
//!
//! - Instantiate `EventHandler` and use its methods to process events and poll transitions.
//! - Implement required AL and DL service traits for integration with the rest of the IO-Link stack.
//!
//! # References
//!
//! - IO-Link Specification, Section 8.3.3.2: Event State Machine of the Device AL
//! - Table 77: State and transitions of the Event state machine

use heapless::Vec;
use iolinke_types::handlers::event::DlEventReq;
use iolinke_types::{
    custom::{IoLinkError, IoLinkResult},
    handlers::event::EventEntry,
};

use core::default::Default;
use core::option::{
    Option,
    Option::{None, Some},
};
pub use core::result::Result::{Err, Ok};

use crate::al::services;
use crate::{al::services::AlEventReq, dl};

struct Events {
    event_code: u8,
    event_entries: heapless::Vec<EventEntry, 6>,
}

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
    _Activate,
    /// {Deactivate} See 8.3.3.2, Triggers T2
    _Deactivate,
    /// {AL_Event_request} See 8.3.3.2, Triggers T3
    AlEventRequest,
    /// {DL_EventTrigger_conf} See 8.3.3.2, Triggers T4
    DlEventTriggerConf,
}

/// Event State Machine implementation
pub struct EventHandler {
    state: EventStateMachineState,
    exec_transition: Transition,
    events: Option<Events>,
}

impl EventHandler {
    /// Create a new Event State Machine
    pub fn new() -> Self {
        Self {
            state: EventStateMachineState::EventInactive,
            exec_transition: Transition::Tn,
            events: None,
        }
    }

    /// Process an event
    pub fn process_event(&mut self, event: EventStateMachineEvent) -> IoLinkResult<()> {
        use EventStateMachineEvent as Event;
        use EventStateMachineState as State;

        let (new_transition, new_state) = match (self.state.clone(), event) {
            // Valid transitions according to Table 77
            (State::EventInactive, Event::_Activate) => (Transition::T1, State::EventIdle),
            (State::EventIdle, Event::_Deactivate) => (Transition::T2, State::EventInactive),
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
    pub fn poll<ALS: services::AlEventCnf>(
        &mut self,
        application: &mut ALS,
        data_link_layer: &mut dl::DataLinkLayer,
    ) -> IoLinkResult<()> {
        // Process pending transitions
        match self.exec_transition {
            Transition::Tn => {
                // No transition to process
            }
            Transition::T1 => {
                // Transition T1: Activate -> EventIdle
                self.execute_t1()?;
            }
            Transition::T2 => {
                // Transition T2: Deactivate -> EventInactive
                self.execute_t2()?;
            }
            Transition::T3 => {
                // Transition T3: AlEventRequest -> AwaitEventResponse
                self.execute_t3(data_link_layer)?;
            }
            Transition::T4 => {
                // Transition T4: DlEventTriggerConf -> EventIdle
                self.execute_t4(application)?;
            }
        }
        Ok(())
    }

    /// Execute transition T1: EventInactive -> EventIdle
    fn execute_t1(&mut self) -> IoLinkResult<()> {
        // T1: Activate -> EventIdle
        // No action needed according to specification
        Ok(())
    }

    /// Execute transition T2: EventIdle -> EventInactive
    fn execute_t2(&mut self) -> IoLinkResult<()> {
        // T2: Deactivate -> EventInactive
        // No action needed according to specification
        Ok(())
    }

    /// Execute transition T3: EventIdle -> AwaitEventResponse
    fn execute_t3(&mut self, data_link_layer: &mut dl::DataLinkLayer) -> IoLinkResult<()> {
        // T3: AlEventRequest -> AwaitEventResponse
        // Action: An AL_Event request triggers a DL_Event and the corresponding
        // DL_EventTrigger service. The DL_Event carries the diagnosis information
        // from AL to DL. The DL_EventTrigger sets the Event flag within the cyclic
        // data exchange.

        // DL_Event and DL_EventTrigger service calls
        if let Some(events) = &self.events {
            let _ = data_link_layer.dl_event_req(events.event_code, &events.event_entries);
            let _ = data_link_layer.dl_event_trigger_req();
        }

        Ok(())
    }

    /// Execute transition T4: AwaitEventResponse -> EventIdle
    fn execute_t4<ALS: services::AlEventCnf>(&mut self, application: &mut ALS) -> IoLinkResult<()> {
        // T4: DlEventTriggerConf -> EventIdle
        // Action: A DL_EventTrigger confirmation triggers an AL_Event confirmation

        // TODO: Implement AL_Event confirmation
        application.al_event_cnf()?;
        Ok(())
    }
}

impl AlEventReq for EventHandler {
    fn al_event_req(&mut self, event_count: u8, event_entries: &[EventEntry]) -> IoLinkResult<()> {
        let mut event_entries_vec: Vec<EventEntry, 6> = Vec::new();
        let _ = event_entries_vec.extend_from_slice(event_entries);
        self.events = Some(Events {
            event_code: event_count,
            event_entries: event_entries_vec,
        });
        self.process_event(EventStateMachineEvent::AlEventRequest)
    }
}

impl dl::DlEventTriggerConf for EventHandler {
    fn event_trigger_conf(&mut self) -> IoLinkResult<()> {
        self.process_event(EventStateMachineEvent::DlEventTriggerConf)
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
