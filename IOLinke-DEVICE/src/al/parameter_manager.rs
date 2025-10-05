//! Parameter Manager
//!
//! This module implements the Parameter Manager as defined in
//! IO-Link Specification v1.1.4

use iolinke_derived_config::device::vendor_specifics::storage_config::ParameterStorage;
use iolinke_macros::isdu_error_code;
use iolinke_macros::system_commands;
use iolinke_types::custom::{IoLinkError, IoLinkResult};
use iolinke_types::handlers;
use iolinke_types::handlers::ds::DsCommand;
use iolinke_types::handlers::pm::ValidityCheckResult;
use iolinke_types::handlers::pm::{
    DataStorageIndexSubIndex, DeviceParametersIndex, DsState, StateProperty, SubIndex,
};
use iolinke_util::{log_state_transition, log_state_transition_error};

use core::convert::TryFrom;
use core::option::{
    Option,
    Option::{None, Some},
};
use core::result::{
    Result,
    Result::{Err, Ok},
};

use crate::al::ApplicationLayerReadWriteInd;
use crate::al::data_storage;
use crate::al::od_handler;
use crate::al::services;
use crate::al::services::{AlReadRsp, AlWriteRsp};

/// Macro to configure block parameterization support and conditional code sections.
///
/// Usage:
/// ```ignore
/// block_param_support! {
///     supported {
///         // code if block parameterization is supported
///     }
///     not_supported {
///         // code if block parameterization is not supported
///     }
/// }
/// ```
///
/// If block parameterization is not supported, the code will be ignored.
///
/// If block parameterization is supported, the code will be executed.
#[macro_export]
macro_rules! block_param_support {
    (supported { $($supported_code:tt)* } not_supported { $($not_supported_code:tt)* }) => {
        #[cfg(feature = "block_parameterization")]
        {
            $($supported_code)*
        }
        #[cfg(not(feature = "block_parameterization"))]
        {
            $($not_supported_code)*
        }
    };
}

system_commands!(SystemCommand);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LockState {
    /// Access is locked
    Locked,
    /// Access is unlocked
    Unlocked,
}

enum WriteCycleStatus {
    /// Write cycle is not active
    Successful,
    /// Write cycle is active
    Failed,
    /// Write cycle is waiting for request
    WaitingForRequest,
}

#[derive(Debug, PartialEq, Eq)]
enum Transition {
    /// No transition to execute
    Tn,
    /// T1: State: Idle (0) -> ValidityCheck (1)
    /// Event: [Single Parameter]
    /// Action: -
    T1,
    /// T2: State: Idle (0) -> ValidityCheck (1)
    /// Event: [DownloadStore]
    /// Action: Set "StoreRequest" (= TRUE)
    T2,
    /// T3: State: Idle (0) -> ValidityCheck (1)
    /// Event: [Local Parameter]
    /// Action: Set "StoreRequest" (= TRUE)
    T3,
    /// T4: State: ValidityCheck (1) -> Idle (0)
    /// Event: [DataValid & DS_StoreRequest]
    /// Action: Mark parameter set as valid; invoke DS_ParUpload.req to DS; enable positive acknowledge of transmission; reset "StoreRequest" (= FALSE)
    T4,
    /// T5: State: ValidityCheck (1) -> Idle (0)
    /// Event: [DataValid & not DS_StoreRequest]
    /// Action: Mark parameter set as valid; enable positive acknowledge of transmission
    T5,
    /// T6: State: ValidityCheck (1) -> Idle (0)
    /// Event: [DataInvalid]
    /// Action: Mark parameter set as invalid; enable negative acknowledgment of transmission; reset "StoreRequest" (= FALSE); discard parameter buffer
    T6,
    /// T7: State: Idle (0) -> Download (2)
    /// Event: [DownloadStart]
    /// Action: Lock local parameter access
    T7,
    /// T8: State: Download (2) -> Idle (0)
    /// Event: [ParamBreak or UploadEnd]
    /// Action: Unlock local parameter access; discard parameter buffer
    T8,
    /// T9: State: Download (2) -> Idle (0)
    /// Event: DeviceMode_Change
    /// Action: Unlock local parameter access; discard parameter buffer
    T9,
    /// T10: State: Idle (0) -> Upload (3)
    /// Event: [UploadStart]
    /// Action: Lock local parameter access
    T10,
    /// T11: State: Upload (3) -> Idle (0)
    /// Event: [UploadEnd or ParamBreak or DownloadEnd]
    /// Action: Unlock local parameter access
    T11,
    /// T12: State: Upload (3) -> Idle (0)
    /// Event: DeviceMode_Change
    /// Action: Unlock local parameter access
    T12,
    /// T13: State: Download (2) -> ValidityCheck (1)
    /// Event: [DownloadEnd]
    /// Action: Unlock local parameter access
    T13,
    /// T14: State: Download (2) -> ValidityCheck (1)
    /// Event: [DownloadStore]
    /// Action: Unlock local parameter access; set "StoreRequest" (= TRUE)
    T14,
    /// T15: State: Upload (3) -> Upload (3)
    /// Event: [UploadStart]
    /// Action: Lock local parameter access
    T15,
    /// T16: State: Download (2) -> Download (2)
    /// Event: [DownloadStart]
    /// Action: Discard parameter buffer, so that a possible second start will not be blocked.
    T16,
    /// T17: State: Upload (3) -> ValidityCheck (1)
    /// Event: [DownloadStore]
    /// Action: Unlock local parameter access; set "StoreRequest" (= TRUE)
    T17,
    /// T18: State: Download (2) -> Upload (3)
    /// Event: [UploadStart]
    /// Action: Discard parameter buffer, so that a possible second start will not be blocked.
    T18,
    /// T19: State: Upload (3) -> Download (2)
    /// Event: [DownloadStart]
    /// Action: -
    T19,
    /// T20: State: Idle (0) -> Idle (0)
    /// Event: [UploadEnd or ParamBreak or DownloadEnd]
    /// Action: Return ErrorType 0x8036 – Function temporarily unavailable if Block Parameterization supported or ErrorType 0x8035 – Function not available if Block Parameterization is not supported.
    T20,
    /// T21: State: Download (2) -> Idle (0)
    /// Event: [SysCmdReset]
    /// Action: Unlock local parameter access; discard parameter buffer
    T21,
    /// T22: State: Upload (3) -> Idle (0)
    /// Event: [SysCmdReset]
    /// Action: Unlock local parameter access
    T22,
}

