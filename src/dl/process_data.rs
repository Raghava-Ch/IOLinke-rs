//! Process Data Handler
//!
//! This module implements the Process Data Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 7.2

use crate::types::{self, IoLinkError, IoLinkResult, ProcessData};

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

/// Process Data Handler implementation
pub struct ProcessDataHandler {
    state: ProcessDataHandlerState,
    exec_transition: Transition,
    input_data: ProcessData,
    output_data: ProcessData,
    cycle_time: u16,
}

impl ProcessDataHandler {
    /// Create a new Process Data Handler
    pub fn new() -> Self {
        Self {
            state: ProcessDataHandlerState::Inactive,
            exec_transition: Transition::Tn,
            input_data: ProcessData::default(),
            output_data: ProcessData::default(),
            cycle_time: 100, // 10ms default
        }
    }

     /// Process an event
    pub fn process_event(&mut self, event: ProcessDataHandlerEvent) -> IoLinkResult<()> {
        use ProcessDataHandlerEvent as Event;
        use ProcessDataHandlerState as State;

        let (new_transition, new_state) = match (self.state, event) {
            (State::Inactive, Event::PDInd) => {
                (Transition::T1, State::Inactive)
            }
            (State::Inactive, Event::PDConfActive) => {
                (Transition::T2, State::PDActive)
            }
            (State::PDActive, Event::DlPDInputUpdate) => {
                (Transition::T3, State::PDActive)
            }
            (State::PDActive, Event::PDInd) => {
                (Transition::T4, State::HandlePD)
            }
            (State::PDActive, Event::PDConfInactive) => {
                (Transition::T8, State::Inactive)
            }
            (State::HandlePD, Event::PDIncomplete) => {
                (Transition::T5, State::PDActive)
            }
            (State::HandlePD, Event::PDComplete) => {
                (Transition::T6, State::PDActive)
            }
            (State::HandlePD, Event::CycleComplete) => {
                (Transition::T7, State::PDActive)
            }
            _ => return Err(IoLinkError::InvalidEvent),
        };
        self.exec_transition = new_transition;
        self.state = new_state;
        
        Ok(())
    }

    /// Poll the process data handler
    /// See IO-Link v1.1.4 Section 7.2
    pub fn poll(&mut self) -> IoLinkResult<()> {
        match self.exec_transition {
            Transition::Tn => {
                // No transition to execute
            }
            Transition::T1 => {
                // State: Inactive (0) -> Inactive (0)
                // Action: Ignore Process Data
                self.state = ProcessDataHandlerState::Inactive;
            }
            Transition::T2 => {
                // State: Inactive (0) -> PDActive (1)
                self.state = ProcessDataHandlerState::PDActive;
            }
            Transition::T3 => {
                // State: PDActive (1) -> PDActive (1)
                // Action: Prepare input Process Data for PD.rsp for next message handler demand
            }
            Transition::T4 => {
                // State: PDActive (1) -> HandlePD (2)
                self.state = ProcessDataHandlerState::HandlePD;
            }
            Transition::T5 => {
                // State: HandlePD (2) -> PDActive (1)
                self.state = ProcessDataHandlerState::PDActive;
            }
            Transition::T6 => {
                // State: HandlePD (2) -> PDActive (1)
                // Action: Invoke DL_PDOutputTransport.ind
            }
            Transition::T7 => {
                // State: HandlePD (2) -> PDActive (1)
                // Action: Invoke DL_PDCycle.ind
            }
            Transition::T8 => {
                // State: PDActive (1) -> Inactive (0)
                self.state = ProcessDataHandlerState::Inactive;
            }
            
        }
        Ok(())
    }

    /// Handle Process Data configuration changes
    /// See 7.3.4.4 State machine of the Device Process Data handler
    pub fn pd_conf(&mut self, state: types::PdConfState) -> IoLinkResult<()> {
        match state {
            types::PdConfState::Active => self.process_event(ProcessDataHandlerEvent::PDConfActive),
            types::PdConfState::Inactive => self.process_event(ProcessDataHandlerEvent::PDConfInactive),
        }?;

        Ok(())
    }

    /// Process cyclic data exchange
    fn process_cyclic_data(&mut self) -> IoLinkResult<()> {
        // Implementation would handle cyclic process data
        Ok(())
    }

    /// Set input data
    pub fn set_input_data(&mut self, data: ProcessData) {
        self.input_data = data;
    }

    /// Get output data
    pub fn get_output_data(&self) -> &ProcessData {
        &self.output_data
    }

    /// Get current state
    pub fn state(&self) -> ProcessDataHandlerState {
        self.state
    }
}

impl Default for ProcessDataHandler {
    fn default() -> Self {
        Self::new()
    }
}
