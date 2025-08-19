//! Data Link Mode Handler
//!
//! This module implements the DL-Mode Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 6.2

use crate::types::{IoLinkError, IoLinkMode, IoLinkResult};
use crate::{dl, pl, system_management, types};
use crate::{DlMode, MHInfo, MasterCommand};

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
enum DlModeEvent {
    /// Table 45 – T1
    PlWakeUp,
    /// Mode change request
    /// Table 45 – T2
    ComChange(IoLinkMode),
    /// Table 45 – T3
    MasterCommandPreoperate,
    /// Table 45 – T4, T5
    MasterCommandOperate, // TODO: Check the T5 what is V1.0-supp
    /// Table 45 – T6, T7
    MasterCommandStartup,
    /// Table 45 – T8, T9
    MasterCommandFallback,
    /// Table 45 – T10
    TimerElapsedTdsio,
    /// Table 45 – T11, T12
    MHInfoIllegalMessagetype,
}

/// DL-Mode Handler state machine
pub struct DlModeHandler {
    state: DlModeState,
    exec_transition: Transition,
    current_mode: IoLinkMode,
    target_mode: IoLinkMode,
}

impl DlModeHandler {
    /// Create a new DL-Mode Handler
    pub fn new() -> Self {
        Self {
            state: DlModeState::Idle,
            exec_transition: Transition::Tn,
            current_mode: IoLinkMode::Sio,
            target_mode: IoLinkMode::Sio,
        }
    }

    /// Process an event
    fn process_event(&mut self, event: DlModeEvent) -> IoLinkResult<()> {
        use DlModeEvent::*;
        use DlModeState::*;

        let (new_transition, new_state) = match (self.state, event) {
            (Idle, PlWakeUp) => (Transition::T1, EstablishCom),
            (EstablishCom, ComChange(mode)) => {
                self.target_mode = mode;
                (Transition::T2, Startup)
            }
            (EstablishCom, TimerElapsedTdsio) => (Transition::T10, Idle),
            (Startup, MasterCommandPreoperate) => (Transition::T3, Preoperate),
            (Startup, MasterCommandOperate) => (Transition::T5, Operate),
            (Preoperate, MasterCommandOperate) => (Transition::T4, Operate),
            (Preoperate, MasterCommandStartup) => (Transition::T6, Startup),
            (Preoperate, MasterCommandFallback) => (Transition::T8, Idle),
            (Preoperate, MHInfoIllegalMessagetype) => (Transition::T12, Startup),
            (Operate, MasterCommandStartup) => (Transition::T7, Startup),
            (Operate, MasterCommandFallback) => (Transition::T9, Idle),
            (Operate, MHInfoIllegalMessagetype) => (Transition::T11, Startup),
            _ => return Err(IoLinkError::InvalidEvent),
        };

        self.exec_transition = new_transition;
        self.state = new_state;
        Ok(())
    }

    /// Poll the state machine
    /// See IO-Link v1.1.4 Section 7.3.2.5
    pub fn poll(
        &mut self,
        isdu_handler: &mut dl::isdu_handler::IsduHandler,
        event_handler: &mut dl::event_handler::EventHandler,
        command_handler: &mut dl::command_handler::CommandHandler,
        od_handler: &mut dl::od_handler::OnRequestDataHandler,
        pd_handler: &mut dl::pd_handler::ProcessDataHandler,
        message_handler: &mut dl::message_handler::MessageHandler,
        system_management: &mut system_management::SystemManagement,
    ) -> IoLinkResult<()> {
        match self.exec_transition {
            Transition::Tn => {
                // No transition to execute
            }
            Transition::T1 => {
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t1(message_handler, system_management);
            }
            Transition::T2 => {
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t2(od_handler, command_handler, system_management);
            }
            Transition::T3 => {
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t3(isdu_handler, event_handler, system_management);
            }
            Transition::T4 => {
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t4(pd_handler, system_management);
            }
            Transition::T5 => {
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t5(
                    isdu_handler,
                    event_handler,
                    pd_handler,
                    system_management,
                );
            }
            Transition::T6 => {
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t6(isdu_handler, event_handler, system_management);
            }
            Transition::T7 => {
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t7(
                    isdu_handler,
                    event_handler,
                    pd_handler,
                    system_management,
                );
            }
            Transition::T8 => {
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t8(
                    isdu_handler,
                    event_handler,
                    command_handler,
                    od_handler,
                    pd_handler,
                    message_handler,
                    system_management,
                );
            }
            Transition::T9 => {
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t9(
                    isdu_handler,
                    event_handler,
                    command_handler,
                    od_handler,
                    pd_handler,
                    message_handler,
                    system_management,
                );
            }
            Transition::T10 => {
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t10(
                    isdu_handler,
                    event_handler,
                    command_handler,
                    od_handler,
                    pd_handler,
                    message_handler,
                    system_management,
                );
            }
            Transition::T11 => {
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t11(
                    isdu_handler,
                    event_handler,
                    pd_handler,
                    system_management,
                );
            }
            Transition::T12 => {
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t12(isdu_handler, event_handler, system_management);
            }
        }
        Ok(())
    }

