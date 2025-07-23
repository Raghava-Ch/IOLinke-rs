//! Application Layer API implementation
//!
//! This module implements the Application Layer interface as defined in
//! IO-Link Specification v1.1.4 Section 8.4

use crate::types::{Event, EventType, IoLinkError, IoLinkResult, Isdu, ProcessData};
use heapless::Vec;

/// Application Layer trait defining all request/indication methods
/// See IO-Link v1.1.4 Section 8.4
pub trait ApplicationLayer {
    /// Get input data from the device
    /// See IO-Link v1.1.4 Section 8.4.2.1
    fn al_get_input_req(&mut self) -> IoLinkResult<ProcessData>;

    /// Set output data to the device
    /// See IO-Link v1.1.4 Section 8.4.2.2
    fn al_set_output_req(&mut self, data: &ProcessData) -> IoLinkResult<()>;

    /// Read parameter via ISDU
    /// See IO-Link v1.1.4 Section 8.4.3.1
    fn al_read_req(&mut self, index: u16, sub_index: u8) -> IoLinkResult<Vec<u8, 32>>;

    /// Write parameter via ISDU
    /// See IO-Link v1.1.4 Section 8.4.3.2
    fn al_write_req(&mut self, index: u16, sub_index: u8, data: &[u8]) -> IoLinkResult<()>;

    /// Get event from device
    /// See IO-Link v1.1.4 Section 8.4.4.1
    fn al_get_event_req(&mut self) -> IoLinkResult<Option<Event>>;

    /// Control event handling
    /// See IO-Link v1.1.4 Section 8.4.4.2
    fn al_control_req(&mut self, control_code: u8) -> IoLinkResult<()>;

    /// Get device identification
    /// See IO-Link v1.1.4 Section 8.4.5.1
    fn al_get_device_id_req(&mut self) -> IoLinkResult<crate::types::DeviceIdentification>;

    /// Get minimum cycle time
    /// See IO-Link v1.1.4 Section 8.4.5.2
    fn al_get_min_cycle_time_req(&mut self) -> IoLinkResult<u8>;

    /// Abort current operation
    /// See IO-Link v1.1.4 Section 8.4.6
    fn al_abort_req(&mut self) -> IoLinkResult<()>;
}

/// Application Layer state machine events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationLayerEvent {
    /// Initialize application layer
    Initialize,
    /// Start communication
    StartCommunication,
    /// Stop communication
    StopCommunication,
    /// Process data received
    ProcessDataReceived,
    /// ISDU request received
    IsduReceived,
    /// Event occurred
    EventOccurred,
    /// Error condition
    Error,
}

/// Application Layer state machine states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationLayerState {
    /// Idle state
    Idle,
    /// Wait for startup
    WaitStartup,
    /// Operate state
    Operate,
    /// Wait for fallback
    WaitFallback,
    /// Error state
    Error,
}

/// Simple state machine implementation (smlang alternative for compatibility)
pub struct ApplicationLayerStateMachine {
    state: ApplicationLayerState,
    context: ApplicationLayerContext,
}

impl ApplicationLayerStateMachine {
    pub fn new() -> Self {
        Self {
            state: ApplicationLayerState::Idle,
            context: ApplicationLayerContext::new(),
        }
    }

    pub fn state(&self) -> &ApplicationLayerState {
        &self.state
    }

    pub fn context(&self) -> &ApplicationLayerContext {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut ApplicationLayerContext {
        &mut self.context
    }

    pub fn process_event(&mut self, event: ApplicationLayerEvent) -> Result<(), ()> {
        use ApplicationLayerEvent as Event;
        use ApplicationLayerState as State;

        let new_state = match (self.state, event) {
            (State::Idle, Event::Initialize) => State::WaitStartup,
            (State::WaitStartup, Event::StartCommunication) => State::Operate,
            (State::Operate, Event::ProcessDataReceived) => State::Operate,
            (State::Operate, Event::IsduReceived) => State::Operate,
            (State::Operate, Event::EventOccurred) => State::Operate,
            (State::Operate, Event::StopCommunication) => State::WaitFallback,
            (State::WaitFallback, Event::Initialize) => State::WaitStartup,
            
            // Error transitions from any state
            (_, Event::Error) => State::Error,
            (State::Error, Event::Initialize) => State::WaitStartup,
            
            // Invalid transitions remain in current state
            _ => return Err(()),
        };

        self.state = new_state;
        Ok(())
    }
}

/// Application Layer implementation
pub struct ApplicationLayerImpl {
    /// State machine instance
    state_machine: ApplicationLayerStateMachine,
    /// Current process data
    process_data: ProcessData,
    /// Event queue
    event_queue: heapless::Deque<Event, 16>,
    /// Device identification
    device_id: crate::types::DeviceIdentification,
    /// Minimum cycle time in 100μs units
    min_cycle_time: u8,
}

/// Context for the application layer state machine
pub struct ApplicationLayerContext {
    /// Error counter
    pub error_count: u32,
    /// Communication active flag
    pub communication_active: bool,
}

impl ApplicationLayerContext {
    /// Create new context
    pub fn new() -> Self {
        Self {
            error_count: 0,
            communication_active: false,
        }
    }
}

impl ApplicationLayerImpl {
    /// Create a new Application Layer instance
    pub fn new() -> Self {
        Self {
            state_machine: ApplicationLayerStateMachine::new(),
            process_data: ProcessData::default(),
            event_queue: heapless::Deque::new(),
            device_id: crate::types::DeviceIdentification {
                vendor_id: 0x0000,
                device_id: 0x00000000,
                function_id: 0x0000,
                reserved: 0x00,
            },
            min_cycle_time: 10, // 1ms default
        }
    }

