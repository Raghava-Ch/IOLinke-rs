//! Data Link Mode Handler
//!
//! This module implements the DL-Mode Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 6.2

use crate::{dl, pl, sm};
use crate::types::{IoLinkError, IoLinkMode, IoLinkResult};
use crate::{DlMode, MHInfo, MasterCommand, Timer};

/// DL indications to other modules
pub trait DlInd {
    /// See 7.2.1.14 DL_Mode
    /// The DL uses the DL_Mode service to report to System Management that a certain operating
    /// status has been reached. The parameters of the service primitives are listed in Table 29.
    fn dl_mode_ind(&mut self, mode: DlMode) -> IoLinkResult<()>;
    
    /// See 7.2.1.5 DL_Write
    /// The DL_Write service is used by System Management to write a Device parameter value to
    /// the Device via the page communication channel. The parameters of the service primitives are
    /// listed in Table 20.
    fn dl_write_ind(&mut self, address: u8, value: u8) -> IoLinkResult<()>;
   
    /// 7.2.1.4 DL_Read
    /// The DL_Read service is used by System Management to read a Device parameter value via
    /// the page communication channel. The parameters of the service primitives are listed in Table 19.
    fn dl_read_ind(&mut self, address: u8) -> IoLinkResult<()>;
}
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
enum Transition {
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
    exec_transition: Transition,
    current_mode: IoLinkMode,
    target_mode: IoLinkMode,
    error_count: u32,
}

