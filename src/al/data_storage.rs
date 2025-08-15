//! Data Storage
//!
//! This module implements the Data Storage functionality as defined in
//! IO-Link Specification v1.1.4

use crate::types::{IoLinkError, IoLinkResult};
use modular_bitfield::prelude::*;


/// Data Storage State Property (8 bits)
/// Bit 0: Reserved
/// Bit 1-2: State of Data Storage (see `DsState`)
/// Bit 3-6: Reserved
/// Bit 7: DS_UPLOAD_FLAG ("1": DS_UPLOAD_REQ pending)
#[bitfield(bits = 8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateProperty {
    /// Bit 0: Reserved
    #[skip] __: B1,
    /// Bits 1-2: State of Data Storage
    #[bits = 2]
    pub ds_state: DsState,
    /// Bits 3-6: Reserved
    #[skip] __: B4,
    /// Bit 7: DS_UPLOAD_FLAG
    pub ds_upload_flag: bool,
}

/// State of Data Storage (for bits 1-2 of StateProperty)
#[derive(Specifier, Debug, Clone, Copy, PartialEq, Eq)]
#[bits = 2]
pub enum DsState {
    Inactive = 0b00,
    Upload = 0b01,
    Download = 0b10,
    Locked = 0b11,
}



/// See 8.3.3.2 Event state machine of the Device AL
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DtatStorageStateMachineState {
    /// {DSStateCheck_0}
    /// Check activation state after initialization.
    DSStateCheck,
    /// {DSLocked_1}
    /// Waiting on Data Storage state machine to become unlocked.
    /// This state will become obsolete in future releases since Device access lock "Data Storage"
    /// shall not be used anymore (see Table B.12)
    DSLocked,
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
    T1,
    /// T2: DSLocked -> DSLocked
    /// Set DS_UPLOAD_FLAG = TRUE
    T2,
    /// T3: DSLocked -> DSIdle
    /// Set State_Property = "Inactive"
    T3,
    /// T4: DSLocked -> DSIdle
    /// Invoke AL_EVENT.req (EventCode: DS_UPLOAD_REQ), Set State_Property = "Inactive"
    T4,
    /// T5: DSIdle -> DSLocked
    /// Set State_Property = "Data Storage access locked"
    T5,
    /// T6: DSStateCheck -> DSIdle
    /// Set State_Property = "Inactive"
    T6,
    /// T7: DSIdle -> DSIdle
    /// Set DS_UPLOAD_FLAG = TRUE, invoke AL_EVENT.req (EventCode: DS_UPLOAD_REQ)
    T7,
    /// T8: DsIdle -> DsActivity
    /// Lock local parameter access, set State_Property = "Upload" or "Download"
    T8,
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
    Unlocked,
    /// [Locked] Event for DSStateCheck_0 (Triggers T1)
    Locked,
    /// DS_ParUpload_ind event (Triggers T2, T4, or T7)
    DsParUploadInd,
    /// [Unlocked & not DS_UPLOAD_FLAG] (Triggers T3)
    UnlockedAndNotDsUploadFlag,
    /// [Unlocked & DS_UPLOAD_FLAG] (Triggers T4)
    UnlockedAndDsUploadFlag,
    /// [Locked] (Triggers T5)
    LockedEvent,
    /// [TransmissionStart] (Triggers T8)
    TransmissionStart,
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
        use DtatStorageStateMachineState as State;
        use DataStorageTransition as Transition;

        let (new_transition, new_state) = match (self.state, event) {
            // DSStateCheck_0
            (State::DSStateCheck, Event::Locked) => (Transition::T1, State::DSLocked),
            (State::DSStateCheck, Event::Unlocked) => (Transition::T6, State::DSIdle),
            // DSLocked_1
            (State::DSLocked, Event::DsParUploadInd) => (Transition::T2, State::DSLocked),
            (State::DSLocked, Event::UnlockedAndNotDsUploadFlag) => (Transition::T3, State::DSIdle),
            (State::DSLocked, Event::UnlockedAndDsUploadFlag) => (Transition::T4, State::DSIdle),
            // DSIdle_2
            (State::DSIdle, Event::DsParUploadInd) => (Transition::T7, State::DSIdle),
            (State::DSIdle, Event::TransmissionStart) => (Transition::T8, State::DSActivity),
            (State::DSIdle, Event::LockedEvent) => (Transition::T5, State::DSLocked),
            (State::DSIdle, Event::TransmissionEnd) => (Transition::T11, State::DSIdle),
            // DSActivity_3
            (State::DSActivity, Event::TransmissionEnd) => (Transition::T9, State::DSIdle),
            (State::DSActivity, Event::TransmissionBreak) => (Transition::T10, State::DSIdle),
            // Default: No transition
            _ => (Transition::Tn, self.state),
        };

        self.exec_transition = new_transition;
        self.state = new_state;
        Ok(())
    }

    pub fn poll(&mut self) -> IoLinkResult<()> {
        use DataStorageTransition as Transition;

        match self.exec_transition {
            Transition::Tn => {
                // No transition to process
            }
            Transition::T1 => {
                // Transition T1: DSStateCheck -> DSLocked
                self.exec_transition = Transition::Tn;
            }
            Transition::T2 => {
                // Transition T2: DSLocked -> DSLocked
                self.exec_transition = Transition::Tn;
            }
            Transition::T3 => {
                // Transition T3: DSLocked -> DSIdle
                self.exec_transition = Transition::Tn;
            }
            Transition::T4 => {
                // Transition T4: DSLocked -> DSIdle
                self.exec_transition = Transition::Tn;
            }
            Transition::T5 => {
                // Transition T5: DSIdle -> DSLocked
                self.exec_transition = Transition::Tn;
            }
            Transition::T6 => {
                // Transition T6: DSStateCheck -> DSIdle
                self.exec_transition = Transition::Tn;
            }
            Transition::T7 => {
                // Transition T7: DSIdle -> DSIdle
                self.exec_transition = Transition::Tn;
            }
            Transition::T8 => {
                // Transition T8: DsIdle -> DsActivity
                self.exec_transition = Transition::Tn;
            }
            Transition::T9 => {
                // Transition T9: DsActivity -> DsIdle
                self.exec_transition = Transition::Tn;
            }
            Transition::T10 => {
                // Transition T10: DsActivity -> DsIdle
                self.exec_transition = Transition::Tn;
            }
            Transition::T11 => {
                // Transition T11: DsIdle -> DsIdle
                self.exec_transition = Transition::Tn;
            }
        };

        Ok(())
    }


    /// Executes transition T1: DSStateCheck -> DSLocked
    ///
    /// Action: Set State_Property = "Data Storage access locked"
    /// TODO: Implement setting the state property to locked.
    pub fn execute_t1(&mut self) -> IoLinkResult<()> {
        // TODO: Set State_Property = "Data Storage access locked"
        Ok(())
    }

    /// Executes transition T2: DSLocked -> DSLocked
    ///
    /// Action: Set DS_UPLOAD_FLAG = TRUE
    /// TODO: Implement setting the DS_UPLOAD_FLAG.
    pub fn execute_t2(&mut self) -> IoLinkResult<()> {
        // TODO: Set DS_UPLOAD_FLAG = TRUE
        Ok(())
    }

    /// Executes transition T3: DSLocked -> DSIdle
    ///
    /// Action: Set State_Property = "Inactive"
    /// TODO: Implement setting the state property to inactive.
    pub fn execute_t3(&mut self) -> IoLinkResult<()> {
        // TODO: Set State_Property = "Inactive"
        Ok(())
    }

    /// Executes transition T4: DSLocked -> DSIdle
    ///
    /// Action: Invoke AL_EVENT.req (EventCode: DS_UPLOAD_REQ), Set State_Property = "Inactive"
    /// TODO: Implement event invocation and set state property to inactive.
    pub fn execute_t4(&mut self) -> IoLinkResult<()> {
        // TODO: Invoke AL_EVENT.req (EventCode: DS_UPLOAD_REQ)
        // TODO: Set State_Property = "Inactive"
        Ok(())
    }

    /// Executes transition T5: DSIdle -> DSLocked
    ///
    /// Action: Set State_Property = "Data Storage access locked"
    /// TODO: Implement setting the state property to locked.
    pub fn execute_t5(&mut self) -> IoLinkResult<()> {
        // TODO: Set State_Property = "Data Storage access locked"
        Ok(())
    }

    /// Executes transition T6: DSStateCheck -> DSIdle
    ///
    /// Action: Set State_Property = "Inactive"
    /// TODO: Implement setting the state property to inactive.
    pub fn execute_t6(&mut self) -> IoLinkResult<()> {
        // TODO: Set State_Property = "Inactive"
        Ok(())
    }

    /// Executes transition T7: DSIdle -> DSIdle
    ///
    /// Action: Set DS_UPLOAD_FLAG = TRUE, invoke AL_EVENT.req (EventCode: DS_UPLOAD_REQ)
    /// TODO: Implement setting DS_UPLOAD_FLAG and event invocation.
    pub fn execute_t7(&mut self) -> IoLinkResult<()> {
        // TODO: Set DS_UPLOAD_FLAG = TRUE
        // TODO: Invoke AL_EVENT.req (EventCode: DS_UPLOAD_REQ)
        Ok(())
    }

    /// Executes transition T8: DsIdle -> DsActivity
    ///
    /// Action: Lock local parameter access
    /// TODO: Implement parameter access locking.
    pub fn execute_t8(&mut self) -> IoLinkResult<()> {
        // TODO: Lock local parameter access
        Ok(())
    }

    /// Executes transition T9: DsActivity -> DsIdle
    ///
    /// Action: Unlock local parameter access
    /// TODO: Implement parameter access unlocking.
    pub fn execute_t9(&mut self) -> IoLinkResult<()> {
        // TODO: Unlock local parameter access
        Ok(())
    }

    /// Executes transition T10: DsActivity -> DsIdle
    ///
    /// Action: Unlock local parameter access
    /// TODO: Implement parameter access unlocking.
    pub fn execute_t10(&mut self) -> IoLinkResult<()> {
        // TODO: Unlock local parameter access
        Ok(())
    }

    /// Executes transition T11: DsIdle -> DsIdle
    ///
    /// Action: No operation (no transition)
    pub fn execute_t11(&mut self) -> IoLinkResult<()> {
        // No operation for T11
        Ok(())
    }

}