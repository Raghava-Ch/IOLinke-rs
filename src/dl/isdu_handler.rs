//! ISDU (Index Service Data Unit) Handler
//!
//! This module implements the ISDU Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 8.4.3
use crate::{
    dl::{self, od_handler::OdIndData as IsduIdnData},
    types::{self, IoLinkError, IoLinkResult},
    utils,
};
use heapless::Vec;
use iolinke_macros::flow_ctrl;
use modular_bitfield::prelude::*;

pub trait DlIsduAbort {
    /// See 7.3.6.5 DL_ISDUAbort
    fn isdu_abort(&mut self) -> IoLinkResult<()>;
}

pub trait DlIsduTransportInd {
    /// See 7.2.1.6 DL_ISDUTransport
    /// The DL_ISDUTransport service is used to transport an ISDU. This service is used by the
    /// Master to send a service request from the Master application layer to the Device. It is used by
    /// the Device to send a service response to the Master from the Device application layer. The
    /// parameters of the service primitives are listed in Table 21.
    fn isdu_transport_ind(&mut self, isdu: Isdu) -> IoLinkResult<()>;
}

pub trait DlIsduTransportRsp {
    /// See
    ///
    fn dl_isdu_transport_read_rsp(&mut self, length: u8, data: &[u8]) -> IoLinkResult<()>;
    fn dl_isdu_transport_write_rsp(&mut self) -> IoLinkResult<()>;
    fn dl_isdu_transport_read_error_rsp(
        &mut self,
        error: u8,
        additional_error: u8,
    ) -> IoLinkResult<()>;
    fn dl_isdu_transport_write_error_rsp(
        &mut self,
        error: u8,
        additional_error: u8,
    ) -> IoLinkResult<()>;
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
    T2(IsduIdnData),
    /// T3: State: ISDURequest (2) -> ISDURequest (2)
    /// Action: Receive ISDU request data
    T3(IsduIdnData),
    /// T4: State: ISDURequest (2) -> ISDUWait (3)
    /// Action: Invoke DL_ISDUTransport.ind to AL (see 7.2.1.6)
    T4(IsduIdnData),
    /// T5: State: ISDUWait (3) -> ISDUWait (3)
    /// Action: Invoke OD.rsp with "busy" indication (see Table A.14)
    T5(IsduIdnData),
    /// T6: State: ISDUWait (3) -> ISDUResponse (4)
    /// Action: -
    T6,
    /// T7: State: ISDUResponse (4) -> ISDUResponse (4)
    /// Action: Invoke OD.rsp with ISDU response data
    T7(IsduIdnData),
    /// T8: State: ISDUResponse (4) -> Idle (1)
    /// Action: -
    T8(IsduIdnData),
    /// T9: State: ISDURequest (2) -> Idle (1)
    /// Action: -
    T9(IsduIdnData),
    /// T10: State: ISDUWait (3) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    T10(IsduIdnData),
    /// T11: State: ISDUResponse (4) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    T11(IsduIdnData),
    /// T12: State: Idle (1) -> Inactive (0)
    /// Action: -
    T12,
    /// T13: State: ISDURequest (2) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    T13,
    /// T14: State: Idle (1) -> Idle (1)
    /// Action: Invoke OD.rsp with "no service" indication (see Table A.12 and Table A.14)
    T14(IsduIdnData),
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
    IsduRead(IsduIdnData),
    /// {IH_Conf_ACTIVE}
    IhConfActive,
    /// {ISDUWrite}
    IsduWrite(IsduIdnData),
    /// {ISDUStart}
    IsduStart(IsduIdnData),
    /// {[ISDUError]}
    IsduError,
    /// {IH_Conf_INACTIVE}
    IhConfInactive,
    /// {[ISDUSendComplete]}
    IsduSendComplete(IsduIdnData),
    /// {ISDUAbort}
    IsduAbort(IsduIdnData),
    /// {[ISDURecComplete]}
    IsduRecComplete(IsduIdnData),
    /// {ISDURespStart}
    IsduRespStart,
}

