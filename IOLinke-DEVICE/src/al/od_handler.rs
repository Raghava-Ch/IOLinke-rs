//! On-request Data Handler for IO-Link Device Application Layer
//!
//! This module implements the state machine and event handling logic for the On-request Data (OD) Handler
//! as specified in IO-Link standard section 8.3.2.2. It manages transitions between OD states in response
//! to events from the Data Link Layer (DL) and Application Layer (AL), including parameter read/write and
//! ISDU transport operations.
//!
//! # Features
//! - Implements the OD state machine with all defined states and transitions (see Table 75 of the IO-Link specification).
//! - Handles AL and DL events, including parameter read/write indications, ISDU transport, and abort scenarios.
//! - Provides transition execution logic for AL_Read, AL_Write, DL_ReadParam, DL_WriteParam, and ISDU transport services.
//! - Ensures correct state transitions and error handling according to the IO-Link protocol.
//!
//! # Usage
//! The main entry point is [`OnRequestDataHandler`], which should be integrated with the device's
//! parameter manager and data link layer. Events are processed via trait implementations, and the
//! state machine is advanced by calling [`OnRequestDataHandler::poll`].
//!
//! # Example
//! ```rust
//! let mut handler = OnRequestDataHandler::new();
//! handler.dl_write_param_ind(index, data)?;
//! handler.poll(&mut parameter_manager, &mut data_link_layer)?;
//! ```
//!
//! # References
//! - IO-Link Specification, Section 8.3.2.2 (OD state machine)
//! - Table 75 – States and transitions for the OD state machine of the Device AL

use iolinke_types::custom::{IoLinkError, IoLinkResult};
use iolinke_types::handlers;
use iolinke_types::handlers::isdu::DlIsduTransportRsp;
use iolinke_types::handlers::od::DlParamRsp;

use core::default::Default;
use core::result::Result::{Err, Ok};

use crate::al::ApplicationLayerReadWriteInd;
use crate::{
    al::{parameter_manager, services},
    dl,
};