    /// Execute transition T1: Wakeup detected, activate message handler
    /// See IO-Link v1.1.4 Table 45, T1
    fn execute_t1(
        &mut self,
        message_handler: &mut dl::message_handler::MessageHandler,
        system_management: &mut system_management::SystemManagement,
    ) -> IoLinkResult<()> {
        // Activate message handler (call MH_Conf_ACTIVE in Figure 44)
        message_handler.mh_conf_update(crate::MhConfState::Active);
        // Indicate state via service DL_Mode.ind(ESTABCOM) to SM
        let _ = system_management.dl_mode_ind(DlMode::Estabcom);
        log::debug!("DL-Mode: T1 - Wakeup detected, activating message handler");

        Ok(())
    }

    /// Execute transition T2: Communication mode established
    /// See IO-Link v1.1.4 Table 45, T2
    fn execute_t2(
        &mut self,
        od_handler: &mut dl::od_handler::OnRequestDataHandler,
        command_handler: &mut dl::command_handler::CommandHandler,
        system_management: &mut system_management::SystemManagement,
    ) -> IoLinkResult<()> {
        log::debug!(
            "DL-Mode: T2 - Communication mode {:?} established",
            self.target_mode
        );
        self.current_mode = self.target_mode;
        // Activate On-request Data (call OH_Conf_ACTIVE in Figure 49)
        let _ = od_handler.oh_conf(crate::ChConfState::Active);
        // Activate command handler (call CH_Conf_ACTIVE in Figure 54)
        let _ = command_handler.ch_conf(crate::ChConfState::Active);
        // Indicate state via service DL_Mode.ind (COM1, COM2, or COM3) to SM
        // TODO: Implement the actual COMx mode handling
        let dl_mode = match self.current_mode {
            IoLinkMode::Com1 => DlMode::Com1,
            IoLinkMode::Com2 => DlMode::Com2,
            IoLinkMode::Com3 => DlMode::Com3,
            _ => return Err(IoLinkError::InvalidEvent),
        };
        let _ = system_management.dl_mode_ind(dl_mode);

        Ok(())
    }

    /// Execute transition T3: PREOPERATE command received
    /// See IO-Link v1.1.4 Table 45, T3
    fn execute_t3(
        &mut self,
        isdu_handler: &mut dl::isdu_handler::IsduHandler,
        event_handler: &mut dl::event_handler::EventHandler,
        system_management: &mut system_management::SystemManagement,
    ) -> IoLinkResult<()> {
        log::debug!("DL-Mode: T3 - PREOPERATE command received");
        // Activate ISDU (call IH_Conf_ACTIVE in Figure 52)
        let _ = isdu_handler.ih_conf(types::IhConfState::Active);
        // Activate Event handler (call EH_Conf_ACTIVE in Figure 56)
        let _ = event_handler.eh_conf(types::EhConfState::Active);
        // Indicate state via service DL_Mode.ind (PREOPERATE) to SM
        let _ = system_management.dl_mode_ind(DlMode::Preoperate);

        Ok(())
    }

    /// Execute transition T4: OPERATE command received from PREOPERATE
    /// See IO-Link v1.1.4 Table 45, T4
    fn execute_t4(
        &mut self,
        pd_handler: &mut dl::pd_handler::ProcessDataHandler,
        system_management: &mut system_management::SystemManagement,
    ) -> IoLinkResult<()> {
        log::debug!("DL-Mode: T4 - OPERATE command received from PREOPERATE");
        // Activate Process Data handler (call PD_Conf_ACTIVE in Figure 47)
        let _ = pd_handler.pd_conf(types::PdConfState::Active);
        // Indicate state via service DL_Mode.ind (OPERATE) to SM
        let _ = system_management.dl_mode_ind(DlMode::Operate);

        Ok(())
    }