const MAX_ISDU_LENGTH: usize = 238;

/// ISDU (Index Service Data Unit) structure
/// See IO-Link v1.1.4 Section 8.4.3
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Isdu {
    /// Parameter index
    pub index: u16,
    /// Sub-index
    pub sub_index: u8,
    /// Data payload
    pub data: Vec<u8, MAX_ISDU_LENGTH>,
    /// Read/Write operation flag
    pub is_write: bool,
}

/// See A.5.2 I-Service
/// Figure A.16 shows the structure of the I-Service octet.
#[bitfield]
pub struct IsduService {
    /// I-Service octet
    pub i_service: B4,
    /// Transfer length
    pub length: B4,
}

/// ISDU Handler implementation
pub struct IsduHandler {
    state: IsduHandlerState,
    exec_transition: Transition,
    message_buffer: Vec<u8, MAX_ISDU_LENGTH>,
}

impl IsduHandler {
    /// Create a new ISDU Handler
    pub fn new() -> Self {
        Self {
            state: IsduHandlerState::Idle,
            exec_transition: Transition::Tn,
            message_buffer: Vec::new(),
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
                self.exec_transition = Transition::Tn;
                // State: Inactive (0) -> Idle (1)
                self.state = IsduHandlerState::Idle;
            }
            Transition::T2(od_ind_data) => {
                self.exec_transition = Transition::Tn;
                // State: Idle (1) -> ISDURequest (2)
                self.state = IsduHandlerState::ISDURequest;
            }
            Transition::T3(od_ind_data) => {
                self.exec_transition = Transition::Tn;
                // State: ISDURequest (2) -> ISDURequest (2)
                // Continue receiving ISDU request data
            }
            Transition::T4(od_ind_data) => {
                self.exec_transition = Transition::Tn;
                // State: ISDURequest (2) -> ISDUWait (3)
                self.state = IsduHandlerState::ISDUWait;
            }
            Transition::T5(od_ind_data) => {
                self.exec_transition = Transition::Tn;
                // State: ISDUWait (3) -> ISDUWait (3)
                // Invoke OD.rsp with "busy" indication
            }
            Transition::T6 => {
                self.exec_transition = Transition::Tn;
                // State: ISDUWait (3) -> ISDUResponse (4)
                self.state = IsduHandlerState::ISDUResponse;
            }
            Transition::T7(od_ind_data) => {
                self.exec_transition = Transition::Tn;
                // State: ISDUResponse (4) -> ISDUResponse (4)
                // Invoke OD.rsp with ISDU response data
            }
            Transition::T8(od_ind_data) => {
                self.exec_transition = Transition::Tn;
                // State: ISDUResponse (4) -> Idle (1)
                self.state = IsduHandlerState::Idle;
            }
            Transition::T9(od_ind_data) => {
                self.exec_transition = Transition::Tn;
                // State: ISDURequest (2) -> Idle (1)
                self.state = IsduHandlerState::Idle;
            }
            Transition::T10(od_ind_data) => {
                self.exec_transition = Transition::Tn;
                // State: ISDUWait (3) -> Idle (1)
                self.state = IsduHandlerState::Idle;
            }
            Transition::T11(od_ind_data) => {
                self.exec_transition = Transition::Tn;
                // State: ISDUResponse (4) -> Idle (1)
                self.state = IsduHandlerState::Idle;
            }
            Transition::T12 => {
                self.exec_transition = Transition::Tn;
                // State: Idle (1) -> Inactive (0)
                self.state = IsduHandlerState::Inactive;
            }
            Transition::T13 => {
                self.exec_transition = Transition::Tn;
                // State: ISDURequest (2) -> Idle (1)
                self.state = IsduHandlerState::Idle;
            }
            Transition::T14(od_ind_data) => {
                self.exec_transition = Transition::Tn;
                // State: Idle (1) -> Idle (1)
                // Invoke OD.rsp with "no service" indication
            }
            Transition::T15 => {
                self.exec_transition = Transition::Tn;
                // State: ISDUWait (3) -> Idle (1)
                self.state = IsduHandlerState::Idle;
            }
            Transition::T16 => {
                self.exec_transition = Transition::Tn;
                // State: ISDUResponse (4) -> Idle (1)
                self.state = IsduHandlerState::Idle;
            }
        }
        Ok(())
    }

    /// Execute transition T1: Inactive (0) -> Idle (1)
    /// Action: -
    fn execute_t1(&mut self) -> IoLinkResult<()> {
        Ok(())
    }

    /// Execute transition T2: Idle (1) -> ISDURequest (2)
    /// Action: Start receiving of ISDU request data
    fn execute_t2(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        Ok(())
    }

    /// Execute transition T3: ISDURequest (2) -> ISDURequest (2)
    /// Action: Receive ISDU request data
    fn execute_t3(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        let (i_service, index, subindex, data) =
            match utils::frame_fromat::isdu::parse_isdu_write_request(&od_ind_data.data) {
                Ok(result) => result,
                Err(_) => {
                    self.process_event(IsduHandlerEvent::IsduError)?;
                    return Err(IoLinkError::InvalidData);
                }
            };
        self.add_an_entry(index, subindex, data);
        todo!()
    }

    /// Execute transition T4: ISDURequest (2) -> ISDUWait (3)
    /// Action: Invoke DL_ISDUTransport.ind to AL (see 7.2.1.6)
    fn execute_t4(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        // self.invoke_dl_isdu_transport_ind(od_ind_data)
        todo!()
    }

    /// Execute transition T5: ISDUWait (3) -> ISDUWait (3)
    /// Action: Invoke OD.rsp with "busy" indication (see Table A.14)
    fn execute_t5(
        &mut self,
        od_ind_data: &IsduIdnData,
        message_handler: &mut dl::message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        let isdu_busy = utils::frame_fromat::isdu::compile_isdu_busy_failure_response()?;
        message_handler.od_rsp(isdu_busy.len() as u8, &isdu_busy)
    }

    /// Execute transition T6: ISDUWait (3) -> ISDUResponse (4)
    /// Action: -
    fn execute_t6(&mut self) -> IoLinkResult<()> {
        Ok(())
    }

    /// Execute transition T7: ISDUResponse (4) -> ISDUResponse (4)
    /// Action: Invoke OD.rsp with ISDU response data
    fn execute_t7(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        todo!()
    }

    /// Execute transition T8: ISDUResponse (4) -> Idle (1)
    /// Action: -
    fn execute_t8(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        todo!()
    }

    /// Execute transition T9: ISDURequest (2) -> Idle (1)
    /// Action: -
    fn execute_t9(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        todo!()
    }

    /// Execute transition T10: ISDUWait (3) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    fn execute_t10(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        todo!()
    }

    /// Execute transition T11: ISDUResponse (4) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    fn execute_t11(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        todo!()
    }

    /// Execute transition T12: Idle (1) -> Inactive (0)
    /// Action: -
    fn execute_t12(&mut self) -> IoLinkResult<()> {
        Ok(())
    }

    /// Execute transition T13: ISDURequest (2) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    fn execute_t13(&mut self) -> IoLinkResult<()> {
        todo!()
    }

    /// Execute transition T14: Idle (1) -> Idle (1)
    /// Action: Invoke OD.rsp with "no service" indication (see Table A.12 and Table A.14)
    fn execute_t14(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        todo!()
    }

    /// Execute transition T15: ISDUWait (3) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    fn execute_t15(&mut self) -> IoLinkResult<()> {
        todo!()
    }

    /// Execute transition T16: ISDUResponse (4) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    fn execute_t16(&mut self) -> IoLinkResult<()> {
        todo!()
    }

    pub fn dl_isdu_transport_read_rsp(
        &mut self,
        length: u8,
        data: &[u8],
        message_handler: &mut dl::message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        utils::frame_fromat::isdu::compile_isdu_read_success_response(
            length,
            data,
            &mut self.message_buffer,
        );
        message_handler.od_rsp(self.message_buffer.len() as u8, &self.message_buffer)
    }

    pub fn dl_isdu_transport_write_rsp(
        &mut self,
        message_handler: &mut dl::message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        utils::frame_fromat::isdu::compile_isdu_write_success_response(&mut self.message_buffer);
        message_handler.od_rsp(self.message_buffer.len() as u8, &self.message_buffer)
    }

    pub fn dl_isdu_transport_read_error_rsp(
        &mut self,
        error: u8,
        additional_error: u8,
        message_handler: &mut dl::message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        utils::frame_fromat::isdu::compile_isdu_read_failure_response(
            error,
            additional_error,
            &mut self.message_buffer,
        );
        message_handler.od_rsp(self.message_buffer.len() as u8, &self.message_buffer)
    }

    pub fn dl_isdu_transport_write_error_rsp(
        &mut self,
        error: u8,
        additional_error: u8,
        message_handler: &mut dl::message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        utils::frame_fromat::isdu::compile_isdu_write_failure_response(
            error,
            additional_error,
            &mut self.message_buffer,
        );
        message_handler.od_rsp(self.message_buffer.len() as u8, &self.message_buffer)
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

    pub fn add_an_entry(&mut self, index: u16, subindex: u8, data: &[u8]) -> IoLinkResult<()> {
        if data.len() > 238 {
            return Err(IoLinkError::InvalidParameter);
        }
        self.message_buffer
            .extend_from_slice(data)
            .map_err(|_| IoLinkError::IsduVolatileMemoryFull)?;
        Ok(())
    }
}

