//! Message Handler
//!
//! This module implements the Message Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 6.3

use crate::dl::{self, od_handler};
use crate::pl::{self, physical_layer};
use crate::types::{self, IoLinkError, IoLinkResult, MHInfo, MessageType, MhConfState};
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
struct IoLinkMessage {
    /// Message type
    message_type: Option<MessageType>,
    /// Communication channel
    channel: Option<types::ComChannel>,
    /// Event flag
    event_flag: bool,
    /// On Request Data (OD) response data
    od: Option<Vec<u8, MAX_MESSAGE_SIZE>>,
    /// Process Data (PD) response data
    pd: Option<Vec<u8, MAX_MESSAGE_SIZE>>,
    /// Process Data input status
    pd_in_status: Option<types::PdInStatus>,
}

impl Default for IoLinkMessage {
    fn default() -> Self {
        Self {
            message_type: None,
            channel: None,
            event_flag: false,
            od: None,
            pd: None,
            pd_in_status: None,
        }
    }
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
    /// Table 47 – T1
    MhConfActive,
    /// Table 47 – T11
    MhConfInactive,
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
    /// Table 47 – T10
    TimerMaxCycle,
    /// Table 47 – T9
    TimerMaxUARTFrame
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
    current_message: IoLinkMessage,
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
            current_message: IoLinkMessage::default(),
            error_count: 0,
        }
    }

    /// Process an event
    fn process_event(&mut self, event: MessageHandlerEvent) -> IoLinkResult<()> {
        use MessageHandlerEvent as Event;
        use MessageHandlerState as State;

        let (new_transition, new_state) = match (self.state, event) {
            (State::Inactive, Event::MhConfActive) => {
                (Transition::T1, State::Idle)
            }
            (State::Inactive, Event::MhConfInactive) => {
                (Transition::Tn, State::Inactive)
            }
            (State::Idle, Event::PlTransfer) => (Transition::T2, State::GetMessage),
            (State::Idle, Event::TimerMaxCycle) => {
                (Transition::T10, State::Idle)
            }
            (State::GetMessage, Event::PlTransfer) => (Transition::T3, State::GetMessage),
            (State::GetMessage, Event::Completed) => (Transition::T4, State::CheckMessage),
            (State::GetMessage, Event::TimerMaxUARTFrame) => {
                (Transition::T9, State::Idle)
            }
            (State::CheckMessage, Event::NoError) => (Transition::T5, State::CreateMessage),
            (State::CheckMessage, Event::ChecksumError) => (Transition::T7, State::Idle),
            (State::CheckMessage, Event::TypeError) => (Transition::T8, State::Idle),
            (State::CreateMessage, Event::Ready) => (Transition::T6, State::Idle),
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
        event_handler: &mut dl::event_handler::EventHandler,
        isdu_handler: &mut dl::isdu_handler::IsduHandler,
        od_handler: &mut dl::od_handler::OnRequestDataHandler,
        mode_handler: &mut dl::mode_handler::DlModeHandler,
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
                self.execute_t2(
                    physical_layer,
                )?;
            }
            Transition::T3 => {
                self.exec_transition = Transition::Tn;
                self.execute_t3(
                    physical_layer,
                )?;
            }
            Transition::T4 => {
                self.exec_transition = Transition::Tn;
                self.execute_t4(
                    physical_layer,
                )?;
            }
            Transition::T5 => {
                self.exec_transition = Transition::Tn;
                self.execute_t5(
                    event_handler,
                    isdu_handler,
                    od_handler,
                )?;
            }
            Transition::T6 => {
                self.exec_transition = Transition::Tn;
                self.execute_t6(
                    physical_layer,
                )?;
            }
            Transition::T7 => {
                self.exec_transition = Transition::Tn;
                self.execute_t7()?;
            }
            Transition::T8 => {
                self.exec_transition = Transition::Tn;
                self.execute_t8(
                    mode_handler,
                )?;
            }
            Transition::T9 => {
                self.exec_transition = Transition::Tn;
                self.execute_t9(
                    physical_layer,
                )?;
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
        Ok(())
    }

    /// Transition T2: Idle -> GetMessage (start timers)
    /// Start "MaxUARTframeTime" and "MaxCycleTime" when in OPERATE
    fn execute_t2(
        &mut self,
        physical_layer: &mut pl::physical_layer::PhysicalLayer,
    ) -> IoLinkResult<()> {
        // Start MaxUARTframeTime and MaxCycleTime timers (handled externally)
        physical_layer.start_timer(pl::physical_layer::Timer::MaxCycleTime, 0)?;
        Ok(())
    }

    /// Transition T3: GetMessage -> GetMessage (restart MaxUARTframeTime)
    /// Restart timer "MaxUARTframeTime"
    fn execute_t3(
        &mut self,
        physical_layer: &mut pl::physical_layer::PhysicalLayer,
    ) -> IoLinkResult<()> {
        physical_layer.restart_timer(pl::physical_layer::Timer::MaxUARTframeTime, 0)?;
        Ok(())
    }

    /// Transition T4: GetMessage -> CheckMessage (reset MaxUARTframeTime)
    /// Reset timer "MaxUARTframeTime"
    fn execute_t4(
        &mut self,
        physical_layer: &mut pl::physical_layer::PhysicalLayer,
    ) -> IoLinkResult<()> {
        physical_layer.stop_timer(pl::physical_layer::Timer::MaxUARTframeTime)?;
        Ok(())
    }

    /// Transition T5: CheckMessage -> CreateMessage
    /// Invoke OD.ind and PD.ind service indications
    fn execute_t5(
        &mut self,
        event_handler: &mut dl::event_handler::EventHandler,
        isdu_handler: &mut dl::isdu_handler::IsduHandler,
        od_handler: &mut dl::od_handler::OnRequestDataHandler,
    ) -> IoLinkResult<()> {
        // OD.ind and PD.ind service indications handled externally
        let _ = event_handler.od_ind(
            types::RwDirection::Read,
            types::ComChannel::Page,
            0,
            0,
            &[],
        );
        let _ = isdu_handler.od_ind(
            types::RwDirection::Read,
            types::ComChannel::Page,
            0,
            0,
            &[],
        );
        let _ = od_handler.od_ind(
            types::RwDirection::Read,
            types::ComChannel::Page,
            0,
            0,
            &[],
        );

        todo!("Implement PD.ind service indication values");
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

        Ok(())
    }

    /// Transition T7: CheckMessage -> Idle
    /// No message (e.g., timeout)
    fn execute_t7(&mut self) -> IoLinkResult<()> {
        Ok(())
    }

    /// Transition T8: CheckMessage -> Idle
    /// Indicate error to DL-mode handler via MHInfo (ILLEGAL_MESSAGETYPE)
    fn execute_t8(
        &mut self,
        mode_handler: &mut dl::mode_handler::DlModeHandler,
    ) -> IoLinkResult<()> {
        // mode_handler.mh_info(MHInfo::IllegalMessagetype);
        // Error indication handled externally
        mode_handler.mh_info(MHInfo::IllegalMessagetype);
        Ok(())
    }

    /// Transition T9: GetMessage -> Idle
    /// Reset both timers "MaxUARTframeTime" and "MaxCycleTime"
    fn execute_t9(
        &mut self,
        physical_layer: &mut pl::physical_layer::PhysicalLayer,
    ) -> IoLinkResult<()> {
        physical_layer.stop_timer(pl::physical_layer::Timer::MaxUARTframeTime)?;
        physical_layer.stop_timer(pl::physical_layer::Timer::MaxCycleTime)?;
        Ok(())
    }

    /// Transition T10: Idle -> Idle
    /// Indicate error to actuator technology that shall observe this information
    /// and take corresponding actions (see 10.2 and 10.8.3)
    fn execute_t10(&mut self) -> IoLinkResult<()> {
        // TODO: Implement error indication logic
        // This could involve sending an error message or setting an error state
        // Indicate error to actuator technology (see 10.2, 10.8.3)
        Ok(())
    }

    /// Transition T11: Idle -> Inactive
    /// Device message handler changes state to Inactive_0
    fn execute_t11(&mut self) -> IoLinkResult<()> {
        Ok(())
    }

    /// This call causes the message handler to send a message with the
    /// requested transmission rate of COMx and with M-sequence TYPE_0 (see Table 46).
    pub fn mh_conf_update(&mut self, mh_conf: MhConfState) {
        let event = if mh_conf == MhConfState::Active {
            MessageHandlerEvent::MhConfActive
        } else {
            MessageHandlerEvent::MhConfInactive
        };
        let _ = self.process_event(event);
    }

    /// See 7.2.2.4 EventFlag
    /// The EventFlag service sets or signals the status of
    /// the "Event flag" (see A.1.5) during cyclic
    /// communication. The parameters of the service primitives are listed in Table 37.
    pub fn event_flag(&mut self, flag: bool) {
        self.current_message.event_flag = flag;
    }

    /// See 7.2.2.2 OD
    /// The OD service is used to set up the On-request Data for the next message to be sent. In
    /// turn, the confirmation of the service contains the data from the receiver. The parameters of
    /// the service primitives are listed in Table 35.
    pub fn od_rsp(&mut self, length: u8, data: &[u8]) -> IoLinkResult<()> {
        let od = if let Some(od) = &mut self.current_message.od {
            od
        } else {
            self.current_message.od = Some(Vec::new());
            self.current_message.od.as_mut().unwrap()
        };
        od.clear();
        od.extend_from_slice(data)
            .map_err(|_| IoLinkError::InvalidParameter)?;
        Ok(())
    }

    /// See 7.2.2.3 PD
    /// The PD service is used to setup the Process Data to be sent through the process
    /// communication channel. The confirmation of the service contains the data from the receiver.
    /// The parameters of the service primitives are listed in Table 36.
    pub fn pd_rsp(&mut self, length: u8, data: &[u8]) -> IoLinkResult<()> {
        let pd = if let Some(pd) = &mut self.current_message.pd {
            pd
        } else {
            self.current_message.pd = Some(Vec::new());
            self.current_message.pd.as_mut().unwrap()
        };
        pd.clear();
        pd.extend_from_slice(data)
            .map_err(|_| IoLinkError::InvalidParameter)?;
        Ok(())
    }

    /// 7.2.2.5 PDInStatus
    /// The service PDInStatus sets and signals the validity qualifier
    /// of the input Process Data.
    pub fn pd_in_status_req(&mut self, valid: types::PdInStatus) -> IoLinkResult<()> {
        let pd_instatus = if let Some(status) = &mut self.current_message.pd_in_status {
            status
        } else {
            self.current_message.pd_in_status = Some(types::PdInStatus::VALID);
            self.current_message.pd_in_status.as_mut().unwrap()
        };
        *pd_instatus = valid;
        Ok(())
    }

    /// Parse IO-Link message from buffer
    /// See IO-Link v1.1.4 Section 6.1
    fn parse_message(&self) -> Result<IoLinkMessage, IoLinkError> {
        // if self.rx_buffer.is_empty() {
        //     return Err(IoLinkError::InvalidFrame);
        // }

        // match parse_iolink_frame(&self.rx_buffer) {
        //     Ok((_, message)) => Ok(message),
        //     Err(_) => Err(IoLinkError::InvalidFrame),
        // }
        todo!("Implement message parsing logic using nom or similar crate");
    }

    /// Process a received message
    fn process_message(&mut self, message: &IoLinkMessage) -> IoLinkResult<()> {
        // match message.message_type {
        //     MessageType::ProcessData => {
        //         // Handle process data message
        //         self.handle_process_data(message)?;
        //     }
        //     MessageType::DeviceCommand => {
        //         // Handle device command
        //         self.handle_device_command(message)?;
        //     }
        //     MessageType::ParameterCommand => {
        //         // Handle parameter command
        //         self.handle_parameter_command(message)?;
        //     }
        //     _ => {
        //         return Err(IoLinkError::InvalidEvent);
        //     }
        // }
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
        self.current_message = IoLinkMessage::default();
    }
}

