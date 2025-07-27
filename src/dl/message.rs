//! Message Handler
//!
//! This module implements the Message Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 6.3

use crate::types;
use crate::types::{IoLinkError, IoLinkResult, MessageType};
use crate::{dl, pl, MHInfo, MhConfState, Timer};
use heapless::Vec;
use nom::{bytes::complete::take, number::complete::u8, IResult};

pub trait OdInd {
    /// Invoke OD.ind service with the provided data
    fn od_ind(
        &mut self,
        rw_direction: types::RwDirection,
        com_channel: types::ComChannel,
        address_ctrl: u8,
        length: u8,
        data: &[u8],
    ) -> IoLinkResult<()>;
}

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
enum MessageHandlerState {
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
enum Transition {
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
enum MessageHandlerEvent {
    /// Table 47 – T1 T11
    MsgHandlerConf(MhConfState),
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

/// Trait for message handler operations in bw modules
pub trait MsgHandlerInfo {
    /// See 7.2.2.6 MHInfo
    /// The service MHInfo signals an exceptional operation within the message handler. The
    /// parameters of the service are listed in Table 39.
    fn mh_info(&mut self, mh_info: MHInfo);
}

/// Message Handler implementation
pub struct MessageHandler {
    state: MessageHandlerState,
    exec_transition: Transition,
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
            exec_transition: Transition::Tn,
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

        let (new_transition, new_state) = match (self.state, event) {
            (State::Inactive, Event::MsgHandlerConf(conf)) => {
                if conf == MhConfState::Active {
                    (Transition::T1, State::Idle)
                } else {
                    (Transition::Tn, State::Inactive)
                }
            }
            (State::Idle, Event::PlTransfer) => {
                (Transition::T2, State::GetMessage)
            }
            (State::Idle, Event::TimerElapsed(timer)) => {
                if timer == Timer::MaxCycleTime {
                    (Transition::T10, State::Idle)
                } else {
                    return Err(IoLinkError::InvalidParameter);
                }
            }
            (State::GetMessage, Event::PlTransfer) => {
                (Transition::T3, State::GetMessage)
            }
            (State::GetMessage, Event::Completed) => {
                (Transition::T4, State::CheckMessage)
            }
            (State::GetMessage, Event::TimerElapsed(timer)) => {
                if timer == Timer::MaxUARTFrameTime {
                    (Transition::T9, State::Idle)
                } else {
                    return Err(IoLinkError::InvalidParameter);
                }
            }
            (State::CheckMessage, Event::NoError) => {
                (Transition::T5, State::CreateMessage)
            }
            (State::CheckMessage, Event::ChecksumError) => {
                (Transition::T7, State::Idle)
            }
            (State::CheckMessage, Event::TypeError) => {
                (Transition::T8, State::Idle)
            }
            (State::CreateMessage, Event::Ready) => {
                (Transition::T6, State::Idle)
            }
            _ => return Err(IoLinkError::InvalidParameter),
        };

        self.exec_transition = new_transition;
        self.state = new_state;
        Ok(())
    }

    /// Poll the message handler
    /// See IO-Link v1.1.4 Section 6.3
    pub fn poll(
        &mut self,
        dl_mode: &mut dl::dl_mode::DlModeHandler,
        physical_layer: &mut pl::physical_layer::PhysicalLayer,
    ) -> IoLinkResult<()> {
        match self.exec_transition {
            Transition::Tn => {
                // No transition, remain in current state
            }
            Transition::T1 => {
                self.exec_transition = Transition::Tn;
                self.execute_t1()?;
            }
            Transition::T2 => {
                self.exec_transition = Transition::Tn;
                self.execute_t2()?;
            }
            Transition::T3 => {
                self.exec_transition = Transition::Tn;
                self.execute_t3()?;
            }
            Transition::T4 => {
                self.exec_transition = Transition::Tn;
                self.execute_t4()?;
            }
            Transition::T5 => {
                self.exec_transition = Transition::Tn;
                self.execute_t5()?;
            }
            Transition::T6 => {
                self.exec_transition = Transition::Tn;
                self.execute_t6(physical_layer)?;
            }
            Transition::T7 => {
                self.exec_transition = Transition::Tn;
                self.execute_t7()?;
            }
            Transition::T8 => {
                self.exec_transition = Transition::Tn;
                self.execute_t8(dl_mode)?;
            }
            Transition::T9 => {
                self.exec_transition = Transition::Tn;
                self.execute_t9()?;
            }
            Transition::T10 => {
                self.exec_transition = Transition::Tn;
                self.execute_t10()?;
            }
            Transition::T11 => {
                self.exec_transition = Transition::Tn;
                self.execute_t11()?;
            }
        }
        Ok(())
    }

    /// Transition T1: Inactive -> Idle (activation by DL-mode handler)
    /// See Table 47 – State transition tables of the Device message handler
    fn execute_t1(&mut self) -> IoLinkResult<()> {
        self.exec_transition = Transition::Tn;
        Ok(())
    }

    /// Transition T2: Idle -> GetMessage (start timers)
    /// Start "MaxUARTframeTime" and "MaxCycleTime" when in OPERATE
    fn execute_t2(&mut self) -> IoLinkResult<()> {
        // Start MaxUARTframeTime and MaxCycleTime timers (handled externally)
        self.exec_transition = Transition::Tn;
        Ok(())
    }

    /// Transition T3: GetMessage -> GetMessage (restart MaxUARTframeTime)
    /// Restart timer "MaxUARTframeTime"
    fn execute_t3(&mut self) -> IoLinkResult<()> {
        // Timer restart handled externally
        self.exec_transition = Transition::Tn;
        Ok(())
    }

