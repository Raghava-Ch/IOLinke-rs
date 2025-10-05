//! ISDU (Index Service Data Unit) Handler
//!
//! This module implements the ISDU Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 8.4.3
use heapless::Vec;
use iolinke_derived_config::device as derived_config;
use iolinke_types::frame::isdu::IsduFlowCtrl;
use iolinke_types::handlers::isdu::DlIsduAbort;
use iolinke_types::handlers::isdu::DlIsduTransportInd;
use iolinke_types::handlers::isdu::MAX_ISDU_LENGTH;
use iolinke_types::{
    custom::{IoLinkError, IoLinkResult},
    handlers,
};
use iolinke_util::frame_fromat::isdu::RxIsduMessageBuffer;
use iolinke_util::frame_fromat::isdu::TxIsduMessageBuffer;
use iolinke_util::{log_state_transition, log_state_transition_error};

use core::default::Default;
use core::option::Option::Some;
use core::result::Result::{Err, Ok};

use crate::services;
use crate::{al, dl::message_handler};

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
#[derive(Clone, Debug, PartialEq, Eq)]
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
    T6(u8, u8), // (Segment number , Number of bytes) to read
    /// T7: State: ISDUResponse (4) -> ISDUResponse (4)
    /// Action: Invoke OD.rsp with ISDU response data
    T7(u8, u8), // (Segment number , Number of bytes) to read
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
    IsduRead(u8, u8), // (Segment number , Number of bytes) to read
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
    IsduRespStart(u8, u8), // (Segment number, Number of bytes)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MessageBuffer {
    tx_buffer: TxIsduMessageBuffer,
    rx_buffer: RxIsduMessageBuffer,
}

/// ISDU Handler implementation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IsduHandler {
    state: IsduHandlerState,
    exec_transition: Transition,
    message_buffer: MessageBuffer,
    expected_segment: u8,
}

