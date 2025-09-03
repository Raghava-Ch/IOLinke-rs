use crate::{utils::event_type::{EventQualifier, StatusCodeType1, StatusCodeType2}, IoLinkError, IoLinkResult};

const MAX_EVENT_MEMORY_SIZE: usize = 19;


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
        bytes[0] = self.event_qualifier.into_bits();
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