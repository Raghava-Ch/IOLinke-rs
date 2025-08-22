//! ISDU (Index Service Data Unit) Handler
//!
//! This module implements the ISDU Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 8.4.3
use crate::{
    al::{self},
    dl::{self, od_handler},
    isdu_read_request_index_code, isdu_read_request_index_index_subindex_code,
    isdu_read_request_index_subindex_code, isdu_write_request_index_code,
    isdu_write_request_index_index_subindex_code, isdu_write_request_index_subindex_code,
    log_state_transition, log_state_transition_error,
    types::{self, IoLinkError, IoLinkResult},
    utils::{self, frame_fromat::isdu::Isdu},
};
use heapless::Vec;
use iolinke_macros::flow_ctrl;
use crate::utils::frame_fromat::isdu::MAX_ISDU_LENGTH;

pub trait DlIsduAbort {
    /// See 7.3.6.5 DL_ISDUAbort
    fn dl_isdu_abort(&mut self) -> IoLinkResult<()>;
}

pub trait DlIsduTransportInd {
    /// See 7.2.1.6 DL_ISDUTransport
    /// The DL_ISDUTransport service is used to transport an ISDU. This service is used by the
    /// Master to send a service request from the Master application layer to the Device. It is used by
    /// the Device to send a service response to the Master from the Device application layer. The
    /// parameters of the service primitives are listed in Table 21.
    fn dl_isdu_transport_ind(&mut self, isdu: Isdu) -> IoLinkResult<()>;
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
enum IsduHandlerEvent {
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
    message_buffer: Vec<u8, MAX_ISDU_LENGTH>,
    od_ind_data_length: u8,
}

impl IsduHandler {
    /// Create a new ISDU Handler
    pub fn new() -> Self {
        Self {
            state: IsduHandlerState::Idle,
            exec_transition: Transition::Tn,
            message_buffer: Vec::new(),
            od_ind_data_length: 0,
        }
    }