    /// Execute transition T5: OPERATE command received from STARTUP
    /// See IO-Link v1.1.4 Table 45, T5
    fn execute_t5(
        &mut self,
        isdu_handler: &mut dl::isdu_handler::IsduHandler,
        event_handler: &mut dl::event_handler::EventHandler,
        pd_handler: &mut dl::pd_handler::ProcessDataHandler,
        system_management: &mut system_management::SystemManagement,
    ) -> IoLinkResult<()> {
        log::debug!("DL-Mode: T5 - OPERATE command received from STARTUP");
        // Activate Process Data handler (call PD_Conf_ACTIVE in Figure 47)
        let _ = pd_handler.pd_conf(types::PdConfState::Active);
        // Activate ISDU (call IH_Conf_ACTIVE in Figure 52)
        let _ = isdu_handler.ih_conf(types::IhConfState::Active);
        // Activate Event handler (call EH_Conf_ACTIVE in Figure 56)
        let _ = event_handler.eh_conf(types::EhConfState::Active);
        // Indicate state via service DL_Mode.ind (OPERATE) to SM
        let _ = system_management.dl_mode_ind(DlMode::Operate);

        Ok(())
    }

    /// Execute transition T6: STARTUP command received from PREOPERATE
    /// See IO-Link v1.1.4 Table 45, T6
    fn execute_t6(
        &mut self,
        isdu_handler: &mut dl::isdu_handler::IsduHandler,
        event_handler: &mut dl::event_handler::EventHandler,
        system_management: &mut system_management::SystemManagement,
    ) -> IoLinkResult<()> {
        log::debug!("DL-Mode: T6 - STARTUP command received from PREOPERATE");
        // Deactivate ISDU (call IH_Conf_INACTIVE in Figure 52)
        let _ = isdu_handler.ih_conf(types::IhConfState::Inactive);
        // Deactivate Event handler (call EH_Conf_INACTIVE in Figure 56)
        let _ = event_handler.eh_conf(types::EhConfState::Inactive);
        // Indicate state via service DL_Mode.ind (STARTUP) to SM
        let _ = system_management.dl_mode_ind(DlMode::Startup);

        Ok(())
    }

    /// Execute transition T7: STARTUP command received from OPERATE
    /// See IO-Link v1.1.4 Table 45, T7
    fn execute_t7(
        &mut self,
        isdu_handler: &mut dl::isdu_handler::IsduHandler,
        event_handler: &mut dl::event_handler::EventHandler,
        pd_handler: &mut dl::pd_handler::ProcessDataHandler,
        system_management: &mut system_management::SystemManagement,
    ) -> IoLinkResult<()> {
        log::debug!("DL-Mode: T7 - STARTUP command received from OPERATE");
        // Deactivate Process Data handler (call PD_Conf_INACTIVE in Figure 47)
        let _ = pd_handler.pd_conf(types::PdConfState::Inactive);
        // Deactivate ISDU (call IH_Conf_INACTIVE in Figure 52)
        let _ = isdu_handler.ih_conf(types::IhConfState::Inactive);
        // Deactivate Event handler (call EH_Conf_INACTIVE in Figure 56)
        let _ = event_handler.eh_conf(types::EhConfState::Inactive);
        // Indicate state via service DL_Mode.ind (STARTUP) to SM
        let _ = system_management.dl_mode_ind(DlMode::Startup);

        Ok(())
    }

