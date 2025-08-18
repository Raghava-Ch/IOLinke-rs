use modular_bitfield::prelude::*;

use crate::{IoLinkError, IoLinkResult};

const MAX_EVENT_MEMORY_SIZE: usize = 19;

/// See A.6.2 StatusCode type 1 (no details)
/// Figure A.21 shows the structure of this StatusCode.
#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StatusCodeType1 {
    event_details: B1,
    pd_valid: B1,
    reserved: B1,
    event_code: B5,
}

/// A.6.3 StatusCode type 2 (with details)
/// Figure A.22 shows the structure of the StatusCode type 2.
#[bitfield]
#[derive(Debug, Clone, PartialEq, Eq)]
struct StatusCodeType2 {
    event_details: B1,
    reserved: B1,
    activated_event_slot1: B1,
    activated_event_slot2: B1,
    activated_event_slot3: B1,
    activated_event_slot4: B1,
    activated_event_slot5: B1,
    activated_event_slot6: B1,
}

/// See A.6.4 EventQualifier
/// The structure of the EventQualifier is shown in Figure A.24.
#[bitfield]
#[derive(Specifier, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventQualifier {
    #[bits = 2]
    pub eq_mode: EventMode,
    #[bits = 2]
    pub eq_type: EventType,
    #[bits = 1]
    pub eq_source: EventSource,
    #[bits = 3]
    pub eq_instance: EventInstance,
}


/// EventQualifier INSTANCE field (Bits 0..=2)
/// See Table A.17 – Values of INSTANCE
#[derive(Specifier, Debug, Clone, Copy, PartialEq, Eq)]
#[bits = 3]
pub enum EventInstance {
    Unknown = 0,
    // Reserved = 1..=3,
    Application = 4,
    System = 5,
    // Reserved = 6..=7,
}

/// EventQualifier SOURCE field (Bits 3)
/// See Table A.18 – Values of SOURCE
#[derive(Specifier, Debug, Clone, Copy, PartialEq, Eq)]
#[bits = 1]
pub enum EventSource {
    /// Device (remote)
    Device = 0,
    /// Master/Port
    Master = 1,
}

/// EventQualifier TYPE field (Bits 4..=5)
/// See Table A.19 – Values of TYPE
#[derive(Specifier, Debug, Clone, Copy, PartialEq, Eq)]
#[bits = 2]
pub enum EventType {
    // Reserved = 0
    Notification = 1,
    Warning = 2,
    Error = 3,
}

/// EventQualifier MODE field (Bits 6..=7)
/// See Table A.20 – Values of MODE
#[derive(Specifier, Debug, Clone, Copy, PartialEq, Eq)]
#[bits = 2]
pub enum EventMode {
    // Reserved = 0,
    SingleShot = 1,
    Disappears = 2,
    Appears = 3,
}



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventEntry {
    pub event_qualifier: EventQualifier,
    pub event_code: u16, // device_event_code macro to be used
}

impl EventEntry {
    pub fn new(event_qualifier: EventQualifier, event_code: u16) -> Self {
        Self {
            event_qualifier,
            event_code,
        }
    }

    /// Convert EventEntry to bytes representation
    pub fn to_bytes(&self) -> [u8; 3] {
        let mut bytes = [0u8; 3];
        bytes[0] = (self.event_qualifier.into_bytes())[0];
        bytes[1] = (self.event_code & 0xFF) as u8;
        bytes[2] = ((self.event_code >> 8) & 0xFF) as u8;
        bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EventCode {
    StatusCodeType1(StatusCodeType1),
    StatusCodeType2(StatusCodeType2),
}

/// See 7.3.8.1 Events and Table 58 – Event memory
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventMemory {
    event_code: EventCode,
    events: heapless::Vec<u8, 19>,
    read_only: bool,
}

impl EventMemory {
    /// Create a new EventMemory instance
    pub fn new() -> Self {
        Self {
            event_code: EventCode::StatusCodeType1(StatusCodeType1::new()),
            events: heapless::Vec::new(),
            read_only: false,
        }
    }

    /// Add an event entry to the memory
    pub fn add_event_details(&mut self, entries: &[EventEntry]) -> IoLinkResult<()> {
        if self.read_only {
            return Err(IoLinkError::ReadOnlyError);
        }

        // Push all entries to the self.events as bytes
        for entry in entries.iter() {
            let bytes = entry.to_bytes();
            // Only push if there is enough space left
            if self.events.len() + bytes.len() > self.events.capacity() {
                return Err(IoLinkError::EventMemoryFull);
            }
            for b in bytes.iter() {
                self.events.push(*b).map_err(|_| IoLinkError::EventMemoryFull)?;
            }
        }
        Ok(())
    }

    pub fn clear_all_event(&mut self) -> IoLinkResult<()> {
        if self.read_only {
            return Err(IoLinkError::ReadOnlyError);
        }
        // Clear all event entries
        self.events.clear();
        // Reset the event code to StatusCodeType1
        self.event_code = EventCode::StatusCodeType1(StatusCodeType1::new());
        Ok(())
    }

    pub fn get_event_detail(&self, address: usize, length: usize) -> IoLinkResult<&[u8]> {
        if address + length > MAX_EVENT_MEMORY_SIZE {
            return Err(IoLinkError::InvalidAddress);
        }
        if self.events.is_empty() {
            return Err(IoLinkError::InvalidEvent);
        }
        // Extract the event entry from the events vector
        self.events.get(address..address + length).ok_or(IoLinkError::InvalidData)
    }

    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }
}