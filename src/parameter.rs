//! Parameter Manager
//!
//! This module implements the Parameter Manager as defined in
//! IO-Link Specification v1.1.4

use crate::types::{IoLinkError, IoLinkResult};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterManagerEvent {
    /// {[Single Parameter]}
    SingleParameter,
    /// {[DownloadStore]}
    DownloadStore,
    /// {[Local Parameter]}
    LocalParameter,
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
    UploadEndOrParamBreakOrDownloadEnd,
    /// {[DownloadEnd]}
    DownloadEnd,
    /// {[SysCmdReset]}
    SysCmdReset,
}
/// Process Data Handler implementation
pub struct ParameterManager {
    state: ParameterManagerState,
    exec_transition: Transition,
}

impl ParameterManager {
    /// Create a new Process Data Handler
    pub fn new() -> Self {
        Self {
            state: ParameterManagerState::Idle,
            exec_transition: Transition::Tn,
        }
    }

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
            (State::Idle, Event::LocalParameter) => (Transition::T3, State::ValidityCheck),

            // T4: ValidityCheck -> Idle on [DataValid & DS_StoreRequest]
            (State::ValidityCheck, Event::DataValidAndStoreRequest) => (Transition::T4, State::Idle),

            // T5: ValidityCheck -> Idle on [DataValid & not DS_StoreRequest]
            (State::ValidityCheck, Event::DataValidAndNotStoreRequest) => (Transition::T5, State::Idle),

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
            (State::Upload, Event::UploadEndOrParamBreakOrDownloadEnd) => (Transition::T11, State::Idle),

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
            (State::Idle, Event::UploadEndOrParamBreakOrDownloadEnd) => (Transition::T20, State::Idle),

            // T21: Download -> Idle on [SysCmdReset]
            (State::Download, Event::SysCmdReset) => (Transition::T21, State::Idle),

            // T22: Upload -> Idle on [SysCmdReset]
            (State::Upload, Event::SysCmdReset) => (Transition::T22, State::Idle),
            _ => return Err(IoLinkError::InvalidEvent),
        };
        self.exec_transition = new_transition;
        self.state = new_state;

        Ok(())
    }

    /// Poll the process data handler
    /// See IO-Link v1.1.4 Section 7.2
    pub fn poll(&mut self) -> IoLinkResult<()> {
        match self.exec_transition {
            Transition::Tn => {
                // No transition to execute
            }
            Transition::T1 => {
                // T1: Idle -> ValidityCheck on [Single Parameter]
                // Action: -
            }
            Transition::T2 => {
                // T2: Idle -> ValidityCheck on [DownloadStore]
                // Action: Set "StoreRequest" (= TRUE)
                // TODO: Implement store request setting
            }
            Transition::T3 => {
                // T3: Idle -> ValidityCheck on [Local Parameter]
                // Action: Set "StoreRequest" (= TRUE)
                // TODO: Implement store request setting
            }
            Transition::T4 => {
                // T4: ValidityCheck -> Idle on [DataValid & DS_StoreRequest]
                // Action: Mark parameter set as valid; invoke DS_ParUpload.req to DS; enable positive acknowledge of transmission; reset "StoreRequest" (= FALSE)
                // TODO: Implement parameter validation, data storage upload, and acknowledgment
            }
            Transition::T5 => {
                // T5: ValidityCheck -> Idle on [DataValid & not DS_StoreRequest]
                // Action: Mark parameter set as valid; enable positive acknowledge of transmission
                // TODO: Implement parameter validation and acknowledgment
            }
            Transition::T6 => {
                // T6: ValidityCheck -> Idle on [DataInvalid]
                // Action: Mark parameter set as invalid; enable negative acknowledgment of transmission; reset "StoreRequest" (= FALSE); discard parameter buffer
                // TODO: Implement parameter invalidation, negative acknowledgment, and buffer cleanup
            }
            Transition::T7 => {
                // T7: Idle -> Download on [DownloadStart]
                // Action: Lock local parameter access
                // TODO: Implement parameter access locking
            }
            Transition::T8 => {
                // T8: Download -> Idle on [ParamBreak or UploadEnd]
                // Action: Unlock local parameter access; discard parameter buffer
                // TODO: Implement parameter access unlocking and buffer cleanup
            }
            Transition::T9 => {
                // T9: Download -> Idle on DeviceMode_Change
                // Action: Unlock local parameter access; discard parameter buffer
                // TODO: Implement parameter access unlocking and buffer cleanup
            }
            Transition::T10 => {
                // T10: Idle -> Upload on [UploadStart]
                // Action: Lock local parameter access
                // TODO: Implement parameter access locking
            }
            Transition::T11 => {
                // T11: Upload -> Idle on [UploadEnd or ParamBreak or DownloadEnd]
                // Action: Unlock local parameter access
                // TODO: Implement parameter access unlocking
            }
            Transition::T12 => {
                // T12: Upload -> Idle on DeviceMode_Change
                // Action: Unlock local parameter access
                // TODO: Implement parameter access unlocking
            }
            Transition::T13 => {
                // T13: Download -> ValidityCheck on [DownloadEnd]
                // Action: Unlock local parameter access
                // TODO: Implement parameter access unlocking
            }
            Transition::T14 => {
                // T14: Download -> ValidityCheck on [DownloadStore]
                // Action: Unlock local parameter access; set "StoreRequest" (= TRUE)
                // TODO: Implement parameter access unlocking and store request setting
            }
            Transition::T15 => {
                // T15: Upload -> Upload on [UploadStart]
                // Action: Lock local parameter access
                // TODO: Implement parameter access locking
            }
            Transition::T16 => {
                // T16: Download -> Download on [DownloadStart]
                // Action: Discard parameter buffer, so that a possible second start will not be blocked
                // TODO: Implement parameter buffer cleanup
            }
            Transition::T17 => {
                // T17: Upload -> ValidityCheck on [DownloadStore]
                // Action: Unlock local parameter access; set "StoreRequest" (= TRUE)
                // TODO: Implement parameter access unlocking and store request setting
            }
            Transition::T18 => {
                // T18: Download -> Upload on [UploadStart]
                // Action: Discard parameter buffer, so that a possible second start will not be blocked
                // TODO: Implement parameter buffer cleanup
            }
            Transition::T19 => {
                // T19: Upload -> Download on [DownloadStart]
                // Action: -
            }
            Transition::T20 => {
                // T20: Idle -> Idle on [UploadEnd or ParamBreak or DownloadEnd]
                // Action: Return ErrorType 0x8036 – Function temporarily unavailable if Block Parameterization supported or ErrorType 0x8035 – Function not available if Block Parameterization is not supported
                // TODO: Implement error response based on Block Parameterization support
                // return Err(IoLinkError::FunctionNotAvailable);
            }
            Transition::T21 => {
                // T21: Download -> Idle on [SysCmdReset]
                // Action: Unlock local parameter access; discard parameter buffer
                // TODO: Implement parameter access unlocking and buffer cleanup
            }
            Transition::T22 => {
                // T22: Upload -> Idle on [SysCmdReset]
                // Action: Unlock local parameter access
                // TODO: Implement parameter access unlocking
            }
        }
        Ok(())
    }
}

impl Default for ParameterManager {
    fn default() -> Self {
        Self::new()
    }
}
