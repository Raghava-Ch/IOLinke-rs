//! Process Data Handler
//!
//! This module implements the Process Data Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 7.2

use heapless::Vec;

use crate::{
    al, config, dl, log_state_transition, log_state_transition_error, types::{self, IoLinkError, IoLinkResult}
};

pub const PD_INPUT_LENGTH: usize = config::process_data::pd_in::param_length() as usize;
pub const PD_OUTPUT_LENGTH: usize = config::process_data::pd_out::length() as usize;

/// Process data input/output structure
/// See IO-Link v1.1.4 Section 8.4.2
#[derive(Debug, Default, Clone)]
pub struct ProcessData {
    /// Input data from device
    input: Vec<u8, PD_INPUT_LENGTH>,
    input_length: u8,
    /// Output data to device
    output: Vec<u8, PD_OUTPUT_LENGTH>,
    output_length: u8,
    /// Data validity flag
    valid: bool,
}

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
    T4,
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
    PDInd,
    /// {PD_Conf_ACTIVE}
    PDConfActive,
    /// {DL_PDInputUpdate}
    DlPDInputUpdate,
    /// {PD_Conf_INACTIVE}
    PDConfInactive,
    /// {[PD incomplete]}
    PDIncomplete,
    /// {[PD complete]}
    PDComplete,
    /// {[Cycle complete]}
    CycleComplete,
}

pub trait DlPDInputUpdate {
    fn dl_pd_input_update_req(&mut self, length: u8, input_data: &[u8]) -> IoLinkResult<()>;
}

pub trait DlPDOutputTransportInd {
    fn dl_pd_output_transport_ind(
        &mut self,
        pd_out: &Vec<u8, PD_OUTPUT_LENGTH>,
    ) -> IoLinkResult<()>;
    fn dl_pd_cycle_ind(&mut self) -> IoLinkResult<()>;
}

/// Process Data Handler implementation
pub struct ProcessDataHandler {
    state: ProcessDataHandlerState,
    exec_transition: Transition,
    process_data: ProcessData,
    cycle_time: u16,
}

impl ProcessDataHandler {
    /// Create a new Process Data Handler
    pub fn new() -> Self {
        Self {
            state: ProcessDataHandlerState::Inactive,
            exec_transition: Transition::Tn,
            process_data: ProcessData::default(),
            cycle_time: 100, // 10ms default
        }
    }

    /// Process an event
    fn process_event(&mut self, event: ProcessDataHandlerEvent) -> IoLinkResult<()> {
        use ProcessDataHandlerEvent as Event;
        use ProcessDataHandlerState as State;

        let (new_transition, new_state) = match (self.state, event) {
            (State::Inactive, Event::PDInd) => (Transition::T1, State::Inactive),
            (State::Inactive, Event::PDConfActive) => (Transition::T2, State::PDActive),
            (State::PDActive, Event::DlPDInputUpdate) => (Transition::T3, State::PDActive),
            (State::PDActive, Event::PDInd) => (Transition::T4, State::HandlePD),
            (State::PDActive, Event::PDConfInactive) => (Transition::T8, State::Inactive),
            (State::HandlePD, Event::PDIncomplete) => (Transition::T5, State::PDActive),
            (State::HandlePD, Event::PDComplete) => (Transition::T6, State::PDActive),
            (State::HandlePD, Event::CycleComplete) => (Transition::T7, State::PDActive),
            _ => {
                log_state_transition_error!(
                    module_path!(),
                    "process_event",
                    self.state,
                    event
                );
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
            Transition::T4 => {
                // State: PDActive (1) -> HandlePD (2)
                self.exec_transition = Transition::Tn;
                self.execute_t4(message_handler)?;
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

    fn execute_t3(
        &mut self,
    ) -> IoLinkResult<()> {
        // State: PDActive (1) -> PDActive (1)
        // Action: Prepare input Process Data for PD.rsp for next message handler demand
        // PD is ready to be sent in "dl_pd_input_update_req"
        Ok(())
    }

    fn execute_t4(&mut self, message_handler: &mut dl::message_handler::MessageHandler) -> IoLinkResult<()> {
        // State: PDActive (1) -> HandlePD (2)
        // Action: Message handler demands input PD via a PD.ind service and delivers output PD or segment of output PD. Invoke PD.rsp with input Process Data when in non-interleave mode (see 7.2.2.3).
        let _ = message_handler.pd_rsp(self.process_data.input_length, &self.process_data.input);
        let _ = self.process_event(ProcessDataHandlerEvent::PDComplete);
        Ok(())
    }

    fn execute_t5(&mut self) -> IoLinkResult<()> {
        // State: HandlePD (2) -> PDActive (1)
        Ok(())
    }

    fn execute_t6(&mut self, application_layer: &mut al::ApplicationLayer) -> IoLinkResult<()> {
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
    pub fn pd_conf(&mut self, state: types::PdConfState) -> IoLinkResult<()> {
        match state {
            types::PdConfState::Active => self.process_event(ProcessDataHandlerEvent::PDConfActive),
            types::PdConfState::Inactive => {
                self.process_event(ProcessDataHandlerEvent::PDConfInactive)
            }
        }?;

        Ok(())
    }

    /// See 7.2.2.3 PD
    /// The PD service is used to setup the Process Data to be sent through the process
    /// communication channel. The confirmation of the service contains the data from the receiver.
    /// The parameters of the service primitives are listed in Table 36.
    pub fn pd_ind(
        &mut self,
        _pd_in_address: u8, // Not required, because of legacy specification
        _pd_in_length: u8,  // Not required, because of legacy specification
        pd_out: &Vec<u8, PD_OUTPUT_LENGTH>,
        _pd_out_address: u8, // Not required, because of legacy specification
        pd_out_length: u8,
    ) -> IoLinkResult<()> {
        self.process_data.output = pd_out.clone();
        self.process_data.output_length = pd_out_length;
        self.process_event(ProcessDataHandlerEvent::PDInd)?;

        Ok(())
    }

    /// See 7.2.1.10 DL_PDInputUpdate
    /// The Device's application layer uses the DL_PDInputUpdate service to update the input data
    /// (Process Data from Device to Master) on the data link layer. The parameters of the service
    /// primitives are listed in Table 25.
    pub fn dl_pd_input_update_req(&mut self, length: u8, input_data: &[u8]) -> IoLinkResult<()> {
        self.process_data.input.fill(0);
        self.process_data.input_length = length;
        for (i, &byte) in input_data.iter().enumerate() {
            if i < length as usize {
                self.process_data.input[i] = byte;
            } else {
                break; // Avoid out of bounds access
            }
        }
        self.process_event(ProcessDataHandlerEvent::DlPDInputUpdate)?;
        Ok(())
    }
}

impl Default for ProcessDataHandler {
    fn default() -> Self {
        Self::new()
    }
}