    /// Poll the application layer state machine
    /// See IO-Link v1.1.4 Section 8.4
    pub fn poll(&mut self) -> IoLinkResult<()> {
        // Process state machine
        match self.state_machine.state() {
            ApplicationLayerState::Idle => {
                // Initialize if needed
                if !self.state_machine.context().communication_active {
                    let _ = self.state_machine.process_event(ApplicationLayerEvent::Initialize);
                }
            }
            ApplicationLayerState::WaitStartup => {
                // Wait for startup conditions
                if self.state_machine.context().communication_active {
                    let _ = self.state_machine.process_event(ApplicationLayerEvent::StartCommunication);
                }
            }
            ApplicationLayerState::Operate => {
                // Normal operation - process events from queue
                self.process_events()?;
            }
            ApplicationLayerState::WaitFallback => {
                // Wait for fallback completion
                if !self.state_machine.context().communication_active {
                    let _ = self.state_machine.process_event(ApplicationLayerEvent::Initialize);
                }
            }
            ApplicationLayerState::Error => {
                // Handle error condition
                self.handle_error()?;
            }
        }

        Ok(())
    }

    /// Process pending events
    fn process_events(&mut self) -> IoLinkResult<()> {
        // This would typically process incoming messages and events
        // Implementation depends on the specific protocol requirements
        Ok(())
    }

    /// Handle error conditions
    fn handle_error(&mut self) -> IoLinkResult<()> {
        self.state_machine.context_mut().error_count += 1;
        
        // Reset after error handling
        let _ = self.state_machine.process_event(ApplicationLayerEvent::Initialize);
        
        Ok(())
    }

    /// Add an event to the queue
    pub fn add_event(&mut self, event: Event) -> IoLinkResult<()> {
        self.event_queue.push_back(event)
            .map_err(|_| IoLinkError::BufferOverflow)
    }

    /// Set device identification
    pub fn set_device_id(&mut self, device_id: crate::types::DeviceIdentification) {
        self.device_id = device_id;
    }

    /// Set minimum cycle time
    pub fn set_min_cycle_time(&mut self, cycle_time: u8) {
        self.min_cycle_time = cycle_time;
    }

    /// Get current state
    pub fn state(&self) -> ApplicationLayerState {
        *self.state_machine.state()
    }
}

impl ApplicationLayer for ApplicationLayerImpl {
    fn al_get_input_req(&mut self) -> IoLinkResult<ProcessData> {
        if *self.state_machine.state() != ApplicationLayerState::Operate {
            return Err(IoLinkError::DeviceNotReady);
        }
        Ok(self.process_data.clone())
    }

    fn al_set_output_req(&mut self, data: &ProcessData) -> IoLinkResult<()> {
        if *self.state_machine.state() != ApplicationLayerState::Operate {
            return Err(IoLinkError::DeviceNotReady);
        }
        self.process_data.output = data.output.clone();
        Ok(())
    }

    fn al_read_req(&mut self, index: u16, sub_index: u8) -> IoLinkResult<Vec<u8, 32>> {
        if *self.state_machine.state() != ApplicationLayerState::Operate {
            return Err(IoLinkError::DeviceNotReady);
        }
        
        // Implementation would read from parameter storage
        // For now, return empty data
        Ok(Vec::new())
    }

    fn al_write_req(&mut self, _index: u16, _sub_index: u8, _data: &[u8]) -> IoLinkResult<()> {
        if *self.state_machine.state() != ApplicationLayerState::Operate {
            return Err(IoLinkError::DeviceNotReady);
        }
        
        // Implementation would write to parameter storage
        Ok(())
    }

    fn al_get_event_req(&mut self) -> IoLinkResult<Option<Event>> {
        Ok(self.event_queue.pop_front())
    }

    fn al_control_req(&mut self, _control_code: u8) -> IoLinkResult<()> {
        // Implementation would handle control codes
        Ok(())
    }

    fn al_get_device_id_req(&mut self) -> IoLinkResult<crate::types::DeviceIdentification> {
        Ok(self.device_id.clone())
    }

    fn al_get_min_cycle_time_req(&mut self) -> IoLinkResult<u8> {
        Ok(self.min_cycle_time)
    }

    fn al_abort_req(&mut self) -> IoLinkResult<()> {
        // Implementation would abort current operations
        let _ = self.state_machine.process_event(ApplicationLayerEvent::Error);
        Ok(())
    }
}
