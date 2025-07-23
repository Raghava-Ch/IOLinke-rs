//! Data Link Mode Handler
//!
//! This module implements the DL-Mode Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 6.2

use crate::hal::PhysicalLayer;
use crate::types::{IoLinkError, IoLinkMode, IoLinkResult};
use crate::{MHInfo, MasterCommand, Timer};

/// DL-Mode Handler states
/// See IO-Link v1.1.4 Section 7.3.2.5
/// See Table 45 – State transition tables of the Device DL-mode handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DlModeState {
    /// Inactive state
    Idle,
    /// Establish Communication
    EstablishCom,
    /// Startup state
    Startup,
    /// Preoperate state
    Preoperate,
    /// Operate state
    Operate,
}

/// See Table 45 – State transition tables of the Device DL-mode handler
#[derive(Debug, PartialEq, Eq)]
enum Transitions {
    /// Nothing to transit.
    Tn,
    /// Source State:0 Target State:1 Wakeup current pulse detected. Activate message handler (call
    /// MH_Conf_ACTIVE in Figure 44). Indicate state via service DL_Mode.ind
    /// ,(ESTABCOM) to SM.
    T1,
    /// Source State:1 Target State:2. One out of the three transmission rates of COM3, COM2, or COM1 mode
    /// established. Activate On-request Data (call OH_Conf_ACTIVE in Figure
    /// 49) and command handler (call CH_Conf_ACTIVE in Figure 54). Indicate
    /// state via service DL_Mode.ind (COM1, COM2, or COM3) to SM.
    T2,
    /// Source State:2 Target State:3. Device command handler received MasterCommand
    /// (MCmd_PREOPERATE). Activate ISDU (call IH_Conf_ACTIVE in Figure
    /// 52) and Event handler (call EH_Conf_ACTIVE in Figure 56). Indicate state
    /// via service DL_Mode.ind (PREOPERATE) to SM.
    T3,
    /// Source State:3 Target State:4. Device command handler received MasterCommand (MCmd_OPERATE).
    /// Activate Process Data handler (call PD_Conf_ACTIVE in Figure 47).
    /// Indicate state via service DL_Mode.ind (OPERATE) to SM.
    T4,
    /// Source State:2 Target State:4. Device command handler received MasterCommand (MCmd_OPERATE).
    /// Activate Process Data handler (call PD_Conf_ACTIVE in Figure 47), ISDU
    /// (call IH_Conf_ACTIVE in Figure 52), and Event handler (call
    /// EH_Conf_ACTIVE in Figure 56). Indicate state via service DL_Mode.ind
    /// (OPERATE) to SM.
    T5,
    /// Source State:3 Target State:2. Device command handler received MasterCommand (MCmd_STARTUP).
    /// Deactivate ISDU (call IH_Conf_INACTIVE in Figure 52) and Event handler
    /// (call EH_Conf_INACTIVE in Figure 56). Indicate state via service
    /// DL_Mode.ind (STARTUP) to SM.
    T6,
    /// Source State:4 Target State:2. Device command handler received MasterCommand (MCmd_STARTUP).
    /// Deactivate Process Data handler (call PD_Conf_INACTIVE in Figure 47),
    /// ISDU (call IH_Conf_INACTIVE in Figure 52), and Event handler (call
    /// EH_Conf_INACTIVE in Figure 56). Indicate state via service DL_Mode.ind
    /// (STARTUP) to SM.
    T7,
    /// Source State:3 Target State:0. Device command handler received MasterCommand (MCmd_FALLBACK).
    /// Wait until TFBD elapsed, and then deactivate all handlers (call
    /// xx_Conf_INACTIVE). Indicate state via service DL_Mode.ind (INACTIVE)
    /// to SM (see Figure 81 and Table 95).
    T8,
    /// Source State:4 Target State:0. Device command handler received MasterCommand (MCmd_FALLBACK).
    /// Wait until TFBD elapsed, and then deactivate all handlers (call
    /// xx_Conf_INACTIVE). Indicate state via service DL_Mode.ind (INACTIVE)
    /// to SM (see Figure 81 and Table 95).
    T9,
    /// Source State:1 Target State:0. After unsuccessful wakeup procedures (see Figure 32) the Device
    /// establishes the configured SIO mode after an elapsed time TDSIO (see
    /// Figure 33). Deactivate all handlers (call xx_Conf_INACTIVE). Indicate
    /// state via service DL_Mode.ind (INACTIVE) to SM.
    T10,
    /// Source State:4 Target State:2. Message handler detected an illegal M-sequence type. Deactivate Process
    /// Data (call PD_Conf_INACTIVE in Figure 47), ISDU (call
    /// IH_Conf_INACTIVE in Figure 52), and Event handler (call
    /// EH_Conf_INACTIVE in Figure 56). Indicate state via service DL_Mode.ind
    /// (STARTUP) to SM (see Figure 81 and Table 95).
    T11,
    /// Message handler detected an illegal M-sequence type. Deactivate ISDU
    /// (call IH_Conf_INACTIVE in Figure 52) and Event handler (call
    /// EH_Conf_INACTIVE in Figure 56). Indicate state via service DL_Mode.ind
    /// (STARTUP) to SM (see Figure 81 and Table 95).
    T12,
}
/// DL-Mode Handler events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DlModeEvent {
    /// Table 45 – T1
    PlWakeUp,
    /// Mode change request
    /// Table 45 – T2
    ComChange(IoLinkMode),
    /// Table 45 – T3, T4, T5, T6, T7, T8, T9,
    MasterCmd(MasterCommand), // TODO: Check the T5 what is V1.0-supp
    /// Table 45 – T10
    TimerElapsed(Timer),
    /// Table 45 – T11
    MsgHandlerError(MHInfo),
}

