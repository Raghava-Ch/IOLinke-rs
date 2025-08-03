//! On request Data Handler
//!
//! This module implements the On request Data Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 7.3.5.3

use crate::{
    dl,
    types::{self, IoLinkError, IoLinkResult},
};

pub trait OdInd {
    /// Invoke OD.ind service with the provided data
    fn od_ind(&mut self, od_ind_data: &OdIndData) -> IoLinkResult<()>;
}

pub trait DlWriteParamInd {
    /// See 7.2.1.3 DL_WriteParam
    /// The DL_WriteParam service is used by the AL to write a parameter value to the Device via
    /// the page communication channel. The parameters of the service primitives are listed in Table 18.
    fn write_param_ind(
        &mut self,
        index: u8,
        data: u8,
    ) -> IoLinkResult<()>;
}

pub trait DlReadParamInd {
    /// See 7.2.1.2 DL_ReadParam
    /// The DL_ReadParam service is used by the AL to read a parameter value from the Device via
    /// the page communication channel. The parameters of the service primitives are listed in Table 17.
    fn read_param_ind(&mut self, address: u8) -> IoLinkResult<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OdIndData {
    pub rw_direction: types::RwDirection,
    pub com_channel: types::ComChannel,
    pub address_ctrl: u8,
    pub length: u8,
    pub data: [u8; 32],
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
    pub fn poll(
        &mut self,
        command_handler: &mut dl::command_handler::CommandHandler,
        isdu_handler: &mut dl::isdu_handler::IsduHandler,
        event_handler: &mut dl::event_handler::EventHandler,
    ) -> IoLinkResult<()> {
        match self.exec_transition {
            Transition::Tn => {
                self.exec_transition = Transition::Tn;
                // No transition to execute
            }
            Transition::T1 => {
                self.exec_transition = Transition::Tn;
                // Transition from Inactive to Idle
                self.execute_t1()?;
            }
            Transition::T2(od_ind_data) => {
                self.exec_transition = Transition::Tn;
                // Provide data content of requested parameter or perform appropriate write action
                self.execute_t2(od_ind_data, command_handler)?;
            }
            Transition::T3(od_ind_data) => {
                self.exec_transition = Transition::Tn;
                // Redirect to command handler
                self.execute_t3(od_ind_data, command_handler)?;
            }
            Transition::T4(od_ind_data) => {
                self.exec_transition = Transition::Tn;
                // Redirect to ISDU handler
                self.execute_t4(od_ind_data, isdu_handler)?;
            }
            Transition::T5(od_ind_data) => {
                self.exec_transition = Transition::Tn;
                // Redirect to Event handler
                self.execute_t5(od_ind_data, event_handler)?;
            }
            Transition::T6 => {
                self.exec_transition = Transition::Tn;
                // Transition from Idle to Inactive
                self.execute_t6()?;
            }
        }
        Ok(())
    }

    /// Handle transition T1: Inactive (0) -> Idle (1)
    /// Action: -
    fn execute_t1(&mut self) -> IoLinkResult<()> {
        Ok(())
    }

    /// Handle transition T2: Idle (1) -> Idle (1)
    /// Action: Provide data content of requested parameter or perform appropriate write action
    fn execute_t2(
        &mut self,
        od_ind_data: OdIndData,
        command_handler: &mut dl::command_handler::CommandHandler,
    ) -> IoLinkResult<()> {
        command_handler.od_ind(&od_ind_data)?;
        Ok(())
    }

    /// Handle transition T3: Idle (1) -> Idle (1)
    /// Action: Redirect to command handler
    fn execute_t3(
        &mut self,
        od_ind_data: OdIndData,
        command_handler: &mut dl::command_handler::CommandHandler,
    ) -> IoLinkResult<()> {
        command_handler.od_ind(&od_ind_data)?;
        Ok(())
    }

    /// Handle transition T4: Idle (1) -> Idle (1)
    /// Action: Redirect to ISDU handler
    fn execute_t4(
        &mut self,
        od_ind_data: OdIndData,
        isdu_handler: &mut dl::isdu_handler::IsduHandler,
    ) -> IoLinkResult<()> {
        isdu_handler.od_ind(&od_ind_data)?;
        Ok(())
    }

    /// Handle transition T5: Idle (1) -> Idle (1)
    /// Action: Redirect to Event handler
    fn execute_t5(
        &mut self,
        od_ind_data: OdIndData,
        event_handler: &mut dl::event_handler::EventHandler,
    ) -> IoLinkResult<()> {
        event_handler.od_ind(&od_ind_data)?;
        Ok(())
    }

    /// Handle transition T6: Idle (1) -> Inactive (0)
    /// Action: -
    fn execute_t6(&mut self) -> IoLinkResult<()> {
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

impl OdInd for OnRequestDataHandler {
    fn od_ind(&mut self, od_ind_data: &OdIndData) -> IoLinkResult<()> {
        let event = match od_ind_data.com_channel {
            types::ComChannel::Page => {
                if od_ind_data.address_ctrl == 0
                    && od_ind_data.rw_direction == types::RwDirection::Write
                {
                    OnRequestHandlerEvent::OdIndCommand(od_ind_data.clone())
                } else if (1..=31).contains(&od_ind_data.address_ctrl) {
                    OnRequestHandlerEvent::OdIndParam(od_ind_data.clone())
                } else {
                    return Err(IoLinkError::InvalidEvent);
                }
            }
            types::ComChannel::Isdu => OnRequestHandlerEvent::OdIndIsdu(od_ind_data.clone()),
            types::ComChannel::Diagnosis => OnRequestHandlerEvent::OdIndEvent(od_ind_data.clone()),
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
