//! On-request Data Handler
//!
//! This module implements the On-request Data Handler state machine as defined in
//! IO-Link Specification v1.1.4

use crate::types::{IoLinkError, IoLinkResult};

/// See 8.3.2.2 OD state machine of the Device AL
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnRequestHandlerState {
    /// {Idle_0}
    Idle,
    /// {Await_AL_Write_rsp_1}
    AwaitAlWriteRsp,
    /// {Await_AL_Read_rsp_2}
    AwaitAlReadRsp,
    /// {Await_AL_RW_rsp_3}
    AwaitAlRwRsp,
}

/// See Table 75 – States and transitions for the OD state machine of the Device AL
#[derive(Debug, PartialEq, Eq)]
enum Transition {
    /// Tn: No transition
    Tn,
    /// T1: State: Idle (0) -> AwaitAlWriteRsp (1)
    /// Action: Invoke AL_Write
    T1,
    /// T2: State: AwaitAlWriteRsp (1) -> Idle (0)
    /// Action: Invoke DL_WriteParam (16 to 31)
    T2,
    /// T3: State: Idle (0) -> AwaitAlReadRsp (2)
    /// Action: Invoke AL_Read
    T3,
    /// T4: State: AwaitAlReadRsp (2) -> Idle (0)
    /// Action: Invoke DL_ReadParam (0 to 31)
    T4,
    /// T5: State: Idle (0) -> AwaitAlRwRsp (3)
    /// Action: Invoke AL_Read
    T5,
    /// T6: State: Idle (0) -> AwaitAlRwRsp (3)
    /// Action: Invoke AL_Write
    T6,
    /// T7: State: AwaitAlRwRsp (3) -> Idle (0)
    /// Action: Invoke DL_ISDUTransport (read)
    T7,
    /// T8: State: AwaitAlRwRsp (3) -> Idle (0)
    /// Action: Invoke DL_ISDUTransport (write)
    T8,
    /// T9: State: AwaitAlRwRsp (3) -> Idle (0)
    /// Action: Current AL_Read or AL_Write abandoned 
    /// upon this asynchronous AL_Abort service call.
    /// Return negative DL_ISDUTransport (see 3.3.7)
    T9,
    /// T10: State: AwaitAlRwRsp (3) -> Idle (0)
    /// Action: Current waiting on AL_Read or AL_Write abandoned
    T10,
    /// T11: State: Idle (0) -> Idle (0)
    /// Action: Current DL_ISDUTransport abandoned. All OD are set to "0"
    T11,
}

/// See 8.3.2.2 OD state machine of the Device AL
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnRequestHandlerEvent {
    /// {AL_Abort}
    AlAbort,
    /// {DL_WriteParam_ind}
    DlWriteParamInd,
    /// {AL_Write_rsp}
    AlWriteRsp,
    /// {DL_ReadParam_ind}
    DlReadParamInd,
    /// {AL_Read_rsp}
    AlReadRsp,
    /// {DL_ISDUTransport_ind[DirRead]}
    DlIsduTransportIndDirRead,
    /// {DL_ISDUTransport_ind[DirWrite]}
    DlIsduTransportIndDirWrite,
    /// {DL_ISDUAbort}
    DlIsduAbort,
}

/// On-request Data Handler implementation
pub struct OnRequestHandler {
    state: OnRequestHandlerState,
    exec_transition: Transition,
}

impl OnRequestHandler {
    /// Create a new On-request Data Handler
    pub fn new() -> Self {
        Self {
            state: OnRequestHandlerState::Idle,
            exec_transition: Transition::Tn,
        }
    }

    /// Process an event
    pub fn process_event(&mut self, event: OnRequestHandlerEvent) -> IoLinkResult<()> {
        use OnRequestHandlerState as State;
        use OnRequestHandlerEvent as Event;

        let (new_transition, new_state) = match (self.state, event) {
            // Valid transitions according to Table 75
            (State::Idle, Event::AlAbort) => (Transition::T11, State::Idle),
            (State::Idle, Event::DlWriteParamInd) => (Transition::T1, State::AwaitAlWriteRsp),
            (State::Idle, Event::DlReadParamInd) => (Transition::T3, State::AwaitAlReadRsp),
            (State::Idle, Event::DlIsduTransportIndDirRead) => (Transition::T5, State::AwaitAlRwRsp),
            (State::Idle, Event::DlIsduTransportIndDirWrite) => (Transition::T6, State::AwaitAlRwRsp),
            (State::AwaitAlWriteRsp, Event::AlWriteRsp) => (Transition::T2, State::Idle),
            (State::AwaitAlReadRsp, Event::AlReadRsp) => (Transition::T4, State::Idle),
            (State::AwaitAlRwRsp, Event::DlIsduAbort) => (Transition::T10, State::Idle),
            (State::AwaitAlRwRsp, Event::AlReadRsp) => (Transition::T7, State::Idle),
            (State::AwaitAlRwRsp, Event::AlWriteRsp) => (Transition::T8, State::Idle),
            // Invalid transitions - no state change
            _ => return Err(IoLinkError::InvalidEvent),
        };
        self.exec_transition = new_transition;
        self.state = new_state;

        Ok(())
    }

    /// Poll the state machine
    pub fn poll(&mut self) -> IoLinkResult<()> {
        // Process pending transitions
        match self.exec_transition {
            Transition::Tn => {
                // No transition, do nothing
            }
            Transition::T1 => {
                // Invoke AL_Write
            }
            Transition::T2 => {
                // Invoke DL_WriteParam (16 to 31)
            }
            Transition::T3 => {
                // Invoke AL_Read
            }
            Transition::T4 => {
                // Invoke DL_ReadParam (0 to 31)
            }
            Transition::T5 => {
                // Invoke AL_Read
            }
            Transition::T6 => {
                // Invoke AL_Write
            }
            Transition::T7 => {
                // Invoke DL_ISDUTransport (read)
            }
            Transition::T8 => {
                // Invoke DL_ISDUTransport (write)
            }
            Transition::T9 => {
                // Handle abort scenarios
            }
            Transition::T10 => {
                // Handle abort scenarios
            }
            Transition::T11 => {
                // Handle abort scenarios
            }
        }

        Ok(())
    }
}

impl Default for OnRequestHandler {
    fn default() -> Self {
        Self::new()
    }
}