// Physical layer indication trait implementation would go here
// when the physical layer module is properly defined
impl pl::physical_layer::PhysicalLayerInd for MessageHandler {
    fn pl_transfer_ind(&mut self, rx_buffer: &mut [u8]) -> IoLinkResult<()> {
        let _ = self.process_event(MessageHandlerEvent::PlTransfer);
        Ok(())
    }
}

impl pl::physical_layer::IoLinkTimer for MessageHandler {
    /// Any MasterCommand received by the Device command handler
    /// (see Table 44 and Figure 54, state "CommandHandler_2")
    fn timer_elapsed(&mut self, timer: pl::physical_layer::Timer) -> bool {
        let event = match timer {
            pl::physical_layer::Timer::MaxCycleTime => {
                MessageHandlerEvent::TimerMaxCycle
            }
            pl::physical_layer::Timer::MaxUARTframeTime => {
                MessageHandlerEvent::TimerMaxUARTFrame
            },
            _ => return false,
            
        };
        let _ = self.process_event(event);
        true
    }
}

/// Parse IO-Link frame using nom
/// See IO-Link v1.1.4 Section 6.1
// fn parse_iolink_frame(input: &[u8]) -> IResult<&[u8], IoLinkMessage> {
//     let (input, message_type_raw) = u8(input)?;
//     let (input, channel) = u8(input)?;
//     let (input, length) = u8(input)?;

