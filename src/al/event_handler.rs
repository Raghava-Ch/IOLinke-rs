//! Event State Machine
//!
//! This module implements the Event State Machine as defined in
//! IO-Link Specification v1.1.4

use crate::{
    al::{self, services::AlEventCnf}, dl::{self, DlEventReq}, storage, types::{IoLinkError, IoLinkResult}
};

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
enum Transition<'a> {
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
    T3(
        u8, // event_count
        &'a [storage::event_memory::EventEntry], // event_entries
    ),
    /// T4: State: AwaitEventResponse (2) -> EventIdle (1)
    /// Action: A DL_EventTrigger confirmation triggers an AL_Event confirmation.
    T4,
}

/// See 8.3.3.2 Event state machine of the Device AL
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventStateMachineEvent<'a> {
    /// {Activate} See 8.3.3.2 , Triggers T1
    Activate,
    /// {Deactivate} See 8.3.3.2, Triggers T2
    Deactivate,
    /// {AL_Event_request} See 8.3.3.2, Triggers T3
    AlEventRequest(
        u8, // event_count
        &'a [storage::event_memory::EventEntry], // event_entries
    ),
    /// {DL_EventTrigger_conf} See 8.3.3.2, Triggers T4
    DlEventTriggerConf,
}

/// Event State Machine implementation
pub struct EventHandler<'a> {
    state: EventStateMachineState,
    exec_transition: Transition<'a>,
}

impl<'a> EventHandler<'a> {
    /// Create a new Event State Machine
    pub fn new() -> Self {
        Self {
            state: EventStateMachineState::EventInactive,
            exec_transition: Transition::Tn,
        }
    }

    /// Process an event
    pub fn process_event(&mut self, event: EventStateMachineEvent<'a>) -> IoLinkResult<()> {
        use EventStateMachineEvent as Event;
        use EventStateMachineState as State;

        let (new_transition, new_state) = match (self.state, event) {
            // Valid transitions according to Table 77
            (State::EventInactive, Event::Activate) => (Transition::T1, State::EventIdle),
            (State::EventIdle, Event::Deactivate) => (Transition::T2, State::EventInactive),
            (
                State::EventIdle,
                Event::AlEventRequest(
                    event_count,
                    event_entries,
                ),
            ) => (
                Transition::T3(
                    event_count,
                    event_entries,
                ),
                State::AwaitEventResponse,
            ),
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
    pub fn poll(
        &mut self,
        application: &mut al::services::ApplicationLayerServices,
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
            Transition::T3(event_count, event_entries) => {
                // Transition T3: AlEventRequest -> AwaitEventResponse
                self.execute_t3(
                    event_count,
                    event_entries,
                    data_link_layer,
                )?;
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
    fn execute_t3(
        &mut self,
        event_count: u8,
        event_entries: &[storage::event_memory::EventEntry],
        data_link_layer: &mut dl::DataLinkLayer,
    ) -> IoLinkResult<()> {
        // T3: AlEventRequest -> AwaitEventResponse
        // Action: An AL_Event request triggers a DL_Event and the corresponding
        // DL_EventTrigger service. The DL_Event carries the diagnosis information
        // from AL to DL. The DL_EventTrigger sets the Event flag within the cyclic
        // data exchange.

        // TODO: Implement DL_Event and DL_EventTrigger service calls
        data_link_layer.dl_event_req(event_count, event_entries)?;

        Ok(())
    }

    /// Execute transition T4: AwaitEventResponse -> EventIdle
    fn execute_t4(
        &mut self,
        application: &mut al::services::ApplicationLayerServices,
    ) -> IoLinkResult<()> {
        // T4: DlEventTriggerConf -> EventIdle
        // Action: A DL_EventTrigger confirmation triggers an AL_Event confirmation

        // TODO: Implement AL_Event confirmation
        application.al_event_cnf()?;
        Ok(())
    }
}

impl<'a> al::services::AlEventReq<'a> for EventHandler<'a> {
    fn al_event_req(
        &mut self,
        event_count: u8,
        event_entries: &'a [storage::event_memory::EventEntry],
    ) -> IoLinkResult<()> {
        self.process_event(EventStateMachineEvent::AlEventRequest(
            event_count,
            event_entries,
        ))
    }
}

impl<'a> dl::DlEventTriggerConf for EventHandler<'a> {
    fn event_trigger_conf(&mut self) -> IoLinkResult<()> {
        self.process_event(EventStateMachineEvent::DlEventTriggerConf)
    }
}

impl<'a> Default for EventHandler<'a> {
    fn default() -> Self {
        Self::new()
    }
}
