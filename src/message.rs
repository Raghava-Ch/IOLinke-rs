//! Message Handler
//!
//! This module implements the Message Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 6.3

use crate::{types::{IoLinkError, IoLinkResult, MessageType}, MHConf, Timer};
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
    /// Waiting for activation by the Device DL-mode handler through MH_Conf_ACTIVE 
    /// (see Table 45, Transition T1).
    Inactive,
    /// Waiting on first UART frame of the Master message through PL_Transfer service
    /// indication. Check whether time "MaxCycleTime" elapsed.
    Idle,
    /// Receive a Master message UART frame. Check number of received UART frames
    /// (Device detects M-sequence type by means of the first two received octets depending
    /// on the current communication state and thus knows the number of the UART frames).
    /// Check whether the time "MaxUARTframeTime" elapsed.
    GetMessage,
    /// Check M-sequence type and checksum of received message.
    CheckMessage,
    /// Compile message from OD.rsp, PD.rsp, EventFlag, and PDStatus services.
    CreateMessage,
}

/// See Table 47 – State transition tables of the Device message handler
#[derive(Debug, PartialEq, Eq)]
enum Transitions {
    /// Nothing to transit.
    Tn,
    /// 0 1 –
    T1,
    /// 1 2 Start "MaxUARTframeTime" and "MaxCycleTime" when in OPERATE.
    T2,
    /// 2 2 Restart timer "MaxUARTframeTime".
    T3,
    /// 2 3 Reset timer "MaxUARTframeTime".
    T4,
    /// 3 4 Invoke OD.ind and PD.ind service indications
    T5,
    /// 4 1 Compile and invoke PL_Transfer.rsp service response 
    /// (Device sends response message)
    T6,
    /// 3 1 –
    T7,
    /// 3 1 Indicate error to DL-mode handler via MHInfo (ILLEGAL_MESSAGETYPE)
    T8,
    /// 2 1 Reset both timers "MaxUARTframeTime" and "MaxCycleTime".
    T9,
    /// 1 1 Indicate error to actuator technology that shall observe 
    /// this information and take corresponding actions (see 10.2 and 10.8.3).
    T10,
    /// 1 0 Device message handler changes state to Inactive_0.
    T11,
}

/// Message Handler events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageHandlerEvent {
    /// Table 47 – T1 T11
    MsgHandlerConf(MHConf),
    /// See 5.2.2.3 PL_Transfer
    /// Table 47 – T2, T3
    PlTransfer,
    /// Table 47 – T4
    Completed,
    /// Table 47 – T5
    NoError,
    /// Table 47 – T6
    Ready,
    /// Table 47 – T7
    ChecksumError,
    /// Table 47 – T8
    TypeError,
    /// Table 47 – T9, T10
    TimerElapsed(Timer),
}