impl IsduHandler {
    /// Create a new ISDU Handler
    pub fn new() -> Self {
        Self {
            state: IsduHandlerState::Inactive,
            exec_transition: Transition::Tn,
            message_buffer: MessageBuffer {
                tx_buffer: TxIsduMessageBuffer::new(),
                rx_buffer: RxIsduMessageBuffer::new(),
            },
            expected_segment: 0,
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
            (State::Idle, Event::IsduRead(_, _)) => (Transition::T14, State::Idle),
            (State::ISDURequest, Event::IsduWrite) => (Transition::T3, State::ISDURequest),
            (State::ISDURequest, Event::IsduRecComplete) => (Transition::T4, State::ISDUWait),
            (State::ISDURequest, Event::IsduError) => (Transition::T13, State::Idle),
            (State::ISDURequest, Event::IsduAbort) => (Transition::T9, State::Idle),
            (State::ISDUWait, Event::IsduRead(_, _)) => (Transition::T5, State::ISDUWait),
            (State::ISDUWait, Event::IsduRespStart(segment, bytes)) => {
                (Transition::T6(segment, bytes), State::ISDUResponse)
            }
            (State::ISDUWait, Event::IsduAbort) => (Transition::T10, State::Idle),
            (State::ISDUWait, Event::IsduError) => (Transition::T15, State::Idle),
            (State::ISDUResponse, Event::IsduRead(segment, bytes)) => {
                (Transition::T7(segment, bytes), State::ISDUResponse)
            }
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
    pub fn poll<
        ALS: services::ApplicationLayerServicesInd
            + handlers::sm::SystemManagementCnf
            + services::AlEventCnf,
    >(
        &mut self,
        message_handler: &mut message_handler::MessageHandler,
        application_layer: &mut al::ApplicationLayer<ALS>,
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
                let _ = self.execute_t2(message_handler);
            }
            Transition::T3 => {
                // State: ISDURequest (2) -> ISDURequest (2)
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t3(message_handler);
            }
            Transition::T4 => {
                self.exec_transition = Transition::Tn;
                // State: ISDURequest (2) -> ISDUWait (3)
                let _ = self.execute_t4(application_layer, message_handler);
            }
            Transition::T5 => {
                // State: ISDUWait (3) -> ISDUWait (3)
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t5(message_handler);
            }
            Transition::T6(segment, bytes) => {
                // State: ISDUWait (3) -> ISDUResponse (4)
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t6(segment, bytes, message_handler);
            }
            Transition::T7(segment, bytes) => {
                // State: ISDUResponse (4) -> ISDUResponse (4)
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t7(segment, bytes, message_handler);
            }
            Transition::T8 => {
                // State: ISDUResponse (4) -> Idle (1)
                self.exec_transition = Transition::Tn;
                let _ = self.execute_t8(message_handler);
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
    fn execute_t1(&mut self) -> IoLinkResult<()> {
        self.expected_segment = 0;
        self.message_buffer.tx_buffer.clear();
        Ok(())
    }

    /// Execute transition T2: Idle (1) -> ISDURequest (2)
    /// Action: Start receiving of ISDU request data
    fn execute_t2(
        &mut self,
        message_handler: &mut message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        self.expected_segment = 0;
        self.message_buffer.tx_buffer.clear();
        // Hanled in od_ind function
        let _ = message_handler.od_rsp(0, &[]);
        Ok(())
    }

    /// Execute transition T3: ISDURequest (2) -> ISDURequest (2)
    /// Action: Receive ISDU request data
    fn execute_t3(
        &mut self,
        message_handler: &mut message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        // Hanled in od_ind function
        let _ = message_handler.od_rsp(0, &[]);
        Ok(())
    }

    /// Execute transition T4: ISDURequest (2) -> ISDUWait (3)
    /// Action: Invoke DL_ISDUTransport.ind to AL (see 7.2.1.6)
    fn execute_t4<
        ALS: services::ApplicationLayerServicesInd
            + handlers::sm::SystemManagementCnf
            + services::AlEventCnf,
    >(
        &mut self,
        application_layer: &mut al::ApplicationLayer<ALS>,
        message_handler: &mut message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        use iolinke_types::frame::isdu::IsduIServiceCode;
        use iolinke_types::frame::msequence::RwDirection;
        use iolinke_types::handlers::isdu::IsduMessage;

        self.expected_segment = 0;
        self.message_buffer.tx_buffer.compile_isdu_busy_response();
        let isdu_busy = self.message_buffer.tx_buffer.get_as_slice();
        let _ = message_handler.od_rsp(isdu_busy.len() as u8, &isdu_busy);
        // self.invoke_dl_isdu_transport_ind(od_ind_data)
        let (i_service, index, sub_index, isdu_data) =
            match &self.message_buffer.rx_buffer.extract_isdu_data() {
                Ok(result) => *result,
                Err(_) => {
                    self.process_event(IsduHandlerEvent::IsduError)?;
                    return Err(IoLinkError::InvalidData);
                }
            };
        let i_service = i_service.i_service();
        let is_write = if i_service == IsduIServiceCode::ReadRequestIndex
            || i_service == IsduIServiceCode::ReadRequestIndexSubindex
            || i_service == IsduIServiceCode::ReadRequestIndexIndexSubindex
        {
            RwDirection::Read
        } else if i_service == IsduIServiceCode::WriteRequestIndex
            || i_service == IsduIServiceCode::WriteRequestIndexSubindex
            || i_service == IsduIServiceCode::WriteRequestIndexIndexSubindex
        {
            RwDirection::Write
        } else {
            self.process_event(IsduHandlerEvent::IsduError)?;
            return Err(IoLinkError::InvalidData);
        };
        let data: Vec<u8, MAX_ISDU_LENGTH> = Vec::from_slice(isdu_data.unwrap_or_default())
            .map_err(|_| IoLinkError::InvalidLength)?;
        application_layer.dl_isdu_transport_ind(IsduMessage {
            index,
            sub_index,
            data,
            direction: is_write,
        })
    }

    /// Execute transition T5: ISDUWait (3) -> ISDUWait (3)
    /// Action: Invoke OD.rsp with "busy" indication (see Table A.14)
    fn execute_t5(
        &mut self,
        message_handler: &mut message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        self.message_buffer.tx_buffer.compile_isdu_busy_response();
        let isdu_busy = self.message_buffer.tx_buffer.get_as_slice();
        message_handler.od_rsp(isdu_busy.len() as u8, &isdu_busy)
    }

    /// Execute transition T6: ISDUWait (3) -> ISDUResponse (4)
    /// Action: -
    fn execute_t6(
        &mut self,
        segment: u8,
        bytes: u8,
        message_handler: &mut message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        // T7 hase same activity as T6, so we can call it here
        self.execute_t7(segment, bytes, message_handler)?;
        Ok(())
    }

    /// Execute transition T7: ISDUResponse (4) -> ISDUResponse (4)
    /// Action: Invoke OD.rsp with ISDU response data
    fn execute_t7(
        &mut self,
        segment: u8,
        data_length: u8,
        message_handler: &mut message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        if segment != self.expected_segment % 16 {
            let _ = self.process_event(IsduHandlerEvent::IsduError);
            return Err(IoLinkError::InvalidIndex);
        }
        const MAX_POSSIBLE_OD_SIZE: u8 = derived_config::on_req_data::max_possible_od_length();
        // Extract ISDU response segment from self.message_buffer.tx_buffer
        let from = (data_length as usize) * (self.expected_segment as usize);
        let to = from + (data_length as usize);
        let buffer_len = self.message_buffer.tx_buffer.len();
        self.expected_segment += 1; // Next expected segment number

        // If 'to' exceeds buffer length, pad with zeros
        let isdu_response = if from < buffer_len {
            if to > buffer_len {
                // Not enough data, pad with zeros
                let mut segment_data: Vec<u8, { MAX_POSSIBLE_OD_SIZE as usize }> =
                    Vec::from_slice(&self.message_buffer.tx_buffer[from..buffer_len])
                        .map_err(|_| IoLinkError::InvalidLength)?;
                segment_data
                    .resize(data_length as usize, 0)
                    .map_err(|_| IoLinkError::InvalidLength)?;
                segment_data
            } else {
                Vec::from_slice(&self.message_buffer.tx_buffer[from..to])
                    .map_err(|_| IoLinkError::InvalidLength)?
            }
        } else {
            // No data left, this should not happen, So create ISDUerror event
            self.process_event(IsduHandlerEvent::IsduError)?;
            return Err(IoLinkError::InvalidEvent);
        };

        let _ = message_handler.od_rsp(isdu_response.len() as u8, &isdu_response);

        Ok(())
    }

    /// Execute transition T8: ISDUResponse (4) -> Idle (1)
    /// Action: -
    fn execute_t8(
        &mut self,
        message_handler: &mut message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        self.expected_segment = 0;
        self.message_buffer
            .tx_buffer
            .compile_isdu_no_service_response();
        let isdu_no_service = self.message_buffer.tx_buffer.get_as_slice();
        message_handler.od_rsp(isdu_no_service.len() as u8, &isdu_no_service)?;
        Ok(())
    }

    /// Execute transition T9: ISDURequest (2) -> Idle (1)
    /// Action: -
    fn execute_t9(&mut self) -> IoLinkResult<()> {
        self.expected_segment = 0;
        self.message_buffer.tx_buffer.clear();
        Ok(())
    }

    /// Execute transition T10: ISDUWait (3) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    fn execute_t10<
        ALS: services::ApplicationLayerServicesInd
            + handlers::sm::SystemManagementCnf
            + services::AlEventCnf,
    >(
        &mut self,
        application_layer: &mut al::ApplicationLayer<ALS>,
    ) -> IoLinkResult<()> {
        self.expected_segment = 0;
        self.message_buffer.tx_buffer.clear();
        application_layer.dl_isdu_abort()
    }

    /// Execute transition T11: ISDUResponse (4) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    fn execute_t11<
        ALS: services::ApplicationLayerServicesInd
            + handlers::sm::SystemManagementCnf
            + services::AlEventCnf,
    >(
        &mut self,
        application_layer: &mut al::ApplicationLayer<ALS>,
    ) -> IoLinkResult<()> {
        self.expected_segment = 0;
        self.message_buffer.tx_buffer.clear();
        application_layer.dl_isdu_abort()
    }

    /// Execute transition T12: Idle (1) -> Inactive (0)
    /// Action: -
    fn execute_t12(&self) -> IoLinkResult<()> {
        Ok(())
    }

    /// Execute transition T13: ISDURequest (2) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    fn execute_t13<
        ALS: services::ApplicationLayerServicesInd
            + handlers::sm::SystemManagementCnf
            + services::AlEventCnf,
    >(
        &mut self,
        application_layer: &mut al::ApplicationLayer<ALS>,
    ) -> IoLinkResult<()> {
        self.expected_segment = 0;
        self.message_buffer.tx_buffer.clear();
        application_layer.dl_isdu_abort()
    }

    /// Execute transition T14: Idle (1) -> Idle (1)
    /// Action: Invoke OD.rsp with "no service" indication (see Table A.12 and Table A.14)
    fn execute_t14(
        &mut self,
        message_handler: &mut message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        self.message_buffer
            .tx_buffer
            .compile_isdu_no_service_response();
        let isdu_no_service = self.message_buffer.tx_buffer.get_as_slice();
        message_handler.od_rsp(isdu_no_service.len() as u8, &isdu_no_service)
    }

    /// Execute transition T15: ISDUWait (3) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    fn execute_t15<
        ALS: services::ApplicationLayerServicesInd
            + handlers::sm::SystemManagementCnf
            + services::AlEventCnf,
    >(
        &mut self,
        application_layer: &mut al::ApplicationLayer<ALS>,
    ) -> IoLinkResult<()> {
        self.expected_segment = 0;
        self.message_buffer.tx_buffer.clear();
        application_layer.dl_isdu_abort()
    }

    /// Execute transition T16: ISDUResponse (4) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    fn execute_t16<
        ALS: services::ApplicationLayerServicesInd
            + handlers::sm::SystemManagementCnf
            + services::AlEventCnf,
    >(
        &mut self,
        application_layer: &mut al::ApplicationLayer<ALS>,
    ) -> IoLinkResult<()> {
        self.expected_segment = 0;
        self.message_buffer.tx_buffer.clear();
        application_layer.dl_isdu_abort()
    }

    pub fn dl_isdu_transport_read_rsp(&mut self, length: u8, data: &[u8]) -> IoLinkResult<()> {
        self.message_buffer.tx_buffer.clear();
        let _ = self
            .message_buffer
            .tx_buffer
            .compile_isdu_read_success_response(length, data);

        Ok(())
    }

    pub fn dl_isdu_transport_write_rsp(&mut self) -> IoLinkResult<()> {
        let _ = self
            .message_buffer
            .tx_buffer
            .compile_isdu_write_success_response();
        Ok(())
    }

    pub fn dl_isdu_transport_read_error_rsp(
        &mut self,
        error: u8,
        additional_error: u8,
    ) -> IoLinkResult<()> {
        let _ = self
            .message_buffer
            .tx_buffer
            .compile_isdu_read_failure_response(error, additional_error);
        Ok(())
    }

    pub fn dl_isdu_transport_write_error_rsp(
        &mut self,
        error: u8,
        additional_error: u8,
    ) -> IoLinkResult<()> {
        let _ = self
            .message_buffer
            .tx_buffer
            .compile_isdu_write_failure_response(error, additional_error);
        Ok(())
    }

    /// Handle ISDU configuration changes
    /// See 7.3.6.4 State machine of the Device ISDU handler
    pub fn ih_conf(&mut self, state: handlers::isdu::IhConfState) -> IoLinkResult<()> {
        use handlers::isdu::IhConfState;
        let _ = match state {
            IhConfState::Active => self.process_event(IsduHandlerEvent::IhConfActive),
            IhConfState::Inactive => self.process_event(IsduHandlerEvent::IhConfInactive),
        };
        Ok(())
    }
}

impl handlers::od::OdInd for IsduHandler {
    fn od_ind(&mut self, od_ind_data: &handlers::od::OdIndData) -> IoLinkResult<()> {
        use iolinke_types::frame::msequence::RwDirection::{Read, Write};

        // Only handle ISDU channel
        if od_ind_data.com_channel != iolinke_types::frame::msequence::ComChannel::Isdu {
            return Err(IoLinkError::InvalidEvent);
        }
        let flow_ctrl = if let Some(flow_ctrl) = IsduFlowCtrl::from_u8(od_ind_data.address_ctrl) {
            flow_ctrl
        } else {
            self.process_event(IsduHandlerEvent::IsduError)?;
            return Err(IoLinkError::InvalidEvent);
        };
        match (od_ind_data.rw_direction, flow_ctrl) {
            // ISDUWrite: OD.ind(W, ISDU, FlowCtrl, Data)
            (Write, IsduFlowCtrl::Count(0x00..=0x0F)) => {
                let isdu_data = &od_ind_data.data;
                if isdu_data.len() + self.message_buffer.rx_buffer.len() > 238 {
                    return Err(IoLinkError::InvalidLength);
                }
                self.message_buffer.rx_buffer.extend(isdu_data);
                return self.process_event(IsduHandlerEvent::IsduWrite);
            }

            // ISDURead: OD.ind(R, ISDU, Start or FlowCtrl, ...)
            (Read, IsduFlowCtrl::Count(0x00..=0x0F)) => {
                if self.state != IsduHandlerState::ISDURequest
                    && self.state != IsduHandlerState::Inactive
                {
                    return self.process_event(IsduHandlerEvent::IsduRead(
                        od_ind_data.address_ctrl,
                        od_ind_data.req_length,
                    ));
                } else {
                    return self.process_event(IsduHandlerEvent::IsduError);
                }
            }

            // ISDURecComplete: If OD.ind(R, ISDU, Start, ...) received
            (Read, IsduFlowCtrl::Start) => {
                if self.state == IsduHandlerState::ISDURequest {
                    return self.process_event(IsduHandlerEvent::IsduRecComplete);
                } else if self.state == IsduHandlerState::ISDUWait {
                    if self.message_buffer.tx_buffer.is_ready() {
                        return self.process_event(IsduHandlerEvent::IsduRespStart(
                            0,
                            od_ind_data.req_length,
                        ));
                    } else {
                        // Isdu response is not ready,
                        // so we need to send a busy isdu response,
                        // So segment number is dont care
                        return self
                            .process_event(IsduHandlerEvent::IsduRead(0, od_ind_data.req_length));
                    }
                }
                return self.process_event(IsduHandlerEvent::IsduError);
            }

            // ISDUStart: OD.ind(W, ISDU, Start, Data)
            (Write, IsduFlowCtrl::Start) => {
                let isdu_data = &od_ind_data.data;
                self.message_buffer.rx_buffer.clear();
                self.message_buffer.rx_buffer.extend(isdu_data);
                return self.process_event(IsduHandlerEvent::IsduStart);
            }

            // ISDUSendComplete: If OD.ind(R, ISDU, IDLE, ...) received
            (Read, IsduFlowCtrl::Idle1) => {
                return self.process_event(IsduHandlerEvent::IsduSendComplete);
            }

            // ISDUAbort: OD.ind(R/W, ISDU, Abort, ...)
            (_, IsduFlowCtrl::Abort) => return self.process_event(IsduHandlerEvent::IsduAbort),

            // ISDUError: If ISDU structure is incorrect or FlowCTRL error detected
            _ => return self.process_event(IsduHandlerEvent::IsduError),
        }
    }
}

impl Default for IsduHandler {
    fn default() -> Self {
        Self::new()
    }
}