    /// Transition T4: GetMessage -> CheckMessage (reset MaxUARTframeTime)
    /// Reset timer "MaxUARTframeTime"
    fn execute_t4(&mut self) -> IoLinkResult<()> {
        self.exec_transition = Transition::Tn;
        Ok(())
    }

    /// Transition T5: CheckMessage -> CreateMessage
    /// Invoke OD.ind and PD.ind service indications
    fn execute_t5(&mut self) -> IoLinkResult<()> {
        // OD.ind and PD.ind service indications handled externally
        self.exec_transition = Transition::Tn;
        Ok(())
    }

    /// Transition T6: CreateMessage -> Idle
    /// Compile and invoke PL_Transfer.rsp service response (Device sends response message)
    fn execute_t6(
        &mut self,
        physical_layer: &mut pl::physical_layer::PhysicalLayer,
    ) -> IoLinkResult<()> {
        // Compile and send response via PL_Transfer.rsp (handled externally)
        physical_layer.pl_transfer_req(&self.tx_buffer)?;
        self.exec_transition = Transition::Tn;
        Ok(())
    }

    /// Transition T7: CheckMessage -> Idle
    /// No message (e.g., timeout)
    fn execute_t7(&mut self) -> IoLinkResult<()> {
        self.exec_transition = Transition::Tn;
        Ok(())
    }

    /// Transition T8: CheckMessage -> Idle
    /// Indicate error to DL-mode handler via MHInfo (ILLEGAL_MESSAGETYPE)
    fn execute_t8(&mut self, dl_mode: &mut dl::dl_mode::DlModeHandler) -> IoLinkResult<()> {
        dl_mode.mh_info(MHInfo::IllegalMessagetype);
        Ok(())
    }

    /// Transition T9: GetMessage -> Idle
    /// Reset both timers "MaxUARTframeTime" and "MaxCycleTime"
    fn execute_t9(&mut self) -> IoLinkResult<()> {
        self.exec_transition = Transition::Tn;
        Ok(())
    }

    /// Transition T10: Idle -> Idle
    /// Indicate error to actuator technology that shall observe this information
    /// and take corresponding actions (see 10.2 and 10.8.3)
    fn execute_t10(&mut self) -> IoLinkResult<()> {
        // Indicate error to actuator technology (see 10.2, 10.8.3)
        self.exec_transition = Transition::Tn;
        Ok(())
    }

    /// Transition T11: Idle -> Inactive
    /// Device message handler changes state to Inactive_0
    fn execute_t11(&mut self) -> IoLinkResult<()> {
        self.exec_transition = Transition::Tn;
        Ok(())
    }

    /// Any MasterCommand received by the Device command handler
    /// (see Table 44 and Figure 54, state "CommandHandler_2")
    pub fn timer_update(&mut self, timer: Timer) {
        let _ = self.process_event(MessageHandlerEvent::TimerElapsed(timer));
    }

    /// This call causes the message handler to send a message with the
    /// requested transmission rate of COMx and with M-sequence TYPE_0 (see Table 46).
    pub fn mh_conf_update(&mut self, mh_conf: MhConfState) {
        let _ = self.process_event(MessageHandlerEvent::MsgHandlerConf(mh_conf));
    }

    /// See 7.2.2.4 EventFlag
    /// The EventFlag service sets or signals the status of 
    /// the "Event flag" (see A.1.5) during cyclic
    /// communication. The parameters of the service primitives are listed in Table 37.
    pub fn event_flag(flag: bool) {
        // Update the checksum / status (CKS) bit 7 wrt to the flag argument
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

    /// 7.2.2.5 PDInStatus
    /// The service PDInStatus sets and signals the validity qualifier
    /// of the input Process Data.
    pub fn pd_in_status_req(&mut self, valid: types::PdInStatus) -> IoLinkResult<()> {
        match valid {
            types::PdInStatus::VALID => self.process_event(MessageHandlerEvent::NoError),
            types::PdInStatus::INVALID => self.process_event(MessageHandlerEvent::ChecksumError),
        };
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

impl pl::physical_layer::PhysicalLayerInd for MessageHandler {
    fn pl_transfer_ind(&mut self, rx_buffer: &mut [u8]) -> IoLinkResult<()> {
        let _ = self.process_event(MessageHandlerEvent::PlTransfer);
        Ok(())
    }
}

/// Parse IO-Link frame using nom
/// See IO-Link v1.1.4 Section 6.1
fn parse_iolink_frame(input: &[u8]) -> IResult<&[u8], IoLinkMessage> {
    let (input, message_type_raw) = u8(input)?;
    let (input, channel) = u8(input)?;
    let (input, length) = u8(input)?;

    if length > MAX_MESSAGE_SIZE as u8 {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::TooLarge,
        )));
    }

    let (input, data_bytes) = take(length)(input)?;
    let (input, checksum) = u8(input)?;

    // Convert message type
    let message_type = match message_type_raw & 0x03 {
        0 => MessageType::ProcessData,
        1 => MessageType::DeviceCommand,
        2 => MessageType::ParameterCommand,
        _ => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Alt,
            )))
        }
    };

    // Create data vector
    let mut data = Vec::new();
    for &byte in data_bytes {
        data.push(byte).map_err(|_| {
            nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::TooLarge,
            ))
        })?;
    }

    // Verify checksum (simple XOR for example)
    let calculated_checksum = calculate_checksum(message_type_raw, channel, length, &data);
    if calculated_checksum != checksum {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        )));
    }

    Ok((
        input,
        IoLinkMessage {
            message_type,
            channel,
            data,
            checksum,
        },
    ))
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