impl DlModeHandler {
    /// Create a new DL-Mode Handler
    pub fn new() -> Self {
        Self {
            state: DlModeState::Idle,
            exec_transition: Transition::Tn,
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
                self.exec_transition = Transition::T1;
                EstablishCom
            }
            (EstablishCom, ComChange(mode)) => {
                self.exec_transition = Transition::T2;
                self.target_mode = mode;
                Startup
            }
            (EstablishCom, TimerElapsed(timer)) => {
                if timer == Timer::Tdsio {
                    self.exec_transition = Transition::T10;
                    Idle
                } else {
                    return Err(IoLinkError::InvalidEvent);
                }
            }
            (Startup, MasterCmd(cmd)) => {
                if cmd == MasterCommand::PREOPERATE {
                    self.exec_transition = Transition::T3;
                    Preoperate
                } else if cmd == MasterCommand::OPERATE {
                    self.exec_transition = Transition::T5;
                    Operate
                } else {
                    return Err(IoLinkError::InvalidEvent);
                }
            }
            (Preoperate, MasterCmd(cmd)) => {
                if cmd == MasterCommand::OPERATE {
                    self.exec_transition = Transition::T4;
                    Operate
                } else if cmd == MasterCommand::STARTUP {
                    self.exec_transition = Transition::T6;
                    Startup
                } else if cmd == MasterCommand::FALLBACK {
                    self.exec_transition = Transition::T8;
                    Idle
                } else {
                    return Err(IoLinkError::InvalidEvent);
                }
            }
            (Preoperate, MsgHandlerError(error)) => {
                if error == MHInfo::IllegalMessagetype {
                    self.exec_transition = Transition::T12;
                    Startup
                } else {
                    return Err(IoLinkError::InvalidEvent);
                }
            }
            (Operate, MasterCmd(cmd)) => {
                if cmd == MasterCommand::STARTUP {
                    self.exec_transition = Transition::T7;
                    Startup
                } else if cmd == MasterCommand::FALLBACK {
                    self.exec_transition = Transition::T9;
                    Idle
                } else {
                    return Err(IoLinkError::InvalidEvent);
                }
            }
            (Operate, MsgHandlerError(error)) => {
                if error == MHInfo::IllegalMessagetype {
                    self.exec_transition = Transition::T11;
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
    pub fn poll(
        &mut self,
        message_handler: &mut dl::message::MessageHandler,
        system_management: &mut sm::SystemManagement,
    ) -> IoLinkResult<()> {
        match self.exec_transition {
            Transition::Tn => {
                // No transition to execute
            }
            Transition::T1 => {
                self.exec_transition = Transition::Tn;
                self.execute_t1(message_handler, system_management);
            }
            Transition::T2 => {
                self.exec_transition = Transition::Tn;
                self.execute_t2(system_management);
            }
            Transition::T3 => {
                self.exec_transition = Transition::Tn;
                self.execute_t3(system_management);
            }
            Transition::T4 => {
                self.exec_transition = Transition::Tn;
                self.execute_t4(system_management);
            }
            Transition::T5 => {
                self.exec_transition = Transition::Tn;
                self.execute_t5(system_management);
            }
            Transition::T6 => {
                self.exec_transition = Transition::Tn;
                self.execute_t6(system_management);
            }
            Transition::T7 => {
                self.exec_transition = Transition::Tn;
                self.execute_t7(system_management);
            }
            Transition::T8 => {
                self.exec_transition = Transition::Tn;
                self.execute_t8(message_handler, system_management);
            }
            Transition::T9 => {
                self.exec_transition = Transition::Tn;
                self.execute_t9(message_handler, system_management);
            }
            Transition::T10 => {
                self.exec_transition = Transition::Tn;
                self.execute_t10(message_handler, system_management);
            }
            Transition::T11 => {
                self.exec_transition = Transition::Tn;
                self.execute_t11(message_handler, system_management);
            }
            Transition::T12 => {
                self.exec_transition = Transition::Tn;
                self.execute_t12(system_management);
            }
        }
        Ok(())
    }

    /// Execute transition T1: Wakeup detected, activate message handler
    /// See IO-Link v1.1.4 Table 45, T1
    fn execute_t1(
        &mut self,
        message_handler: &mut dl::message::MessageHandler,
        system_management: &mut sm::SystemManagement
    ) -> IoLinkResult<()> {
        // Activate message handler (call MH_Conf_ACTIVE in Figure 44)
        message_handler.mh_conf_update(crate::MhConf::Active);
        // Indicate state via service DL_Mode.ind(ESTABCOM) to SM
        system_management.dl_mode_ind(DlMode::Estabcom);
        log::debug!("DL-Mode: T1 - Wakeup detected, activating message handler");

        Ok(())
    }

    /// Execute transition T2: Communication mode established
    /// See IO-Link v1.1.4 Table 45, T2
    fn execute_t2(
        &mut self,
        system_management: &mut sm::SystemManagement
    ) -> IoLinkResult<()> {
        // Activate On-request Data (call OH_Conf_ACTIVE in Figure 49)
        // Activate command handler (call CH_Conf_ACTIVE in Figure 54)
        // Indicate state via service DL_Mode.ind (COM1, COM2, or COM3) to SM
        log::debug!("DL-Mode: T2 - Communication mode {:?} established", self.target_mode);
        self.current_mode = self.target_mode;
        // TODO: Implement OH_Conf_ACTIVE call
        // TODO: Implement CH_Conf_ACTIVE call
        let dl_mode = match self.current_mode {
            IoLinkMode::Com1 => DlMode::Com1,
            IoLinkMode::Com2 => DlMode::Com2,
            IoLinkMode::Com3 => DlMode::Com3,
            _ => return Err(IoLinkError::InvalidEvent),
        };
        system_management.dl_mode_ind(dl_mode);
        
        Ok(())
    }

    /// Execute transition T3: PREOPERATE command received
    /// See IO-Link v1.1.4 Table 45, T3
    fn execute_t3(
        &mut self,
        system_management: &mut sm::SystemManagement
    ) -> IoLinkResult<()> {
        // Activate ISDU (call IH_Conf_ACTIVE in Figure 52)
        // Activate Event handler (call EH_Conf_ACTIVE in Figure 56)
        // Indicate state via service DL_Mode.ind (PREOPERATE) to SM
        log::debug!("DL-Mode: T3 - PREOPERATE command received");
        // TODO: Implement IH_Conf_ACTIVE call
        // TODO: Implement EH_Conf_ACTIVE call
        system_management.dl_mode_ind(DlMode::Preoperate);

        Ok(())
    }

    /// Execute transition T4: OPERATE command received from PREOPERATE
    /// See IO-Link v1.1.4 Table 45, T4
    fn execute_t4(
        &mut self,
        system_management: &mut sm::SystemManagement
    ) -> IoLinkResult<()> {
        // Activate Process Data handler (call PD_Conf_ACTIVE in Figure 47)
        // Indicate state via service DL_Mode.ind (OPERATE) to SM
        log::debug!("DL-Mode: T4 - OPERATE command received from PREOPERATE");
        // TODO: Implement PD_Conf_ACTIVE call
        system_management.dl_mode_ind(DlMode::Operate);
        
        Ok(())
    }

    /// Execute transition T5: OPERATE command received from STARTUP
    /// See IO-Link v1.1.4 Table 45, T5
    fn execute_t5(
        &mut self,
        system_management: &mut sm::SystemManagement
    ) -> IoLinkResult<()> {
        // Activate Process Data handler (call PD_Conf_ACTIVE in Figure 47)
        // Activate ISDU (call IH_Conf_ACTIVE in Figure 52)
        // Activate Event handler (call EH_Conf_ACTIVE in Figure 56)
        // Indicate state via service DL_Mode.ind (OPERATE) to SM
        log::debug!("DL-Mode: T5 - OPERATE command received from STARTUP");
        // TODO: Implement PD_Conf_ACTIVE call
        // TODO: Implement IH_Conf_ACTIVE call
        // TODO: Implement EH_Conf_ACTIVE call
        system_management.dl_mode_ind(DlMode::Operate);

        Ok(())
    }

    /// Execute transition T6: STARTUP command received from PREOPERATE
    /// See IO-Link v1.1.4 Table 45, T6
    fn execute_t6(
        &mut self,
        system_management: &mut sm::SystemManagement
    ) -> IoLinkResult<()> {
        // Deactivate ISDU (call IH_Conf_INACTIVE in Figure 52)
        // Deactivate Event handler (call EH_Conf_INACTIVE in Figure 56)
        // Indicate state via service DL_Mode.ind (STARTUP) to SM
        log::debug!("DL-Mode: T6 - STARTUP command received from PREOPERATE");
        // TODO: Implement IH_Conf_INACTIVE call
        // TODO: Implement EH_Conf_INACTIVE call
        system_management.dl_mode_ind(DlMode::Startup);

        Ok(())
    }

    /// Execute transition T7: STARTUP command received from OPERATE
    /// See IO-Link v1.1.4 Table 45, T7
    fn execute_t7(
        &mut self,
        system_management: &mut sm::SystemManagement
    ) -> IoLinkResult<()> {
        // Deactivate Process Data handler (call PD_Conf_INACTIVE in Figure 47)
        // Deactivate ISDU (call IH_Conf_INACTIVE in Figure 52)
        // Deactivate Event handler (call EH_Conf_INACTIVE in Figure 56)
        // Indicate state via service DL_Mode.ind (STARTUP) to SM
        log::debug!("DL-Mode: T7 - STARTUP command received from OPERATE");
        // TODO: Implement PD_Conf_INACTIVE call
        // TODO: Implement IH_Conf_INACTIVE call
        // TODO: Implement EH_Conf_INACTIVE call
        system_management.dl_mode_ind(DlMode::Startup);

        Ok(())
    }

    /// Execute transition T8: FALLBACK command received from PREOPERATE
    /// See IO-Link v1.1.4 Table 45, T8
    fn execute_t8(
        &mut self,
        message_handler: &mut dl::message::MessageHandler,
        system_management: &mut sm::SystemManagement,
    ) -> IoLinkResult<()> {
        // Wait until TFBD elapsed, then deactivate all handlers (call xx_Conf_INACTIVE)
        // Indicate state via service DL_Mode.ind (INACTIVE) to SM
        log::debug!("DL-Mode: T8 - FALLBACK command received from PREOPERATE");
        // TODO: Implement TFBD timer wait
        // TODO: Implement all xx_Conf_INACTIVE calls
        message_handler.mh_conf_update(crate::MhConf::Inactive);
        system_management.dl_mode_ind(DlMode::Inactive);

        Ok(())
    }

    /// Execute transition T9: FALLBACK command received from OPERATE
    /// See IO-Link v1.1.4 Table 45, T9
    fn execute_t9(
        &mut self,
        message_handler: &mut dl::message::MessageHandler,
        system_management: &mut sm::SystemManagement,
    ) -> IoLinkResult<()> {
        // Wait until TFBD elapsed, then deactivate all handlers (call xx_Conf_INACTIVE)
        // Indicate state via service DL_Mode.ind (INACTIVE) to SM
        log::debug!("DL-Mode: T9 - FALLBACK command received from OPERATE");
        // TODO: Implement TFBD timer wait
        // TODO: Implement all xx_Conf_INACTIVE calls
        message_handler.mh_conf_update(crate::MhConf::Inactive);
        system_management.dl_mode_ind(DlMode::Inactive);

        Ok(())
    }

    /// Execute transition T10: Unsuccessful wakeup procedures
    /// See IO-Link v1.1.4 Table 45, T10
    fn execute_t10(
        &mut self,
        message_handler: &mut dl::message::MessageHandler,
        system_management: &mut sm::SystemManagement,
    ) -> IoLinkResult<()> {
        // After unsuccessful wakeup procedures, establish configured SIO mode
        // Deactivate all handlers (call xx_Conf_INACTIVE)
        // Indicate state via service DL_Mode.ind (INACTIVE) to SM
        log::debug!("DL-Mode: T10 - Unsuccessful wakeup, establishing SIO mode");
        self.current_mode = IoLinkMode::Sio;
        // TODO: Implement all xx_Conf_INACTIVE calls
        message_handler.mh_conf_update(crate::MhConf::Inactive);
        system_management.dl_mode_ind(DlMode::Inactive);

        Ok(())
    }

    /// Execute transition T11: Illegal M-sequence type from OPERATE
    /// See IO-Link v1.1.4 Table 45, T11
    fn execute_t11(
        &mut self,
        message_handler: &mut dl::message::MessageHandler,
        system_management: &mut sm::SystemManagement,
    ) -> IoLinkResult<()> {
        // Deactivate Process Data (call PD_Conf_INACTIVE in Figure 47)
        // Deactivate ISDU (call IH_Conf_INACTIVE in Figure 52)
        // Deactivate Event handler (call EH_Conf_INACTIVE in Figure 56)
        // Indicate state via service DL_Mode.ind (STARTUP) to SM
        log::debug!("DL-Mode: T11 - Illegal M-sequence type from OPERATE");
        // TODO: Implement PD_Conf_INACTIVE call
        // TODO: Implement IH_Conf_INACTIVE call
        // TODO: Implement EH_Conf_INACTIVE call
        system_management.dl_mode_ind(DlMode::Startup);

        Ok(())
    }

    /// Execute transition T12: Illegal M-sequence type from PREOPERATE
    /// See IO-Link v1.1.4 Table 45, T12
    fn execute_t12(
        &mut self,
        system_management: &mut sm::SystemManagement,
    ) -> IoLinkResult<()> {
        // Deactivate ISDU (call IH_Conf_INACTIVE in Figure 52)
        // Deactivate Event handler (call EH_Conf_INACTIVE in Figure 56)
        // Indicate state via service DL_Mode.ind (STARTUP) to SM
        log::debug!("DL-Mode: T12 - Illegal M-sequence type from PREOPERATE");
        // TODO: Implement IH_Conf_INACTIVE call
        // TODO: Implement EH_Conf_INACTIVE call
        system_management.dl_mode_ind(DlMode::Startup);
        Ok(())
    }
    /// One out of three possible SDCI communication modes COM1, COM2, or COM3
    pub fn com_mode_update(&mut self, com_mode: IoLinkMode) {
        let _ = self.process_event(DlModeEvent::ComChange(com_mode));
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

impl pl::physical_layer::PhysicalLayerInd for DlModeHandler {
    /// The PL-WakeUp service initiates or indicates a specific sequence which prepares the
    /// Physical Layer to send and receive communication requests (see 5.3.3.3). This unconfirmed
    /// service has no parameters. Its success can only be verified by a Master by attempting to
    /// communicate with the Device. The service primitives are listed in Table 3.
    fn pl_wake_up_ind(&mut self) -> IoLinkResult<()> {
        let _ = self.process_event(DlModeEvent::PlWakeUp);
        Ok(())
    }
}

impl dl::message::MsgHandlerInfo for DlModeHandler {
    /// See 7.2.2.6 MHInfo
    /// The service MHInfo signals an exceptional operation within the message handler. The
    /// parameters of the service are listed in Table 39.
    fn mh_info_update(&mut self, mh_info: MHInfo) {
        let _ = self.process_event(DlModeEvent::MsgHandlerError(mh_info));
    }
}