/// On request Data Handler states
/// See Figure 86 – The Parameter Manager (PM) state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterManagerState {
    /// {Idle_0}
    Idle,
    /// {ValidityCheck_1}
    ValidityCheck,
    /// {Download_2}
    Download,
    /// {Upload_3}
    Upload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterManagerEvent {
    /// {[Single Parameter]}
    SingleParameter,
    /// {[DownloadStore]}
    DownloadStore,
    /// {[Local Parameter]}
    _LocalParameter,
    /// {[DataValid & DS_StoreRequest]}
    DataValidAndStoreRequest,
    /// {[DataValid & not DS_StoreRequest]}
    DataValidAndNotStoreRequest,
    /// {[DataInvalid]}
    DataInvalid,
    /// {[DownloadStart]}
    DownloadStart,
    /// {[ParamBreak or UploadEnd]}
    ParamBreakOrUploadEnd,
    /// {DeviceMode_Change}
    DeviceModeChange,
    /// {[UploadStart]}
    UploadStart,
    /// {[UploadEnd or ParamBreak or DownloadEnd]}
    _UploadEndOrParamBreakOrDownloadEnd,
    /// {[DownloadEnd]}
    DownloadEnd,
    /// {[SysCmdReset]}
    SysCmdReset,
}
/// Process Data Handler implementation
pub struct ParameterManager {
    state: ParameterManagerState,
    exec_transition: Transition,
    store_request: bool,
    data_valid: ValidityCheckResult,
    local_parameter_access: LockState,
    ds_store_request: bool,
    param_storage: ParameterStorage,
    read_cycle: Option<(u16, u8)>, // (index, sub_index)
    write_cycle_status: WriteCycleStatus,
    ds_command: Option<DsCommand>,
}

impl ParameterManager {
    /// Create a new Process Data Handler
    pub fn new() -> Self {
        Self {
            state: ParameterManagerState::Idle,
            exec_transition: Transition::Tn,
            store_request: false,
            data_valid: ValidityCheckResult::YetToBeValidated,
            local_parameter_access: LockState::Unlocked,
            ds_store_request: false,
            param_storage: ParameterStorage::new(),
            read_cycle: None,
            write_cycle_status: WriteCycleStatus::WaitingForRequest,
            ds_command: None,
        }
    }

    // pub fn set_param_storage(&mut self, param_storage: &'b mut ParameterStorage) -> IoLinkResult<()> {
    //     self.param_storage = Some(param_storage);
    //     Ok(())
    // }