/// DL-Mode Handler state machine
pub struct DlModeHandler {
    state: DlModeState,
    exec_transition: Transitions,
    current_mode: IoLinkMode,
    target_mode: IoLinkMode,
    error_count: u32,
}

impl DlModeHandler {
    /// Create a new DL-Mode Handler
    pub fn new() -> Self {
        Self {
            state: DlModeState::Idle,
            exec_transition: Transitions::Tn,
            current_mode: IoLinkMode::Sio,
            target_mode: IoLinkMode::Sio,
            error_count: 0,
        }
    }

    /// Process an event
    pub fn process_event(&mut self, event: DlModeEvent) -> IoLinkResult<()> {
        use DlModeEvent::*;
        use DlModeState::*;

        let new_state = match (self.state, event) {
            (Idle, PlWakeUp) => {
                self.exec_transition = Transitions::T1;
                EstablishCom
            }
            (EstablishCom, ComChange(mode)) => {
                self.exec_transition = Transitions::T2;
                self.target_mode = mode;
                Startup
            }
            (EstablishCom, TimerElapsed(timer)) => {
                if timer == Timer::Tdsio {
                    self.exec_transition = Transitions::T10;
                    Idle
                } else {
                    return Err(IoLinkError::InvalidEvent);
                }
            }
            (Startup, MasterCmd(cmd)) => {
                if cmd == MasterCommand::PREOPERATE {
                    self.exec_transition = Transitions::T3;
                    Preoperate
                } else if cmd == MasterCommand::OPERATE {
                    self.exec_transition = Transitions::T5;
                    Operate
                } else {
                    return Err(IoLinkError::InvalidEvent);
                }
            }
            (Preoperate, MasterCmd(cmd)) => {
                if cmd == MasterCommand::OPERATE {
                    self.exec_transition = Transitions::T4;
                    Operate
                } else if cmd == MasterCommand::STARTUP {
                    self.exec_transition = Transitions::T6;
                    Startup
                } else if cmd == MasterCommand::FALLBACK {
                    self.exec_transition = Transitions::T8;
                    Idle
                } else {
                    return Err(IoLinkError::InvalidEvent);
                }
            }
            (Preoperate, MsgHandlerError(error)) => {
                if error == MHInfo::IllegalMessagetype {
                    self.exec_transition = Transitions::T12;
                    Startup
                } else {
                    return Err(IoLinkError::InvalidEvent);
                }
            }
            (Operate, MasterCmd(cmd)) => {
                if cmd == MasterCommand::STARTUP {
                    self.exec_transition = Transitions::T7;
                    Startup
                } else if cmd == MasterCommand::FALLBACK {
                    self.exec_transition = Transitions::T9;
                    Idle
                } else {
                    return Err(IoLinkError::InvalidEvent);
                }
            }
            (Operate, MsgHandlerError(error)) => {
                if error == MHInfo::IllegalMessagetype {
                    self.exec_transition = Transitions::T11;
                    Startup
                } else {
                    return Err(IoLinkError::InvalidEvent);
                }
            }
            _ => return Err(IoLinkError::InvalidEvent),
        };

        self.state = new_state;
        Ok(())
    }

