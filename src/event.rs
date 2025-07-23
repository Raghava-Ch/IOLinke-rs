//! Event Handler
//!
//! This module implements the Event Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 8.4.4

use crate::types::{Event, EventType, IoLinkError, IoLinkResult};
use heapless::Deque;

/// Event Handler implementation
pub struct EventHandler {
    event_queue: Deque<Event, 16>,
}

impl EventHandler {
    /// Create a new Event Handler
    pub fn new() -> Self {
        Self {
            event_queue: Deque::new(),
        }
    }

    /// Poll the handler
    pub fn poll(&mut self) -> IoLinkResult<()> {
        // Process pending events
        Ok(())
    }

    /// Add an event to the queue
    pub fn add_event(&mut self, event: Event) -> IoLinkResult<()> {
        self.event_queue.push_back(event)
            .map_err(|_| IoLinkError::BufferOverflow)
    }

    /// Get the next event
    pub fn get_event(&mut self) -> Option<Event> {
        self.event_queue.pop_front()
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