/// See 8.3.2.2 OD state machine of the Device AL
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OnRequestDataHandlerState {
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
#[derive(Clone, Debug, PartialEq, Eq)]
enum Transition {
    /// Tn: No transition
    Tn,
    /// T1: State: Idle (0) -> AwaitAlWriteRsp (1)
    /// Action: Invoke AL_Write
    T1(u8, u8), // (index, data)
    /// T2: State: AwaitAlWriteRsp (1) -> Idle (0)
    /// Action: Invoke DL_WriteParam (16 to 31)
    T2,
    /// T3: State: Idle (0) -> AwaitAlReadRsp (2)
    /// Action: Invoke AL_Read
    T3(u8), // (address)
    /// T4: State: AwaitAlReadRsp (2) -> Idle (0)
    /// Action: Invoke DL_ReadParam (0 to 31)
    T4(u8, [u8; handlers::isdu::MAX_ISDU_LENGTH]), // (length, data)
    /// T5: State: Idle (0) -> AwaitAlRwRsp (3)
    /// Action: Invoke AL_Read
    T5(dl::IsduMessage),
    /// T6: State: Idle (0) -> AwaitAlRwRsp (3)
    /// Action: Invoke AL_Write
    T6(dl::IsduMessage),
    /// T7: State: AwaitAlRwRsp (3) -> Idle (0)
    /// Action: Invoke DL_ISDUTransport (read)
    T7(u16, u8, u8, [u8; handlers::isdu::MAX_ISDU_LENGTH]), // (index, sub_index, length, data)
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
#[derive(Debug, Clone, PartialEq, Eq)]
enum OnRequestHandlerEvent {
    /// {AL_Abort}
    AlAbort,
    /// {DL_WriteParam_ind}
    DlWriteParamInd(u8, u8), // (index, data)
    /// {AL_Write_rsp}
    AlWriteRsp,
    /// {DL_ReadParam_ind}
    DlReadParamInd(u8), // (address)
    /// {AL_Read_rsp}
    AlReadRsp(u8, [u8; handlers::isdu::MAX_ISDU_LENGTH]), // (length, data)
    /// {DL_ISDUTransport_ind[DirRead]}
    DlIsduTransportIndDirRead(dl::IsduMessage),
    /// {DL_ISDUTransport_ind[DirWrite]}
    DlIsduTransportIndDirWrite(dl::IsduMessage),
    /// {DL_ISDUAbort}
    DlIsduAbort,
}

/// On-request Data Handler implementation
#[derive(Debug, Clone)]
pub struct OnRequestDataHandler {
    state: OnRequestDataHandlerState,
    exec_transition: Transition,
    read_cycle: bool,
}

impl OnRequestDataHandler {
    /// Create a new On-request Data Handler
    pub fn new() -> Self {
        Self {
            state: OnRequestDataHandlerState::Idle,
            exec_transition: Transition::Tn,
            read_cycle: false,
        }
    }

    /// Process an event
    fn process_event(&mut self, event: OnRequestHandlerEvent) -> IoLinkResult<()> {
        use OnRequestDataHandlerState as State;
        use OnRequestHandlerEvent as Event;

        let (new_transition, new_state) = match (self.state.clone(), event) {
            // Valid transitions according to Table 75
            (State::Idle, Event::AlAbort) => (Transition::T11, State::Idle),
            (State::Idle, Event::DlWriteParamInd(index, data)) => {
                (Transition::T1(index, data), State::AwaitAlWriteRsp)
            }
            (State::Idle, Event::DlReadParamInd(address)) => {
                (Transition::T3(address), State::AwaitAlReadRsp)
            }
            (State::Idle, Event::DlIsduTransportIndDirRead(isdu)) => {
                (Transition::T5(isdu), State::AwaitAlRwRsp)
            }
            (State::Idle, Event::DlIsduTransportIndDirWrite(isdu)) => {
                (Transition::T6(isdu), State::AwaitAlRwRsp)
            }
            (State::AwaitAlWriteRsp, Event::AlWriteRsp) => (Transition::T2, State::Idle),
            (State::AwaitAlReadRsp, Event::AlReadRsp(length, data)) => {
                (Transition::T4(length, data), State::Idle)
            }
            (State::AwaitAlRwRsp, Event::DlIsduAbort) => (Transition::T10, State::Idle),
            (State::AwaitAlRwRsp, Event::AlReadRsp(length, data)) => {
                (Transition::T7(0, 0, length, data), State::Idle)
            }
            (State::AwaitAlRwRsp, Event::AlWriteRsp) => (Transition::T8, State::Idle),
            (State::AwaitAlRwRsp, Event::AlAbort) => (Transition::T9, State::Idle),
            // Invalid transitions - no state change
            _ => return Err(IoLinkError::InvalidEvent),
        };
        self.exec_transition = new_transition;
        self.state = new_state;

        Ok(())
    }

    /// Poll the state machine
    pub fn poll(
        &mut self,
        parameter_manager: &mut parameter_manager::ParameterManager,
        data_link_layer: &mut dl::DataLinkLayer,
    ) -> IoLinkResult<()> {
        let exec_transition = self.exec_transition.clone();
        // Process pending transitions
        match exec_transition {
            Transition::Tn => {
                self.exec_transition = Transition::Tn;
                // No transition, do nothing
            }
            Transition::T1(index, data) => {
                self.exec_transition = Transition::Tn;
                self.execute_t1(index, data, parameter_manager)?;
            }
            Transition::T2 => {
                self.exec_transition = Transition::Tn;
                self.execute_t2(data_link_layer)?;
            }
            Transition::T3(address) => {
                self.exec_transition = Transition::Tn;
                self.execute_t3(address, parameter_manager)?;
            }
            Transition::T4(_length, data) => {
                self.exec_transition = Transition::Tn;
                self.execute_t4(&data, data_link_layer)?;
            }
            Transition::T5(isdu) => {
                self.exec_transition = Transition::Tn;
                self.execute_t5(isdu, parameter_manager)?;
            }
            Transition::T6(isdu) => {
                self.exec_transition = Transition::Tn;
                self.execute_t6(isdu, parameter_manager)?;
            }
            Transition::T7(index, _sub_index, length, data) => {
                self.exec_transition = Transition::Tn;
                self.execute_t7(index, length, &data, data_link_layer)?;
            }
            Transition::T8 => {
                self.exec_transition = Transition::Tn;
                self.execute_t8(data_link_layer)?;
            }
            Transition::T9 => {
                self.exec_transition = Transition::Tn;
                self.execute_t9(data_link_layer)?;
            }
            Transition::T10 => {
                self.exec_transition = Transition::Tn;
                self.execute_t10()?; // Current waiting on AL_Read or AL_Write abandoned
            }
            Transition::T11 => {
                self.exec_transition = Transition::Tn;
                self.execute_t11()?; // Current DL_ISDUTransport abandoned. All OD are set to "0"
            }
        }

        Ok(())
    }

    /// Execute transition T1: Invoke AL_Write
    fn execute_t1(
        &mut self,
        index: u8,
        data: u8,
        parameter_manager: &mut parameter_manager::ParameterManager,
    ) -> IoLinkResult<()> {
        // TODO: Invoke AL_Write
        parameter_manager.al_write_ind(index as u16, 0, &[data])?;
        Ok(())
    }

    /// Execute transition T2: Invoke DL_WriteParam (16 to 31)
    fn execute_t2(&mut self, data_link_layer: &mut dl::DataLinkLayer) -> IoLinkResult<()> {
        // TODO: Invoke DL_WriteParam (16 to 31)?
        data_link_layer.dl_write_param_rsp()?;
        Ok(())
    }

    /// Execute transition T3: Invoke AL_Read
    fn execute_t3(
        &mut self,
        address: u8,
        parameter_manager: &mut parameter_manager::ParameterManager,
    ) -> IoLinkResult<()> {
        // TODO: Invoke AL_Read
        if !(0..=31).contains(&address) {
            return Err(IoLinkError::InvalidAddress);
        }
        parameter_manager.al_read_ind(0, address)?;
        Ok(())
    }

    /// Execute transition T4: Invoke DL_ReadParam (0 to 31)
    fn execute_t4(
        &mut self,
        data: &[u8; handlers::isdu::MAX_ISDU_LENGTH],
        data_link_layer: &mut dl::DataLinkLayer,
    ) -> IoLinkResult<()> {
        // TODO: Invoke DL_ReadParam (0 to 31)
        data_link_layer.dl_read_param_rsp(1, data[0])?;
        Ok(())
    }

    /// Execute transition T5: Invoke AL_Read
    fn execute_t5(
        &mut self,
        isdu: dl::IsduMessage,
        parameter_manager: &mut parameter_manager::ParameterManager,
    ) -> IoLinkResult<()> {
        self.read_cycle = true;
        // TODO: Invoke AL_Read
        parameter_manager.al_read_ind(isdu.index, isdu.sub_index)?;
        Ok(())
    }

    /// Execute transition T6: Invoke AL_Write
    fn execute_t6(
        &mut self,
        isdu: dl::IsduMessage,
        parameter_manager: &mut parameter_manager::ParameterManager,
    ) -> IoLinkResult<()> {
        self.read_cycle = false;
        // TODO: Invoke AL_Write
        parameter_manager.al_write_ind(isdu.index, isdu.sub_index, &isdu.data)?;
        Ok(())
    }

    /// Execute transition T7: Invoke DL_ISDUTransport (read)
    fn execute_t7(
        &mut self,
        _index: u16,
        length: u8,
        data: &[u8; handlers::isdu::MAX_ISDU_LENGTH],
        data_link_layer: &mut dl::DataLinkLayer,
    ) -> IoLinkResult<()> {
        // TODO: Invoke DL_ISDUTransport (read)
        data_link_layer.dl_isdu_transport_read_rsp(length, data)?;
        Ok(())
    }

    /// Execute transition T8: Invoke DL_ISDUTransport (write)
    fn execute_t8(&mut self, data_link_layer: &mut dl::DataLinkLayer) -> IoLinkResult<()> {
        // TODO: Invoke DL_ISDUTransport (write)
        data_link_layer.dl_isdu_transport_write_rsp()?;
        Ok(())
    }

    /// Execute transition T9: Handle abort scenarios
    fn execute_t9(&mut self, data_link_layer: &mut dl::DataLinkLayer) -> IoLinkResult<()> {
        // TODO: Current AL_Read or AL_Write abandoned upon AL_Abort service call
        const ABORT_APP_ERROR_CODE: (u8, u8) = iolinke_macros::isdu_error_code!(APP_DEV);
        if self.read_cycle {
            data_link_layer
                .dl_isdu_transport_read_error_rsp(ABORT_APP_ERROR_CODE.0, ABORT_APP_ERROR_CODE.1)?;
        } else {
            data_link_layer.dl_isdu_transport_write_error_rsp(
                ABORT_APP_ERROR_CODE.0,
                ABORT_APP_ERROR_CODE.1,
            )?;
        }
        Ok(())
    }

    /// Execute transition T10: Handle abort scenarios
    fn execute_t10(&mut self) -> IoLinkResult<()> {
        // TODO: Current waiting on AL_Read or AL_Write abandoned
        Ok(())
    }

    /// Execute transition T11: Handle abort scenarios
    fn execute_t11(&mut self) -> IoLinkResult<()> {
        // TODO: Current DL_ISDUTransport abandoned. All OD are set to "0"
        Ok(())
    }
}

impl dl::DlWriteParamInd for OnRequestDataHandler {
    fn dl_write_param_ind(&mut self, index: u8, data: u8) -> IoLinkResult<()> {
        // Handle the write parameter indication
        self.process_event(OnRequestHandlerEvent::DlWriteParamInd(index, data))?;
        Ok(())
    }
}

impl dl::DlReadParamInd for OnRequestDataHandler {
    fn dl_read_param_ind(&mut self, address: u8) -> IoLinkResult<()> {
        // Handle the read parameter indication
        self.process_event(OnRequestHandlerEvent::DlReadParamInd(address))?;
        Ok(())
    }
}

impl dl::DlIsduAbort for OnRequestDataHandler {
    fn dl_isdu_abort(&mut self) -> IoLinkResult<()> {
        // Handle ISDU abort
        self.process_event(OnRequestHandlerEvent::DlIsduAbort)?;
        Ok(())
    }
}

impl dl::DlIsduTransportInd for OnRequestDataHandler {
    fn dl_isdu_transport_ind(&mut self, isdu: dl::IsduMessage) -> IoLinkResult<()> {
        use iolinke_types::frame::msequence::RwDirection;

        match isdu.direction {
            RwDirection::Write => {
                self.process_event(OnRequestHandlerEvent::DlIsduTransportIndDirWrite(isdu))?;
            }
            RwDirection::Read => {
                self.process_event(OnRequestHandlerEvent::DlIsduTransportIndDirRead(isdu))?;
            }
        }
        Ok(())
    }
}

impl services::AlReadRsp for OnRequestDataHandler {
    fn al_read_rsp(&mut self, result: services::AlResult<(u8, &[u8])>) -> IoLinkResult<()> {
        // Handle AL_Read response
        let (length, data) = result.map_err(|_| IoLinkError::InvalidData)?;
        let mut data_array = [0; handlers::isdu::MAX_ISDU_LENGTH];
        data_array[..length as usize].copy_from_slice(data);
        self.process_event(OnRequestHandlerEvent::AlReadRsp(length, data_array))?;
        Ok(())
    }
}

impl services::AlWriteRsp for OnRequestDataHandler {
    fn al_write_rsp(&mut self, result: services::AlResult<()>) -> IoLinkResult<()> {
        // Handle AL_Write response
        match result {
            Ok(_) => {
                self.process_event(OnRequestHandlerEvent::AlWriteRsp)?;
            }
            Err(_) => {
                return Err(IoLinkError::InvalidData);
            }
        }
        self.process_event(OnRequestHandlerEvent::AlWriteRsp)?;
        Ok(())
    }
}

impl Default for OnRequestDataHandler {
    fn default() -> Self {
        Self::new()
    }
}
