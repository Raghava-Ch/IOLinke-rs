//! Message Handler
//!
//! This module implements the Message Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 6.3

use crate::types::{IoLinkError, IoLinkResult, MessageType};
use heapless::Vec;
use nom::{
    bytes::complete::take,
    number::complete::u8,
    IResult,
};

/// Maximum message buffer size
const MAX_MESSAGE_SIZE: usize = 32;

/// IO-Link message structure
/// See IO-Link v1.1.4 Section 6.1
#[derive(Debug, Clone)]
pub struct IoLinkMessage {
    /// Message type
    pub message_type: MessageType,
    /// Communication channel
    pub channel: u8,
    /// Message data
    pub data: Vec<u8, MAX_MESSAGE_SIZE>,
    /// Checksum
    pub checksum: u8,
}

/// Message Handler states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageHandlerState {
    /// Idle state
    Idle,
    /// Receiving message
    Receiving,
    /// Processing message
    Processing,
    /// Sending response
    Sending,
    /// Error state
    Error,
}

/// Message Handler events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageHandlerEvent {
    /// Start receiving
    StartReceive,
    /// Message received
    MessageReceived,
    /// Start processing
    StartProcessing,
    /// Processing complete
    ProcessingComplete,
    /// Start sending
    StartSending,
    /// Sending complete
    SendingComplete,
    /// Error occurred
    Error,
    /// Reset
    Reset,
}

/// Message Handler implementation
pub struct MessageHandler {
    state: MessageHandlerState,
    rx_buffer: Vec<u8, MAX_MESSAGE_SIZE>,
    tx_buffer: Vec<u8, MAX_MESSAGE_SIZE>,
    current_message: Option<IoLinkMessage>,
    error_count: u32,
}

impl MessageHandler {
    /// Create a new Message Handler
    pub fn new() -> Self {
        Self {
            state: MessageHandlerState::Idle,
            rx_buffer: Vec::new(),
            tx_buffer: Vec::new(),
            current_message: None,
            error_count: 0,
        }
    }

    /// Process an event
    pub fn process_event(&mut self, event: MessageHandlerEvent) -> IoLinkResult<()> {
        use MessageHandlerEvent as Event;
        use MessageHandlerState as State;

        let new_state = match (self.state, event) {
            (State::Idle, Event::StartReceive) => State::Receiving,
            (State::Receiving, Event::MessageReceived) => State::Processing,
            (State::Receiving, Event::Error) => State::Error,
            (State::Processing, Event::ProcessingComplete) => State::Sending,
            (State::Processing, Event::Error) => State::Error,
            (State::Sending, Event::SendingComplete) => State::Idle,
            (State::Sending, Event::Error) => State::Error,
            (State::Error, Event::Reset) => State::Idle,
            _ => return Err(IoLinkError::InvalidParameter),
        };

        self.state = new_state;
        Ok(())
    }

    /// Poll the message handler
    /// See IO-Link v1.1.4 Section 6.3
    pub fn poll(&mut self) -> IoLinkResult<()> {
        match self.state {
            MessageHandlerState::Idle => {
                // Ready to receive new messages
                if !self.rx_buffer.is_empty() {
                    self.process_event(MessageHandlerEvent::StartReceive)?;
                }
            }
            MessageHandlerState::Receiving => {
                // Try to parse received data
                if let Ok(message) = self.parse_message() {
                    self.current_message = Some(message);
                    self.process_event(MessageHandlerEvent::MessageReceived)?;
                } else if self.rx_buffer.len() >= MAX_MESSAGE_SIZE {
                    // Buffer full but no valid message
                    self.process_event(MessageHandlerEvent::Error)?;
                }
            }
            MessageHandlerState::Processing => {
                // Process the current message
                if let Some(message) = self.current_message.take() {
                    self.process_message(&message)?;
                    self.process_event(MessageHandlerEvent::ProcessingComplete)?;
                }
            }
            MessageHandlerState::Sending => {
                // Send response (would integrate with physical layer)
                self.process_event(MessageHandlerEvent::SendingComplete)?;
            }
            MessageHandlerState::Error => {
                // Handle error condition
                self.error_count += 1;
                self.rx_buffer.clear();
                self.tx_buffer.clear();
                self.current_message = None;
                self.process_event(MessageHandlerEvent::Reset)?;
            }
        }
        Ok(())
    }

    /// Add received data to buffer
    pub fn add_rx_data(&mut self, data: &[u8]) -> IoLinkResult<()> {
        for &byte in data {
            self.rx_buffer.push(byte)
                .map_err(|_| IoLinkError::BufferOverflow)?;
        }
        Ok(())
    }

    /// Get transmit data
    pub fn get_tx_data(&mut self) -> Vec<u8, MAX_MESSAGE_SIZE> {
        let data = self.tx_buffer.clone();
        self.tx_buffer.clear();
        data
    }

