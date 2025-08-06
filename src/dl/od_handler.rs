//! On request Data Handler
//!
//! This module implements the On request Data Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 7.3.5.3

use crate::{
    al, dl,
    types::{self, IoLinkError, IoLinkResult},
};

pub trait OdInd<'a> {
    /// Invoke OD.ind service with the provided data
    fn od_ind(&mut self, od_ind_data: &'a OdIndData) -> IoLinkResult<()>;
}

pub trait OdRsp {
    /// Invoke OD.rsp service with the provided data
    fn od_rsp(&mut self, length: u8, data: &[u8]) -> IoLinkResult<()>;
}

pub trait DlWriteParamInd {
    /// See 7.2.1.3 DL_WriteParam
    /// The DL_WriteParam service is used by the AL to write a parameter value to the Device via
    /// the page communication channel. The parameters of the service primitives are listed in Table 18.
    fn write_param_ind(&mut self, index: u8, data: u8) -> IoLinkResult<()>;
}

pub trait DlParamRsp {
    /// See 7.2.1.4 DL_ReadParam.rsp
    fn dl_read_param_rsp(&mut self, length: u8, data: &[u8]) -> IoLinkResult<()>;
    fn dl_write_param_rsp(&mut self) -> IoLinkResult<()>;
}

pub trait DlReadParamInd {
    /// See 7.2.1.2 DL_ReadParam
    /// The DL_ReadParam service is used by the AL to read a parameter value from the Device via
    /// the page communication channel. The parameters of the service primitives are listed in Table 17.
    fn read_param_ind(&mut self, address: u8) -> IoLinkResult<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OdIndData<'a> {
    pub rw_direction: types::RwDirection,
    pub com_channel: types::ComChannel,
    pub address_ctrl: u8,
    pub length: u8,
    pub data: &'a [u8],
}

impl<'a> OdIndData<'a> {
    pub fn new() -> Self {
        Self {
            rw_direction: types::RwDirection::Read,
            com_channel: types::ComChannel::Page,
            address_ctrl: 0,
            length: 0,
            data: &[],
        }
    }
}

impl<'a> Default for OdIndData<'a> {
    fn default() -> Self {
        Self::new()
    }
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
enum Transition<'a> {
    /// Tn: No transition
    Tn,
    /// T1: State: Inactive (0) -> Idle (1)
    /// Action: -
    T1,
    /// T2: State: Idle (1) -> Idle (1)
    /// Action: Provide data content of requested parameter or perform appropriate write action
    T2(&'a OdIndData<'a>),
    /// T3: State: Idle (1) -> Idle (1)
    /// Action: Redirect to command handler
    T3(&'a OdIndData<'a>),
    /// T4: State: Idle (1) -> Idle (1)
    /// Action: Redirect to ISDU handler
    T4(&'a OdIndData<'a>),
    /// T5: State: Idle (1) -> Idle (1)
    /// Action: Redirect to Event handler
    T5(&'a OdIndData<'a>),
    /// T6: State: Idle (1) -> Inactive (0)
    /// Action: -
    T6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OnRequestHandlerEvent<'a> {
    /// {OD_ind_Command}
    OdIndCommand(&'a OdIndData<'a>),
    /// {OD_ind_Param}
    OdIndParam(&'a OdIndData<'a>),
    /// {OH_Conf_ACTIVE}
    OhConfActive,
    /// {OH_Conf_INACTIVE}
    OhConfInactive,
    /// {OD_ind_ISDU}
    OdIndIsdu(&'a OdIndData<'a>),
    /// {OD_ind_Event}
    OdIndEvent(&'a OdIndData<'a>),
}

/// Process Data Handler implementation
pub struct OnRequestDataHandler<'a> {
    state: OnRequestHandlerState,
    exec_transition: Transition<'a>,
}

impl<'a> OnRequestDataHandler<'a> {
    /// Create a new Process Data Handler
    pub fn new() -> Self {
        Self {
            state: OnRequestHandlerState::Inactive,
            exec_transition: Transition::Tn,
        }
    }

    /// Process an event
    pub fn process_event(&mut self, event: OnRequestHandlerEvent<'a>) -> IoLinkResult<()> {
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
        isdu_handler: &mut dl::isdu_handler::IsduHandler<'a>,
        event_handler: &mut dl::event_handler::EventHandler<'a>,
        application_layer: &mut al::ApplicationLayer,
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
                self.execute_t2(od_ind_data, command_handler, application_layer)?;
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
        od_ind_data: &OdIndData,
        command_handler: &mut dl::command_handler::CommandHandler,
        application_layer: &mut al::ApplicationLayer,
    ) -> IoLinkResult<()> {
        if od_ind_data.com_channel == types::ComChannel::Page {
            if od_ind_data.rw_direction == types::RwDirection::Read {
                // Provide data content of requested parameter
                application_layer.read_param_ind(od_ind_data.address_ctrl)?;
            } else if od_ind_data.rw_direction == types::RwDirection::Write {
                // Perform appropriate write action
                application_layer.write_param_ind(od_ind_data.address_ctrl, od_ind_data.data[0])?;
            }
        } else {
            return Err(IoLinkError::InvalidEvent);
        }
        command_handler.od_ind(&od_ind_data)?;
        Ok(())
    }

    /// Handle transition T3: Idle (1) -> Idle (1)
    /// Action: Redirect to command handler
    fn execute_t3(
        &mut self,
        od_ind_data: &OdIndData,
        command_handler: &mut dl::command_handler::CommandHandler,
    ) -> IoLinkResult<()> {
        command_handler.od_ind(od_ind_data)?;
        Ok(())
    }

    /// Handle transition T4: Idle (1) -> Idle (1)
    /// Action: Redirect to ISDU handler
    fn execute_t4(
        &mut self,
        od_ind_data: &'a OdIndData,
        isdu_handler: &mut dl::isdu_handler::IsduHandler<'a>,
    ) -> IoLinkResult<()> {
        isdu_handler.od_ind(od_ind_data)?;
        Ok(())
    }

    /// Handle transition T5: Idle (1) -> Idle (1)
    /// Action: Redirect to Event handler
    fn execute_t5(
        &mut self,
        od_ind_data: &'a OdIndData<'a>,
        event_handler: &mut dl::event_handler::EventHandler<'a>,
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

    pub fn od_rsp(
        &mut self,
        length: u8,
        data: &[u8],
        message_handler: &mut dl::message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        message_handler.od_rsp(length, data)?;
        Ok(())
    }

    pub fn dl_read_param_rsp(
        &mut self,
        length: u8,
        data: &[u8],
        message_handler: &mut dl::message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        self.od_rsp(length, data, message_handler);
        Ok(())
    }
}

impl<'a> OdInd<'a> for OnRequestDataHandler<'a> {
    fn od_ind(&mut self, od_ind_data: &'a OdIndData<'a>) -> IoLinkResult<()> {
        let event = match od_ind_data.com_channel {
            types::ComChannel::Page => {
                if od_ind_data.address_ctrl == 0
                    && od_ind_data.rw_direction == types::RwDirection::Write
                {
                    OnRequestHandlerEvent::OdIndCommand(od_ind_data)
                } else if (1..=31).contains(&od_ind_data.address_ctrl) {
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

impl<'a> Default for OnRequestDataHandler<'a> {
    fn default() -> Self {
        Self::new()
    }
}
