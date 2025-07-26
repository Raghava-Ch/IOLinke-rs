//! Command Handler
//!
//! This module implements the Command Handler state machine as defined in
//! IO-Link Specification v1.1.4

use crate::{
    dl::{self, dl_mode::DlModeHandler},
    types::{self, IoLinkError, IoLinkResult},
    ChConfState, MasterCommand,
};

pub trait MasterCommandInd {
    /// Any MasterCommand received by the Device command handler
    /// (see Table 44 and Figure 54, state "CommandHandler_2")
    fn master_command_ind(&mut self, master_command: MasterCommand) -> IoLinkResult<()>;
}

/// See 7.3.7.3 State machine of the Device command handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandHandlerState {
    ///{Inactive_0} Waiting on activation
    Inactive,
    ///{Idle_1} Waiting on next MasterCommand
    Idle,
    ///{CommandHandler_2} Decompose MasterCommand and invoke specific actions (see B.1.2):
    /// If MasterCommand = 0x5A then change Device state to INACTIVE.
    /// If MasterCommand = 0x97 then change Device state to STARTUP.
    /// If MasterCommand = 0x9A then change Device state to PREOPERATE.
    /// If MasterCommand = 0x99 then change Device state to OPERATE.
    /// Refer the master_command macro for implimentation details.
    CommandHandler,
}

/// See Table 57 – State transition tables of the Device command handler
#[derive(Debug, PartialEq, Eq)]
enum Transition {
    /// Tn: No transition
    Tn,
    /// T1: State: Inactive (0) -> Idle (1)
    /// Action: -
    T1,
    /// T2: State: Idle (1) -> Idle (1)
    /// Action: Invoke DL_Control.ind (PDOUTVALID) if received MasterCommand =
    /// 0x98. Invoke DL_Control.ind (PDOUTINVALID) if received
    /// MasterCommand = 0x99.
    T2,
    /// T3: State: Idle (1) -> Idle (1)
    /// Action: If service DL_Control.req (VALID) then invoke PDInStatus.req (VALID).
    /// If service DL_Control.req (INVALID) then invoke PDInStatus.req
    /// (INVALID). Message handler uses PDInStatus service to set/reset the PD
    /// status flag (see A.1.5)
    T3(types::DlControlCodes),
    /// T4: State: Idle (1) -> CommandHandler (2)
    /// Action: -
    T4,
    /// T5: State: CommandHandler (2) -> Idle (1)
    /// Action: -
    T5,
    /// T6: State: CommandHandler (2) -> Inactive (0)
    /// Action: -
    T6,
}

/// See Table 57 – State transition tables of the Device command handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandHandlerEvent {
    /// {CH_Conf_ACTIVE} See Table 57, Triggers T1
    ChConfActive,
    /// {[Received MasterCmd PDOUT]} See Table 57, Triggers T2
    ReceivedMasterCmdPdout,
    /// {DL_Control_PDIn} See Table 57, Triggers T3
    DlControlPdIn(types::DlControlCodes),
    /// {[Received MasterCmd DEVICEMODE]} See Table 57, Triggers T4
    ReceivedMasterCmdDevicemode,
    /// {[Accomplished]} See Table 57, Triggers T5
    Accomplished,
    /// {CH_Conf_INACTIVE} See Table 57, Triggers T6
    ChConfInactive,
}

/// Command Handler implementation
pub struct CommandHandler {
    /// Current state of the Command Handler
    state: CommandHandlerState,
    exec_transition: Transition,
}

impl CommandHandler {
    /// Create a new Command Handler
    pub fn new() -> Self {
        Self {
            state: CommandHandlerState::Inactive,
            exec_transition: Transition::Tn,
        }
    }

    /// Process an event
    pub fn process_event(&mut self, event: CommandHandlerEvent) -> IoLinkResult<()> {
        use CommandHandlerEvent as Event;
        use CommandHandlerState as State;

        let (new_transition, new_state) = match (self.state, event) {
            // Valid transitions according to Table 57
            (State::Inactive, Event::ChConfActive) => (Transition::T1, State::Idle),
            (State::Idle, Event::ReceivedMasterCmdPdout) => (Transition::T2, State::Idle),
            (State::Idle, Event::DlControlPdIn(code)) => (Transition::T3(code), State::Idle),
            (State::Idle, Event::ReceivedMasterCmdDevicemode) => {
                (Transition::T4, State::CommandHandler)
            }
            (State::CommandHandler, Event::Accomplished) => (Transition::T5, State::Idle),
            (State::CommandHandler, Event::ChConfInactive) => (Transition::T6, State::Inactive),
            // Invalid transitions - no state change
            _ => (Transition::Tn, self.state),
        };

        self.exec_transition = new_transition;
        self.state = new_state;
        Ok(())
    }

