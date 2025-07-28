//! ISDU (Index Service Data Unit) Handler
//!
//! This module implements the ISDU Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 8.4.3

use crate::{
    dl,
    types::{self, IoLinkError, IoLinkResult, Isdu},
};
use heapless::Vec;
use iolinke_macros::flow_ctrl;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OdIndData {
    rw_direction: types::RwDirection,
    com_channel: types::ComChannel,
    address_ctrl: u8,
    length: u8,
    data: [u8; 32],
}
/// See 7.3.6.4 State machine of the Device ISDU handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IsduHandlerState {
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
    T2(OdIndData),
    /// T3: State: ISDURequest (2) -> ISDURequest (2)
    /// Action: Receive ISDU request data
    T3(OdIndData),
    /// T4: State: ISDURequest (2) -> ISDUWait (3)
    /// Action: Invoke DL_ISDUTransport.ind to AL (see 7.2.1.6)
    T4(OdIndData),
    /// T5: State: ISDUWait (3) -> ISDUWait (3)
    /// Action: Invoke OD.rsp with "busy" indication (see Table A.14)
    T5(OdIndData),
    /// T6: State: ISDUWait (3) -> ISDUResponse (4)
    /// Action: -
    T6,
    /// T7: State: ISDUResponse (4) -> ISDUResponse (4)
    /// Action: Invoke OD.rsp with ISDU response data
    T7(OdIndData),
    /// T8: State: ISDUResponse (4) -> Idle (1)
    /// Action: -
    T8(OdIndData),
    /// T9: State: ISDURequest (2) -> Idle (1)
    /// Action: -
    T9(OdIndData),
    /// T10: State: ISDUWait (3) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    T10(OdIndData),
    /// T11: State: ISDUResponse (4) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    T11(OdIndData),
    /// T12: State: Idle (1) -> Inactive (0)
    /// Action: -
    T12,
    /// T13: State: ISDURequest (2) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    T13,
    /// T14: State: Idle (1) -> Idle (1)
    /// Action: Invoke OD.rsp with "no service" indication (see Table A.12 and Table A.14)
    T14(OdIndData),
    /// T15: State: ISDUWait (3) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    T15,
    /// T16: State: ISDUResponse (4) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    T16,
}

