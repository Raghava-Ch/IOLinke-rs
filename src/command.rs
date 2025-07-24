//! Command Handler
//!
//! This module implements the Command Handler state machine as defined in
//! IO-Link Specification v1.1.4

use crate::types::{IoLinkError, IoLinkResult};

/// See 7.3.7.3 State machine of the Device command handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandHandlerState {
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
    /// Action: -
    T2,
    /// T3: State: Idle (1) -> Idle (1)
    /// Action: Invoke DL_Control.ind (PDOUTVALID) if received MasterCommand =
    /// 0x98. Invoke DL_Control.ind (PDOUTINVALID) if received
    /// MasterCommand = 0x99.
    T3,
    /// T4: State: Idle (1) -> CommandHandler (2)
    /// Action: If service DL_Control.req (VALID) then invoke PDInStatus.req (VALID).
    /// If service DL_Control.req (INVALID) then invoke PDInStatus.req
    /// (INVALID). Message handler uses PDInStatus service to set/reset the PD
    /// status flag (see A.1.5)
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
pub enum EventHandlerEvent {
    /// {CH_Conf_ACTIVE} See Table 57, Triggers T1
    ChConfActive,
    /// {[Received MasterCmd PDOUT]} See Table 57, Triggers T2
    ReceivedMasterCmdPdout,
    /// {DL_Control_PDIn} See Table 57, Triggers T3
    DlControlPdIn,
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
    pub fn process_event(&mut self, event: EventHandlerEvent) -> IoLinkResult<()> {
        use EventHandlerEvent as Event;
        use CommandHandlerState as State;

        let (new_transition, new_state) = match (self.state, event) {
            // Valid transitions according to Table 57
            (State::Inactive, Event::ChConfActive) => {(Transition::T1, State::Idle)},
            (State::Idle, Event::ReceivedMasterCmdPdout) => {(Transition::T2, State::Idle)},
            (State::Idle, Event::DlControlPdIn) => {(Transition::T3, State::Idle)},
            (State::Idle, Event::ReceivedMasterCmdDevicemode) => {(Transition::T4, State::CommandHandler)},
            (State::CommandHandler, Event::Accomplished) => {(Transition::T5, State::Idle)},
            (State::CommandHandler, Event::ChConfInactive) => {(Transition::T6, State::Inactive)},
            // Invalid transitions - no state change
            _ => (Transition::Tn, self.state),
        };

        self.exec_transition = new_transition;
        self.state = new_state;
        Ok(())
    }

    /// Poll the handler
    pub fn poll(&mut self) -> IoLinkResult<()> {
        // Process pending events
        match self.exec_transition {
            Transition::Tn => {

            }
            Transition::T1 => {

            }
            Transition::T2 => {

            }
            Transition::T3 => {

            }
            Transition::T4 => {

            }
            Transition::T5 => {

            }
            Transition::T6 => {

            }
        }
        Ok(())
    }
}

impl Default for CommandHandler {
    fn default() -> Self {
        Self::new()
    }
}
