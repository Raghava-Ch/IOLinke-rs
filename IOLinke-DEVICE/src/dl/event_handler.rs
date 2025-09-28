//! Event Handler
//!
//! This module implements the Event Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 8.4.4

use iolinke_types::{
    custom::{IoLinkError, IoLinkResult},
    handlers,
};
use iolinke_util::{log_state_transition, log_state_transition_error};

use core::result::Result::{Ok, Err};
use core::default::Default;

use crate::{dl::message_handler, storage};

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

type AddressCtrl = u8;
type DataLength = u8;
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
    T4(AddressCtrl, DataLength),
    /// T5: State: FreezeEventMemory (2) -> Idle (1)
    /// Action: Invoke service EventFlag.req (Flag = FALSE) to indicate Event deactivation to the Master via the "Event flag" bit. Mark all Event slots in memory as invalid according to A.6.3.
    T5,
    /// T6: State: Idle (1) -> Idle (1)
    /// Action: Send contents of Event memory by invoking OD.rsp with Event data
    T6(AddressCtrl, DataLength),
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
    EventRead(AddressCtrl, DataLength),
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
    active_event_count: u8,
}

impl EventHandler {
    /// Create a new Event Handler
    pub fn new() -> Self {
        Self {
            state: EventHandlerState::Inactive,
            exec_transition: Transition::Tn,
            event_memory: storage::event_memory::EventMemory::new(),
            active_event_count: 0,
        }
    }

    /// Process an event
    fn process_event(&mut self, event: EventHandlerEvent) -> IoLinkResult<()> {
        use EventHandlerEvent as Event;
        use EventHandlerState as State;

        let (new_transition, new_state) = match (self.state, event) {
            (State::Inactive, Event::EhConfActive) => (Transition::T1, State::Idle),
            (State::Idle, Event::DlEvent) => (Transition::T2, State::Idle),
            (State::Idle, Event::DLEventTrigger) => (Transition::T3, State::FreezeEventMemory),
            (State::FreezeEventMemory, Event::EventRead(address_ctrl, data_length)) => (
                Transition::T4(address_ctrl, data_length),
                State::FreezeEventMemory,
            ),
            (State::FreezeEventMemory, Event::EventConf) => (Transition::T5, State::Idle),
            (State::Idle, Event::EventRead(address_ctrl, data_length)) => {
                (Transition::T6(address_ctrl, data_length), State::Idle)
            }
            (State::Idle, Event::EhConfInactive) => (Transition::T7, State::Inactive),
            (State::FreezeEventMemory, Event::EhConfInactive) => (Transition::T8, State::Inactive),
            _ => {
                log_state_transition_error!(module_path!(), "process_event", self.state, event);
                (Transition::Tn, self.state)
            }
        };
        log_state_transition!(
            module_path!(),
            "process_event",
            self.state,
            new_state,
            event
        );
        // Update the state and transition
        self.state = new_state;
        self.exec_transition = new_transition;
        Ok(())
    }

    /// Poll the handler
    pub fn poll(
        &mut self,
        message_handler: &mut message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        // Process pending events
        match self.exec_transition {
            Transition::Tn => {
                // No transition to execute
            }
            Transition::T1 => {
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t1();
            }
            Transition::T2 => {
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t2();
            }
            Transition::T3 => {
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t3(message_handler);
            }
            Transition::T4(address_ctrl, data_length) => {
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t4(address_ctrl, data_length, message_handler);
            }
            Transition::T5 => {
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t5(message_handler);
            }
            Transition::T6(address_ctrl, data_length) => {
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t6(address_ctrl, data_length, message_handler);
            }
            Transition::T7 => {
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t7();
            }
            Transition::T8 => {
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t8();
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
        // This should update the Event memory entries according to Table 58
        // This transition activity is done in the dl_event_req function aka T2 transition trigger
        Ok(())
    }

    /// Execute T3 transition: Idle (1) -> FreezeEventMemory (2)
    /// Action: Invoke service EventFlag.req (Flag = TRUE) to indicate Event activation to the Master via the "Event flag" bit. Mark all Event slots in memory as not changeable.
    fn execute_t3(
        &mut self,
        message_handler: &mut message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        self.event_memory.set_read_only(true);
        message_handler.event_flag(true);
        Ok(())
    }

    /// Execute T4 transition: FreezeEventMemory (2) -> FreezeEventMemory (2)
    /// Action: Master requests Event memory data via EventRead (= OD.ind). Send Event data by invoking OD.rsp with Event data of the requested Event memory address.
    fn execute_t4(
        &self,
        address_ctrl: u8,
        data_length: u8,
        message_handler: &mut message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        // Use od_ind_data.address_ctrl to determine which Event memory address to read
        let event_data = self
            .event_memory
            .get_event_detail(address_ctrl as usize, data_length as usize)
            .map_err(|_| IoLinkError::InvalidAddress)?;
        let _ = message_handler.od_rsp(data_length, event_data);
        Ok(())
    }

    /// Execute T5 transition: FreezeEventMemory (2) -> Idle (1)
    /// Action: Invoke service EventFlag.req (Flag = FALSE) to indicate Event deactivation to the Master via the "Event flag" bit. Mark all Event slots in memory as invalid according to A.6.3.
    fn execute_t5(
        &mut self,
        message_handler: &mut message_handler::MessageHandler,
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
        address_ctrl: u8,
        data_length: u8,
        message_handler: &mut message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        // This transition activity is done in the execute_t4 function aka T4
        self.execute_t4(address_ctrl, data_length, message_handler)?;
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
    pub fn eh_conf(&mut self, state: handlers::event::EhConfState) -> IoLinkResult<()> {
        use handlers::event::EhConfState;
        // Update the event handler configuration
        match state {
            EhConfState::Active => self.process_event(EventHandlerEvent::EhConfActive),
            EhConfState::Inactive => self.process_event(EventHandlerEvent::EhConfInactive),
        }?;

        Ok(())
    }

    /// See 7.2.1.15 DL_Event
    /// The service DL_Event indicates a pending status or error information. The cause for an Event
    /// is located in a Device and the Device application triggers the Event transfer. The parameters
    /// of the service primitives are listed in Table 30.
    pub fn dl_event_req(
        &mut self,
        event_count: u8, // TODO: Check what to do with this?
        event_entries: &[handlers::event::EventEntry],
    ) -> IoLinkResult<()> {
        // TODO: Implement the DL_Event request to event memory handling
        self.active_event_count = event_count;
        self.event_memory.add_event_details(event_entries)?;
        // Process the DL_Event request
        let _ = self.process_event(EventHandlerEvent::DlEvent);
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
        let _ = self.process_event(EventHandlerEvent::DLEventTrigger);
        Ok(())
    }
}

impl handlers::od::OdInd for EventHandler {
    /// Handle the OD.ind event
    fn od_ind(&mut self, od_ind_data: &handlers::od::OdIndData) -> IoLinkResult<()> {
        use iolinke_types::frame::msequence::{ComChannel, RwDirection};
        // Process the incoming data
        let event = if od_ind_data.rw_direction == RwDirection::Read
            && od_ind_data.com_channel == ComChannel::Diagnosis
        {
            EventHandlerEvent::EventRead(od_ind_data.address_ctrl, od_ind_data.req_length)
        } else if od_ind_data.rw_direction == RwDirection::Write
            && od_ind_data.com_channel == ComChannel::Diagnosis
            && od_ind_data.address_ctrl == 0x00
        {
            EventHandlerEvent::EventConf
        } else {
            return Err(IoLinkError::InvalidEvent);
        };

        let _ = self.process_event(event);
        Ok(())
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
