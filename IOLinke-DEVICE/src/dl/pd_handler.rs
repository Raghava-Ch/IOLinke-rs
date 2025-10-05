//! Process Data Handler
//!
//! This module implements the Process Data Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 7.2
use heapless::Vec;
use iolinke_derived_config as derived_config;
use iolinke_types::handlers;
use iolinke_types::handlers::pd::DlPDOutputTransportInd;
use iolinke_types::{
    custom::IoLinkResult,
    handlers::pd::{PD_OUTPUT_LENGTH, PdConfState, ProcessData},
};
use iolinke_util::{log_state_transition, log_state_transition_error};

use core::clone::Clone;
use core::default::Default;
use core::result::Result::{Err, Ok};

use crate::{al, dl::message_handler, services};

/// Process Data Handler states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProcessDataHandlerState {
    /// {Inactive_0}
    Inactive,
    /// {PDActive_1}
    PDActive,
    /// {HandlePD_2}
    HandlePD,
}

#[derive(Debug, PartialEq, Eq)]
enum Transition {
    /// Tn: No transition
    Tn,
    /// T1: State: Inactive (0) -> Inactive (0)
    /// Action: Ignore Process Data
    T1,
    /// T2: State: Inactive (0) -> PDActive (1)
    /// Action: -
    T2,
    /// T3: State: PDActive (1) -> PDActive (1)
    /// Action: Prepare input Process Data for PD.rsp for next message handler demand
    T3,
    /// T4: State: PDActive (1) -> HandlePD (2)
    /// Action: Message handler demands input PD via a PD.ind service and delivers output PD or segment of output PD. Invoke PD.rsp with input Process Data when in non-interleave mode (see 7.2.2.3).
    T4(u8), // (pd_in demand length)
    /// T5: State: HandlePD (2) -> PDActive (1)
    /// Action: -
    T5,
    /// T6: State: HandlePD (2) -> PDActive (1)
    /// Action: Invoke DL_PDOutputTransport.ind (see 7.2.1.9)
    T6,
    /// T7: State: HandlePD (2) -> PDActive (1)
    /// Action: Invoke DL_PDCycle.ind (see 7.2.1.12)
    T7,
    /// T8: State: PDActive (1) -> Inactive (0)
    /// Action: -
    T8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProcessDataHandlerEvent {
    /// {PD_ind}
    PDInd(u8), // (pd_in demand length)
    /// {PD_Conf_ACTIVE}
    PDConfActive,
    /// {DL_PDInputUpdate}
    DlPDInputUpdate,
    /// {PD_Conf_INACTIVE}
    PDConfInactive,
    /// {[PD incomplete]}
    _PDIncomplete,
    /// {[PD complete]}
    PDComplete,
    /// {[Cycle complete]}
    _CycleComplete,
}

/// Process Data Handler implementation
pub struct ProcessDataHandler {
    state: ProcessDataHandlerState,
    exec_transition: Transition,
    process_data: ProcessData,
}

impl ProcessDataHandler {
    /// Create a new Process Data Handler
    pub fn new() -> Self {
        Self {
            state: ProcessDataHandlerState::Inactive,
            exec_transition: Transition::Tn,
            process_data: ProcessData::default(),
        }
    }

    /// Process an event
    fn process_event(&mut self, event: ProcessDataHandlerEvent) -> IoLinkResult<()> {
        use ProcessDataHandlerEvent as Event;
        use ProcessDataHandlerState as State;

        let (new_transition, new_state) = match (self.state, event) {
            (State::Inactive, Event::PDInd(_pd_in_length)) => (Transition::T1, State::Inactive),
            (State::Inactive, Event::PDConfActive) => (Transition::T2, State::PDActive),
            (State::PDActive, Event::DlPDInputUpdate) => (Transition::T3, State::PDActive),
            (State::PDActive, Event::PDInd(pd_in_length)) => {
                (Transition::T4(pd_in_length), State::HandlePD)
            }
            (State::PDActive, Event::PDConfInactive) => (Transition::T8, State::Inactive),
            (State::HandlePD, Event::_PDIncomplete) => (Transition::T5, State::PDActive),
            (State::HandlePD, Event::PDComplete) => (Transition::T6, State::PDActive),
            (State::HandlePD, Event::_CycleComplete) => (Transition::T7, State::PDActive),
            _ => {
                log_state_transition_error!(module_path!(), "process_event", self.state, event);
                (Transition::Tn, self.state)
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

    /// Poll the process data handler
    /// See IO-Link v1.1.4 Section 7.2
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
                // State: Inactive (0) -> Inactive (0)
                // Action: Ignore Process Data
                self.exec_transition = Transition::Tn;
                self.execute_t1()?;
            }
            Transition::T2 => {
                // State: Inactive (0) -> PDActive (1)
                self.exec_transition = Transition::Tn;
                self.execute_t2()?;
            }
            Transition::T3 => {
                // State: PDActive (1) -> PDActive (1)
                self.exec_transition = Transition::Tn;
                // Action: Prepare input Process Data for PD.rsp for next message handler demand
                self.execute_t3()?;
            }
            Transition::T4(pd_in_length) => {
                // State: PDActive (1) -> HandlePD (2)
                self.exec_transition = Transition::Tn;
                self.execute_t4(pd_in_length, application_layer, message_handler)?;
            }
            Transition::T5 => {
                // State: HandlePD (2) -> PDActive (1)
                self.exec_transition = Transition::Tn;
                self.execute_t5()?;
            }
            Transition::T6 => {
                // State: HandlePD (2) -> PDActive (1)
                self.exec_transition = Transition::Tn;
                // Action: Invoke DL_PDOutputTransport.ind
                self.execute_t6(application_layer)?;
            }
            Transition::T7 => {
                // State: HandlePD (2) -> PDActive (1)
                self.exec_transition = Transition::Tn;
                // Action: Invoke DL_PDCycle.ind
                self.execute_t7()?;
            }
            Transition::T8 => {
                // State: PDActive (1) -> Inactive (0)
                self.exec_transition = Transition::Tn;
                self.execute_t8()?;
            }
        }
        Ok(())
    }

