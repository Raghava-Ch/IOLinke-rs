//! Message Handler
//!
//! This module implements the Message Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 6.3

use crate::config;
use crate::dl;
use crate::pl;
use crate::types::{self, IoLinkError, IoLinkResult};
use crate::{
    construct_u8, get_bit_0, get_bit_1, get_bit_2, get_bit_3, get_bit_4, get_bit_5, get_bit_6,
    get_bit_7, get_bits_0_4, get_bits_5_6, get_bits_6_7, set_bit_6, set_bit_7, set_bits_0_5,
};

use heapless::Vec;

/// Mask to set bits 0-5 to zero while preserving bits 6-7
/// This macro clears the revceived checksum bits (0-5) in a byte,
/// leaving bits 6 and 7 unchanged.
#[macro_export]
macro_rules! clear_checksum_bits_0_to_5 {
    ($byte:expr) => {
        ($byte) & 0xC0
    };
}

/// Extract checksum bits (bits 0-5) from byte
#[macro_export]
macro_rules! extract_checksum_bits {
    ($byte:expr) => {
        ($byte) & 0x3F
    };
}

/// Extract RW direction from first byte (bit 2)
/// 0 = Read, 1 = Write
macro_rules! extract_rw_direction {
    ($byte:expr) => {
        get_bit_7!($byte)
    };
}

/// Extract communication channel from first byte (bits 5-6)
/// 00 = Process, 01 = Page, 10 = Diagnosis, 11 = ISDU
macro_rules! extract_com_channel {
    ($byte:expr) => {
        get_bits_5_6!($byte)
    };
}
/// Extract address from first byte for M-sequence byte (bits 0-4)
macro_rules! extract_address_fctrl {
    ($byte:expr) => {
        get_bits_0_4!($byte)
    };
}

/// Extract the message type (TYPE_0, TYPE_1, or TYPE_2)
macro_rules! extract_message_type {
    ($msg_type:expr) => {
        get_bits_6_7!($msg_type)
    };
}

/// Compile `Event flag` for CKS byte
macro_rules! compile_event_flag {
    ($data:expr, $event_flag:expr) => {
        set_bit_7!($data, $event_flag)
    };
}

/// Compile `PD status` for CKS byte
macro_rules! compile_pd_status {
    ($data:expr, $pd_status:expr) => {
        set_bit_6!($data, $pd_status)
    };
}

/// Compile Checksum byte Bit: 0 to Bit: 5
macro_rules! compile_checksum {
    ($data:expr, $checksum:expr) => {
        set_bits_0_5!($data, $checksum)
    };
}

/// See A.1.5 Checksum / status (CKS)
/// Compile Checksum / Status (CKS) byte
macro_rules! compile_checksum_status {
    ($data:expr, $event_flag:expr, $pd_status:expr, $checksum:expr) => {{
        compile_event_flag!($data, $event_flag);
        compile_pd_status!($data, $pd_status);
        compile_checksum!($data, $checksum);
        $data
    }};
}

// /// Extract payload data (excluding header and checksum)
// macro_rules! extract_payload {
//     ($data:expr, $start_idx:expr, $length:expr) => {
//         &$data[$start_idx..($start_idx + $length as usize)]
//     };
// }

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

const HEADER_SIZE: usize = 2; // Header size is 2 bytes (MC and length)
/// Maximum message buffer size for OD
/// This is the maximum size of the message buffer used for OD messages.
const MAX_OD_SIZE: usize = 32;
/// Maximum message buffer size for PD
/// This is the maximum size of the message buffer used for PD messages.
const MAX_PD_SIZE: usize = 32;
/// Maximum frame size for IO-Link messages
const MAX_FRAME_SIZE: usize = MAX_OD_SIZE + MAX_PD_SIZE + HEADER_SIZE;

/// IO-Link message structure
/// See IO-Link v1.1.4 Section 6.1
#[derive(Debug, Clone, PartialEq, Eq)]
struct IoLinkMessage {
    /// Read/Write direction
    read_write: Option<types::RwDirection>,
    /// Message type
    message_type: Option<types::MessageType>,
    /// Communication channel
    com_channel: Option<types::ComChannel>,
    /// Contains the address or flow control value (see A.1.2).
    address_fctrl: Option<u8>,
    /// Event flag
    event_flag: bool,
    /// On Request Data (OD) response data
    od: Option<Vec<u8, MAX_OD_SIZE>>,
    /// Process Data (PD) response data
    pd: Option<Vec<u8, MAX_PD_SIZE>>,
    /// Process Data input status
    pd_in_status: Option<types::PdInStatus>,
}

