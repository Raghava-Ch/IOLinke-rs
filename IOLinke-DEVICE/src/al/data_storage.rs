//! Data Storage
//!
//! This module implements the Data Storage functionality as defined in
//! IO-Link Specification v1.1.4
use crate::al::services::AlEventReq;
use iolinke_macros::device_event_code;
use iolinke_types::handlers::ds::DsCommand;
use iolinke_types::{
    custom::{IoLinkError, IoLinkResult},
    handlers,
};
use iolinke_util::{log_state_transition, log_state_transition_error};

use crate::al::{event_handler, parameter_manager};

/// See 8.3.3.2 Event state machine of the Device AL
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DtatStorageStateMachineState {
    /// {DSLocked_1}
    /// Waiting on Data Storage state machine to become unlocked.
    /// This state will become obsolete in future releases since Device access lock "Data Storage"
    /// shall not be used anymore (see Table B.12)
    /// ! {Obsolete} This state is not recommended to use from v1.1.4.
    // DSLocked,

    /// {DSStateCheck_0}
    /// Check activation state after initialization.
    DSStateCheck,
    /// {DSIdle_2}
    /// Waiting on Data Storage activities.
    /// Any unhandled DS-Command shall be rejected with the ErrorType "0x8036 Function temporarily not available"
    DSIdle,
    /// {DSActivity_3}
    /// Provide parameter set; local parameterization locked.
    DSActivity,
}

/// Data Storage State Machine Transitions
#[derive(Debug, PartialEq, Eq)]
pub enum DataStorageTransition {
    /// No transition
    Tn,
    /// T1: DSStateCheck -> DSLocked
    /// Set State_Property = "Data Storage access locked"
    /// ! {Obsolete} This transition is not recommended to use in v1.1.4.
    // T1,
    /// T2: DSLocked -> DSLocked
    /// Set DS_UPLOAD_FLAG = TRUE
    /// ! {Obsolete} This transition is not recommended to use in v1.1.4.
    // T2,
    /// T3: DSLocked -> DSIdle
    /// Set State_Property = "Inactive"
    /// ! {Obsolete} This transition is not recommended to use in v1.1.4.
    // T3,
    /// T4: DSLocked -> DSIdle
    /// Invoke AL_EVENT.req (EventCode: DS_UPLOAD_REQ), Set State_Property = "Inactive"
    /// ! {Obsolete} This transition is not recommended to use in v1.1.4.
    // T4,
    /// T5: DSIdle -> DSLocked
    /// Set State_Property = "Data Storage access locked"
    /// ! {Obsolete} This transition is not recommended to use in v1.1.4.
    // T5,
    /// T6: DSStateCheck -> DSIdle
    /// Set State_Property = "Inactive"
    /// ! {Obsolete} This transition is not recommended to use in v1.1.4.
    // T6,

    /// T7: DSIdle -> DSIdle
    /// Set DS_UPLOAD_FLAG = TRUE, invoke AL_EVENT.req (EventCode: DS_UPLOAD_REQ)
    T7,
    /// T8: DsIdle -> DsActivity
    /// Lock local parameter access, set State_Property = "Upload" or "Download"
    T8(DsCommand),
    /// T9: DsActivity -> DsIdle
    /// Set DS_UPLOAD_FLAG = FALSE, unlock local parameter access, set State_Property = "Inactive"
    T9,
    /// T10: DsActivity -> DsIdle
    /// Unlock local parameter access; Set State_Property = "Inactive"
    T10,
    /// T11: DsIdle -> DsIdle
    /// Set DS_UPLOAD_FLAG = FALSE
    T11,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataStorageStateMachineEvent {
    /// [Unlocked] Event for DSStateCheck_0 (Triggers T1 or T6)
    /// ! {Obsolete} This event is not used in v1.1.4.
    // Unlocked,
    /// [Locked] Event for DSStateCheck_0 (Triggers T1)
    /// ! {Obsolete} This event is not used in v1.1.4.
    // Locked,
    /// [Unlocked & not DS_UPLOAD_FLAG] (Triggers T3)
    /// ! {Obsolete} This event is not used in v1.1.4.
    // UnlockedAndNotDsUploadFlag,
    /// [Unlocked & DS_UPLOAD_FLAG] (Triggers T4)
    /// ! {Obsolete} This event is not used in v1.1.4.
    // UnlockedAndDsUploadFlag,
    /// [Locked] (Triggers T5)
    /// ! {Obsolete} This event is not used in v1.1.4.
    // LockedEvent,
    /// DS_ParUpload_ind event (Triggers T2, T4, or T7)
    DsParUploadInd,
    /// [TransmissionStart] (Triggers T8)
    TransmissionStart(DsCommand),
    /// [TransmissionEnd] (Triggers T9 or T11)
    TransmissionEnd,
    /// [TransmissionBreak] (Triggers T10)
    TransmissionBreak,
}

pub struct DataStorage {
    state: DtatStorageStateMachineState,
    exec_transition: DataStorageTransition,
}

impl DataStorage {
    pub fn new() -> Self {
        Self {
            state: DtatStorageStateMachineState::DSStateCheck,
            exec_transition: DataStorageTransition::Tn,
        }
    }

