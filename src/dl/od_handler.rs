//! On request Data Handler
//!
//! This module implements the On request Data Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 7.3.5.3

use crate::{
    dl,
    types::{self, IoLinkError, IoLinkResult},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OdIndData {
    rw_direction: types::RwDirection,
    com_channel: types::ComChannel,
    address_ctrl: u8,
    length: u8,
    data: [u8; 32],
}

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
    T2(OdIndData),
    /// T3: State: Idle (1) -> Idle (1)
    /// Action: Redirect to command handler
    T3(OdIndData),
    /// T4: State: Idle (1) -> Idle (1)
    /// Action: Redirect to ISDU handler
    T4(OdIndData),
    /// T5: State: Idle (1) -> Idle (1)
    /// Action: Redirect to Event handler
    T5(OdIndData),
    /// T6: State: Idle (1) -> Inactive (0)
    /// Action: -
    T6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OnRequestHandlerEvent {
    /// {OD_ind_Command}
    OdIndCommand(OdIndData),
    /// {OD_ind_Param}
    OdIndParam(OdIndData),
    /// {OH_Conf_ACTIVE}
    OhConfActive,
    /// {OH_Conf_INACTIVE}
    OhConfInactive,
    /// {OD_ind_ISDU}
    OdIndIsdu(OdIndData),
    /// {OD_ind_Event}
    OdIndEvent(OdIndData),
}

/// Process Data Handler implementation
pub struct OnRequestDataHandler {
    state: OnRequestHandlerState,
    exec_transition: Transition,
}

impl OnRequestDataHandler {
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
            (State::Idle, Event::OdIndParam(od_ind_data)) => {
                (Transition::T2(od_ind_data), State::Idle)
            }
            (State::Idle, Event::OdIndCommand(od_ind_data)) => {
                (Transition::T3(od_ind_data), State::Idle)
            }
            (State::Idle, Event::OdIndIsdu(od_ind_data)) => {
                (Transition::T4(od_ind_data), State::Idle)
            }
            (State::Idle, Event::OdIndEvent(od_ind_data)) => {
                (Transition::T5(od_ind_data), State::Idle)
            }
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
            Transition::T2(od_ind_data) => {
                // Provide data content of requested parameter or perform appropriate write action
                // This is a placeholder for actual implementation
            }
            Transition::T3(od_ind_data) => {
                // Redirect to command handler
                // This is a placeholder for actual implementation
            }
            Transition::T4(od_ind_data) => {
                // Redirect to ISDU handler
                // This is a placeholder for actual implementation
            }
            Transition::T5(od_ind_data) => {
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
            types::ChConfState::Inactive => {
                self.process_event(OnRequestHandlerEvent::OhConfInactive)
            }
        }
    }
}

impl dl::message_handler::OdInd for OnRequestDataHandler {
    fn od_ind(
        &mut self,
        rw_direction: types::RwDirection,
        com_channel: types::ComChannel,
        address_ctrl: u8,
        length: u8,
        data: &[u8],
    ) -> IoLinkResult<()> {
        let od_ind_data = OdIndData {
            rw_direction,
            com_channel,
            address_ctrl,
            length,
            data: data.try_into().map_err(|_| IoLinkError::InvalidData)?,
        };
        let event = match com_channel {
            types::ComChannel::Page => {
                if address_ctrl == 0 {
                    OnRequestHandlerEvent::OdIndCommand(od_ind_data)
                } else if rw_direction == types::RwDirection::Write {
                    OnRequestHandlerEvent::OdIndParam(od_ind_data)
                } else {
                    return Err(IoLinkError::InvalidEvent);
                }
            }
            types::ComChannel::Isdu => OnRequestHandlerEvent::OdIndIsdu(od_ind_data),
            types::ComChannel::Diagnosis => OnRequestHandlerEvent::OdIndEvent(od_ind_data),
            _ => return Err(IoLinkError::InvalidEvent),
        };

        self.process_event(event)?;
        Ok(())
    }
}

impl Default for OnRequestDataHandler {
    fn default() -> Self {
        Self::new()
    }
}