impl Default for IoLinkMessage {
    fn default() -> Self {
        Self {
            read_write: None,
            message_type: None,
            com_channel: None,
            address_fctrl: None,
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
    TimerMaxUARTFrame,
}

/// Trait for message handler operations in bw modules
pub trait MsgHandlerInfo {
    /// See 7.2.2.6 MHInfo
    /// The service MHInfo signals an exceptional operation within the message handler. The
    /// parameters of the service are listed in Table 39.
    fn mh_info(&mut self, mh_info: types::MHInfo);
}

struct Buffers {
    rx_buffer: [u8; MAX_FRAME_SIZE],
    rx_buffer_len: u8,
    tx_buffer: [u8; MAX_FRAME_SIZE],
    tx_buffer_len: u8,
}

/// Message Handler implementation
pub struct MessageHandler {
    state: MessageHandlerState,
    exec_transition: Transition,
    buffers: Buffers,
    tx_message: IoLinkMessage,
    rx_message: IoLinkMessage,
    device_operate_state: types::MasterCommand,
    od_transfer_pending: bool,
}

impl MessageHandler {
    /// Create a new Message Handler
    pub fn new() -> Self {
        Self {
            state: MessageHandlerState::Idle,
            exec_transition: Transition::Tn,
            buffers: Buffers {
                rx_buffer: [0; MAX_FRAME_SIZE],
                rx_buffer_len: 0,
                tx_buffer: [0; MAX_FRAME_SIZE],
                tx_buffer_len: 0,
            },
            tx_message: IoLinkMessage::default(),
            rx_message: IoLinkMessage::default(),
            device_operate_state: types::MasterCommand::INACTIVE,
            od_transfer_pending: false,
        }
    }

    /// Process an event
    fn process_event(&mut self, event: MessageHandlerEvent) -> IoLinkResult<()> {
        use MessageHandlerEvent as Event;
        use MessageHandlerState as State;

        let (new_transition, new_state) = match (self.state, event) {
            (State::Inactive, Event::MhConfActive) => (Transition::T1, State::Idle),
            (State::Inactive, Event::MhConfInactive) => (Transition::Tn, State::Inactive),
            (State::Idle, Event::PlTransfer) => (Transition::T2, State::GetMessage),
            (State::Idle, Event::TimerMaxCycle) => (Transition::T10, State::Idle),
            (State::GetMessage, Event::PlTransfer) => (Transition::T3, State::GetMessage),
            (State::GetMessage, Event::Completed) => (Transition::T4, State::CheckMessage),
            (State::GetMessage, Event::TimerMaxUARTFrame) => (Transition::T9, State::Idle),
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
                self.execute_t2(physical_layer)?;
            }
            Transition::T3 => {
                self.exec_transition = Transition::Tn;
                self.execute_t3(physical_layer)?;
            }
            Transition::T4 => {
                self.exec_transition = Transition::Tn;
                self.execute_t4(physical_layer)?;
            }
            Transition::T5 => {
                self.exec_transition = Transition::Tn;
                self.execute_t5(event_handler, isdu_handler, od_handler)?;
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
                self.execute_t8(mode_handler)?;
            }
            Transition::T9 => {
                self.exec_transition = Transition::Tn;
                self.execute_t9(physical_layer)?;
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

        match self.state {
            MessageHandlerState::CheckMessage => {
                let io_link_message = match self.parse_message() {
                    Ok(message) => message,
                    Err(e) => {
                        // Handle parsing error
                        match e {
                            IoLinkError::ChecksumError => {
                                self.process_event(MessageHandlerEvent::ChecksumError)?;
                                return Err(e);
                            }
                            IoLinkError::InvalidMseqType => {
                                self.process_event(MessageHandlerEvent::TypeError)?;
                                return Err(e);
                            }
                            _ => return Err(e),
                        }
                    }
                };
                self.rx_message = io_link_message;
                self.process_event(MessageHandlerEvent::NoError)?;
            }
            MessageHandlerState::CreateMessage => {
                // Check the response is ready to be sent
                if self.tx_message.od.is_none()
                    && self.tx_message.pd.is_none()
                    && self.tx_message.address_fctrl.is_none()
                    && self.tx_message.com_channel.is_none()
                    && self.tx_message.read_write.is_none()
                    && self.tx_message.message_type.is_none()
                    && self.tx_message.pd_in_status.is_none()
                {
                    // No data to send, transition to Idle
                } else {
                    // Data is ready, proceed to send
                    self.compile_message()?;
                    self.process_event(MessageHandlerEvent::Ready)?;
                }
            }
            _ => {
                // All other states are not expected to perform actions
                // No specific action required for other states
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
        if let (Some(rw), Some(channel), Some(addr_ctrl), Some(ref od_data)) = (
            self.rx_message.read_write,
            self.rx_message.com_channel,
            self.rx_message.address_fctrl,
            self.rx_message.od.as_ref(),
        ) {
            let _ = event_handler.od_ind(rw, channel, addr_ctrl, od_data.len() as u8, od_data);
            let _ = isdu_handler.od_ind(rw, channel, addr_ctrl, od_data.len() as u8, od_data);
            let _ = od_handler.od_ind(rw, channel, addr_ctrl, od_data.len() as u8, od_data);
            // TODO: Implement PD.ind service indication values
        }
        todo!("Implement PD.ind service indication values");
        Ok(())
    }

    /// Transition T6: CreateMessage -> Idle
    /// Compile and invoke PL_Transfer.rsp service response (Device sends response message)
    fn execute_t6(
        &mut self,
        physical_layer: &mut pl::physical_layer::PhysicalLayer,
    ) -> IoLinkResult<()> {
        // Compiled and send response via PL_Transfer.rsp (handled externally)
        physical_layer.pl_transfer_req(&self.buffers.tx_buffer)?;

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
        mode_handler.mh_info(types::MHInfo::IllegalMessagetype);
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
    pub fn mh_conf_update(&mut self, mh_conf: types::MhConfState) {
        let event = if mh_conf == types::MhConfState::Active {
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
        self.tx_message.event_flag = flag;
    }

    /// See 7.2.2.2 OD
    /// The OD service is used to set up the On-request Data for the next message to be sent. In
    /// turn, the confirmation of the service contains the data from the receiver. The parameters of
    /// the service primitives are listed in Table 35.
    pub fn od_rsp(&mut self, _length: u8, data: &[u8]) -> IoLinkResult<()> {
        let od = if let Some(od) = &mut self.tx_message.od {
            od
        } else {
            self.tx_message.od = Some(Vec::new());
            self.tx_message.od.as_mut().unwrap()
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
    pub fn pd_rsp(&mut self, _length: u8, data: &[u8]) -> IoLinkResult<()> {
        let pd = if let Some(pd) = &mut self.tx_message.pd {
            pd
        } else {
            self.tx_message.pd = Some(Vec::new());
            self.tx_message.pd.as_mut().unwrap()
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
        let pd_instatus = if let Some(status) = &mut self.tx_message.pd_in_status {
            status
        } else {
            self.tx_message.pd_in_status = Some(types::PdInStatus::VALID);
            self.tx_message.pd_in_status.as_mut().unwrap()
        };
        *pd_instatus = valid;
        Ok(())
    }

    /// Compile IO-Link message from buffer
    /// See IO-Link v1.1.4 `Annex A` (normative)
    /// Codings, timing constraints, and errors
    /// A.1 General structure and encoding of M-sequences
    fn compile_message(&mut self) -> IoLinkResult<()> {
        let io_link_message = &self.tx_message;
        let tx_buffer = &mut self.buffers.tx_buffer;
        *tx_buffer = [0; MAX_FRAME_SIZE];
        let length = match self.device_operate_state {
            types::MasterCommand::STARTUP => {
                compile_iolink_startup_frame(tx_buffer, io_link_message)?
            }
            types::MasterCommand::PREOPERATE => {
                compile_iolink_preoperate_frame(tx_buffer, io_link_message)?
            }
            types::MasterCommand::OPERATE => {
                compile_iolink_operate_frame(tx_buffer, io_link_message, self.od_transfer_pending)?
            }
            _ => return Err(IoLinkError::InvalidMseqType),
        };
        self.buffers.tx_buffer_len = length as u8;
        todo!("Implement IO-Link message compilation logic");
    }

    /// Parse IO-Link message from buffer
    /// See IO-Link v1.1.4 Section 6.1
    fn parse_message(&mut self) -> IoLinkResult<IoLinkMessage> {
        let rx_buffer: &mut [u8; 66] = &mut self.buffers.rx_buffer;
        if validate_checksum(rx_buffer.len() as u8, rx_buffer) == false {
            // If checksum is invalid, indicate error
            return Err(IoLinkError::ChecksumError);
        }
        let io_link_message = match self.device_operate_state {
            types::MasterCommand::STARTUP => {
                let m_seq_type: u8 = types::MsequenceType::Type0.into();
                let rxed_m_seq_base_type: u8 = extract_message_type!(rx_buffer[1]);
                if m_seq_type != rxed_m_seq_base_type {
                    return Err(IoLinkError::InvalidMseqType);
                }
                parse_iolink_startup_frame(rx_buffer)
            }
            types::MasterCommand::PREOPERATE => {
                const M_SEQ_TYPE: u8 = config::m_seq_capability::preoperate_m_sequence();
                let rxed_m_seq_base_type: u8 = extract_message_type!(rx_buffer[1]);
                if rxed_m_seq_base_type != M_SEQ_TYPE {
                    return Err(IoLinkError::InvalidMseqType);
                }
                parse_iolink_pre_operate_frame(rx_buffer)
            }
            types::MasterCommand::OPERATE => {
                const M_SEQ_TYPE: u8 = config::m_seq_capability::operate_m_sequence_base_type();
                let com_channel = types::ComChannel::try_from(extract_com_channel!(rx_buffer[0]))?;
                if com_channel != types::ComChannel::Process {
                    self.od_transfer_pending = true;
                }
                let rxed_m_seq_base_type: u8 = extract_message_type!(rx_buffer[1]);
                if rxed_m_seq_base_type != M_SEQ_TYPE {
                    return Err(IoLinkError::InvalidMseqType);
                }
                parse_iolink_operate_frame(rx_buffer)
            }
            _ => return Err(IoLinkError::InvalidMseqType),
        };

        io_link_message
    }

    /// Clear buffers
    fn clear_buffers(&mut self) {
        self.buffers.rx_buffer = [0; MAX_FRAME_SIZE];
        self.buffers.rx_buffer_len = 0;
        self.buffers.tx_buffer = [0; MAX_FRAME_SIZE];
        self.buffers.tx_buffer_len = 0;
        self.tx_message = IoLinkMessage::default();
    }
}

// Physical layer indication trait implementation would go here
// when the physical layer module is properly defined
impl pl::physical_layer::PhysicalLayerInd for MessageHandler {
    fn pl_transfer_ind(&mut self, rx_buffer: &mut [u8]) -> IoLinkResult<()> {
        self.buffers.rx_buffer = rx_buffer.try_into().map_err(|_| IoLinkError::InvalidData)?;
        let _ = self.process_event(MessageHandlerEvent::PlTransfer);
        Ok(())
    }
}

impl pl::physical_layer::IoLinkTimer for MessageHandler {
    /// Any MasterCommand received by the Device command handler
    /// (see Table 44 and Figure 54, state "CommandHandler_2")
    fn timer_elapsed(&mut self, timer: pl::physical_layer::Timer) -> bool {
        let event = match timer {
            pl::physical_layer::Timer::MaxCycleTime => MessageHandlerEvent::TimerMaxCycle,
            pl::physical_layer::Timer::MaxUARTframeTime => MessageHandlerEvent::TimerMaxUARTFrame,
            _ => return false,
        };
        let _ = self.process_event(event);
        true
    }
}

fn compile_iolink_startup_frame(
    tx_buffer: &mut [u8],
    io_link_message: &IoLinkMessage,
) -> Result<u8, IoLinkError> {
    tx_buffer[0] = io_link_message
        .od
        .as_ref()
        .ok_or(IoLinkError::InvalidData)?[0];
    tx_buffer[1] = compile_checksum_status!(
        tx_buffer[1],
        io_link_message.event_flag,
        io_link_message.pd_in_status,
        0 // Checksum will be calculated later
    );
    let checksum = calculate_checksum(2, &tx_buffer);
    tx_buffer[1] = compile_checksum!(tx_buffer[1], checksum);
    Ok(2)
}

fn compile_iolink_preoperate_frame(
    tx_buffer: &mut [u8],
    io_link_message: &IoLinkMessage,
) -> Result<u8, IoLinkError> {
    const OD_LENGTH: u8 = config::m_seq_capability::operate_m_sequence_legacy_od_len();
    if io_link_message.od.is_none() {
        return Err(IoLinkError::InvalidData);
    }
    for (i, &byte) in io_link_message.od.as_ref().unwrap().iter().enumerate() {
        if i < OD_LENGTH as usize {
            tx_buffer[i] = byte;
        } else {
            break; // Avoid out of bounds access
        }
    }
    tx_buffer[OD_LENGTH as usize] = compile_checksum_status!(
        tx_buffer[OD_LENGTH as usize],
        io_link_message.event_flag,
        io_link_message.pd_in_status,
        0 // Checksum will be calculated later
    );
    let checksum = calculate_checksum(OD_LENGTH + 1, &tx_buffer);
    tx_buffer[OD_LENGTH as usize] = compile_checksum!(tx_buffer[OD_LENGTH as usize], checksum);
    Ok(OD_LENGTH + 1)
}

fn compile_iolink_native_operate_frame(
    tx_buffer: &mut [u8],
    io_link_message: &IoLinkMessage,
) -> Result<u8, IoLinkError> {
    const OD_LENGTH: u8 = config::m_seq_capability::operate_m_sequence_legacy_od_len();
    const PD_LENGTH_BITS: u8 = config::m_seq_capability::operate_m_sequence_legacy_pd_out_len();
    const PD_LENGTH: u8 = if PD_LENGTH_BITS & 0x01 == 0 {
        PD_LENGTH_BITS / 8
    } else {
        // The ceiling division technique:
        // Instead of using floating-point math like ceil(bits / 8.0), this uses the mathematical identity:
        // Formula is ceil(a/b) = (a + b - 1) / b
        (PD_LENGTH_BITS + 7) / 8 // Integer ceiling division
    };
    if io_link_message.od.is_none() {
        return Err(IoLinkError::InvalidData);
    }
    for (i, &byte) in io_link_message.od.as_ref().unwrap().iter().enumerate() {
        if i < OD_LENGTH as usize {
            tx_buffer[i] = byte;
        } else {
            break; // Avoid out of bounds access
        }
    }
    tx_buffer[OD_LENGTH as usize] = compile_checksum_status!(
        tx_buffer[OD_LENGTH as usize],
        io_link_message.event_flag,
        io_link_message.pd_in_status,
        0 // Checksum will be calculated later
    );
    tx_buffer[PD_LENGTH as usize] = compile_checksum_status!(
        tx_buffer[PD_LENGTH as usize],
        io_link_message.event_flag,
        io_link_message.pd_in_status,
        0 // Checksum will be calculated later
    );
    let checksum = calculate_checksum(OD_LENGTH + PD_LENGTH + 1, &tx_buffer);
    tx_buffer[OD_LENGTH as usize] =
        compile_checksum!(tx_buffer[(OD_LENGTH + PD_LENGTH) as usize], checksum);
    Ok(OD_LENGTH + PD_LENGTH + 1)
}

fn compile_iolink_interleaved_operate_frame_od(
    tx_buffer: &mut [u8],
    io_link_message: &IoLinkMessage,
) -> Result<u8, IoLinkError> {
    const OD_LENGTH: u8 = config::m_seq_capability::operate_m_sequence_od_len();
    if io_link_message.od.is_none() {
        return Err(IoLinkError::InvalidData);
    }
    for (i, &byte) in io_link_message.od.as_ref().unwrap().iter().enumerate() {
        if i < OD_LENGTH as usize {
            tx_buffer[i] = byte;
        } else {
            break; // Avoid out of bounds access
        }
    }
    tx_buffer[OD_LENGTH as usize] = compile_checksum_status!(
        tx_buffer[OD_LENGTH as usize],
        io_link_message.event_flag,
        io_link_message.pd_in_status,
        0 // Checksum will be calculated later
    );
    let checksum = calculate_checksum(OD_LENGTH + 1, &tx_buffer);
    tx_buffer[OD_LENGTH as usize] = compile_checksum!(tx_buffer[OD_LENGTH as usize], checksum);
    Ok(OD_LENGTH + 1)
}

fn compile_iolink_interleaved_operate_frame_pd(
    tx_buffer: &mut [u8],
    io_link_message: &IoLinkMessage,
) -> Result<u8, IoLinkError> {
    const PD_LENGTH_BITS: (u8, bool) = config::m_seq_capability::operate_m_sequence_pd_out_len();
    const PD_LENGTH: u8 = if PD_LENGTH_BITS.1 {
        PD_LENGTH_BITS.0
    } else {
        // The ceiling division technique:
        // Instead of using floating-point math like ceil(bits / 8.0), this uses the mathematical identity:
        // Formula is ceil(a/b) = (a + b - 1) / b
        (PD_LENGTH_BITS.0 + 7) / 8 as u8 // Integer ceiling division
    };
    if io_link_message.od.is_none() {
        return Err(IoLinkError::InvalidData);
    }
    for (i, &byte) in io_link_message.od.as_ref().unwrap().iter().enumerate() {
        if i < PD_LENGTH as usize {
            tx_buffer[i] = byte;
        } else {
            break; // Avoid out of bounds access
        }
    }
    tx_buffer[PD_LENGTH as usize] = compile_checksum_status!(
        tx_buffer[PD_LENGTH as usize],
        io_link_message.event_flag,
        io_link_message.pd_in_status,
        0 // Checksum will be calculated later
    );
    let checksum = calculate_checksum(PD_LENGTH + 1, &tx_buffer);
    tx_buffer[PD_LENGTH as usize] = compile_checksum!(tx_buffer[PD_LENGTH as usize], checksum);
    Ok(PD_LENGTH + 1)
}

fn compile_iolink_operate_frame(
    tx_buffer: &mut [u8],
    io_link_message: &IoLinkMessage,
    od_transfer_pending: bool,
) -> IoLinkResult<u8> {
    const INTERLEAVED_MODE: bool = config::m_seq_capability::interleaved_mode();
    let length = if INTERLEAVED_MODE {
        // If interleaved mode is enabled, we need to handle both OD and PD frames
        // Check if the message is an OD frame
        if od_transfer_pending {
            compile_iolink_interleaved_operate_frame_pd(tx_buffer, io_link_message)
        } else {
            compile_iolink_interleaved_operate_frame_od(tx_buffer, io_link_message)
        }
    } else {
        compile_iolink_native_operate_frame(tx_buffer, io_link_message)
    }?;

    Ok(length)
}

/// Parse IO-Link frame using nom
/// See IO-Link v1.1.4 Section 6.1
fn parse_iolink_startup_frame(input: &[u8]) -> IoLinkResult<IoLinkMessage> {
    // Extracting `MC` byte properties
    let read_write = types::RwDirection::try_from(extract_rw_direction!(input[0]))?;
    let com_channel = types::ComChannel::try_from(extract_com_channel!(input[0]))?;
    let address_fctrl = extract_address_fctrl!(input[0]);

    // Extracting `CKT` byte properties
    let message_type = types::MessageType::try_from(extract_message_type!(input[1]))?;
    // check for M-sequenceCapability
    // For STARTUP, we expect TYPE_0 (0b00), thus we can check directly
    // because message sequence is always TYPE_0.
    // OD length in startup is always 1 byte
    let od: Option<Vec<u8, MAX_OD_SIZE>> = if input.len() > 2 {
        let mut vec = Vec::new();
        vec.push(input[2]).map_err(|_| IoLinkError::InvalidData)?;
        Some(vec)
    } else {
        None
    };

    Ok(IoLinkMessage {
        read_write: Some(read_write),
        message_type: Some(message_type),
        com_channel: Some(com_channel),
        address_fctrl: Some(address_fctrl),
        event_flag: false, // Event flag is not set in startup frame
        od,
        pd: None,           // No PD in startup frame
        pd_in_status: None, // No PD status in startup frame
    })
}

fn parse_iolink_pre_operate_frame(input: &[u8]) -> IoLinkResult<IoLinkMessage> {
    // On-request Data (OD) length
    const OD_LENGTH: u8 = config::m_seq_capability::preoperate_m_sequence_od_len();
    // Extracting `MC` byte properties
    let read_write = types::RwDirection::try_from(extract_rw_direction!(input[0]))?;
    let com_channel = types::ComChannel::try_from(extract_com_channel!(input[0]))?;
    let address_fctrl = extract_address_fctrl!(input[0]);

    // Extracting `CKT` byte properties
    let rxed_message_type = types::MessageType::try_from(extract_message_type!(input[1]))?;
    let od = if input.len() > 2 {
        let mut vec: Vec<u8, MAX_OD_SIZE> = Vec::new();
        // Extract OD data
        for i in 2..(2 + OD_LENGTH as usize) {
            if i < input.len() {
                vec.push(input[i]).map_err(|_| IoLinkError::InvalidData)?;
            } else {
                break; // Avoid out of bounds access
            }
        }
        Some(vec)
    } else {
        None
    };

    Ok(IoLinkMessage {
        read_write: Some(read_write),
        message_type: Some(rxed_message_type),
        com_channel: Some(com_channel),
        address_fctrl: Some(address_fctrl),
        event_flag: false, // Event flag is not set in pre-operate frame
        od,
        pd: None,           // No PD in pre-operate frame
        pd_in_status: None, // No PD status in pre-operate frame
    })
}

fn parse_iolink_operate_frame(input: &[u8]) -> IoLinkResult<IoLinkMessage> {
    if input.len() < 3 {
        return Err(IoLinkError::InvalidData);
    }
    const INTERLEAVED_MODE: bool = config::m_seq_capability::interleaved_mode();
    // Extracting `MC` byte properties
    let read_write = types::RwDirection::try_from(extract_rw_direction!(input[0]))?;
    let com_channel = types::ComChannel::try_from(extract_com_channel!(input[0]))?;
    let address_fctrl = extract_address_fctrl!(input[0]);

    // Extracting `CKT` byte properties
    let message_type = types::MessageType::try_from(extract_message_type!(input[1]))?;

    let (od, pd) = if INTERLEAVED_MODE {
        // If interleaved mode is enabled, we need to handle both OD and PD frames
        // Check if the message is an OD frame
        if message_type == types::MessageType::ProcessData.into() {
            parse_iolink_interleaved_operate_frame_pd(input)
        } else {
            parse_iolink_interleaved_operate_frame_od(input)
        }
    } else {
        parse_iolink_native_operate_frame(input)
    }?;

    Ok(IoLinkMessage {
        read_write: Some(read_write),
        message_type: Some(message_type),
        com_channel: Some(com_channel),
        address_fctrl: Some(address_fctrl),
        event_flag: false, // Event flag is not set in operate frame
        od,
        pd,
        pd_in_status: None, // No PD status in operate frame
    })
}

fn parse_iolink_native_operate_frame(
    input: &[u8],
) -> IoLinkResult<(Option<Vec<u8, MAX_OD_SIZE>>, Option<Vec<u8, MAX_PD_SIZE>>)> {
    const OD_LENGTH_OCTETS: u8 = config::m_seq_capability::operate_m_sequence_od_len();
    const PD_LENGTH_BITS: (u8, bool) = config::m_seq_capability::operate_m_sequence_pd_out_len();
    const PD_LENGTH: u8 = if PD_LENGTH_BITS.1 {
        PD_LENGTH_BITS.0
    } else {
        // The ceiling division technique:
        // Instead of using floating-point math like ceil(bits / 8.0), this uses the mathematical identity:
        // Formula is ceil(a/b) = (a + b - 1) / b
        (PD_LENGTH_BITS.0 + 7) / 8 as u8 // Integer ceiling division
    };
    if input.len() != HEADER_SIZE + OD_LENGTH_OCTETS as usize + PD_LENGTH as usize {
        return Err(IoLinkError::InvalidData);
    }
    let mut od: Vec<u8, MAX_OD_SIZE> = Vec::new();
    let mut pd: Vec<u8, MAX_PD_SIZE> = Vec::new();

    for i in 2..(2 + OD_LENGTH_OCTETS as usize) {
        if i < input.len() {
            if let Err(_) = od.push(input[i]) {
                return Err(IoLinkError::InvalidData);
            }
        } else {
            break; // Avoid out of bounds access
        }
    }

    for i in (2 + OD_LENGTH_OCTETS as usize)..(2 + OD_LENGTH_OCTETS as usize + PD_LENGTH as usize) {
        if i < input.len() {
            if let Err(_) = pd.push(input[i]) {
                return Err(IoLinkError::InvalidData);
            }
        } else {
            break; // Avoid out of bounds access
        }
    }

    Ok((Some(od), Some(pd)))
}
fn parse_iolink_interleaved_operate_frame_od(
    input: &[u8],
) -> IoLinkResult<(Option<Vec<u8, MAX_OD_SIZE>>, Option<Vec<u8, MAX_PD_SIZE>>)> {
    const OD_LENGTH: u8 = config::m_seq_capability::operate_m_sequence_legacy_od_len();
    if input.len() != HEADER_SIZE + OD_LENGTH as usize {
        return Err(IoLinkError::InvalidData);
    }

    let mut od: Vec<u8, MAX_OD_SIZE> = Vec::new();
    for i in 2..(2 + OD_LENGTH as usize) {
        if i < input.len() {
            if let Err(_) = od.push(input[i]) {
                return Err(IoLinkError::InvalidData);
            }
        } else {
            break; // Avoid out of bounds access
        }
    }

    Ok((Some(od), None))
}

fn parse_iolink_interleaved_operate_frame_pd(
    input: &[u8],
) -> IoLinkResult<(Option<Vec<u8, MAX_OD_SIZE>>, Option<Vec<u8, MAX_PD_SIZE>>)> {
    const PD_LENGTH_BITS: u8 = config::m_seq_capability::operate_m_sequence_legacy_pd_out_len();
    const PD_LENGTH: u8 = if PD_LENGTH_BITS & 0x01 == 0 {
        PD_LENGTH_BITS / 8
    } else {
        // The ceiling division technique:
        // Instead of using floating-point math like ceil(bits / 8.0), this uses the mathematical identity:
        // Formula is ceil(a/b) = (a + b - 1) / b
        (PD_LENGTH_BITS + 7) / 8 // Integer ceiling division
    };
    if input.len() != HEADER_SIZE + PD_LENGTH as usize {
        return Err(IoLinkError::InvalidData);
    }

    let mut pd: Vec<u8, MAX_PD_SIZE> = Vec::new();
    for i in 2..(2 + PD_LENGTH as usize) {
        if i < input.len() {
            if let Err(_) = pd.push(input[i]) {
                return Err(IoLinkError::InvalidData);
            }
        } else {
            break; // Avoid out of bounds access
        }
    }

    Ok((None, Some(pd)))
}

fn validate_checksum(length: u8, data: &mut [u8]) -> bool {
    // Validate the checksum of the received IO-Link message
    let received_checksum = extract_checksum_bits!(data[1]);
    // clear the received checksum bits (0-5), Before calculating the checksum
    data[1] = clear_checksum_bits_0_to_5!(data[1]);
    let calculated_checksum = calculate_checksum(length, &data);
    calculated_checksum == received_checksum
}

/// See A.1.6 Calculation of the checksum
/// Calculate message checksum
fn calculate_checksum(length: u8, data: &[u8]) -> u8 {
    // Seed value as per IO-Link spec
    let mut checksum = 0x52u8;
    for i in 0..length as usize {
        if i < data.len() {
            checksum ^= data[i];
        }
    }
    let d_bit0 = get_bit_0!(checksum);
    let d_bit1 = get_bit_1!(checksum);
    let d_bit2 = get_bit_2!(checksum);
    let d_bit3 = get_bit_3!(checksum);
    let d_bit4 = get_bit_4!(checksum);
    let d_bit5 = get_bit_5!(checksum);
    let d_bit6 = get_bit_6!(checksum);
    let d_bit7 = get_bit_7!(checksum);

    let checksum_bit0 = d_bit1 ^ d_bit0;
    let checksum_bit1 = d_bit3 ^ d_bit2;
    let checksum_bit2 = d_bit5 ^ d_bit4;
    let checksum_bit3 = d_bit7 ^ d_bit6;
    let checksum_bit4 = d_bit6 ^ d_bit4 ^ d_bit2 ^ d_bit0;
    let checksum_bit5 = d_bit7 ^ d_bit5 ^ d_bit3 ^ d_bit1;

    let checksum = construct_u8!(
        0,
        0,
        checksum_bit5,
        checksum_bit4,
        checksum_bit3,
        checksum_bit2,
        checksum_bit1,
        checksum_bit0
    );

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
    fn test_message_parsing() {}

    #[test]
    fn test_checksum_calculation() {}
}