    /// Poll the handler
    pub fn poll(&mut self, message_handler: &mut dl::message::MessageHandler) -> IoLinkResult<()> {
        // Process pending events
        match self.exec_transition {
            Transition::Tn => {}
            Transition::T1 => {
                self.exec_transition = Transition::Tn;
                self.execute_t1();
            }
            Transition::T2 => {
                self.exec_transition = Transition::Tn;
                self.execute_t2();
            }
            Transition::T3(code) => {
                self.exec_transition = Transition::Tn;
                self.execute_t3(code, message_handler);
            }
            Transition::T4 => {
                self.exec_transition = Transition::Tn;
                self.execute_t4();
            }
            Transition::T5 => {
                self.exec_transition = Transition::Tn;
                self.execute_t5();
            }
            Transition::T6 => {
                self.exec_transition = Transition::Tn;
                self.execute_t6();
            }
        }
        Ok(())
    }

    /// Execute transition T1: Inactive -> Idle
    fn execute_t1(&mut self) -> IoLinkResult<()> {
        // T1: State: Inactive (0) -> Idle (1)
        // Action: -
        // No specific action required for this transition
        Ok(())
    }

    /// Execute transition T2: Idle -> Idle (PDOUT commands)
    fn execute_t2(&mut self) -> IoLinkResult<()> {
        // T2: State: Idle (1) -> Idle (1)
        // Action: Invoke DL_Control.ind (PDOUTVALID) if received MasterCommand = 0x98.
        // Invoke DL_Control.ind (PDOUTINVALID) if received MasterCommand = 0x99.
        // TODO: Implement DL_Control.ind invocation based on received MasterCommand
        // TODO: Determine how to access the specific MasterCommand that triggered this transition
        Ok(())
    }

    /// Execute transition T3: Idle -> Idle (DL_Control PDIn)
    fn execute_t3(
        &mut self,
        code: types::DlControlCodes,
        message_handler: &mut dl::message::MessageHandler,
    ) -> IoLinkResult<()> {
        // T3: State: Idle (1) -> Idle (1)
        // Action: If service DL_Control.req (VALID) then invoke PDInStatus.req (VALID).
        // If service DL_Control.req (INVALID) then invoke PDInStatus.req (INVALID).
        // Message handler uses PDInStatus service to set/reset the PD status flag (see A.1.5)
        match code {
            types::DlControlCodes::VALID => {
                message_handler.pd_in_status_req(types::PdInStatus::VALID);
            }
            types::DlControlCodes::INVALID => {
                message_handler.pd_in_status_req(types::PdInStatus::INVALID);
            }
            _ => {
                return Err(IoLinkError::InvalidEvent);
            }
        }
        Ok(())
    }

    /// Execute transition T4: Idle -> CommandHandler
    fn execute_t4(&mut self) -> IoLinkResult<()> {
        // T4: State: Idle (1) -> CommandHandler (2)
        // Action: -
        // No specific action required for this transition
        Ok(())
    }

    /// Execute transition T5: CommandHandler -> Idle
    fn execute_t5(&mut self) -> IoLinkResult<()> {
        // T5: State: CommandHandler (2) -> Idle (1)
        // Action: -
        // No specific action required for this transition
        Ok(())
    }

    /// Execute transition T6: CommandHandler -> Inactive
    fn execute_t6(&mut self) -> IoLinkResult<()> {
        // T6: State: CommandHandler (2) -> Inactive (0)
        // Action: -
        // No specific action required for this transition
        Ok(())
    }

    /// See 7.3.7.3 State machine of the Device command handler
    pub fn ch_conf(&mut self, state: ChConfState) -> IoLinkResult<()> {
        match state {
            ChConfState::Active => self.process_event(CommandHandlerEvent::ChConfActive),
            ChConfState::Inactive => self.process_event(CommandHandlerEvent::ChConfInactive),
        }?;

        Ok(())
    }

    /// 7.2.1.18 DL_Control
    pub fn dl_control_req(&mut self, control_code: types::DlControlCodes) -> IoLinkResult<()> {
        self.process_event(CommandHandlerEvent::DlControlPdIn(control_code));
        Ok(())
    }
}

impl dl::command::MasterCommandInd for CommandHandler {
    /// Any MasterCommand received by the Device command handler
    /// (see Table 44 and Figure 54, state "CommandHandler_2")
    fn master_command_ind(&mut self, master_command: MasterCommand) -> IoLinkResult<()> {
        match master_command {
            MasterCommand::INACTIVE | MasterCommand::STARTUP | MasterCommand::PREOPERATE => {
                self.process_event(CommandHandlerEvent::ReceivedMasterCmdDevicemode);
            }
            MasterCommand::OPERATE | MasterCommand::PDOUT => {
                self.process_event(CommandHandlerEvent::ReceivedMasterCmdPdout);
            }
            _ => return Err(IoLinkError::InvalidEvent),
        }
        Ok(())
    }
}
impl Default for CommandHandler {
    fn default() -> Self {
        Self::new()
    }
}