    /// Parse IO-Link message from buffer
    /// See IO-Link v1.1.4 Section 6.1
    fn parse_message(&self) -> Result<IoLinkMessage, IoLinkError> {
        if self.rx_buffer.is_empty() {
            return Err(IoLinkError::InvalidFrame);
        }

        match parse_iolink_frame(&self.rx_buffer) {
            Ok((_, message)) => Ok(message),
            Err(_) => Err(IoLinkError::InvalidFrame),
        }
    }

    /// Process a received message
    fn process_message(&mut self, message: &IoLinkMessage) -> IoLinkResult<()> {
        match message.message_type {
            MessageType::ProcessData => {
                // Handle process data message
                self.handle_process_data(message)?;
            }
            MessageType::DeviceCommand => {
                // Handle device command
                self.handle_device_command(message)?;
            }
            MessageType::ParameterCommand => {
                // Handle parameter command
                self.handle_parameter_command(message)?;
            }
        }
        Ok(())
    }

    /// Handle process data message
    fn handle_process_data(&mut self, _message: &IoLinkMessage) -> IoLinkResult<()> {
        // Implementation would handle process data exchange
        Ok(())
    }

    /// Handle device command
    fn handle_device_command(&mut self, _message: &IoLinkMessage) -> IoLinkResult<()> {
        // Implementation would handle device commands
        Ok(())
    }

    /// Handle parameter command
    fn handle_parameter_command(&mut self, _message: &IoLinkMessage) -> IoLinkResult<()> {
        // Implementation would handle parameter access
        Ok(())
    }

    /// Get current state
    pub fn state(&self) -> MessageHandlerState {
        self.state
    }

    /// Clear buffers
    pub fn clear_buffers(&mut self) {
        self.rx_buffer.clear();
        self.tx_buffer.clear();
        self.current_message = None;
    }
}

/// Parse IO-Link frame using nom
/// See IO-Link v1.1.4 Section 6.1
fn parse_iolink_frame(input: &[u8]) -> IResult<&[u8], IoLinkMessage> {
    let (input, message_type_raw) = u8(input)?;
    let (input, channel) = u8(input)?;
    let (input, length) = u8(input)?;
    
    if length > MAX_MESSAGE_SIZE as u8 {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::TooLarge)));
    }
    
    let (input, data_bytes) = take(length)(input)?;
    let (input, checksum) = u8(input)?;
    
    // Convert message type
    let message_type = match message_type_raw & 0x03 {
        0 => MessageType::ProcessData,
        1 => MessageType::DeviceCommand,
        2 => MessageType::ParameterCommand,
        _ => return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Alt))),
    };
    
    // Create data vector
    let mut data = Vec::new();
    for &byte in data_bytes {
        data.push(byte).map_err(|_| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::TooLarge)))?;
    }
    
    // Verify checksum (simple XOR for example)
    let calculated_checksum = calculate_checksum(message_type_raw, channel, length, &data);
    if calculated_checksum != checksum {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify)));
    }
    
    Ok((input, IoLinkMessage {
        message_type,
        channel,
        data,
        checksum,
    }))
}

/// Calculate message checksum
fn calculate_checksum(message_type: u8, channel: u8, length: u8, data: &[u8]) -> u8 {
    let mut checksum = message_type ^ channel ^ length;
    for &byte in data {
        checksum ^= byte;
    }
    checksum
}

impl Default for MessageHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_parsing() {
        // Create a test message: Type 0, Channel 1, Length 2, Data [0xAA, 0xBB], Checksum
        let mut test_data = heapless::Vec::<u8, 16>::new();
        test_data.push(0x00).unwrap();
        test_data.push(0x01).unwrap();
        test_data.push(0x02).unwrap();
        test_data.push(0xAA).unwrap();
        test_data.push(0xBB).unwrap();
        
        let checksum = calculate_checksum(0x00, 0x01, 0x02, &[0xAA, 0xBB]);
        test_data.push(checksum).unwrap();

        let result = parse_iolink_frame(&test_data);
        assert!(result.is_ok());
        
        let (_, message) = result.unwrap();
        assert_eq!(message.message_type, MessageType::ProcessData);
        assert_eq!(message.channel, 1);
        assert_eq!(message.data.len(), 2);
        assert_eq!(message.data[0], 0xAA);
        assert_eq!(message.data[1], 0xBB);
    }

    #[test]
    fn test_message_handler_state_machine() {
        let mut handler = MessageHandler::new();
        assert_eq!(handler.state(), MessageHandlerState::Idle);

        // Add some test data
        handler.add_rx_data(&[0x00, 0x01, 0x02, 0xAA, 0xBB, 0xA8]).unwrap();
        
        // Poll should trigger state transitions
        handler.poll().unwrap();
        // Should be in Receiving state after poll
    }

    #[test]
    fn test_checksum_calculation() {
        let checksum = calculate_checksum(0x00, 0x01, 0x02, &[0xAA, 0xBB]);
        // Calculate manually: 0x00 ^ 0x01 ^ 0x02 ^ 0xAA ^ 0xBB
        // = 0x01 ^ 0x02 ^ 0xAA ^ 0xBB
        // = 0x03 ^ 0xAA ^ 0xBB  
        // = 0xA9 ^ 0xBB
        // = 0x12
        assert_eq!(checksum, 0x12);
    }
}