    /// Process an event
    fn process_event(&mut self, event: IsduHandlerEvent) -> IoLinkResult<()> {
        use IsduHandlerEvent as Event;
        use IsduHandlerState as State;
        let (new_transition, new_state) = match (self.state, event) {
            (State::Inactive, Event::IhConfActive) => (Transition::T1, State::Idle),
            (State::Idle, Event::IsduStart) => (Transition::T2, State::ISDURequest),
            (State::Idle, Event::IhConfInactive) => (Transition::T12, State::Inactive),
            (State::Idle, Event::IsduRead) => (Transition::T14, State::Idle),
            (State::ISDURequest, Event::IsduWrite) => (Transition::T3, State::ISDURequest),
            (State::ISDURequest, Event::IsduRecComplete) => (Transition::T4, State::ISDUWait),
            (State::ISDURequest, Event::IsduError) => (Transition::T13, State::Idle),
            (State::ISDURequest, Event::IsduAbort) => (Transition::T9, State::Idle),
            (State::ISDUWait, Event::IsduRead) => (Transition::T5, State::ISDUWait),
            (State::ISDUWait, Event::IsduRespStart) => (Transition::T6, State::ISDUResponse),
            (State::ISDUWait, Event::IsduAbort) => (Transition::T10, State::Idle),
            (State::ISDUWait, Event::IsduError) => (Transition::T15, State::Idle),
            (State::ISDUResponse, Event::IsduRead) => (Transition::T7, State::ISDUResponse),
            (State::ISDUResponse, Event::IsduSendComplete) => (Transition::T8, State::Idle),
            (State::ISDUResponse, Event::IsduAbort) => (Transition::T11, State::Idle),
            (State::ISDUResponse, Event::IsduError) => (Transition::T16, State::Idle),
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
    /// Poll the ISDU handler
    /// See IO-Link v1.1.4 Section 8.4.3
    pub fn poll(
        &mut self,
        message_handler: &mut dl::message_handler::MessageHandler,
        application_layer: &mut al::ApplicationLayer,
    ) -> IoLinkResult<()> {
        match self.exec_transition {
            Transition::Tn => {
                // No transition to execute
            }
            Transition::T1 => {
                // State: Inactive (0) -> Idle (1)
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t1();
            }
            Transition::T2 => {
                // State: Idle (1) -> ISDURequest (2)
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t2();
            }
            Transition::T3 => {
                // State: ISDURequest (2) -> ISDURequest (2)
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t3();
            }
            Transition::T4 => {
                self.exec_transition = Transition::Tn;
                // State: ISDURequest (2) -> ISDUWait (3)
                let _ = self.execute_t4(application_layer);
            }
            Transition::T5 => {
                // State: ISDUWait (3) -> ISDUWait (3)
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t5(message_handler);
            }
            Transition::T6 => {
                // State: ISDUWait (3) -> ISDUResponse (4)
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t6();
            }
            Transition::T7 => {
                // State: ISDUResponse (4) -> ISDUResponse (4)
                self.exec_transition = Transition::Tn;
                let data_length = self.od_ind_data_length;
                let _ = self.execute_t7(data_length, message_handler);
            }
            Transition::T8 => {
                // State: ISDUResponse (4) -> Idle (1)
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t8();
            }
            Transition::T9 => {
                // State: ISDURequest (2) -> Idle (1)
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t9();
            }
            Transition::T10 => {
                // State: ISDUWait (3) -> Idle (1)
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t10(application_layer);
            }
            Transition::T11 => {
                // State: ISDUResponse (4) -> Idle (1)
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t11(application_layer);
            }
            Transition::T12 => {
                // State: Idle (1) -> Inactive (0)
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t12();
            }
            Transition::T13 => {
                // State: ISDURequest (2) -> Idle (1)
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t13(application_layer);
            }
            Transition::T14 => {
                // State: Idle (1) -> Idle (1)
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t14(message_handler);
            }
            Transition::T15 => {
                // State: ISDUWait (3) -> Idle (1)
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t15(application_layer);
            }
            Transition::T16 => {
                // State: ISDUResponse (4) -> Idle (1)
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t16(application_layer);
            }
        }
        Ok(())
    }

    /// Execute transition T1: Inactive (0) -> Idle (1)
    /// Action: -
    fn execute_t1(&self) -> IoLinkResult<()> {
        Ok(())
    }

    /// Execute transition T2: Idle (1) -> ISDURequest (2)
    /// Action: Start receiving of ISDU request data
    fn execute_t2(&self) -> IoLinkResult<()> {
        // Hanled in od_ind function
        Ok(())
    }

    /// Execute transition T3: ISDURequest (2) -> ISDURequest (2)
    /// Action: Receive ISDU request data
    fn execute_t3(&self) -> IoLinkResult<()> {
        // Hanled in od_ind function

        Ok(())
    }

    /// Execute transition T4: ISDURequest (2) -> ISDUWait (3)
    /// Action: Invoke DL_ISDUTransport.ind to AL (see 7.2.1.6)
    fn execute_t4(&mut self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
        // self.invoke_dl_isdu_transport_ind(od_ind_data)
        let (i_service, index, sub_index) =
            match utils::frame_fromat::isdu::parse_isdu_read_request(&self.message_buffer) {
                Ok(result) => result,
                Err(_) => {
                    self.process_event(IsduHandlerEvent::IsduError)?;
                    return Err(IoLinkError::InvalidData);
                }
            };
        let i_service = i_service.i_service();
        let is_write = if i_service == isdu_read_request_index_code!()
            || i_service == isdu_read_request_index_subindex_code!()
            || i_service == isdu_read_request_index_index_subindex_code!()
        {
            false
        } else if i_service == isdu_write_request_index_code!()
            || i_service == isdu_write_request_index_subindex_code!()
            || i_service == isdu_write_request_index_index_subindex_code!()
        {
            true
        } else {
            self.process_event(IsduHandlerEvent::IsduError)?;
            return Err(IoLinkError::InvalidData);
        };

        application_layer.dl_isdu_transport_ind(Isdu {
            index,
            sub_index,
            data: Vec::new(),
            direction: is_write,
        })
    }

    /// Execute transition T5: ISDUWait (3) -> ISDUWait (3)
    /// Action: Invoke OD.rsp with "busy" indication (see Table A.14)
    fn execute_t5(
        &self,
        message_handler: &mut dl::message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        let isdu_busy = utils::frame_fromat::isdu::compile_isdu_busy_failure_response()?;
        message_handler.od_rsp(isdu_busy.len() as u8, &isdu_busy)
    }

    /// Execute transition T6: ISDUWait (3) -> ISDUResponse (4)
    /// Action: -
    fn execute_t6(&self) -> IoLinkResult<()> {
        Ok(())
    }

    /// Execute transition T7: ISDUResponse (4) -> ISDUResponse (4)
    /// Action: Invoke OD.rsp with ISDU response data
    fn execute_t7(
        &mut self,
        data_length: u8,
        message_handler: &mut dl::message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        // Remove od_ind_data.length bytes from the start of message_buffer and add them to isdu_response
        let mut isdu_response: [u8; MAX_ISDU_LENGTH] = [0; MAX_ISDU_LENGTH];
        for i in 0..data_length as usize {
            if let Some(byte) = self.message_buffer.get(0).cloned() {
                isdu_response[i] = byte;
                self.message_buffer.remove(0);
            } else {
                break;
            }
        }
        let _ = message_handler.od_rsp(0 as u8, &isdu_response);

        Ok(())
    }

    /// Execute transition T8: ISDUResponse (4) -> Idle (1)
    /// Action: -
    fn execute_t8(&self) -> IoLinkResult<()> {
        Ok(())
    }

    /// Execute transition T9: ISDURequest (2) -> Idle (1)
    /// Action: -
    fn execute_t9(&self) -> IoLinkResult<()> {
        Ok(())
    }

    /// Execute transition T10: ISDUWait (3) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    fn execute_t10(&self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
        application_layer.dl_isdu_abort()
    }

    /// Execute transition T11: ISDUResponse (4) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    fn execute_t11(&self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
        application_layer.dl_isdu_abort()
    }