/// See Figure 52 – State machine of the Device ISDU handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IsduHandlerEvent {
    /// {ISDURead}
    IsduRead(OdIndData),
    /// {IH_Conf_ACTIVE}
    IhConfActive,
    /// {ISDUWrite}
    IsduWrite(OdIndData),
    /// {ISDUStart}
    IsduStart(OdIndData),
    /// {[ISDUError]}
    IsduError,
    /// {IH_Conf_INACTIVE}
    IhConfInactive,
    /// {[ISDUSendComplete]}
    IsduSendComplete(OdIndData),
    /// {ISDUAbort}
    IsduAbort(OdIndData),
    /// {[ISDURecComplete]}
    IsduRecComplete(OdIndData),
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
            (State::Inactive, Event::IhConfActive) => (Transition::T1, State::Idle),
            (State::Idle, Event::IsduStart(od_ind_data)) => {
                (Transition::T2(od_ind_data), State::ISDURequest)
            }
            (State::Idle, Event::IhConfInactive) => (Transition::T12, State::Inactive),
            (State::Idle, Event::IsduRead(od_ind_data)) => {
                (Transition::T14(od_ind_data), State::Idle)
            }
            (State::ISDURequest, Event::IsduWrite(od_ind_data)) => {
                (Transition::T3(od_ind_data), State::ISDURequest)
            }
            (State::ISDURequest, Event::IsduRecComplete(od_ind_data)) => {
                (Transition::T4(od_ind_data), State::ISDUWait)
            }
            (State::ISDURequest, Event::IsduError) => (Transition::T13, State::Idle),
            (State::ISDURequest, Event::IsduAbort(od_ind_data)) => {
                (Transition::T9(od_ind_data), State::Idle)
            }
            (State::ISDUWait, Event::IsduRead(od_ind_data)) => {
                (Transition::T5(od_ind_data), State::ISDUWait)
            }
            (State::ISDUWait, Event::IsduRespStart) => (Transition::T6, State::ISDUResponse),
            (State::ISDUWait, Event::IsduAbort(od_ind_data)) => {
                (Transition::T10(od_ind_data), State::Idle)
            }
            (State::ISDUWait, Event::IsduError) => (Transition::T15, State::Idle),
            (State::ISDUResponse, Event::IsduRead(od_ind_data)) => {
                (Transition::T7(od_ind_data), State::ISDUResponse)
            }
            (State::ISDUResponse, Event::IsduSendComplete(od_ind_data)) => {
                (Transition::T8(od_ind_data), State::Idle)
            }
            (State::ISDUResponse, Event::IsduAbort(od_ind_data)) => {
                (Transition::T11(od_ind_data), State::Idle)
            }
            (State::ISDUResponse, Event::IsduError) => (Transition::T16, State::Idle),
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
            Transition::T2(od_ind_data) => {
                // State: Idle (1) -> ISDURequest (2)
                self.state = IsduHandlerState::ISDURequest;
            }
            Transition::T3(od_ind_data) => {
                // State: ISDURequest (2) -> ISDURequest (2)
                // Continue receiving ISDU request data
            }
            Transition::T4(od_ind_data) => {
                // State: ISDURequest (2) -> ISDUWait (3)
                self.state = IsduHandlerState::ISDUWait;
            }
            Transition::T5(od_ind_data) => {
                // State: ISDUWait (3) -> ISDUWait (3)
                // Invoke OD.rsp with "busy" indication
            }
            Transition::T6 => {
                // State: ISDUWait (3) -> ISDUResponse (4)
                self.state = IsduHandlerState::ISDUResponse;
            }
            Transition::T7(od_ind_data) => {
                // State: ISDUResponse (4) -> ISDUResponse (4)
                // Invoke OD.rsp with ISDU response data
            }
            Transition::T8(od_ind_data) => {
                // State: ISDUResponse (4) -> Idle (1)
                self.state = IsduHandlerState::Idle;
            }
            Transition::T9(od_ind_data) => {
                // State: ISDURequest (2) -> Idle (1)
                self.state = IsduHandlerState::Idle;
            }
            Transition::T10(od_ind_data) => {
                // State: ISDUWait (3) -> Idle (1)
                self.state = IsduHandlerState::Idle;
            }
            Transition::T11(od_ind_data) => {
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
            Transition::T14(od_ind_data) => {
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

    /// Handle ISDU configuration changes
    /// See 7.3.6.4 State machine of the Device ISDU handler
    pub fn ih_conf(&mut self, state: types::IhConfState) -> IoLinkResult<()> {
        match state {
            types::IhConfState::Active => self.process_event(IsduHandlerEvent::IhConfActive),
            types::IhConfState::Inactive => self.process_event(IsduHandlerEvent::IhConfInactive),
        };
        Ok(())
    }

    /// Process an ISDU request
    fn process_request(&mut self, request: &Isdu) -> IoLinkResult<()> {
        if request.is_write {
            // Handle write request
            // self.handle_write(request)?;
        } else {
            // Handle read request
            // self.handle_read(request)?;
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

impl dl::message::OdInd for IsduHandler {
    fn od_ind(
        &mut self,
        rw_direction: types::RwDirection,
        com_channel: types::ComChannel,
        address_ctrl: u8,
        length: u8,
        data: &[u8],
    ) -> IoLinkResult<()> {
        use types::RwDirection::Read;
        use types::RwDirection::Write;
        // Process the ISDU request
        let mut data_array = [0u8; 32];
        let copy_len = data.len().min(32);
        data_array[..copy_len].copy_from_slice(&data[..copy_len]);

        let od_data = OdIndData {
            rw_direction,
            com_channel,
            address_ctrl,
            length,
            data: data_array,
        };

        let event = if com_channel == types::ComChannel::Isdu {
            // Determine event based on the OD.ind parameters
            match (rw_direction, address_ctrl) {
                // ISDUStart: OD.ind(W, ISDU, Start, Data)
                (Write, flow_ctrl!(START)) => IsduHandlerEvent::IsduStart(od_data),

                // ISDUWrite: OD.ind(W, ISDU, FlowCtrl, Data)
                (Write, _) => IsduHandlerEvent::IsduWrite(od_data),

                // ISDURecComplete: If OD.ind(R, ISDU, Start, ...) received
                (Read, flow_ctrl!(START)) => IsduHandlerEvent::IsduRecComplete(od_data),

                // ISDURead: OD.ind(R, ISDU, Start or FlowCtrl, ...)
                (types::RwDirection::Read, addr_ctrl)
                    if addr_ctrl == flow_ctrl!(START) || addr_ctrl <= 0x0Fu8 =>
                {
                    IsduHandlerEvent::IsduRead(od_data)
                }

                // ISDUSendComplete: If OD.ind(R, ISDU, IDLE, ...) received
                (types::RwDirection::Read, addr_ctrl)
                    if addr_ctrl == flow_ctrl!(IDLE_1) || addr_ctrl == flow_ctrl!(IDLE_2) =>
                {
                    IsduHandlerEvent::IsduSendComplete(od_data)
                }

                // ISDUAbort: OD.ind(R/W, ISDU, Abort, ...)
                (_, flow_ctrl!(ABORT)) => IsduHandlerEvent::IsduAbort(od_data),

                // ISDUError: If ISDU structure is incorrect or FlowCTRL error detected
                _ => {
                    // Check for structure errors
                    if length > 32 || data.len() != length as usize {
                        IsduHandlerEvent::IsduError
                    } else {
                        return Err(IoLinkError::InvalidParameter);
                    }
                }
            }
        } else {
            return Err(IoLinkError::InvalidEvent);
        };

        self.process_event(event)?;
        Ok(())
    }
}

impl Default for IsduHandler {
    fn default() -> Self {
        Self::new()
    }
}