    /// Process an event
    pub fn process_event(&mut self, event: ParameterManagerEvent) -> IoLinkResult<()> {
        use ParameterManagerEvent as Event;
        use ParameterManagerState as State;

        let (new_transition, new_state) = match (self.state, event) {
            // T1: Idle -> ValidityCheck on [Single Parameter]
            (State::Idle, Event::SingleParameter) => (Transition::T1, State::ValidityCheck),

            // T2: Idle -> ValidityCheck on [DownloadStore]
            (State::Idle, Event::DownloadStore) => (Transition::T2, State::ValidityCheck),

            // T3: Idle -> ValidityCheck on [Local Parameter]
            (State::Idle, Event::_LocalParameter) => (Transition::T3, State::ValidityCheck),

            // T4: ValidityCheck -> Idle on [DataValid & DS_StoreRequest]
            (State::ValidityCheck, Event::DataValidAndStoreRequest) => {
                (Transition::T4, State::Idle)
            }

            // T5: ValidityCheck -> Idle on [DataValid & not DS_StoreRequest]
            (State::ValidityCheck, Event::DataValidAndNotStoreRequest) => {
                (Transition::T5, State::Idle)
            }

            // T6: ValidityCheck -> Idle on [DataInvalid]
            (State::ValidityCheck, Event::DataInvalid) => (Transition::T6, State::Idle),

            // T7: Idle -> Download on [DownloadStart]
            (State::Idle, Event::DownloadStart) => (Transition::T7, State::Download),

            // T8: Download -> Idle on [ParamBreak or UploadEnd]
            (State::Download, Event::ParamBreakOrUploadEnd) => (Transition::T8, State::Idle),

            // T9: Download -> Idle on DeviceMode_Change
            (State::Download, Event::DeviceModeChange) => (Transition::T9, State::Idle),

            // T10: Idle -> Upload on [UploadStart]
            (State::Idle, Event::UploadStart) => (Transition::T10, State::Upload),

            // T11: Upload -> Idle on [UploadEnd or ParamBreak or DownloadEnd]
            (State::Upload, Event::_UploadEndOrParamBreakOrDownloadEnd) => {
                (Transition::T11, State::Idle)
            }

            // T12: Upload -> Idle on DeviceMode_Change
            (State::Upload, Event::DeviceModeChange) => (Transition::T12, State::Idle),

            // T13: Download -> ValidityCheck on [DownloadEnd]
            (State::Download, Event::DownloadEnd) => (Transition::T13, State::ValidityCheck),

            // T14: Download -> ValidityCheck on [DownloadStore]
            (State::Download, Event::DownloadStore) => (Transition::T14, State::ValidityCheck),

            // T15: Upload -> Upload on [UploadStart]
            (State::Upload, Event::UploadStart) => (Transition::T15, State::Upload),

            // T16: Download -> Download on [DownloadStart]
            (State::Download, Event::DownloadStart) => (Transition::T16, State::Download),

            // T17: Upload -> ValidityCheck on [DownloadStore]
            (State::Upload, Event::DownloadStore) => (Transition::T17, State::ValidityCheck),

            // T18: Download -> Upload on [UploadStart]
            (State::Download, Event::UploadStart) => (Transition::T18, State::Upload),

            // T19: Upload -> Download on [DownloadStart]
            (State::Upload, Event::DownloadStart) => (Transition::T19, State::Download),

            // T20: Idle -> Idle on [UploadEnd or ParamBreak or DownloadEnd]
            (State::Idle, Event::_UploadEndOrParamBreakOrDownloadEnd) => {
                (Transition::T20, State::Idle)
            }

            // T21: Download -> Idle on [SysCmdReset]
            (State::Download, Event::SysCmdReset) => (Transition::T21, State::Idle),

            // T22: Upload -> Idle on [SysCmdReset]
            (State::Upload, Event::SysCmdReset) => (Transition::T22, State::Idle),
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

    /// Poll the process data handler
    /// See IO-Link v1.1.4 Section 7.2
    pub fn poll(
        &mut self,
        od_handler: &mut od_handler::OnRequestDataHandler,
        data_storage: &mut data_storage::DataStorage,
    ) -> IoLinkResult<()> {
        match self.exec_transition {
            Transition::Tn => {
                // No transition to execute
            }
            Transition::T1 => {
                self.exec_transition = Transition::Tn;
                // T1: Idle -> ValidityCheck on [Single Parameter]
                // Action: -
                self.execute_t1()?;
            }
            Transition::T2 => {
                self.exec_transition = Transition::Tn;
                // T2: Idle -> ValidityCheck on [DownloadStore]
                // Action: Set "StoreRequest" (= TRUE)
                self.execute_t2()?;
            }
            Transition::T3 => {
                self.exec_transition = Transition::Tn;
                // T3: Idle -> ValidityCheck on [Local Parameter]
                // Action: Set "StoreRequest" (= TRUE)
                self.execute_t3()?;
            }
            Transition::T4 => {
                self.exec_transition = Transition::Tn;
                // T4: ValidityCheck -> Idle on [DataValid & DS_StoreRequest]
                // Action: Mark parameter set as valid; invoke DS_ParUpload.req to DS; enable positive acknowledge of transmission; reset "StoreRequest" (= FALSE)
                self.execute_t4(od_handler, data_storage)?;
            }
            Transition::T5 => {
                self.exec_transition = Transition::Tn;
                // T5: ValidityCheck -> Idle on [DataValid & not DS_StoreRequest]
                // Action: Mark parameter set as valid; enable positive acknowledge of transmission
                self.execute_t5()?;
            }
            Transition::T6 => {
                self.exec_transition = Transition::Tn;
                // T6: ValidityCheck -> Idle on [DataInvalid]
                // Action: Mark parameter set as invalid; enable negative acknowledgment of transmission; reset "StoreRequest" (= FALSE); discard parameter buffer
                self.execute_t6(od_handler)?;
            }
            Transition::T7 => {
                self.exec_transition = Transition::Tn;
                // T7: Idle -> Download on [DownloadStart]
                // Action: Lock local parameter access
                self.execute_t7()?;
            }
            Transition::T8 => {
                self.exec_transition = Transition::Tn;
                // T8: Download -> Idle on [ParamBreak or UploadEnd]
                // Action: Unlock local parameter access; discard parameter buffer
                self.execute_t8()?;
            }
            Transition::T9 => {
                self.exec_transition = Transition::Tn;
                // T9: Download -> Idle on DeviceMode_Change
                // Action: Unlock local parameter access; discard parameter buffer
                self.execute_t9()?;
            }
            Transition::T10 => {
                self.exec_transition = Transition::Tn;
                // T10: Idle -> Upload on [UploadStart]
                // Action: Lock local parameter access
                self.execute_t10()?;
            }
            Transition::T11 => {
                self.exec_transition = Transition::Tn;
                // T11: Upload -> Idle on [UploadEnd or ParamBreak or DownloadEnd]
                // Action: Unlock local parameter access
                self.execute_t11()?;
            }
            Transition::T12 => {
                self.exec_transition = Transition::Tn;
                // T12: Upload -> Idle on DeviceMode_Change
                // Action: Unlock local parameter access
                self.execute_t12()?;
            }
            Transition::T13 => {
                self.exec_transition = Transition::Tn;
                // T13: Download -> ValidityCheck on [DownloadEnd]
                // Action: Unlock local parameter access
                self.execute_t13()?;
            }
            Transition::T14 => {
                self.exec_transition = Transition::Tn;
                // T14: Download -> ValidityCheck on [DownloadStore]
                // Action: Unlock local parameter access; set "StoreRequest" (= TRUE)
                self.execute_t14()?;
            }
            Transition::T15 => {
                self.exec_transition = Transition::Tn;
                // T15: Upload -> Upload on [UploadStart]
                // Action: Lock local parameter access
                self.execute_t15()?;
            }
            Transition::T16 => {
                self.exec_transition = Transition::Tn;
                // T16: Download -> Download on [DownloadStart]
                // Action: Discard parameter buffer, so that a possible second start will not be blocked
                self.execute_t16()?;
            }
            Transition::T17 => {
                self.exec_transition = Transition::Tn;
                // T17: Upload -> ValidityCheck on [DownloadStore]
                // Action: Unlock local parameter access; set "StoreRequest" (= TRUE)
                self.execute_t17()?;
            }
            Transition::T18 => {
                self.exec_transition = Transition::Tn;
                // T18: Download -> Upload on [UploadStart]
                // Action: Discard parameter buffer, so that a possible second start will not be blocked
                self.execute_t18()?;
            }
            Transition::T19 => {
                self.exec_transition = Transition::Tn;
                // T19: Upload -> Download on [DownloadStart]
                // Action: -
                self.execute_t19()?;
            }
            Transition::T20 => {
                self.exec_transition = Transition::Tn;
                // T20: Idle -> Idle on [UploadEnd or ParamBreak or DownloadEnd]
                // Action: Return ErrorType 0x8036 – Function temporarily unavailable if Block Parameterization supported or ErrorType 0x8035 – Function not available if Block Parameterization is not supported
                self.execute_t20(od_handler)?;
            }
            Transition::T21 => {
                self.exec_transition = Transition::Tn;
                // T21: Download -> Idle on [SysCmdReset]
                // Action: Unlock local parameter access; discard parameter buffer
                self.execute_t21()?;
            }
            Transition::T22 => {
                self.exec_transition = Transition::Tn;
                // T22: Upload -> Idle on [SysCmdReset]
                // Action: Unlock local parameter access
                self.execute_t22()?;
            }
        }
        let _ = self.poll_active_state(od_handler, data_storage);
        Ok(())
    }

    fn poll_active_state(
        &mut self,
        od_handler: &mut od_handler::OnRequestDataHandler,
        data_storage: &mut data_storage::DataStorage,
    ) -> IoLinkResult<()> {
        use ParameterManagerEvent as Event;
        use ParameterManagerState as State;

        match self.state {
            State::ValidityCheck => {
                if self.data_valid == ValidityCheckResult::Valid && self.ds_store_request {
                    self.process_event(Event::DataValidAndStoreRequest)?;
                } else if self.data_valid == ValidityCheckResult::Valid && !self.ds_store_request {
                    self.process_event(Event::DataValidAndNotStoreRequest)?;
                } else if self.data_valid == ValidityCheckResult::Invalid {
                    self.process_event(Event::DataInvalid)?;
                }
            }
            _ => {}
        }
        match self.read_cycle {
            Some((index, sub_index)) => {
                // Avoid creating a temporary that is dropped while borrowed
                let param: Result<(u8, &[u8]), _> =
                    self.param_storage.get_parameter(index, sub_index);
                match param {
                    Ok((length, data)) => {
                        let result: Result<(u8, &[u8]), services::AlRspError> =
                            services::AlResult::Ok((length, data));
                        od_handler.al_read_rsp(result)?;
                    }
                    Err(_) => {
                        let result =
                            services::AlResult::Err(services::AlRspError::Error(0x81, 0xFF));
                        od_handler.al_read_rsp(result)?;
                    }
                }
                self.read_cycle = None;
            }
            None => {}
        }
        match self.write_cycle_status {
            WriteCycleStatus::Successful => {
                self.write_cycle_status = WriteCycleStatus::WaitingForRequest;
                od_handler.al_write_rsp(Ok(()))?;
            }
            WriteCycleStatus::Failed => {
                self.write_cycle_status = WriteCycleStatus::WaitingForRequest;
                // TODO: Error codes must be verified and handled properly
                let error_codes = isdu_error_code!(SERV_NOTAVAIL);
                od_handler.al_write_rsp(Err(services::AlRspError::Error(
                    error_codes.0,
                    error_codes.1,
                )))?;
            }
            WriteCycleStatus::WaitingForRequest => {}
        }
        if let Some(ds_command) = self.ds_command {
            data_storage.ds_command(ds_command)?;
            self.ds_command = None;
        }
        Ok(())
    }

    /// Executes transition T1: Idle -> ValidityCheck on [Single Parameter]
    ///
    /// Action: -
    /// TODO: Implement any required logic for T1.
    pub fn execute_t1(&mut self) -> IoLinkResult<()> {
        // TODO: Implement any required logic for T1.
        Ok(())
    }

    /// Executes transition T2: Idle -> ValidityCheck on [DownloadStore]
    ///
    /// Action: Set "StoreRequest" (= TRUE)
    pub fn execute_t2(&mut self) -> IoLinkResult<()> {
        self.ds_store_request = true;
        Ok(())
    }

    /// Executes transition T3: Idle -> ValidityCheck on [Local Parameter]
    ///
    /// Action: Set "StoreRequest" (= TRUE)
    pub fn execute_t3(&mut self) -> IoLinkResult<()> {
        // TODO: Implement Local parameter event trigger handling
        self.store_request = true;
        Ok(())
    }

    /// Executes transition T4: ValidityCheck -> Idle on [DataValid & DS_StoreRequest]
    ///
    /// Action: Mark parameter set as valid; invoke DS_ParUpload.req to DS; enable positive acknowledge of transmission; reset "StoreRequest" (= FALSE)
    /// TODO: Mark parameter set as valid, invoke DS_ParUpload.req, enable positive ack, reset StoreRequest.
    pub fn execute_t4(
        &mut self,
        od_handler: &mut od_handler::OnRequestDataHandler,
        data_storage: &mut data_storage::DataStorage,
    ) -> IoLinkResult<()> {
        // Mark parameter set as valid
        self.data_valid = ValidityCheckResult::Valid;
        // Invoke DS_ParUpload.req to DS.
        data_storage.ds_par_upload_ind()?;
        // Send positive acknowledge of transmission
        od_handler.al_write_rsp(Ok(()))?;
        // Reset StoreRequest flag to false.
        self.store_request = false;
        self.data_valid = ValidityCheckResult::YetToBeValidated;
        Ok(())
    }

    /// Executes transition T5: ValidityCheck -> Idle on [DataValid & not DS_StoreRequest]
    ///
    /// Action: Mark parameter set as valid; enable positive acknowledge of transmission
    /// TODO: Mark parameter set as valid, enable positive ack.
    pub fn execute_t5(&mut self) -> IoLinkResult<()> {
        // TODO: Mark parameter set as valid.
        // TODO: Enable positive acknowledge of transmission.

        self.data_valid = ValidityCheckResult::YetToBeValidated;
        Ok(())
    }

    /// Executes transition T6: ValidityCheck -> Idle on [DataInvalid]
    ///
    /// Action: Mark parameter set as invalid; enable negative acknowledgment of transmission; reset "StoreRequest" (= FALSE); discard parameter buffer
    pub fn execute_t6(
        &mut self,
        od_handler: &mut od_handler::OnRequestDataHandler,
    ) -> IoLinkResult<()> {
        // Mark parameter set as invalid
        self.data_valid = ValidityCheckResult::Invalid;
        // Enable negative acknowledgment of transmission
        od_handler.al_write_rsp(Err(services::AlRspError::Error(0x81, 0xFF)))?; // TODO: Check if this is the correct error code
        // Reset StoreRequest flag to false.
        self.store_request = false;
        // Discard parameter buffer
        self.data_valid = ValidityCheckResult::YetToBeValidated;
        self.param_storage.clear();
        Ok(())
    }

    /// Executes transition T7: Idle -> Download on [DownloadStart]
    ///
    /// Action: Lock local parameter access; discard parameter buffer
    pub fn execute_t7(&mut self) -> IoLinkResult<()> {
        // Lock local parameter access.
        self.local_parameter_access = LockState::Locked;
        Ok(())
    }

    /// Executes transition T8: Idle -> Upload on [UploadStart]
    ///
    /// Action: Unlock local parameter access
    pub fn execute_t8(&mut self) -> IoLinkResult<()> {
        // Unlock local parameter access.
        self.local_parameter_access = LockState::Unlocked;
        self.data_valid = ValidityCheckResult::YetToBeValidated;
        // Discard parameter buffer.
        self.param_storage.clear();
        Ok(())
    }

    /// Executes transition T9: Download -> ValidityCheck on [DownloadStore]
    ///
    /// Action: Unlock local parameter access; set "StoreRequest" (= TRUE)
    pub fn execute_t9(&mut self) -> IoLinkResult<()> {
        // Unlock local parameter access.
        self.local_parameter_access = LockState::Unlocked;
        self.data_valid = ValidityCheckResult::YetToBeValidated;
        // Discard parameter buffer.
        self.param_storage.clear();
        Ok(())
    }

    /// Executes transition T10: Download -> Idle on [DownloadEnd]
    ///
    /// Action: Lock local parameter access; discard parameter buffer
    pub fn execute_t10(&mut self) -> IoLinkResult<()> {
        // Lock local parameter access.
        self.local_parameter_access = LockState::Locked;
        Ok(())
    }

    /// Executes transition T11: Upload -> Idle on [UploadEnd]
    ///
    /// Action: Unlock local parameter access
    pub fn execute_t11(&mut self) -> IoLinkResult<()> {
        // Unlock local parameter access.
        self.local_parameter_access = LockState::Unlocked;
        self.data_valid = ValidityCheckResult::YetToBeValidated;
        Ok(())
    }

    /// Executes transition T12: Download -> Idle on [ParamBreak]
    ///
    /// Action: Unlock local parameter access; discard parameter buffer
    pub fn execute_t12(&mut self) -> IoLinkResult<()> {
        // Unlock local parameter access.
        self.local_parameter_access = LockState::Unlocked;
        self.data_valid = ValidityCheckResult::YetToBeValidated;
        // Discard parameter buffer.
        self.param_storage.clear();
        Ok(())
    }

    /// Executes transition T13: Upload -> Idle on [ParamBreak]
    ///
    /// Action: Unlock local parameter access
    pub fn execute_t13(&mut self) -> IoLinkResult<()> {
        // Unlock local parameter access.
        self.local_parameter_access = LockState::Unlocked;
        Ok(())
    }

    /// Executes transition T14: Download -> ValidityCheck on [DownloadStore]
    ///
    /// Action: Unlock local parameter access; set "StoreRequest" (= TRUE)
    pub fn execute_t14(&mut self) -> IoLinkResult<()> {
        // Unlock local parameter access.
        self.local_parameter_access = LockState::Unlocked;
        // Set StoreRequest flag to true.
        self.store_request = true;
        Ok(())
    }

    /// Executes transition T15: Upload -> Upload on [UploadStart]
    ///
    /// Action: Lock local parameter access
    pub fn execute_t15(&mut self) -> IoLinkResult<()> {
        // Lock local parameter access.
        self.local_parameter_access = LockState::Locked;
        Ok(())
    }

    /// Executes transition T16: Download -> Download on [DownloadStart]
    ///
    /// Action: Discard parameter buffer, so that a possible second start will not be blocked
    pub fn execute_t16(&mut self) -> IoLinkResult<()> {
        self.data_valid = ValidityCheckResult::YetToBeValidated;
        // Discard parameter buffer.
        self.param_storage.clear();
        Ok(())
    }

    /// Executes transition T17: Upload -> ValidityCheck on [DownloadStore]
    ///
    /// Action: Unlock local parameter access; set "StoreRequest" (= TRUE)
    pub fn execute_t17(&mut self) -> IoLinkResult<()> {
        // Unlock local parameter access.
        self.local_parameter_access = LockState::Unlocked;
        // Set StoreRequest flag to true.
        self.store_request = true;
        Ok(())
    }

    /// Executes transition T18: Download -> Upload on [UploadStart]
    ///
    /// Action: Discard parameter buffer, so that a possible second start will not be blocked
    pub fn execute_t18(&mut self) -> IoLinkResult<()> {
        self.data_valid = ValidityCheckResult::YetToBeValidated;
        // Discard parameter buffer.
        self.param_storage.clear();
        Ok(())
    }

    /// Executes transition T19: Upload -> Download on [DownloadStart]
    ///
    /// Action: -
    pub fn execute_t19(&mut self) -> IoLinkResult<()> {
        self.data_valid = ValidityCheckResult::YetToBeValidated;
        Ok(())
    }

    /// Executes transition T20: Idle -> Idle on [UploadEnd or ParamBreak or DownloadEnd]
    ///
    /// Action: Return ErrorType 0x8036 – Function temporarily unavailable if Block Parameterization
    /// supported or ErrorType 0x8035 – Function not available if Block Parameterization is not supported
    pub fn execute_t20(
        &mut self,
        od_handler: &mut od_handler::OnRequestDataHandler,
    ) -> IoLinkResult<()> {
        // Implement error response based on Block Parameterization support.
        block_param_support! {
            supported {
                let (error_code, additional_code) = isdu_error_code!(FUNC_UNAVAILTEMP);
                od_handler.al_write_rsp(Err(services::AlRspError::Error(error_code, additional_code)))?;
            }
            not_supported {
                let (error_code, additional_code) = isdu_error_code!(FUNC_NOTAVAIL);
                od_handler.al_write_rsp(Err(services::AlRspError::Error(error_code, additional_code)))?;
            }
        }
        self.data_valid = ValidityCheckResult::YetToBeValidated;
        Ok(())
    }

    /// Executes transition T21: Download -> Idle on [SysCmdReset]
    ///
    /// Action: Unlock local parameter access; discard parameter buffer
    pub fn execute_t21(&mut self) -> IoLinkResult<()> {
        // Unlock local parameter access.
        self.local_parameter_access = LockState::Unlocked;
        // Discard parameter buffer.
        self.data_valid = ValidityCheckResult::YetToBeValidated;
        self.param_storage.clear();
        Ok(())
    }

    /// Executes transition T22: Upload -> Idle on [SysCmdReset]
    ///
    /// Action: Unlock local parameter access
    pub fn execute_t22(&mut self) -> IoLinkResult<()> {
        // Unlock local parameter access.
        self.local_parameter_access = LockState::Unlocked;

        self.data_valid = ValidityCheckResult::YetToBeValidated;
        Ok(())
    }

    fn handle_system_command(&mut self, system_command: SystemCommand) -> IoLinkResult<()> {
        match system_command {
            SystemCommand::ParamUploadStart => {
                self.process_event(ParameterManagerEvent::UploadStart)?;
            }
            SystemCommand::ParamUploadEnd => {
                self.process_event(ParameterManagerEvent::ParamBreakOrUploadEnd)?;
                todo!()
            }
            SystemCommand::ParamDownloadStart => {
                self.process_event(ParameterManagerEvent::DownloadStart)?;
            }
            SystemCommand::ParamDownloadEnd => {
                self.process_event(ParameterManagerEvent::DownloadEnd)?;
            }
            SystemCommand::ParamDownloadStore => {
                self.process_event(ParameterManagerEvent::DownloadStore)?;
            }
            SystemCommand::ParamBreak => {
                self.process_event(ParameterManagerEvent::ParamBreakOrUploadEnd)?;
                todo!()
            }
            SystemCommand::DeviceReset => {
                self.process_event(ParameterManagerEvent::SysCmdReset)?;
                todo!()
            }
            SystemCommand::ApplicationReset => {
                self.process_event(ParameterManagerEvent::SysCmdReset)?;
                todo!()
            }
            SystemCommand::RestoreFactorySettings => {
                self.process_event(ParameterManagerEvent::SysCmdReset)?;
                todo!()
            }
            SystemCommand::BackToBox => {
                self.process_event(ParameterManagerEvent::SysCmdReset)?;
                todo!()
            }
            SystemCommand::VendorSpecific(_) => {
                todo!()
            }
        }

        Ok(())
    }

    /// Set the upload flag in the data storage state property
    ///
    /// # Arguments
    ///
    /// * `upload_flag` - The upload flag to set
    ///
    /// # Returns
    pub fn set_upload_flag(&mut self, upload_flag: bool) -> IoLinkResult<()> {
        const STATE_PROPERTY_INDEX: u16 = DeviceParametersIndex::DataStorageIndex.index();
        const STATE_PROPERTY_SUBINDEX: u8 = DeviceParametersIndex::DataStorageIndex.subindex(
            SubIndex::DataStorageIndex(DataStorageIndexSubIndex::StateProperty),
        );

        let (_length, state_property) = self
            .param_storage
            .get_parameter(STATE_PROPERTY_INDEX, STATE_PROPERTY_SUBINDEX)
            .map_err(|_| IoLinkError::FailedToGetParameter)?;

        let mut state_property = StateProperty::from_bits(state_property[0]);
        state_property.set_ds_upload_flag(upload_flag);

        self.param_storage
            .set_parameter(
                STATE_PROPERTY_INDEX,
                STATE_PROPERTY_SUBINDEX,
                &[state_property.into_bits()],
            )
            .map_err(|_| IoLinkError::FailedToSetParameter)?;
        Ok(())
    }

    /// Set the state of data storage in the state property
    ///
    /// # Arguments
    ///
    /// * `state_of_ds` - The state of data storage to set
    ///
    /// # Returns
    pub fn set_state_property(&mut self, state_of_ds: DsState) -> IoLinkResult<()> {
        const STATE_PROPERTY_INDEX: u16 = DeviceParametersIndex::DataStorageIndex.index();
        const STATE_PROPERTY_SUBINDEX: u8 = DeviceParametersIndex::DataStorageIndex.subindex(
            SubIndex::DataStorageIndex(DataStorageIndexSubIndex::StateProperty),
        );

        let (_length, state_property) = self
            .param_storage
            .get_parameter(STATE_PROPERTY_INDEX, STATE_PROPERTY_SUBINDEX)
            .map_err(|_| IoLinkError::FailedToGetParameter)?;

        let mut state_property = StateProperty::from_bits(state_property[0]);
        state_property.set_ds_state(state_of_ds);

        self.param_storage
            .set_parameter(
                STATE_PROPERTY_INDEX,
                STATE_PROPERTY_SUBINDEX,
                &[state_property.into_bits()],
            )
            .map_err(|_| IoLinkError::FailedToSetParameter)?;
        Ok(())
    }

    pub fn lock_local_parameter_access(&mut self) -> IoLinkResult<()> {
        self.local_parameter_access = LockState::Locked;
        Ok(())
    }

    pub fn unlock_local_parameter_access(&mut self) -> IoLinkResult<()> {
        self.local_parameter_access = LockState::Unlocked;
        Ok(())
    }
}

impl ApplicationLayerReadWriteInd for ParameterManager {
    fn al_read_ind(&mut self, index: u16, sub_index: u8) -> IoLinkResult<()> {
        // For demonstration, return a zeroed array. Replace with actual parameter read logic.
        // In a real implementation, you would look up the parameter by index/sub_index.
        self.read_cycle = Some((index, sub_index));
        Ok(())
    }

    fn al_write_ind(&mut self, index: u16, sub_index: u8, data: &[u8]) -> IoLinkResult<()> {
        // For demonstration, accept the write. Replace with actual parameter write logic.
        // In a real implementation, you would update the parameter by index/sub_index.
        if self.data_valid == ValidityCheckResult::YetToBeValidated
            || self.data_valid == ValidityCheckResult::Valid
        {
            match self.param_storage.set_parameter(index, sub_index, data) {
                Ok(_) => {
                    self.data_valid = ValidityCheckResult::Valid;
                    self.write_cycle_status = WriteCycleStatus::Successful;
                }
                Err(_) => {
                    self.data_valid = ValidityCheckResult::Invalid;
                    self.write_cycle_status = WriteCycleStatus::Failed;
                }
            }
        }

        let param_index = DeviceParametersIndex::from_index(index);
        match param_index {
            Some(DeviceParametersIndex::SystemCommand) => {
                let system_command =
                    SystemCommand::from_u8(data[0]).ok_or(IoLinkError::InvalidData)?;
                self.handle_system_command(system_command)?;
                return Ok(());
            }
            Some(DeviceParametersIndex::DataStorageIndex) => {
                const DS_COMMAND_SUBINDEX: u8 = DeviceParametersIndex::DataStorageIndex.subindex(
                    SubIndex::DataStorageIndex(DataStorageIndexSubIndex::DsCommand),
                );
                if sub_index == DS_COMMAND_SUBINDEX {
                    self.ds_command = Some(DsCommand::try_from(data[0])?);
                }

                return Ok(());
            }
            _ => {
                // Handling single parameter
                self.process_event(ParameterManagerEvent::SingleParameter)?;
            }
        };

        Ok(())
    }

    fn al_abort_ind(&mut self) -> IoLinkResult<()> {
        // Handle abort indication, e.g., reset state or abort current operation.
        self.read_cycle = None;
        Ok(())
    }
}

impl handlers::sm::SystemManagementInd for ParameterManager {
    fn sm_device_mode_ind(
        &mut self,
        device_mode: handlers::sm::DeviceMode,
    ) -> handlers::sm::SmResult<()> {
        use ParameterManagerEvent as Event;
        match device_mode {
            handlers::sm::DeviceMode::Idle | handlers::sm::DeviceMode::Startup => {
                let _ = self.process_event(Event::DeviceModeChange);
            }
            _ => return Ok(()),
        }
        Ok(())
    }
}
