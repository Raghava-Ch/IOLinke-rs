//! Event Handler
//!
//! This module implements the Event Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 8.4.4

use crate::{
    dl::{self, message_handler::OdInd}, storage, types::{self, EventType, IoLinkError, IoLinkResult}
};

/// {EventRead} See Table 60, Tiggers T4, T6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OdIndData {
    rw_direction: types::RwDirection,
    com_channel: types::ComChannel,
    address_ctrl: u8,
    length: u8,
}
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
    T4(OdIndData),
    /// T5: State: FreezeEventMemory (2) -> Idle (1)
    /// Action: Invoke service EventFlag.req (Flag = FALSE) to indicate Event deactivation to the Master via the "Event flag" bit. Mark all Event slots in memory as invalid according to A.6.3.
    T5,
    /// T6: State: Idle (1) -> Idle (1)
    /// Action: Send contents of Event memory by invoking OD.rsp with Event data
    T6(OdIndData),
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
    EventRead(OdIndData),
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
    /// See 7.3.8.1 Events and Table 58 – Event memory
    event_memory: storage::event_memory::EventMemory,
}

impl EventHandler {
    /// Create a new Event Handler
    pub fn new() -> Self {
        Self {
            state: EventHandlerState::Inactive,
            exec_transition: Transition::Tn,
            event_memory: storage::event_memory::EventMemory::new(),
        }
    }

    /// Process an event
    pub fn process_event(&mut self, event: EventHandlerEvent) -> IoLinkResult<()> {
        use EventHandlerEvent as Event;
        use EventHandlerState as State;

        let (new_transition, new_state) = match (self.state, event) {
            (State::Inactive, Event::EhConfActive) => (Transition::T1, State::Idle),
            (State::Idle, Event::DlEvent) => (Transition::T2, State::Idle),
            (State::Idle, Event::DLEventTrigger) => (Transition::T3, State::FreezeEventMemory),
            (State::FreezeEventMemory, Event::EventRead(od_ind_data)) => {
                (Transition::T4(od_ind_data), State::FreezeEventMemory)
            }
            (State::FreezeEventMemory, Event::EventConf) => {
                (Transition::T5, State::Idle)
            }
            (State::Idle, Event::EventRead(od_ind_data)) => {
                (Transition::T6(od_ind_data), State::Idle)
            }
            (State::Idle, Event::EhConfInactive) => (Transition::T7, State::Inactive),
            (State::FreezeEventMemory, Event::EhConfInactive) => (Transition::T8, State::Inactive),
            _ => return Err(IoLinkError::InvalidEvent),
        };
        // Update the state and transition
        self.state = new_state;
        self.exec_transition = new_transition;
        Ok(())
    }

    /// Poll the handler
    pub fn poll(&mut self, message_handler: &mut dl::message_handler::MessageHandler) -> IoLinkResult<()> {
        // Process pending events
        match self.exec_transition {
            Transition::Tn => {
                // No transition to execute
            }
            Transition::T1 => {
                self.exec_transition = Transition::Tn;
                self.execute_t1();
            }
            Transition::T2 => {
                self.exec_transition = Transition::Tn;
                self.execute_t2();
            }
            Transition::T3 => {
                self.exec_transition = Transition::Tn;
                self.execute_t3(message_handler);
            }
            Transition::T4(od_ind_data) => {
                self.exec_transition = Transition::Tn;
                self.execute_t4(od_ind_data, message_handler);
            }
            Transition::T5 => {
                self.exec_transition = Transition::Tn;
                self.execute_t5(message_handler);
            }
            Transition::T6(od_ind_data) => {
                self.exec_transition = Transition::Tn;
                self.execute_t6(od_ind_data, message_handler);
            }
            Transition::T7 => {
                self.exec_transition = Transition::Tn;
                self.execute_t7();
            }
            Transition::T8 => {
                self.exec_transition = Transition::Tn;
                self.execute_t8();
            }
        }
        Ok(())
    }

    /// Execute T1 transition: Inactive (0) -> Idle (1)
    /// Action: -
    fn execute_t1(&mut self) -> IoLinkResult<()> {
        // No specific action required for this transition
        Ok(())
    }

    /// Execute T2 transition: Idle (1) -> Idle (1)
    /// Action: Change Event memory entries with new Event data (see Table 58)
    fn execute_t2(&mut self) -> IoLinkResult<()> {
        // TODO: Implement Event memory update with new Event data
        // TODO: Compose event memory entry for the given data and update it to the event memory in fifo manner
        // This should update the Event memory entries according to Table 58
        Ok(())
    }

    /// Execute T3 transition: Idle (1) -> FreezeEventMemory (2)
    /// Action: Invoke service EventFlag.req (Flag = TRUE) to indicate Event activation to the Master via the "Event flag" bit. Mark all Event slots in memory as not changeable.
    fn execute_t3(
        &mut self,
        message_handler: &mut dl::message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        self.event_memory.set_read_only(true);
        message_handler.event_flag(true);
        Ok(())
    }