impl dl::od_handler::OdInd for IsduHandler {
    fn od_ind(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        use types::RwDirection::Read;
        use types::RwDirection::Write;
        // Process the ISDU request
        let event = if od_ind_data.com_channel == types::ComChannel::Isdu {
            // Determine event based on the OD.ind parameters
            match (od_ind_data.rw_direction, od_ind_data.address_ctrl) {
                // ISDUStart: OD.ind(W, ISDU, Start, Data)
                (Write, flow_ctrl!(START)) => IsduHandlerEvent::IsduStart(od_ind_data.clone()),

                // ISDUWrite: OD.ind(W, ISDU, FlowCtrl, Data)
                (Write, _) => IsduHandlerEvent::IsduWrite(od_ind_data.clone()),

                // ISDURecComplete: If OD.ind(R, ISDU, Start, ...) received
                (Read, flow_ctrl!(START)) => IsduHandlerEvent::IsduRecComplete(od_ind_data.clone()),

                // ISDURead: OD.ind(R, ISDU, Start or FlowCtrl, ...)
                (types::RwDirection::Read, addr_ctrl)
                    if addr_ctrl == flow_ctrl!(START) || addr_ctrl <= 0x0Fu8 =>
                {
                    IsduHandlerEvent::IsduRead(od_ind_data.clone())
                }

                // ISDUSendComplete: If OD.ind(R, ISDU, IDLE, ...) received
                (types::RwDirection::Read, addr_ctrl)
                    if addr_ctrl == flow_ctrl!(IDLE_1) || addr_ctrl == flow_ctrl!(IDLE_2) =>
                {
                    IsduHandlerEvent::IsduSendComplete(od_ind_data.clone())
                }

                // ISDUAbort: OD.ind(R/W, ISDU, Abort, ...)
                (_, flow_ctrl!(ABORT)) => IsduHandlerEvent::IsduAbort(od_ind_data.clone()),

                // ISDUError: If ISDU structure is incorrect or FlowCTRL error detected
                _ => IsduHandlerEvent::IsduError,
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
