//! On request Data Handler
//!
//! This module implements the On request Data Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 7.3.5.3
use iolinke_macros::direct_parameter_address;
use iolinke_types::handlers::mode::DlReadWriteInd;
use iolinke_types::handlers::od::{DlReadParamInd, DlWriteParamInd, OdInd};
use iolinke_types::{
    custom::{IoLinkError, IoLinkResult},
    frame,
    handlers::{self, od::OdIndData},
};
use iolinke_util::{log_state_transition, log_state_transition_error};

use crate::{
    al,
    dl::{command_handler, event_handler, isdu_handler, message_handler},
    system_management,
};

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
pub struct OnRequestDataHandler {
    state: OnRequestHandlerState,
    exec_transition: Transition,
    od_ind_data: handlers::od::OdIndData,
}

impl OnRequestDataHandler {
    /// Create a new Process Data Handler
    pub fn new() -> Self {
        Self {
            state: OnRequestHandlerState::Inactive,
            exec_transition: Transition::Tn,
            od_ind_data: handlers::od::OdIndData::new(),
        }
    }

    /// Process an event
    fn process_event(&mut self, event: OnRequestHandlerEvent) -> IoLinkResult<()> {
        use OnRequestHandlerEvent as Event;
        use OnRequestHandlerState as State;

        let (new_transition, new_state) = match (self.state, event) {
            (State::Inactive, Event::OhConfActive) => (Transition::T1, State::Idle),
            (State::Idle, Event::OdIndParam) => (Transition::T2, State::Idle),
            (State::Idle, Event::OdIndCommand) => (Transition::T3, State::Idle),
            (State::Idle, Event::OdIndIsdu) => (Transition::T4, State::Idle),
            (State::Idle, Event::OdIndEvent) => (Transition::T5, State::Idle),
            (State::Idle, Event::OhConfInactive) => (Transition::T6, State::Inactive),
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
        self.exec_transition = new_transition;
        self.state = new_state;

        Ok(())
    }

    /// Poll the process data handler
    /// See IO-Link v1.1.4 Section 7.2
    pub fn poll(
        &mut self,
        command_handler: &mut command_handler::CommandHandler,
        isdu_handler: &mut isdu_handler::IsduHandler,
        event_handler: &mut event_handler::EventHandler,
        application_layer: &mut al::ApplicationLayer,
        system_management: &mut system_management::SystemManagement,
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
            Transition::T2 => {
                self.exec_transition = Transition::Tn;
                // Provide data content of requested parameter or perform appropriate write action
                let od_ind_data = &(self.od_ind_data);
                self.execute_t2(
                    &od_ind_data,
                    command_handler,
                    application_layer,
                    system_management,
                )?;
            }
            Transition::T3 => {
                self.exec_transition = Transition::Tn;
                // Redirect to command handler
                let od_ind_data = &self.od_ind_data;
                self.execute_t3(
                    od_ind_data,
                    command_handler,
                    application_layer,
                    system_management,
                )?;
            }
            Transition::T4 => {
                self.exec_transition = Transition::Tn;
                // Redirect to ISDU handler
                let od_ind_data = &self.od_ind_data;
                self.execute_t4(od_ind_data, isdu_handler)?;
            }
            Transition::T5 => {
                self.exec_transition = Transition::Tn;
                // Redirect to Event handler
                let od_ind_data = &self.od_ind_data;
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
    fn execute_t1(&self) -> IoLinkResult<()> {
        Ok(())
    }

    /// Handle transition T2: Idle (1) -> Idle (1)
    /// Action: Provide data content of requested parameter or perform appropriate write action
    fn execute_t2(
        &self,
        od_ind_data: &OdIndData,
        command_handler: &mut command_handler::CommandHandler,
        application_layer: &mut al::ApplicationLayer,
        system_management: &mut system_management::SystemManagement,
    ) -> IoLinkResult<()> {
        if od_ind_data.com_channel == frame::msequence::ComChannel::Page {
            if od_ind_data.rw_direction == frame::msequence::RwDirection::Read {
                // Provide data content of requested parameter
                let _ = application_layer.dl_read_param_ind(od_ind_data.address_ctrl);
                // Informing system management about MinCycleTime is read.
                let _ = system_management.dl_read_ind(od_ind_data.address_ctrl);
            } else if od_ind_data.rw_direction == frame::msequence::RwDirection::Write {
                // Perform appropriate write action
                application_layer
                    .dl_write_param_ind(od_ind_data.address_ctrl, od_ind_data.data[0])?;
                if od_ind_data.address_ctrl == direct_parameter_address!(MasterCommand) {
                    command_handler.od_ind(&od_ind_data)?;
                }
            }
        } else {
            return Err(IoLinkError::InvalidEvent);
        }
        Ok(())
    }

    /// Handle transition T3: Idle (1) -> Idle (1)
    /// Action: Redirect to command handler
    fn execute_t3(
        &self,
        od_ind_data: &OdIndData,
        command_handler: &mut command_handler::CommandHandler,
        application_layer: &mut al::ApplicationLayer,
        system_management: &mut system_management::SystemManagement,
    ) -> IoLinkResult<()> {
        let address = od_ind_data.address_ctrl;
        let value = od_ind_data.data[0];
        let _ = application_layer.dl_write_param_ind(address, value);
        let _ = command_handler.od_ind(od_ind_data);
        let _ = system_management.dl_write_ind(address, value);
        Ok(())
    }

    /// Handle transition T4: Idle (1) -> Idle (1)
    /// Action: Redirect to ISDU handler
    fn execute_t4(
        &self,
        od_ind_data: &OdIndData,
        isdu_handler: &mut isdu_handler::IsduHandler,
    ) -> IoLinkResult<()> {
        isdu_handler.od_ind(od_ind_data)?;
        Ok(())
    }

    /// Handle transition T5: Idle (1) -> Idle (1)
    /// Action: Redirect to Event handler
    fn execute_t5(
        &self,
        od_ind_data: &OdIndData,
        event_handler: &mut event_handler::EventHandler,
    ) -> IoLinkResult<()> {
        event_handler.od_ind(&od_ind_data)?;
        Ok(())
    }

    /// Handle transition T6: Idle (1) -> Inactive (0)
    /// Action: -
    fn execute_t6(&self) -> IoLinkResult<()> {
        Ok(())
    }

    /// Handle On request configuration changes
    /// See 7.3.5.3 State machine of the Device On-request Data handler
    pub fn oh_conf(&mut self, state: handlers::command::ChConfState) -> IoLinkResult<()> {
        use handlers::command::ChConfState;

        match state {
            ChConfState::Active => self.process_event(OnRequestHandlerEvent::OhConfActive),
            ChConfState::Inactive => self.process_event(OnRequestHandlerEvent::OhConfInactive),
        }
    }

    pub fn od_rsp(
        &self,
        length: u8,
        data: &[u8],
        message_handler: &mut message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        message_handler.od_rsp(length, data)?;
        Ok(())
    }

    pub fn dl_read_param_rsp(
        &self,
        length: u8,
        data: u8,
        message_handler: &mut message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        let _ = self.od_rsp(length, &[data], message_handler);
        Ok(())
    }

    pub fn dl_write_param_rsp(
        &self,
        message_handler: &mut message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        let _ = self.od_rsp(0, &[], message_handler);
        Ok(())
    }
}

impl handlers::od::OdInd for OnRequestDataHandler {
    fn od_ind(&mut self, od_ind_data: &OdIndData) -> IoLinkResult<()> {
        use frame::msequence::{ComChannel, RwDirection};

        self.od_ind_data = od_ind_data.clone();
        let event = match od_ind_data.com_channel {
            ComChannel::Page => {
                if od_ind_data.address_ctrl == 0 && od_ind_data.rw_direction == RwDirection::Write {
                    OnRequestHandlerEvent::OdIndCommand
                } else if (1..=31).contains(&od_ind_data.address_ctrl) {
                    OnRequestHandlerEvent::OdIndParam
                } else {
                    return Err(IoLinkError::InvalidEvent);
                }
            }
            ComChannel::Isdu => OnRequestHandlerEvent::OdIndIsdu,
            ComChannel::Diagnosis => OnRequestHandlerEvent::OdIndEvent,
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