    pub fn process_event(&mut self, event: DataStorageStateMachineEvent) -> IoLinkResult<()> {
        use DataStorageStateMachineEvent as Event;
        use DataStorageTransition as Transition;
        use DtatStorageStateMachineState as State;

        let (new_transition, new_state) = match (self.state, event) {
            // ! Obsolete transitions are all commented out
            // DSStateCheck_0
            // (State::DSStateCheck, Event::Locked) => (Transition::T1, State::DSLocked),
            // (State::DSStateCheck, Event::Unlocked) => (Transition::T6, State::DSIdle),
            // (State::DSLocked, Event::UnlockedAndNotDsUploadFlag) => (Transition::T3, State::DSIdle),
            // (State::DSLocked, Event::UnlockedAndDsUploadFlag) => (Transition::T4, State::DSIdle),
            // (State::DSIdle, Event::LockedEvent) => (Transition::T5, State::DSLocked),
            // DSLocked_1
            // (State::DSLocked, Event::DsParUploadInd) => (Transition::T2, State::DSLocked),

            // DSIdle_2
            (State::DSIdle, Event::DsParUploadInd) => (Transition::T7, State::DSIdle),
            (State::DSIdle, Event::TransmissionStart(direction)) => {
                (Transition::T8(direction), State::DSActivity)
            }
            (State::DSIdle, Event::TransmissionEnd) => (Transition::T11, State::DSIdle),
            // DSActivity_3
            (State::DSActivity, Event::TransmissionEnd) => (Transition::T9, State::DSIdle),
            (State::DSActivity, Event::TransmissionBreak) => (Transition::T10, State::DSIdle),
            // Default: No transition
            _ => {
                log_state_transition_error!(module_path!(), "process_event", self.state, event);
                return Err(IoLinkError::InvalidEvent);
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

    pub fn poll(
        &mut self,
        event_handler: &mut event_handler::EventHandler,
        parameter_manager: &mut parameter_manager::ParameterManager,
    ) -> IoLinkResult<()> {
        use DataStorageTransition as Transition;

        match self.exec_transition {
            // ! Obsolete transitions are all commented out
            // Transition::T1 => {
            //     // Transition T1: DSStateCheck -> DSLocked
            //     self.exec_transition = Transition::Tn;
            //     self.execute_t1()?;
            // }
            // Transition::T2 => {
            //     // Transition T2: DSLocked -> DSLocked
            //     self.exec_transition = Transition::Tn;
            //     self.execute_t2()?;
            // }
            // Transition::T3 => {
            //     // Transition T3: DSLocked -> DSIdle
            //     self.exec_transition = Transition::Tn;
            //     self.execute_t3()?;
            // }
            // Transition::T4 => {
            //     // Transition T4: DSLocked -> DSIdle
            //     self.exec_transition = Transition::Tn;
            //     self.execute_t4()?;
            // }
            // Transition::T5 => {
            //     // Transition T5: DSIdle -> DSLocked
            //     self.exec_transition = Transition::Tn;
            //     self.execute_t5()?;
            // }
            // Transition::T6 => {
            //     // Transition T6: DSStateCheck -> DSIdle
            //     self.exec_transition = Transition::Tn;
            //     self.execute_t6(parameter_manager)?;
            // }
            Transition::Tn => {
                // No transition to process
            }
            Transition::T7 => {
                // Transition T7: DSIdle -> DSIdle
                self.exec_transition = Transition::Tn;
                self.execute_t7(event_handler, parameter_manager)?;
            }
            Transition::T8(direction) => {
                // Transition T8: DsIdle -> DsActivity
                self.exec_transition = Transition::Tn;
                self.execute_t8(parameter_manager, direction)?;
            }
            Transition::T9 => {
                // Transition T9: DsActivity -> DsIdle
                self.exec_transition = Transition::Tn;
                self.execute_t9(parameter_manager)?;
            }
            Transition::T10 => {
                // Transition T10: DsActivity -> DsIdle
                self.exec_transition = Transition::Tn;
                self.execute_t10(parameter_manager)?;
            }
            Transition::T11 => {
                // Transition T11: DsIdle -> DsIdle
                self.exec_transition = Transition::Tn;
                self.execute_t11()?;
            }
        };

        Ok(())
    }

    /// Executes transition T1: DSStateCheck -> DSLocked
    ///
    /// Action: Set State_Property = "Data Storage access locked"
    /// ! {Obsolete} This transition is not recommended to use in v1.1.4.
    // pub fn execute_t1(&mut self) -> IoLinkResult<()> {
    //     // Obsolete transition
    //     Ok(())
    // }

    /// Executes transition T2: DSLocked -> DSLocked
    ///
    /// Action: Set DS_UPLOAD_FLAG = TRUE
    /// ! {Obsolete} This transition is not recommended to use in v1.1.4.
    // pub fn execute_t2(&mut self) -> IoLinkResult<()> {
    //     // Obsolete transition
    //     Ok(())
    // }

    /// Executes transition T3: DSLocked -> DSIdle
    ///
    /// Action: Set State_Property = "Inactive"
    /// ! {Obsolete} This transition is not recommended to use in v1.1.4.
    // pub fn execute_t3(&mut self) -> IoLinkResult<()> {
    //     // Obsolete transition
    //     Ok(())
    // }

    /// Executes transition T4: DSLocked -> DSIdle
    ///
    /// Action: Invoke AL_EVENT.req (EventCode: DS_UPLOAD_REQ), Set State_Property = "Inactive"
    /// ! {Obsolete} This transition is not recommended to use in v1.1.4.
    // pub fn execute_t4(&mut self) -> IoLinkResult<()> {
    //     // Obsolete transition
    //     Ok(())
    // }

    /// Executes transition T5: DSIdle -> DSLocked
    ///
    /// Action: Set State_Property = "Data Storage access locked"
    /// ! {Obsolete} This transition is not recommended to use in v1.1.4.
    // pub fn execute_t5(&mut self) -> IoLinkResult<()> {
    //     // Obsolete transition
    //     Ok(())
    // }

    /// Executes transition T6: DSStateCheck -> DSIdle
    ///
    /// Action: Set State_Property = "Inactive"
    /// ! {Obsolete} This transition is not recommended to use in v1.1.4.
    // pub fn execute_t6(
    //     &mut self,
    //     parameter_manager: &mut al::parameter_manager::ParameterManager,
    // ) -> IoLinkResult<()> {
    //     // Set State_Property = "Inactive"
    //     parameter_manager.set_state_property(al::parameter_manager::DsState::Inactive)?;
    //     Ok(())
    // }

    /// Executes transition T7: DSIdle -> DSIdle
    ///
    /// Action: Set DS_UPLOAD_FLAG = TRUE, invoke AL_EVENT.req (EventCode: DS_UPLOAD_REQ)
    /// TODO: Implement setting DS_UPLOAD_FLAG and event invocation.
    pub fn execute_t7(
        &mut self,
        event_handler: &mut event_handler::EventHandler,
        parameter_manager: &mut parameter_manager::ParameterManager,
    ) -> IoLinkResult<()> {
        use handlers::event::{EventEntry, EventQualifier};
        // Set DS_UPLOAD_FLAG = TRUE
        parameter_manager.set_upload_flag(true)?;
        // Invoke AL_EVENT.req (EventCode: DS_UPLOAD_REQ)
        const EVENT_QUALIFIER: EventQualifier = ds_upload_event_qualifier();

        const ENTRY: &'static [EventEntry] = &[EventEntry {
            event_qualifier: EVENT_QUALIFIER,
            event_code: device_event_code!(DS_UPLOAD_REQ),
        }];
        event_handler.al_event_req(1, ENTRY)?;
        Ok(())
    }

    /// Executes transition T8: DsIdle -> DsActivity
    ///
    /// Action: Lock local parameter access
    /// TODO: Implement parameter access locking.
    pub fn execute_t8(
        &mut self,
        parameter_manager: &mut parameter_manager::ParameterManager,
        ds_command: DsCommand,
    ) -> IoLinkResult<()> {
        // Lock local parameter access
        parameter_manager.lock_local_parameter_access()?;
        // Set State_Property = "Upload" or "Download"
        match ds_command {
            DsCommand::UploadStart => {
                parameter_manager.set_state_property(handlers::pm::DsState::Upload)?
            }
            DsCommand::DownloadStart => {
                parameter_manager.set_state_property(handlers::pm::DsState::Download)?
            }
            _ => return Err(IoLinkError::InvalidEvent),
        }

        Ok(())
    }

    /// Executes transition T9: DsActivity -> DsIdle
    ///
    /// Action: Unlock local parameter access
    /// TODO: Implement parameter access unlocking.
    pub fn execute_t9(
        &mut self,
        parameter_manager: &mut parameter_manager::ParameterManager,
    ) -> IoLinkResult<()> {
        parameter_manager.set_upload_flag(false)?;
        // Unlock local parameter access
        parameter_manager.unlock_local_parameter_access()?;
        parameter_manager.set_state_property(handlers::pm::DsState::Inactive)?;
        Ok(())
    }

    /// Executes transition T10: DsActivity -> DsIdle
    ///
    /// Action: Unlock local parameter access
    /// TODO: Implement parameter access unlocking.
    pub fn execute_t10(
        &mut self,
        parameter_manager: &mut parameter_manager::ParameterManager,
    ) -> IoLinkResult<()> {
        // Unlock local parameter access
        parameter_manager.unlock_local_parameter_access()?;
        parameter_manager.set_state_property(handlers::pm::DsState::Inactive)?;
        Ok(())
    }

    /// Executes transition T11: DsIdle -> DsIdle
    ///
    /// Action: No operation (no transition)
    pub fn execute_t11(&mut self) -> IoLinkResult<()> {
        // No operation for T11
        Ok(())
    }

    pub fn ds_command(&mut self, command: DsCommand) -> IoLinkResult<()> {
        match command {
            DsCommand::UploadStart | DsCommand::DownloadStart => {
                self.process_event(DataStorageStateMachineEvent::TransmissionStart(command))
            }
            DsCommand::UploadEnd | DsCommand::DownloadEnd => {
                self.process_event(DataStorageStateMachineEvent::TransmissionEnd)
            }
            DsCommand::Break => self.process_event(DataStorageStateMachineEvent::TransmissionBreak),
        }
    }

    pub fn ds_par_upload_ind(&mut self) -> IoLinkResult<()> {
        self.process_event(DataStorageStateMachineEvent::DsParUploadInd)
    }
}

const fn ds_upload_event_qualifier() -> handlers::event::EventQualifier {
    use handlers::event::{
        EventInstance::System, EventMode::SingleShot, EventSource::Device, EventType::Notification,
    };

    let mut event_qualifier = handlers::event::EventQualifier::new();
    event_qualifier.set_eq_mode(SingleShot);
    event_qualifier.set_eq_type(Notification);
    event_qualifier.set_eq_source(Device);
    event_qualifier.set_eq_instance(System);
    event_qualifier
}

impl handlers::sm::SystemManagementInd for DataStorage {
    fn sm_device_mode_ind(
        &mut self,
        device_mode: handlers::sm::DeviceMode,
    ) -> handlers::sm::SmResult<()> {
        match device_mode {
            // Control gets here when the dl_mode_ind(INACTIVE) is invoked.
            handlers::sm::DeviceMode::Idle => {
                let _ = self.process_event(DataStorageStateMachineEvent::TransmissionBreak);
            }
            _ => return Ok(()),
        }
        Ok(())
    }
}