/// Message Handler implementation
pub struct MessageHandler {
    state: MessageHandlerState,
    exec_transition: Transitions,
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
            exec_transition: Transitions::Tn,
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
            (State::Inactive, Event::MsgHandlerConf(conf)) => {
                if conf == MHConf::Active {
                    self.exec_transition = Transitions::T1;
                    State::Idle
                } else {
                    State::Inactive
                }
            },
            (State::Idle, Event::PlTransfer) => {
                self.exec_transition = Transitions::T2;
                State::GetMessage
            },
            (State::Idle, Event::TimerElapsed(timer)) => {
                if timer == Timer::MaxCycleTime {
                    self.exec_transition = Transitions::T10;
                    State::Idle
                } else {
                    return Err(IoLinkError::InvalidParameter);
                }
            },
            (State::GetMessage, Event::PlTransfer) => {
                self.exec_transition = Transitions::T3;
                State::GetMessage
            },
            (State::GetMessage, Event::Completed) => {
                self.exec_transition = Transitions::T4;
                State::CheckMessage
            },
            (State::GetMessage, Event::TimerElapsed(timer)) => {
                if timer == Timer::MaxUARTFrameTime {
                    self.exec_transition = Transitions::T9;
                    State::Idle
                } else {
                    return Err(IoLinkError::InvalidParameter);
                }
            },
            (State::CheckMessage, Event::NoError) => {
                self.exec_transition = Transitions::T5;
                State::CreateMessage
            },
            (State::CheckMessage, Event::ChecksumError) => {
                self.exec_transition = Transitions::T7;
                State::Idle
            },
            (State::CheckMessage, Event::TypeError) => {
                self.exec_transition = Transitions::T8;
                State::Idle
            },
            (State::CreateMessage, Event::Ready) => {
                self.exec_transition = Transitions::T6;
                State::Idle
            },
            _ => return Err(IoLinkError::InvalidParameter),
        };

        self.state = new_state;
        Ok(())
    }

    /// Poll the message handler
    /// See IO-Link v1.1.4 Section 6.3
    pub fn poll(&mut self) -> IoLinkResult<()> {
        match self.exec_transition {
        Transitions::Tn => {
            // No transition, remain in current state
        }
        Transitions::T1 => {
            // Inactive -> Idle (activation by DL-mode handler)
            self.state = MessageHandlerState::Idle;
            self.exec_transition = Transitions::Tn;
        }
        Transitions::T2 => {
            // Idle -> GetMessage (start timers)
            self.state = MessageHandlerState::GetMessage;
            // Start MaxUARTframeTime and MaxCycleTime timers (handled externally)
            self.exec_transition = Transitions::Tn;
        }
        Transitions::T3 => {
            // GetMessage -> GetMessage (restart MaxUARTframeTime)
            // Timer restart handled externally
            self.exec_transition = Transitions::Tn;
        }
        Transitions::T4 => {
            // GetMessage -> CheckMessage (reset MaxUARTframeTime)
            self.state = MessageHandlerState::CheckMessage;
            self.exec_transition = Transitions::Tn;
        }
        Transitions::T5 => {
            // CheckMessage -> CreateMessage (invoke OD.ind, PD.ind)
            self.state = MessageHandlerState::CreateMessage;
            // OD.ind and PD.ind service indications handled externally
            self.exec_transition = Transitions::Tn;
        }
        Transitions::T6 => {
            // CreateMessage -> Idle (send response)
            self.state = MessageHandlerState::Idle;
            // Compile and send response via PL_Transfer.rsp (handled externally)
            self.exec_transition = Transitions::Tn;
        }
        Transitions::T7 => {
            // CheckMessage -> Idle (no message, e.g. timeout)
            self.state = MessageHandlerState::Idle;
            self.exec_transition = Transitions::Tn;
        }
        Transitions::T8 => {
            // CheckMessage -> Idle (illegal message type)
            self.state = MessageHandlerState::Idle;
            self.error_count += 1;
            // Indicate error to DL-mode handler via MHInfo (ILLEGAL_MESSAGETYPE)
            self.exec_transition = Transitions::Tn;
        }
        Transitions::T9 => {
            // GetMessage -> Idle (timeout, reset timers)
            self.state = MessageHandlerState::Idle;
            self.exec_transition = Transitions::Tn;
        }
        Transitions::T10 => {
            // Idle -> Idle (error indication to actuator tech)
            // Indicate error to actuator technology (see 10.2, 10.8.3)
            self.exec_transition = Transitions::Tn;
        }
        Transitions::T11 => {
            // Idle -> Inactive (deactivation)
            self.state = MessageHandlerState::Inactive;
            self.exec_transition = Transitions::Tn;
        }
        }
        Ok(())
    }

    /// 5.2.2.3 PL_Transfer
    /// The PL-Transfer service is used to exchange the SDCI data between Data Link Layer and
    /// Physical Layer. The parameters of the service primitives are listed in Table 4.
    pub fn pl_transfer(&mut self) {
        let _ = self.process_event(MessageHandlerEvent::PlTransfer);
    }

    /// Any MasterCommand received by the Device command handler
    /// (see Table 44 and Figure 54, state "CommandHandler_2")
    pub fn timer_update(&mut self, timer: Timer) {
        let _ = self.process_event(MessageHandlerEvent::TimerElapsed(timer));
    }

    /// This call causes the message handler to send a message with the
    /// requested transmission rate of COMx and with M-sequence TYPE_0 (see Table 46).
    pub fn mh_conf_update(&mut self, mh_conf: MHConf) {
        let _ = self.process_event(MessageHandlerEvent::MsgHandlerConf(mh_conf));
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