//     if length > MAX_MESSAGE_SIZE as u8 {
//         return Err(nom::Err::Error(nom::error::Error::new(
//             input,
//             nom::error::ErrorKind::TooLarge,
//         )));
//     }

//     let (input, data_bytes) = take(length)(input)?;
//     let (input, checksum) = u8(input)?;

//     // Convert message type
//     let message_type = match message_type_raw & 0x03 {
//         0 => MessageType::ProcessData,
//         1 => MessageType::DeviceCommand,
//         2 => MessageType::ParameterCommand,
//         _ => {
//             return Err(nom::Err::Error(nom::error::Error::new(
//                 input,
//                 nom::error::ErrorKind::Alt,
//             )))
//         }
//     };

//     // Create data vector
//     let mut data = Vec::new();
//     for &byte in data_bytes {
//         data.push(byte).map_err(|_| {
//             nom::Err::Error(nom::error::Error::new(
//                 input,
//                 nom::error::ErrorKind::TooLarge,
//             ))
//         })?;
//     }

//     // Verify checksum (simple XOR for example)
//     let calculated_checksum = calculate_checksum(message_type_raw, channel, length, &data);
//     if calculated_checksum != checksum {
//         return Err(nom::Err::Error(nom::error::Error::new(
//             input,
//             nom::error::ErrorKind::Verify,
//         )));
//     }

//     Ok((
//         input,
//         IoLinkMessage {
//             message_type,
//             channel,
//             data,
//             checksum,
//         },
//     ))
// }

// /// Calculate message checksum
// fn calculate_checksum(message_type: u8, channel: u8, length: u8, data: &[u8]) -> u8 {
//     let mut checksum = message_type ^ channel ^ length;
//     for &byte in data {
//         checksum ^= byte;
//     }
//     checksum
// }

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
