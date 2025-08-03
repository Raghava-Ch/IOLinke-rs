//! ISDU (Index Service Data Unit) Handler
//!
//! This module implements the ISDU Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 8.4.3
use crate::{
    dl::{self, od_handler::OdIndData as IsduIdnData}, storage, types::{self, IoLinkError, IoLinkResult, Isdu}
};
use heapless::Vec;
use iolinke_macros::flow_ctrl;
use modular_bitfield::prelude::*;

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_write_failure_code {
    () => {
        0x4
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_write_success_code {
    () => {
        0x5
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_read_failure_code {
    () => {
        0xC
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_read_success_code {
    () => {
        0xD
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_read_request_index_code {
    () => {
        0x9
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_read_request_index_subindex_code {
    () => {
        0xA
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_read_request_index_index_subindex_code {
    () => {
        0xB
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_write_request_index_code {
    () => {
        0x1
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_write_request_index_subindex_code {
    () => {
        0x2
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_write_request_index_index_subindex_code {
    () => {
        0x3
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_extended_length_code {
    () => {
        0x1
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_busy {
    () => {
        0xD
    };
}

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
struct IsduService {
    /// I-Service octet
    i_service: B4,
    /// Transfer length
    length: B4,
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
        let (i_service, index, subindex, data) = match parse_isdu_write_request(&od_ind_data.data) {
            Ok(result) => result,
            Err(_) => {
                self.process_event(IsduHandlerEvent::IsduError)?;
                return Err(IoLinkError::InvalidData)
            },
        };
        self.add_an_entry(
            index,
            subindex,
            data,
        );
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
    fn execute_t5(&mut self, od_ind_data: &IsduIdnData, message_handler: &mut dl::message_handler::MessageHandler) -> IoLinkResult<()> {
        let isdu_busy = compile_isdu_busy_failure_response()?;
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
        self.invoke_od_rsp_with_response_data(od_ind_data)
    }

    /// Execute transition T8: ISDUResponse (4) -> Idle (1)
    /// Action: -
    fn execute_t8(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        self.cleanup_isdu_transaction(od_ind_data)
    }

    /// Execute transition T9: ISDURequest (2) -> Idle (1)
    /// Action: -
    fn execute_t9(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        self.cleanup_isdu_transaction(od_ind_data)
    }

    /// Execute transition T10: ISDUWait (3) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    fn execute_t10(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        self.invoke_dl_isdu_abort(od_ind_data)
    }

    /// Execute transition T11: ISDUResponse (4) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    fn execute_t11(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        self.invoke_dl_isdu_abort(od_ind_data)
    }

    /// Execute transition T12: Idle (1) -> Inactive (0)
    /// Action: -
    fn execute_t12(&mut self) -> IoLinkResult<()> {
        Ok(())
    }

    /// Execute transition T13: ISDURequest (2) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    fn execute_t13(&mut self) -> IoLinkResult<()> {
        self.invoke_dl_isdu_abort_no_data()
    }

    /// Execute transition T14: Idle (1) -> Idle (1)
    /// Action: Invoke OD.rsp with "no service" indication (see Table A.12 and Table A.14)
    fn execute_t14(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        self.invoke_od_rsp_no_service(od_ind_data)
    }

    /// Execute transition T15: ISDUWait (3) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    fn execute_t15(&mut self) -> IoLinkResult<()> {
        self.invoke_dl_isdu_abort_no_data()
    }

    /// Execute transition T16: ISDUResponse (4) -> Idle (1)
    /// Action: Invoke DL_ISDUAbort
    fn execute_t16(&mut self) -> IoLinkResult<()> {
        self.invoke_dl_isdu_abort_no_data()
    }

    /// Start receiving ISDU request data
    /// Action for transition T2
    fn start_receiving_isdu_request(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        // TODO: Initialize ISDU request reception
        // TODO: Parse ISDU header from od_ind_data
        // TODO: Validate ISDU format and length
        // Clear any previous request
        Ok(())
    }

    /// Receive ISDU request data
    /// Action for transition T3
    fn receive_isdu_request_data(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        // TODO: Continue receiving ISDU request data fragments
        // TODO: Assemble complete ISDU request
        // TODO: Validate data integrity
        Ok(())
    }

    /// Invoke DL_ISDUTransport.ind to Application Layer
    /// Action for transition T4 - see IO-Link v1.1.4 Section 7.2.1.6
    fn invoke_dl_isdu_transport_ind(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        // TODO: Complete ISDU request assembly
        // TODO: Validate complete ISDU structure
        // TODO: Forward ISDU to Application Layer via DL_ISDUTransport.ind
        // TODO: Parse ISDU index, subindex, and data from assembled request
        Ok(())
    }

    /// Invoke OD.rsp with "busy" indication
    /// Action for transition T5 - see IO-Link v1.1.4 Table A.14
    fn invoke_od_rsp_busy(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        // TODO: Send OD.rsp with busy status
        // TODO: Use appropriate error code for busy indication
        // TODO: Maintain current ISDU processing state
        Ok(())
    }

    /// Invoke OD.rsp with ISDU response data
    /// Action for transition T7
    fn invoke_od_rsp_with_response_data(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        // TODO: Send OD.rsp with prepared response data
        // TODO: Handle fragmentation if response data exceeds frame size
        // TODO: Update flow control appropriately
        Ok(())
    }

    /// Invoke DL_ISDUAbort with OD indication data
    /// Action for transitions T10, T11
    fn invoke_dl_isdu_abort(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        // TODO: Send DL_ISDUAbort indication
        // TODO: Clean up current ISDU transaction
        // TODO: Reset internal buffers and state
        // self.current_request = None;
        // self.response_data.clear();
        Ok(())
    }

    /// Invoke DL_ISDUAbort without specific OD data
    /// Action for transitions T13, T15, T16
    fn invoke_dl_isdu_abort_no_data(&mut self) -> IoLinkResult<()> {
        // TODO: Send DL_ISDUAbort indication
        // TODO: Clean up current ISDU transaction
        // TODO: Reset internal buffers and state
        // self.current_request = None;
        // self.response_data.clear();
        Ok(())
    }

    /// Invoke OD.rsp with "no service" indication
    /// Action for transition T14 - see IO-Link v1.1.4 Table A.12 and Table A.14
    fn invoke_od_rsp_no_service(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        // TODO: Send OD.rsp with "no service" error code
        // TODO: Use appropriate error code indicating service not available
        // TODO: Maintain idle state
        Ok(())
    }

    /// Clean up ISDU transaction
    /// Helper function for transitions that return to idle without abort
    fn cleanup_isdu_transaction(&mut self, od_ind_data: &IsduIdnData) -> IoLinkResult<()> {
        // TODO: Clean up completed ISDU transaction
        // TODO: Reset internal state for next transaction
        // self.current_request = None;
        // self.response_data.clear();
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
    
    pub fn add_an_entry(&mut self, index: u16, subindex: u8, data: &[u8]) -> IoLinkResult<()> {
        if data.len() > 238 {
            return Err(IoLinkError::InvalidParameter);
        }
        self.message_buffer.extend_from_slice(data).map_err(|_| IoLinkError::IsduVolatileMemoryFull)?;
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
                _ => {
                    IsduHandlerEvent::IsduError
                }
            }
        } else {
            return Err(IoLinkError::InvalidEvent);
        };

        self.process_event(event)?;
        Ok(())
    }
}

fn compile_isdu_write_success_response(buffer: &mut [u8]) -> IoLinkResult<()> {
    let i_service = IsduService::new()
        .with_i_service(isdu_write_success_code!())
        .with_length(2);
    buffer[0] = i_service.into_bytes()[0];
    buffer[1] = 0;
    let chkpdu = calculate_checksum(2, &buffer[0..2]);
    buffer[1] = chkpdu;
    Ok(())
}

fn compile_isdu_write_failure_response(
    error_code: u8,
    additional_error_code: u8,
    buffer: &mut [u8],
) -> IoLinkResult<()> {
    let i_service = IsduService::new()
        .with_i_service(isdu_write_failure_code!())
        .with_length(3);
    buffer[0] = i_service.into_bytes()[0];
    buffer[1] = error_code;
    buffer[2] = additional_error_code;
    buffer[3] = 0;
    let chkpdu = calculate_checksum(4, &buffer[0..4]);
    buffer[3] = chkpdu;
    Ok(())
}

fn compile_isdu_read_success_response(length: u8, data: &mut [u8], buffer: &mut [u8]) -> IoLinkResult<()> {    
    if (1..=15).contains(&length) { // Valid data length range (excluding length byte and checksum)
        let i_service = IsduService::new()
        .with_i_service(isdu_read_success_code!())
        .with_length(length + 2); // +2 for length byte and checksum
        buffer[0] = i_service.into_bytes()[0];
        buffer[1..1 + length as usize].copy_from_slice(&data[..length as usize]);
        let total_length = 1 + length as usize;
        buffer[total_length] = 0;
        let chkpdu = calculate_checksum(total_length as u8, &buffer[0..total_length]);
        buffer[total_length] = chkpdu;
    } else {
        let i_service = IsduService::new()
            .with_i_service(isdu_read_success_code!())
            .with_length(isdu_extended_length_code!());
        buffer[0] = i_service.into_bytes()[0];
        buffer[1] = 2 + length; // Extended length byte
        buffer[2..2 + length as usize].copy_from_slice(&data[..length as usize]);
        let total_length = 2 + length as usize;
        let chkpdu = calculate_checksum(total_length as u8, &buffer[0..total_length]);
        buffer[total_length] = chkpdu;
    }
    Ok(())
}

fn compile_isdu_read_failure_response(error_code: u8, additional_error_code: u8, buffer: &mut [u8]) {
    let i_service = IsduService::new()
        .with_i_service(isdu_read_failure_code!())
        .with_length(4);
    buffer[0] = i_service.into_bytes()[0];
    buffer[1] = error_code;
    buffer[2] = additional_error_code;
    buffer[3] = 0;
    let chkpdu = calculate_checksum(4, &buffer[0..4]);
    buffer[3] = chkpdu;
}

fn compile_isdu_busy_failure_response() -> IoLinkResult<[u8; 1]> {
    let i_service = IsduService::new().with_length(isdu_busy!());
    let buffer = i_service.into_bytes()[0];
    Ok([buffer])
}

fn parse_isdu_write_request(
    buffer: &[u8],
) -> IoLinkResult<(IsduService, u16, u8, &[u8])> {
    if buffer.len() < 3 {
        return Err(IoLinkError::InvalidParameter);
    }
    if calculate_checksum(buffer.len() as u8, buffer) != 0 {
        // Invalid checksum
        return Err(IoLinkError::ChecksumError);
    }
    let i_service: IsduService = IsduService::from_bytes([buffer[0]]);
    if i_service.i_service() != isdu_write_request_index_code!() {
        parse_write_request_with_index(buffer)
    }
    else if i_service.i_service() == isdu_write_request_index_subindex_code!() {
        parse_write_request_with_index_subindex(buffer)
    } else if i_service.i_service() == isdu_write_request_index_index_subindex_code!() {
        parse_write_request_with_index_index_subindex(buffer)
    } else {
        return Err(IoLinkError::InvalidData);
    }
}

fn parse_isdu_read_request(
    buffer: &[u8],
) -> IoLinkResult<(IsduService, u16, u8)> {
    if buffer.len() < 3 {
        return Err(IoLinkError::InvalidParameter);
    }
    if calculate_checksum(buffer.len() as u8, buffer) != 0 {
        // Invalid checksum
        return Err(IoLinkError::ChecksumError);
    }
    let i_service: IsduService = IsduService::from_bytes([buffer[0]]);
    if i_service.i_service() != isdu_read_request_index_code!() {
        parse_read_request_with_index(buffer)
    }
    else if i_service.i_service() == isdu_read_request_index_subindex_code!() {
        parse_read_request_with_index_subindex(buffer)
    } else if i_service.i_service() == isdu_read_request_index_index_subindex_code!() {
        parse_read_request_with_index_index_subindex(buffer)
    } else {
        return Err(IoLinkError::InvalidParameter);
    }
}

fn parse_read_request_with_index(
    buffer: &[u8],
) -> IoLinkResult<(IsduService, u16, u8)> {
    let i_service: IsduService = IsduService::from_bytes([buffer[0]]);
    if i_service.i_service() != isdu_read_request_index_code!() {
        return Err(IoLinkError::InvalidParameter);
    }
    let index = buffer[1];
    Ok((i_service, index as u16, 0))
}

fn parse_read_request_with_index_subindex(
    buffer: &[u8],
) -> IoLinkResult<(IsduService, u16, u8)> {
    let i_service: IsduService = IsduService::from_bytes([buffer[0]]);
    if i_service.i_service() != isdu_read_request_index_subindex_code!() {
        return Err(IoLinkError::InvalidParameter);
    }
    let index = buffer[1];
    let subindex = buffer[2];
    Ok((i_service, index as u16, subindex))
}

fn parse_read_request_with_index_index_subindex(
    buffer: &[u8],
) -> IoLinkResult<(IsduService, u16, u8)> {
    let i_service: IsduService = IsduService::from_bytes([buffer[0]]);
    if i_service.i_service() != isdu_read_request_index_index_subindex_code!() {
        return Err(IoLinkError::InvalidParameter);
    }
    let index = u16::from_le_bytes([buffer[1], buffer[2]]);
    let subindex = buffer[3];
    Ok((i_service, index, subindex))
}

fn parse_write_request_with_index(
    buffer: &[u8],
) -> IoLinkResult<(IsduService, u16, u8, &[u8])> {
    let i_service: IsduService = IsduService::from_bytes([buffer[0]]);
    if i_service.i_service() != isdu_write_request_index_code!() {
        return Err(IoLinkError::InvalidParameter);
    }
    let length = i_service.length();
    if !(2..=15).contains(&length) {
        return Err(IoLinkError::InvalidData);
    }
    let index = buffer[1];
    Ok((i_service, index as u16, 0, &buffer[2..(3 - length as usize)]))
}

fn parse_write_request_with_index_subindex(
    buffer: &[u8],
) -> IoLinkResult<(IsduService, u16, u8, &[u8])> {
    let i_service: IsduService = IsduService::from_bytes([buffer[0]]);
    if i_service.i_service() != isdu_write_request_index_subindex_code!() {
        return Err(IoLinkError::InvalidParameter);
    }
    let length = i_service.length();
    if !(2..=15).contains(&length) {
        return Err(IoLinkError::InvalidData);
    }
    let index = buffer[1];
    let subindex = buffer[2];
    let data = &buffer[3..(3 + length as usize)];
    Ok((i_service, index as u16, subindex, data))
}

fn parse_write_request_with_index_index_subindex(
    buffer: &[u8],
) -> IoLinkResult<(IsduService, u16, u8, &[u8])> {
    let i_service: IsduService = IsduService::from_bytes([buffer[0]]);
    if i_service.i_service() != isdu_write_request_index_index_subindex_code!() {
        return Err(IoLinkError::InvalidParameter);
    }
    if i_service.length() != 1 {
        return Err(IoLinkError::InvalidData);
    }
    let length = buffer[1];
    if !(17..=238).contains(&length) {
        return Err(IoLinkError::InvalidData);
    }
    let index = u16::from_le_bytes([buffer[2], buffer[3]]);
    let subindex = buffer[4];
    let data = &buffer[5..(5 + length as usize)];
    Ok((i_service, index, subindex, data))
}

fn calculate_checksum(length: u8, data: &[u8]) -> u8 {
    let mut checkpdu = 0;
    for byte in data.iter().take(length as usize) {
        checkpdu ^= byte;
    }
    checkpdu
}

impl Default for IsduHandler {
    fn default() -> Self {
        Self::new()
    }
}