    /// Execute transition T8: FALLBACK command received from PREOPERATE
    /// See IO-Link v1.1.4 Table 45, T8
    fn execute_t8(
        &mut self,
        isdu_handler: &mut dl::isdu_handler::IsduHandler,
        event_handler: &mut dl::event_handler::EventHandler,
        command_handler: &mut dl::command_handler::CommandHandler,
        od_handler: &mut dl::od_handler::OnRequestDataHandler,
        pd_handler: &mut dl::pd_handler::ProcessDataHandler,
        message_handler: &mut dl::message_handler::MessageHandler,
        system_management: &mut system_management::SystemManagement,
    ) -> IoLinkResult<()> {
        log::debug!("DL-Mode: T8 - FALLBACK command received from PREOPERATE");
        // Wait until TFBD elapsed,
        // TODO: Implement TFBD timer wait
        //Then deactivate all handlers (call xx_Conf_INACTIVE)
        let _ = isdu_handler.ih_conf(types::IhConfState::Inactive);
        let _ = event_handler.eh_conf(types::EhConfState::Inactive);
        let _ = command_handler.ch_conf(crate::ChConfState::Inactive);
        let _ = od_handler.oh_conf(crate::ChConfState::Inactive);
        let _ = pd_handler.pd_conf(types::PdConfState::Inactive);
        let _ = message_handler.mh_conf_update(crate::MhConfState::Inactive);
        // Indicate state via service DL_Mode.ind (INACTIVE) to SM
        let _ = system_management.dl_mode_ind(DlMode::Inactive);

        Ok(())
    }

    /// Execute transition T9: FALLBACK command received from OPERATE
    /// See IO-Link v1.1.4 Table 45, T9
    fn execute_t9(
        &mut self,
        isdu_handler: &mut dl::isdu_handler::IsduHandler,
        event_handler: &mut dl::event_handler::EventHandler,
        command_handler: &mut dl::command_handler::CommandHandler,
        od_handler: &mut dl::od_handler::OnRequestDataHandler,
        pd_handler: &mut dl::pd_handler::ProcessDataHandler,
        message_handler: &mut dl::message_handler::MessageHandler,
        system_management: &mut system_management::SystemManagement,
    ) -> IoLinkResult<()> {
        log::debug!("DL-Mode: T9 - FALLBACK command received from OPERATE");
        // Wait until TFBD elapsed, 
        // TODO: Implement TFBD timer wait
        // Then deactivate all handlers (call xx_Conf_INACTIVE)
        let _ = isdu_handler.ih_conf(types::IhConfState::Inactive);
        let _ = event_handler.eh_conf(types::EhConfState::Inactive);
        let _ = command_handler.ch_conf(crate::ChConfState::Inactive);
        let _ = od_handler.oh_conf(crate::ChConfState::Inactive);
        let _ = pd_handler.pd_conf(types::PdConfState::Inactive);
        let _ = message_handler.mh_conf_update(crate::MhConfState::Inactive);
        // Indicate state via service DL_Mode.ind (INACTIVE) to SM
        let _ = system_management.dl_mode_ind(DlMode::Inactive);

        Ok(())
    }

    /// Execute transition T10: Unsuccessful wakeup procedures
    /// See IO-Link v1.1.4 Table 45, T10
    fn execute_t10(
        &mut self,
        isdu_handler: &mut dl::isdu_handler::IsduHandler,
        event_handler: &mut dl::event_handler::EventHandler,
        command_handler: &mut dl::command_handler::CommandHandler,
        od_handler: &mut dl::od_handler::OnRequestDataHandler,
        pd_handler: &mut dl::pd_handler::ProcessDataHandler,
        message_handler: &mut dl::message_handler::MessageHandler,
        system_management: &mut system_management::SystemManagement,
    ) -> IoLinkResult<()> {
        log::debug!("DL-Mode: T10 - Unsuccessful wakeup, establishing SIO mode");
        self.current_mode = IoLinkMode::Sio;
        // After unsuccessful wakeup procedures, establish configured SIO mode
        // Deactivate all handlers (call xx_Conf_INACTIVE)
        let _ = isdu_handler.ih_conf(types::IhConfState::Inactive);
        let _ = event_handler.eh_conf(types::EhConfState::Inactive);
        let _ = command_handler.ch_conf(crate::ChConfState::Inactive);
        let _ = od_handler.oh_conf(crate::ChConfState::Inactive);
        let _ = pd_handler.pd_conf(types::PdConfState::Inactive);
        let _ = message_handler.mh_conf_update(crate::MhConfState::Inactive);
        // Indicate state via service DL_Mode.ind (INACTIVE) to SM
        let _ = system_management.dl_mode_ind(DlMode::Inactive);

        Ok(())
    }

