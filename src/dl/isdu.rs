//! ISDU (Index Service Data Unit) Handler
//!
//! This module implements the ISDU Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 8.4.3

use crate::types::{IoLinkError, IoLinkResult, Isdu};
use heapless::Vec;

/// See 7.3.6.4 State machine of the Device ISDU handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsduHandlerState {
    /// {Inactive_0}
    Inactive,
    /// {Idle_1}
    Idle,
    /// {ISDURequest_2}
    ISDURequest,
    /// {ISDUResponse_4}
    ISDUResponse,
    /// {ISDUWait_3}
    ISDUWait,
}

/// See Table 54 – State transition tables of the Device ISDU handler
#[derive(Debug, PartialEq, Eq)]
enum Transition {
    /// Tn: No transition
    Tn,
    /// T1: State: Inactive (0) -> Idle (1)
    /// Action: -
    T1,
    /// T2: State: Idle (1) -> ISDURequest (2)
    /// Action: Start receiving of ISDU request data
    T2,
    /// T3: State: ISDURequest (2) -> ISDURequest (2)
    /// Action: Receive ISDU request data
    T3,
    /// T4: State: ISDURequest (2) -> ISDUWait (3)
    /// Action: Invoke DL_ISDUTransport.ind to AL (see 7.2.1.6)
    T4,
    /// T5: State: ISDUWait (3) -> ISDUWait (3)
    /// Action: Invoke OD.rsp with "busy" indication (see Table A.14)
    T5,
    /// T6: State: ISDUWait (3) -> ISDUResponse (4)
    /// Action: -
    T6,
    /// T7: State: ISDUResponse (4) -> ISDUResponse (4)
    /// Action: Invoke OD.rsp with ISDU response data
    T7,
    /// T8: State: ISDUResponse (4) -> Idle (1)
    /// Action: -
    T8,
    /// T9: State: ISDURequest (2) -> Idle (1)
    /// Action: -
    T9,
    /// T10: State: ISDUWait (3) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    T10,
    /// T11: State: ISDUResponse (4) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    T11,
    /// T12: State: Idle (1) -> Inactive (0)
    /// Action: -
    T12,
    /// T13: State: ISDURequest (2) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    T13,
    /// T14: State: Idle (1) -> Idle (1)
    /// Action: Invoke OD.rsp with "no service" indication (see Table A.12 and Table A.14)
    T14,
    /// T15: State: ISDUWait (3) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    T15,
    /// T16: State: ISDUResponse (4) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    T16,
}

/// See Figure 52 – State machine of the Device ISDU handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsduHandlerEvent {
    /// {ISDURead}
    IsduRead,
    /// {IH_Conf_ACTIVE}
    IhConfActive,
    /// {ISDUWrite}
    IsduWrite,
    /// {ISDUStart}
    IsduStart,
    /// {[ISDUError]}
    IsduError,
    /// {IH_Conf_INACTIVE}
    IhConfInactive,
    /// {[ISDUSendComplete]}
    IsduSendComplete,
    /// {ISDUAbort}
    IsduAbort,
    /// {[ISDURecComplete]}
    IsduRecComplete,
    /// {ISDURespStart}
    IsduRespStart,
}

/// ISDU Handler implementation
pub struct IsduHandler {
    state: IsduHandlerState,
    exec_transition: Transition,
    current_request: Option<Isdu>,
    response_data: Vec<u8, 32>,
}

impl IsduHandler {
    /// Create a new ISDU Handler
    pub fn new() -> Self {
        Self {
            state: IsduHandlerState::Idle,
            exec_transition: Transition::Tn,
            current_request: None,
            response_data: Vec::new(),
        }
    }

