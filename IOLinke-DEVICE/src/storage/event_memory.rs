use iolinke_types::custom::{IoLinkError, IoLinkResult};
use iolinke_types::handlers::event::{EventEntry, StatusCodeType1, StatusCodeType2};

use core::result::Result::{Err, Ok};

const MAX_EVENT_MEMORY_SIZE: usize = 19;

#[derive(Debug, Clone, PartialEq, Eq)]
enum EventCode {
    StatusCodeType1(StatusCodeType1),
    _StatusCodeType2(StatusCodeType2),
}

/// See 7.3.8.1 Events and Table 58 – Event memory
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventMemory {
    event_code: EventCode,
    events: heapless::Vec<u8, MAX_EVENT_MEMORY_SIZE>,
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
                self.events
                    .push(*b)
                    .map_err(|_| IoLinkError::EventMemoryFull)?;
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
        self.events
            .get(address..address + length)
            .ok_or(IoLinkError::InvalidData)
    }

    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }
}