    /// Execute transition T11: Illegal M-sequence type from OPERATE
    /// See IO-Link v1.1.4 Table 45, T11
    fn execute_t11(
        &mut self,
        isdu_handler: &mut dl::isdu_handler::IsduHandler,
        event_handler: &mut dl::event_handler::EventHandler,
        pd_handler: &mut dl::pd_handler::ProcessDataHandler,
        system_management: &mut system_management::SystemManagement,
    ) -> IoLinkResult<()> {
        log::debug!("DL-Mode: T11 - Illegal M-sequence type from OPERATE");
        // Deactivate Process Data (call PD_Conf_INACTIVE in Figure 47)
        let _ = pd_handler.pd_conf(types::PdConfState::Inactive);
        // Deactivate ISDU (call IH_Conf_INACTIVE in Figure 52)
        let _ = isdu_handler.ih_conf(types::IhConfState::Inactive);
        // Deactivate Event handler (call EH_Conf_INACTIVE in Figure 56)
        let _ = event_handler.eh_conf(types::EhConfState::Inactive);
        // Indicate state via service DL_Mode.ind (STARTUP) to SM
        let _ = system_management.dl_mode_ind(DlMode::Startup);

        Ok(())
    }

    /// Execute transition T12: Illegal M-sequence type from PREOPERATE
    /// See IO-Link v1.1.4 Table 45, T12
    fn execute_t12(
        &mut self,
        isdu_handler: &mut dl::isdu_handler::IsduHandler,
        event_handler: &mut dl::event_handler::EventHandler,
        system_management: &mut system_management::SystemManagement,
    ) -> IoLinkResult<()> {
        log::debug!("DL-Mode: T12 - Illegal M-sequence type from PREOPERATE");
        // Deactivate ISDU (call IH_Conf_INACTIVE in Figure 52)
        let _ = isdu_handler.ih_conf(types::IhConfState::Inactive);
        // Deactivate Event handler (call EH_Conf_INACTIVE in Figure 56)
        let _ = event_handler.eh_conf(types::EhConfState::Inactive);
        // Indicate state via service DL_Mode.ind (STARTUP) to SM
        let _ = system_management.dl_mode_ind(DlMode::Startup);
        Ok(())
    }

    /// One out of three possible SDCI communication modes COM1, COM2, or COM3
    /// See Figure 37 – State machine of the Device DL-mode handler
    /// This function is called to indicate communication is successful.
    pub fn com_mode_update(&mut self, com_mode: IoLinkMode) {
        let _ = self.process_event(DlModeEvent::ComChange(com_mode));
    }

    /// Timer elapsed event
    /// This function is called to indicate a Tdsio timer has elapsed.
    pub fn timer_elapsed(&mut self, timer: pl::physical_layer::Timer) {
        if timer == pl::physical_layer::Timer::Tdsio {
            let _ = self.process_event(DlModeEvent::TimerElapsedTdsio);
        }
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

impl dl::message_handler::MsgHandlerInfo for DlModeHandler {
    /// See 7.2.2.6 MHInfo
    /// The service MHInfo signals an exceptional operation within the message handler. The
    /// parameters of the service are listed in Table 39.
    fn mh_info(&mut self, mh_info: MHInfo) {
        if mh_info == types::MHInfo::IllegalMessagetype {
            let _ = self.process_event(DlModeEvent::MHInfoIllegalMessagetype);
        }
    }
}

impl dl::command_handler::MasterCommandInd for DlModeHandler {
    /// Any MasterCommand received by the Device command handler
    /// (see Table 44 and Figure 54, state "CommandHandler_2")
    fn master_command_ind(&mut self, master_command: MasterCommand) -> IoLinkResult<()> {
        match master_command {
            MasterCommand::DevicePreOperate => {
                let _ = self.process_event(DlModeEvent::MasterCommandPreoperate);
            }
            MasterCommand::DeviceOperate => {
                let _ = self.process_event(DlModeEvent::MasterCommandOperate);
            }
            MasterCommand::DeviceStartup => {
                let _ = self.process_event(DlModeEvent::MasterCommandStartup);
            }
            MasterCommand::Fallback => {
                let _ = self.process_event(DlModeEvent::MasterCommandFallback);
            }
            MasterCommand::DeviceIdent => {
                todo!()
            }
            MasterCommand::MasterIdent => {
                todo!()
            }
            _ => return Err(IoLinkError::InvalidEvent),
        }
        Ok(())
    }
}