    fn execute_t1(&mut self) -> IoLinkResult<()> {
        // State: Inactive (0) -> Inactive (0)
        // Action: Ignore Process Data
        Ok(())
    }

    fn execute_t2(&mut self) -> IoLinkResult<()> {
        // State: Inactive (0) -> PDActive (1)
        Ok(())
    }

    fn execute_t3(&mut self) -> IoLinkResult<()> {
        // State: PDActive (1) -> PDActive (1)
        // Action: Prepare input Process Data for PD.rsp for next message handler demand
        // PD is ready to be sent in "dl_pd_input_update_req"
        Ok(())
    }

    fn execute_t4<
        ALS: services::ApplicationLayerServicesInd
            + handlers::sm::SystemManagementCnf
            + services::AlEventCnf,
    >(
        &mut self,
        _pd_in_length: u8,
        application_layer: &mut al::ApplicationLayer<ALS>,
        message_handler: &mut message_handler::MessageHandler,
    ) -> IoLinkResult<()> {
        // State: PDActive (1) -> HandlePD (2)
        // Action: Message handler demands input PD via a PD.ind service and delivers output PD or segment of output PD. Invoke PD.rsp with input Process Data when in non-interleave mode (see 7.2.2.3).
        let _ = message_handler.pd_rsp(self.process_data.input.len(), &self.process_data.input);
        let _ = application_layer.dl_pd_output_transport_ind(&self.process_data.output);
        let _ = self.process_event(ProcessDataHandlerEvent::PDComplete);
        Ok(())
    }

    fn execute_t5(&mut self) -> IoLinkResult<()> {
        // State: HandlePD (2) -> PDActive (1)
        Ok(())
    }

    fn execute_t6<
        ALS: services::ApplicationLayerServicesInd
            + handlers::sm::SystemManagementCnf
            + services::AlEventCnf,
    >(
        &mut self,
        application_layer: &mut al::ApplicationLayer<ALS>,
    ) -> IoLinkResult<()> {
        // State: HandlePD (2) -> PDActive (1)
        // Action: Invoke DL_PDOutputTransport.ind
        let _ = application_layer.dl_pd_output_transport_ind(&self.process_data.output);
        Ok(())
    }

    fn execute_t7(&mut self) -> IoLinkResult<()> {
        // State: HandlePD (2) -> PDActive (1)
        // Action: Invoke DL_PDCycle.ind
        Ok(())
    }

    fn execute_t8(&mut self) -> IoLinkResult<()> {
        // State: PDActive (1) -> Inactive (0)
        Ok(())
    }

    /// See 7.2.2.3 PD
    /// The PD service is used to setup the Process Data to be sent through the process
    /// communication channel. The confirmation of the service contains the data from the receiver.
    /// The parameters of the service primitives are listed in Table 36.
    pub fn pd_conf(&mut self, state: PdConfState) -> IoLinkResult<()> {
        match state {
            PdConfState::Active => self.process_event(ProcessDataHandlerEvent::PDConfActive),
            PdConfState::Inactive => self.process_event(ProcessDataHandlerEvent::PDConfInactive),
        }?;

        Ok(())
    }

    /// See 7.2.2.3 PD
    /// The PD service is used to setup the Process Data to be sent through the process
    /// communication channel. The confirmation of the service contains the data from the receiver.
    /// The parameters of the service primitives are listed in Table 36.
    pub fn pd_ind(
        &mut self,
        _pd_in_address: u8,  // Not required, because of legacy specification
        pd_in_length: u8,    // pd_in demands length
        _pd_out_address: u8, // Not required, because of legacy specification
        pd_out: &Vec<u8, PD_OUTPUT_LENGTH>,
    ) -> IoLinkResult<()> {
        if pd_in_length > derived_config::device::process_data::max_pd_len() {
            return Err(iolinke_types::custom::IoLinkError::InvalidParameter);
        }
        self.process_data.output = pd_out.clone();
        self.process_event(ProcessDataHandlerEvent::PDInd(pd_in_length))?;

        Ok(())
    }

    /// See 7.2.1.10 DL_PDInputUpdate
    /// The Device's application layer uses the DL_PDInputUpdate service to update the input data
    /// (Process Data from Device to Master) on the data link layer. The parameters of the service
    /// primitives are listed in Table 25.
    pub fn dl_pd_input_update_req(&mut self, length: u8, input_data: &[u8]) -> IoLinkResult<()> {
        let _ = length;
        self.process_data.input.clear();
        self.process_data
            .input
            .extend_from_slice(input_data)
            .map_err(|_| iolinke_types::custom::IoLinkError::InvalidParameter)?;
        self.process_event(ProcessDataHandlerEvent::DlPDInputUpdate)?;
        Ok(())
    }
}

impl Default for ProcessDataHandler {
    fn default() -> Self {
        Self::new()
    }
}