    /// Execute transition T12: Idle (1) -> Inactive (0)
    /// Action: -
    fn execute_t12(&self) -> IoLinkResult<()> {
        Ok(())
    }

    /// Execute transition T13: ISDURequest (2) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    fn execute_t13(&self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
        application_layer.dl_isdu_abort()
    }

    /// Execute transition T14: Idle (1) -> Idle (1)
    /// Action: Invoke OD.rsp with "no service" indication (see Table A.12 and Table A.14)
    fn execute_t14(
        &self,
        message_handler: &mut dl::message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        let isdu_no_service = utils::frame_fromat::isdu::compile_isdu_no_service_response()?;
        message_handler.od_rsp(isdu_no_service.len() as u8, &isdu_no_service)
    }

    /// Execute transition T15: ISDUWait (3) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    fn execute_t15(&self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
        application_layer.dl_isdu_abort()
    }

    /// Execute transition T16: ISDUResponse (4) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    fn execute_t16(&mut self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
        application_layer.dl_isdu_abort()
    }

    pub fn dl_isdu_transport_read_rsp(
        &mut self,
        length: u8,
        data: &[u8],
        message_handler: &mut dl::message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        let _ = utils::frame_fromat::isdu::compile_isdu_read_success_response(
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
        let _ = utils::frame_fromat::isdu::compile_isdu_write_success_response(
            &mut self.message_buffer,
        );
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
        let _ = self.process_event(IsduHandlerEvent::IsduRespStart);
        message_handler.od_rsp(self.message_buffer.len() as u8, &self.message_buffer)
    }

    pub fn dl_isdu_transport_write_error_rsp(
        &mut self,
        error: u8,
        additional_error: u8,
        message_handler: &mut dl::message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        let _ = utils::frame_fromat::isdu::compile_isdu_write_failure_response(
            error,
            additional_error,
            &mut self.message_buffer,
        );
        let _ = self.process_event(IsduHandlerEvent::IsduRespStart);
        message_handler.od_rsp(self.message_buffer.len() as u8, &self.message_buffer)
    }

    /// Handle ISDU configuration changes
    /// See 7.3.6.4 State machine of the Device ISDU handler
    pub fn ih_conf(&mut self, state: types::IhConfState) -> IoLinkResult<()> {
        let _ = match state {
            types::IhConfState::Active => self.process_event(IsduHandlerEvent::IhConfActive),
            types::IhConfState::Inactive => self.process_event(IsduHandlerEvent::IhConfInactive),
        };
        Ok(())
    }
}

impl dl::od_handler::OdInd for IsduHandler {
    fn od_ind(&mut self, od_ind_data: &od_handler::OdIndData) -> IoLinkResult<()> {
        use types::RwDirection::Read;
        use types::RwDirection::Write;
        // Process the ISDU request
        let event = if od_ind_data.com_channel == types::ComChannel::Isdu {
            // Determine event based on the OD.ind parameters
            match (od_ind_data.rw_direction, od_ind_data.address_ctrl) {
                // ISDUStart: OD.ind(W, ISDU, Start, Data)
                (Write, flow_ctrl!(START)) => {
                    self.message_buffer.clear();
                    self.message_buffer
                        .extend_from_slice(&od_ind_data.data)
                        .map_err(|_| IoLinkError::IsduVolatileMemoryFull)?;
                    IsduHandlerEvent::IsduStart
                }

                // ISDUWrite: OD.ind(W, ISDU, FlowCtrl, Data)
                (Write, _) => {
                    let isdu_data = od_ind_data.data.clone();
                    if isdu_data.len() + self.message_buffer.len() > 238 {
                        return Err(IoLinkError::InvalidLength);
                    }
                    self.message_buffer.extend(isdu_data);
                    IsduHandlerEvent::IsduWrite
                }

                // ISDURecComplete: If OD.ind(R, ISDU, Start, ...) received
                (Read, flow_ctrl!(START)) => {
                    if od_ind_data.data.len() == 0 {
                        IsduHandlerEvent::IsduRecComplete
                    } else {
                        IsduHandlerEvent::IsduError
                    }
                }

                // ISDURead: OD.ind(R, ISDU, Start or FlowCtrl, ...)
                (Read, addr_ctrl) if addr_ctrl == flow_ctrl!(START) || addr_ctrl <= 0x0Fu8 => {
                    self.od_ind_data_length = od_ind_data.length;
                    IsduHandlerEvent::IsduRead
                }

                // ISDUSendComplete: If OD.ind(R, ISDU, IDLE, ...) received
                (Read, addr_ctrl)
                    if addr_ctrl == flow_ctrl!(IDLE_1) || addr_ctrl == flow_ctrl!(IDLE_2) =>
                {
                    IsduHandlerEvent::IsduSendComplete
                }

                // ISDUAbort: OD.ind(R/W, ISDU, Abort, ...)
                (_, flow_ctrl!(ABORT)) => IsduHandlerEvent::IsduAbort,

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