    /// Process an event
    pub fn process_event(&mut self, event: IsduHandlerEvent) -> IoLinkResult<()> {
        use IsduHandlerEvent as Event;
        use IsduHandlerState as State;

        let (new_transition, new_state) = match (self.state, event) {
            (State::Inactive, Event::IhConfActive) => {
                (Transition::T1, State::Idle)
            }
            (State::Idle, Event::IsduStart) => {
                (Transition::T2, State::ISDURequest)
            }
            (State::Idle, Event::IhConfInactive) => {
                (Transition::T12, State::Inactive)
            }
            (State::Idle, Event::IsduRead) => {
                (Transition::T14, State::Idle)
            }
            (State::ISDURequest, Event::IsduWrite) => {
                (Transition::T3, State::ISDURequest)
            }
            (State::ISDURequest, Event::IsduRecComplete) => {
                (Transition::T4, State::ISDUWait)
            }
            (State::ISDURequest, Event::IsduError) => {
                (Transition::T13, State::Idle)
            }
            (State::ISDURequest, Event::IsduAbort) => {
                (Transition::T9, State::Idle)
            }
            (State::ISDUWait, Event::IsduRead) => {
                (Transition::T5, State::ISDUWait)
            }
            (State::ISDUWait, Event::IsduRespStart) => {
                (Transition::T6, State::ISDUResponse)
            }
            (State::ISDUWait, Event::IsduAbort) => {
                (Transition::T10, State::Idle)
            }
            (State::ISDUWait, Event::IsduError) => {
                (Transition::T15, State::Idle)
            }
            (State::ISDUResponse, Event::IsduRead) => {
                (Transition::T7, State::ISDUResponse)
            }
            (State::ISDUResponse, Event::IsduSendComplete) => {
                (Transition::T8, State::Idle)
            }
            (State::ISDUResponse, Event::IsduAbort) => {
                (Transition::T11, State::Idle)
            }
            (State::ISDUResponse, Event::IsduError) => {
                (Transition::T16, State::Idle)
            }
            _ => return Err(IoLinkError::InvalidEvent),
        };
        self.exec_transition = new_transition;
        self.state = new_state;

        Ok(())

    }
    /// Poll the ISDU handler
    /// See IO-Link v1.1.4 Section 8.4.3
    pub fn poll(&mut self) -> IoLinkResult<()> {
        match self.exec_transition {
            Transition::Tn => {
                // No transition to execute
            }
            Transition::T1 => {
                // State: Inactive (0) -> Idle (1)
                self.state = IsduHandlerState::Idle;
            }
            Transition::T2 => {
                // State: Idle (1) -> ISDURequest (2)
                self.state = IsduHandlerState::ISDURequest;
            }
            Transition::T3 => {
                // State: ISDURequest (2) -> ISDURequest (2)
                // Continue receiving ISDU request data
            }
            Transition::T4 => {
                // State: ISDURequest (2) -> ISDUWait (3)
                self.state = IsduHandlerState::ISDUWait;
            }
            Transition::T5 => {
                // State: ISDUWait (3) -> ISDUWait (3)
                // Invoke OD.rsp with "busy" indication
            }
            Transition::T6 => {
                // State: ISDUWait (3) -> ISDUResponse (4)
                self.state = IsduHandlerState::ISDUResponse;
            }
            Transition::T7 => {
                // State: ISDUResponse (4) -> ISDUResponse (4)
                // Invoke OD.rsp with ISDU response data
            }
            Transition::T8 => {
                // State: ISDUResponse (4) -> Idle (1)
                self.state = IsduHandlerState::Idle;
            }
            Transition::T9 => {
                // State: ISDURequest (2) -> Idle (1)
                self.state = IsduHandlerState::Idle;
            }
            Transition::T10 => {
                // State: ISDUWait (3) -> Idle (1)
                self.state = IsduHandlerState::Idle;
            }
            Transition::T11 => {
                // State: ISDUResponse (4) -> Idle (1)
                self.state = IsduHandlerState::Idle;
            }
            Transition::T12 => {
                // State: Idle (1) -> Inactive (0)
                self.state = IsduHandlerState::Inactive;
            }
            Transition::T13 => {        
                // State: ISDURequest (2) -> Idle (1)
                self.state = IsduHandlerState::Idle;
            }
            Transition::T14 => {
                // State: Idle (1) -> Idle (1)
                // Invoke OD.rsp with "no service" indication
            }
            Transition::T15 => {
                // State: ISDUWait (3) -> Idle (1)
                self.state = IsduHandlerState::Idle;
            }
            Transition::T16 => {
                // State: ISDUResponse (4) -> Idle (1)
                self.state = IsduHandlerState::Idle;
            }
        }
        Ok(())
    }

    /// Process an ISDU request
    fn process_request(&mut self, request: &Isdu) -> IoLinkResult<()> {
        if request.is_write {
            // Handle write request
            self.handle_write(request)?;
        } else {
            // Handle read request
            self.handle_read(request)?;
        }
        Ok(())
    }

    /// Handle ISDU read request
    fn handle_read(&mut self, request: &Isdu) -> IoLinkResult<()> {
        // Implementation would read from parameter storage
        // For now, return dummy data based on index
        self.response_data.clear();
        match request.index {
            0x0000 => {
                // Vendor ID
                self.response_data.push(0x00).ok();
                self.response_data.push(0x01).ok();
            }
            0x0001 => {
                // Device ID
                self.response_data.push(0x00).ok();
                self.response_data.push(0x00).ok();
                self.response_data.push(0x00).ok();
                self.response_data.push(0x01).ok();
            }
            _ => {
                return Err(IoLinkError::InvalidParameter);
            }
        }
        Ok(())
    }
}

impl Default for IsduHandler {
    fn default() -> Self {
        Self::new()
    }
}