    /// Poll the state machine
    /// See IO-Link v1.1.4 Section 7.3.2.5
    pub fn poll<P: PhysicalLayer>(&mut self, phy: &mut P) -> IoLinkResult<()> {
        match self.exec_transition {
            Transitions::Tn => {
                // No transition to execute
            }
            Transitions::T1 => {
                // Wakeup detected, activate message handler, indicate ESTABCOM to SM
                // See IO-Link v1.1.4 Table 45, T1
                // Typically: MH_Conf_ACTIVE, DL_Mode.ind(ESTABCOM)
                // Here, just update state
                self.exec_transition = Transitions::Tn;
            }
            Transitions::T2 => {
                // Communication mode established, activate On-request Data and Command Handler
                // Indicate COMx to SM
                // See IO-Link v1.1.4 Table 45, T2
                self.exec_transition = Transitions::Tn;
            }
            Transitions::T3 => {
                // MasterCommand: PREOPERATE, activate ISDU and Event Handler
                // Indicate PREOPERATE to SM
                self.exec_transition = Transitions::Tn;
            }
            Transitions::T4 => {
                // MasterCommand: OPERATE, activate Process Data Handler
                // Indicate OPERATE to SM
                self.exec_transition = Transitions::Tn;
            }
            Transitions::T5 => {
                // MasterCommand: OPERATE from Startup, activate PD, ISDU, Event Handler
                // Indicate OPERATE to SM
                self.exec_transition = Transitions::Tn;
            }
            Transitions::T6 => {
                // MasterCommand: STARTUP from Preoperate, deactivate ISDU and Event Handler
                // Indicate STARTUP to SM
                self.exec_transition = Transitions::Tn;
            }
            Transitions::T7 => {
                // MasterCommand: STARTUP from Operate, deactivate PD, ISDU, Event Handler
                // Indicate STARTUP to SM
                self.exec_transition = Transitions::Tn;
            }
            Transitions::T8 => {
                // MasterCommand: FALLBACK from Preoperate, wait TFBD, deactivate all handlers
                // Indicate INACTIVE to SM
                self.exec_transition = Transitions::Tn;
            }
            Transitions::T9 => {
                // MasterCommand: FALLBACK from Operate, wait TFBD, deactivate all handlers
                // Indicate INACTIVE to SM
                self.exec_transition = Transitions::Tn;
            }
            Transitions::T10 => {
                // Unsuccessful wakeup, establish SIO mode, deactivate all handlers
                // Indicate INACTIVE to SM
                self.exec_transition = Transitions::Tn;
            }
            Transitions::T11 => {
                // Illegal M-sequence type, deactivate PD, ISDU, Event Handler
                // Indicate STARTUP to SM
                self.exec_transition = Transitions::Tn;
            }
            Transitions::T12 => {
                // Illegal M-sequence type, deactivate ISDU, Event Handler
                // Indicate STARTUP to SM
                self.exec_transition = Transitions::Tn;
            }
        }
        Ok(())
    }

    /// The PL-WakeUp service initiates or indicates a specific sequence which prepares the
    /// Physical Layer to send and receive communication requests (see 5.3.3.3). This unconfirmed
    /// service has no parameters. Its success can only be verified by a Master by attempting to
    /// communicate with the Device. The service primitives are listed in Table 3.
    pub fn pl_wake_up(&mut self) {
        let _ = self.process_event(DlModeEvent::PlWakeUp);
    }

    /// One out of three possible SDCI communication modes COM1, COM2, or COM3
    pub fn com_mode_update(&mut self, com_mode: IoLinkMode) {
        let _ = self.process_event(DlModeEvent::ComChange(com_mode));
    }

    /// See 7.2.2.6 MHInfo
    /// The service MHInfo signals an exceptional operation within the message handler. The
    /// parameters of the service are listed in Table 39.
    pub fn mh_info_update(&mut self, mh_info: MHInfo) {
        let _ = self.process_event(DlModeEvent::MsgHandlerError(mh_info));
    }

    /// Any MasterCommand received by the Device command handler
    /// (see Table 44 and Figure 54, state "CommandHandler_2")
    pub fn master_command_update(&mut self, master_command: MasterCommand) {
        let _ = self.process_event(DlModeEvent::MasterCmd(master_command));
    }

    /// Any MasterCommand received by the Device command handler
    /// (see Table 44 and Figure 54, state "CommandHandler_2")
    pub fn timer_update(&mut self, timer: Timer) {
        let _ = self.process_event(DlModeEvent::TimerElapsed(timer));
    }
}

impl Default for DlModeHandler {
    fn default() -> Self {
        Self::new()
    }
}
