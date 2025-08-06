//! On-request Data Handler
//!
//! This module implements the On-request Data Handler state machine as defined in
//! IO-Link Specification v1.1.4

use crate::{
    al::{self, services::ApplicationLayerServicesInd},
    dl::{self, DlIsduTransportRsp, DlParamRsp},
    types::{IoLinkError, IoLinkResult},
};

/// See 8.3.2.2 OD state machine of the Device AL
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnRequestDataHandlerState {
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
#[derive(Debug, PartialEq, Eq, Clone)]
enum Transition<'a> {
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
    T4(u8, &'a [u8]), // (address, data)
    /// T5: State: Idle (0) -> AwaitAlRwRsp (3)
    /// Action: Invoke AL_Read
    T5(dl::Isdu),
    /// T6: State: Idle (0) -> AwaitAlRwRsp (3)
    /// Action: Invoke AL_Write
    T6(dl::Isdu),
    /// T7: State: AwaitAlRwRsp (3) -> Idle (0)
    /// Action: Invoke DL_ISDUTransport (read)
    T7(u16, u8, &'a [u8]), // (index, sub_index, data)
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
pub enum OnRequestHandlerEvent<'a> {
    /// {AL_Abort}
    AlAbort,
    /// {DL_WriteParam_ind}
    DlWriteParamInd(u8, u8), // (index, data)
    /// {AL_Write_rsp}
    AlWriteRsp,
    /// {DL_ReadParam_ind}
    DlReadParamInd(u8), // (address)
    /// {AL_Read_rsp}
    AlReadRsp(&'a [u8]), // (data)
    /// {DL_ISDUTransport_ind[DirRead]}
    DlIsduTransportIndDirRead(dl::Isdu),
    /// {DL_ISDUTransport_ind[DirWrite]}
    DlIsduTransportIndDirWrite(dl::Isdu),
    /// {DL_ISDUAbort}
    DlIsduAbort,
}

/// On-request Data Handler implementation
pub struct OnRequestDataHandler<'a> {
    state: OnRequestDataHandlerState,
    exec_transition: Transition<'a>,
    read_cycle: bool,
}

impl<'a> OnRequestDataHandler<'a> {
    /// Create a new On-request Data Handler
    pub fn new() -> Self {
        Self {
            state: OnRequestDataHandlerState::Idle,
            exec_transition: Transition::Tn,
            read_cycle: false,
        }
    }

    /// Process an event
    pub fn process_event(&mut self, event: OnRequestHandlerEvent<'a>) -> IoLinkResult<()> {
        use OnRequestHandlerEvent as Event;
        use OnRequestDataHandlerState as State;

        let (new_transition, new_state) = match (self.state, event) {
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
            (State::AwaitAlReadRsp, Event::AlReadRsp(data)) => {
                (Transition::T4(0, data), State::Idle)
            }
            (State::AwaitAlRwRsp, Event::DlIsduAbort) => (Transition::T10, State::Idle),
            (State::AwaitAlRwRsp, Event::AlReadRsp(data)) => {
                (Transition::T7(0, 0, data), State::Idle)
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
        application_layer: &mut al::services::ApplicationLayerServices,
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
                self.execute_t1(index, data, application_layer)?;
            }
            Transition::T2 => {
                self.exec_transition = Transition::Tn;
                self.execute_t2(data_link_layer)?;
            }
            Transition::T3(address) => {
                self.exec_transition = Transition::Tn;
                self.execute_t3(address, application_layer)?;
            }
            Transition::T4(address, data) => {
                self.exec_transition = Transition::Tn;
                self.execute_t4(address, data, data_link_layer)?;
            }
            Transition::T5(isdu) => {
                self.exec_transition = Transition::Tn;
                self.execute_t5(isdu, application_layer)?;
            }
            Transition::T6(isdu) => {
                self.exec_transition = Transition::Tn;
                self.execute_t6(isdu, application_layer)?;
            }
            Transition::T7(index, _, data) => {
                self.exec_transition = Transition::Tn;
                self.execute_t7(index, data, data_link_layer)?;
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
        application_layer: &mut al::services::ApplicationLayerServices,
    ) -> IoLinkResult<()> {
        // TODO: Invoke AL_Write
        application_layer.al_write_ind(index as u16, 0, &[data])?;
        Ok(())
    }

    /// Execute transition T2: Invoke DL_WriteParam (16 to 31)
    fn execute_t2(&mut self, data_link_layer: &mut dl::DataLinkLayer) -> IoLinkResult<()> {
        // TODO: Invoke DL_WriteParam (16 to 31)
        data_link_layer.dl_write_param_rsp()?;
        Ok(())
    }

    /// Execute transition T3: Invoke AL_Read
    fn execute_t3(
        &mut self,
        address: u8,
        application_layer: &mut al::services::ApplicationLayerServices,
    ) -> IoLinkResult<()> {
        // TODO: Invoke AL_Read
        if !(0..=31).contains(&address) {
            return Err(IoLinkError::InvalidAddress);
        }
        application_layer.al_read_ind(address as u16, 0)?;
        Ok(())
    }

    /// Execute transition T4: Invoke DL_ReadParam (0 to 31)
    fn execute_t4(
        &mut self,
        address: u8,
        data: &[u8],
        data_link_layer: &mut dl::DataLinkLayer,
    ) -> IoLinkResult<()> {
        // TODO: Invoke DL_ReadParam (0 to 31)
        data_link_layer.dl_read_param_rsp(address, data)?;
        Ok(())
    }

    /// Execute transition T5: Invoke AL_Read
    fn execute_t5(
        &mut self,
        isdu: dl::Isdu,
        application_layer: &mut al::services::ApplicationLayerServices,
    ) -> IoLinkResult<()> {
        self.read_cycle = true;
        // TODO: Invoke AL_Read
        application_layer.al_read_ind(isdu.index, isdu.sub_index)?;
        Ok(())
    }

    /// Execute transition T6: Invoke AL_Write
    fn execute_t6(
        &mut self,
        isdu: dl::Isdu,
        application_layer: &mut al::services::ApplicationLayerServices,
    ) -> IoLinkResult<()> {
        self.read_cycle = false;
        // TODO: Invoke AL_Write
        application_layer.al_write_ind(isdu.index, isdu.sub_index, &isdu.data)?;
        Ok(())
    }

    /// Execute transition T7: Invoke DL_ISDUTransport (read)
    fn execute_t7(
        &mut self,
        index: u16,
        data: &[u8],
        data_link_layer: &mut dl::DataLinkLayer,
    ) -> IoLinkResult<()> {
        // TODO: Invoke DL_ISDUTransport (read)
        data_link_layer.dl_isdu_transport_read_rsp(index as u8, data)?;
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

impl<'a> dl::DlWriteParamInd for OnRequestDataHandler<'a> {
    fn dl_write_param_ind(&mut self, index: u8, data: u8) -> IoLinkResult<()> {
        // Handle the write parameter indication
        self.process_event(OnRequestHandlerEvent::DlWriteParamInd(index, data))?;
        Ok(())
    }
}

impl<'a> dl::DlReadParamInd for OnRequestDataHandler<'a> {
    fn dl_read_param_ind(&mut self, address: u8) -> IoLinkResult<()> {
        // Handle the read parameter indication
        self.process_event(OnRequestHandlerEvent::DlReadParamInd(address))?;
        Ok(())
    }
}

impl<'a> dl::DlIsduAbort for OnRequestDataHandler<'a> {
    fn dl_isdu_abort(&mut self) -> IoLinkResult<()> {
        // Handle ISDU abort
        self.process_event(OnRequestHandlerEvent::DlIsduAbort)?;
        Ok(())
    }
}

impl<'a> dl::DlIsduTransportInd for OnRequestDataHandler<'a> {
    fn dl_isdu_transport_ind(&mut self, isdu: dl::Isdu) -> IoLinkResult<()> {
        match isdu.direction {
            true => {
                self.process_event(OnRequestHandlerEvent::DlIsduTransportIndDirWrite(isdu))?;
            }
            false => {
                self.process_event(OnRequestHandlerEvent::DlIsduTransportIndDirRead(isdu))?;
            }
        }
        Ok(())
    }
}

impl<'a> al::services::AlReadRsp<'a> for OnRequestDataHandler<'a> {
    fn al_read_rsp(&mut self, result: al::services::AlResult<&'a [u8]>) -> IoLinkResult<()> {
        // Handle AL_Read response
        let data = result.map_err(|_| IoLinkError::InvalidData)?;
        self.process_event(OnRequestHandlerEvent::AlReadRsp(data))?;
        Ok(())
    }
}

impl<'a> al::services::AlWriteRsp for OnRequestDataHandler<'a> {
    fn al_write_rsp(&mut self, result: al::services::AlResult<()>) -> IoLinkResult<()> {
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

impl<'a> al::services::AlAbortReq for OnRequestDataHandler<'a> {
    fn al_abort_req(&mut self) -> IoLinkResult<()> {
        // Handle AL_Abort response
        self.process_event(OnRequestHandlerEvent::AlAbort)?;
        Ok(())
    }
}

impl<'a> Default for OnRequestDataHandler<'a> {
    fn default() -> Self {
        Self::new()
    }
}