    /// Execute T4 transition: FreezeEventMemory (2) -> FreezeEventMemory (2)
    /// Action: Master requests Event memory data via EventRead (= OD.ind). Send Event data by invoking OD.rsp with Event data of the requested Event memory address.
    fn execute_t4(
        &mut self,
        od_ind_data: OdIndData,
        message_handler: &mut dl::message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        // Use od_ind_data.address_ctrl to determine which Event memory address to read
        let event_data = self.event_memory
            .get_event_detail(od_ind_data.address_ctrl as usize, od_ind_data.length as usize)
            .map_err(|_| IoLinkError::InvalidAddress)?;
        let _ = message_handler.od_rsp(od_ind_data.length, event_data);
        Ok(())
    }

    /// Execute T5 transition: FreezeEventMemory (2) -> Idle (1)
    /// Action: Invoke service EventFlag.req (Flag = FALSE) to indicate Event deactivation to the Master via the "Event flag" bit. Mark all Event slots in memory as invalid according to A.6.3.
    fn execute_t5(
        &mut self, 
        message_handler: &mut dl::message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        message_handler.event_flag(false);
        self.event_memory.set_read_only(false);
        self.event_memory.clear_all_event()?;
        Ok(())
    }

    /// Execute T6 transition: Idle (1) -> Idle (1)
    /// Action: Send contents of Event memory by invoking OD.rsp with Event data
    fn execute_t6(
        &mut self,
        od_ind_data: OdIndData,
        message_handler: &mut dl::message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        self.execute_t4(od_ind_data, message_handler);
        // Use od_ind_data.address_ctrl to determine which Event memory data to send
        Ok(())
    }

    /// Execute T7 transition: Idle (1) -> Inactive (0)
    /// Action: -
    fn execute_t7(&mut self) -> IoLinkResult<()> {
        // No specific action required for this transition
        Ok(())
    }

    /// Execute T8 transition: FreezeEventMemory (2) -> Inactive (0)
    /// Action: Discard Event memory data
    fn execute_t8(&mut self) -> IoLinkResult<()> {
        self.event_memory.set_read_only(false);
        self.event_memory.clear_all_event()?;
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

    /// See 7.2.1.15 DL_Event
    /// The service DL_Event indicates a pending status or error information. The cause for an Event
    /// is located in a Device and the Device application triggers the Event transfer. The parameters
    /// of the service primitives are listed in Table 30.
    pub fn dl_event_req(
        &mut self,
        event_instance: types::EventInstance,
        event_type: EventType,
        event_mode: types::EventMode,
        event_code: u16, // device_event_code macro to be used
        events_left: u8,
    ) -> IoLinkResult<()> {
        // TODO: Implement the DL_Event request to event memory handling
        // let entry = storage::event_memory::EventEntry {
        //     event_qualifier: storage::event_memory::EventQualifier::new()
        //         .with_eq_mode(event_mode.into())
        //         .with_eq_type(event_type.into())
        //         .with_eq_source(event_instance.source().into())
        //         .with_eq_instance(event_instance.instance()),
        //     event_code,
        // };
        // self.event_memory.add_event_detail(entry);
        // Process the DL_Event request
        self.process_event(EventHandlerEvent::DlEvent);
        Ok(())
    }

    /// See 7.2.1.17 DL_EventTrigger
    /// The DL_EventTrigger request starts the Event signaling (see Event flag in Figure A.3) and
    /// freezes the Event memory within the DL. The confirmation is returned after the activated
    /// Events have been processed. Additional DL_EventTrigger requests are ignored until the
    /// previous one has been confirmed (see 7.3.8, 8.3.3 and Figure 66). This service has no
    /// parameters. The service primitives are listed in Table 32.
    pub fn dl_event_trigger_req(&mut self) -> IoLinkResult<()> {
        // Process the DL_EventTrigger request
        self.process_event(EventHandlerEvent::DLEventTrigger);
        Ok(())
    }
}

impl OdInd for EventHandler {
    /// Handle the OD.ind event
    fn od_ind(
        &mut self,
        rw_direction: types::RwDirection,
        com_channel: types::ComChannel,
        address_ctrl: u8,
        length: u8,
        _data: &[u8],
    ) -> IoLinkResult<()> {
        // Process the incoming data
        let od_ind_data = OdIndData {
            rw_direction,
            com_channel,
            address_ctrl,
            length,
        };
        let event = if rw_direction == types::RwDirection::Read
            && com_channel == types::ComChannel::Diagnosis
        {
            EventHandlerEvent::EventRead(od_ind_data)
        } else if rw_direction == types::RwDirection::Write
            && com_channel == types::ComChannel::Diagnosis
            && address_ctrl == 0x00
        {
            EventHandlerEvent::EventConf
        } else {
            return Err(IoLinkError::InvalidEvent);
        };

        self.process_event(event)?;
        Ok(())
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
