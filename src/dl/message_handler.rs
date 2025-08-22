//! Message Handler
//!
//! This module implements the Message Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 6.3

use crate::DlMode;
use crate::config;
use crate::dl;
use crate::dl::DlInd;
use crate::dl::{od_handler::OdInd, pd_handler};
use crate::extract_address_fctrl;
use crate::extract_com_channel;
use crate::extract_message_type;
use crate::log_state_transition;
use crate::log_state_transition_error;
use crate::pl;
use crate::types::{self, IoLinkError, IoLinkResult};
use crate::utils;
use crate::utils::frame_fromat::message;
use crate::{get_bits_0_4, get_bits_5_6, get_bits_6_7};
use heapless::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeviceMode {
    Startup,
    PreOperate,
    Operate,
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
    T2(u8),
    /// 2 2 Restart timer "MaxUARTframeTime".
    T3(u8),
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
    PlTransfer(u8),
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
    rx_buffer: Vec<u8, { message::MAX_FRAME_SIZE }>,
    rx_buffer_len: u8,
    tx_buffer: Vec<u8, { message::MAX_FRAME_SIZE }>,
    tx_buffer_len: u8,
}

/// Message Handler implementation
pub struct MessageHandler {
    state: MessageHandlerState,
    exec_transition: Transition,
    buffers: Buffers,
    tx_message: message::IoLinkMessage,
    rx_message: message::IoLinkMessage,
    device_operate_state: types::MasterCommand,
    od_transfer_pending: bool,
    pd_in_valid_status: types::PdStatus,

    device_mode: DeviceMode,
    expected_rx_bytes: u8,
}

impl MessageHandler {
    /// Create a new Message Handler
    pub fn new() -> Self {
        Self {
            state: MessageHandlerState::Idle,
            exec_transition: Transition::Tn,
            buffers: Buffers {
                rx_buffer: Vec::new(),
                rx_buffer_len: 0,
                tx_buffer: Vec::new(),
                tx_buffer_len: 0,
            },
            tx_message: message::IoLinkMessage::default(),
            rx_message: message::IoLinkMessage::default(),
            device_operate_state: types::MasterCommand::Fallback,
            od_transfer_pending: false,
            pd_in_valid_status: types::PdStatus::INVALID,
            device_mode: DeviceMode::Startup,
            expected_rx_bytes: 0, // 0 means not set
        }
    }

    /// Process an event
    fn process_event(&mut self, event: MessageHandlerEvent) -> IoLinkResult<()> {
        use MessageHandlerEvent as Event;
        use MessageHandlerState as State;

        let (new_transition, new_state) = match (self.state, event) {
            (State::Inactive, Event::MhConfActive) => (Transition::T1, State::Idle),
            (State::Inactive, Event::MhConfInactive) => (Transition::T11, State::Inactive),
            (State::Idle, Event::PlTransfer(rx_byte)) => {
                (Transition::T2(rx_byte), State::GetMessage)
            }
            (State::Idle, Event::TimerMaxCycle) => (Transition::T10, State::Idle),
            (State::GetMessage, Event::PlTransfer(rx_byte)) => {
                (Transition::T3(rx_byte), State::GetMessage)
            }
            (State::GetMessage, Event::Completed) => (Transition::T4, State::CheckMessage),
            (State::GetMessage, Event::TimerMaxUARTFrame) => (Transition::T9, State::Idle),
            (State::CheckMessage, Event::NoError) => (Transition::T5, State::CreateMessage),
            (State::CheckMessage, Event::ChecksumError) => (Transition::T7, State::Idle),
            (State::CheckMessage, Event::TypeError) => (Transition::T8, State::Idle),
            (State::CreateMessage, Event::Ready) => (Transition::T6, State::Idle),
            _ => {
                log_state_transition_error!(module_path!(), "process_event", self.state, event);
                (Transition::Tn, self.state)
            }
        };

        log_state_transition!(
            module_path!(),
            "process_event",
            self.state,
            new_state,
            event
        );
        self.exec_transition = new_transition;
        self.state = new_state;

        Ok(())
    }

    /// Poll the message handler
    /// See IO-Link v1.1.4 Section 6.3
    pub fn poll<T: pl::physical_layer::PhysicalLayerReq>(
        &mut self,
        event_handler: &mut dl::event_handler::EventHandler,
        isdu_handler: &mut dl::isdu_handler::IsduHandler,
        od_handler: &mut dl::od_handler::OnRequestDataHandler,
        pd_handler: &mut pd_handler::ProcessDataHandler,
        mode_handler: &mut dl::mode_handler::DlModeHandler,
        physical_layer: &mut T,
    ) -> IoLinkResult<()> {
        let _ = self.poll_active_states();
        match self.exec_transition {
            Transition::Tn => {
                // No transition, remain in current state
            }
            Transition::T1 => {
                self.exec_transition = Transition::Tn;
                self.execute_t1()?;
            }
            Transition::T2(rx_byte) => {
                self.exec_transition = Transition::Tn;
                self.execute_t2(physical_layer, rx_byte)?;
            }
            Transition::T3(rx_byte) => {
                self.exec_transition = Transition::Tn;
                self.execute_t3(physical_layer, rx_byte)?;
            }
            Transition::T4 => {
                self.exec_transition = Transition::Tn;
                self.execute_t4(physical_layer)?;
            }
            Transition::T5 => {
                self.exec_transition = Transition::Tn;
                self.execute_t5(event_handler, isdu_handler, od_handler, pd_handler)?;
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
        Ok(())
    }

    fn poll_active_states(&mut self) -> IoLinkResult<()> {
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
                    && self.tx_message.pd_status.is_none()
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
    fn execute_t2<T: pl::physical_layer::PhysicalLayerReq>(
        &mut self,
        physical_layer: &mut T,
        rx_byte: u8,
    ) -> IoLinkResult<()> {
        self.buffers.rx_buffer.push(rx_byte);
        // Start MaxUARTframeTime and MaxCycleTime timers (handled externally)
        let _ = physical_layer.start_timer(pl::physical_layer::Timer::MaxCycleTime, 0);
        self.expected_rx_bytes = 0;
        Ok(())
    }

    /// Transition T3: GetMessage -> GetMessage (restart MaxUARTframeTime)
    /// Restart timer "MaxUARTframeTime"
    fn execute_t3<T: pl::physical_layer::PhysicalLayerReq>(
        &mut self,
        physical_layer: &mut T,
        rx_byte: u8,
    ) -> IoLinkResult<()> {
        let _ = self.buffers.rx_buffer.push(rx_byte);
        let _ = physical_layer.restart_timer(pl::physical_layer::Timer::MaxUARTframeTime, 0);
        // TODO: Find the number of UART frames to be received using first two bytes of the message
        if self.buffers.rx_buffer.len() == 2 {
            let (mc, ckt) =
                utils::frame_fromat::message::compile_mc_ckt_bytes(&mut self.buffers.rx_buffer)?;
            self.expected_rx_bytes = Self::calculate_expected_rx_bytes(self.device_mode, mc, ckt);
        }
        if self.expected_rx_bytes == self.buffers.rx_buffer.len() as u8 {
            let _ = self.process_event(MessageHandlerEvent::Completed);
        }
        Ok(())
    }

    /// Transition T4: GetMessage -> CheckMessage (reset MaxUARTframeTime)
    /// Reset timer "MaxUARTframeTime"
    fn execute_t4<T: pl::physical_layer::PhysicalLayerReq>(
        &mut self,
        physical_layer: &mut T,
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
        pd_handler: &mut pd_handler::ProcessDataHandler,
    ) -> IoLinkResult<()> {
        if let (Some(rw), Some(channel), Some(addr_ctrl), Some(od_data), Some(pd_data)) = (
            self.rx_message.read_write,
            self.rx_message.com_channel,
            self.rx_message.address_fctrl,
            self.rx_message.od.as_ref(),
            self.rx_message.pd.as_ref(),
        ) {
            let od_ind_data = dl::od_handler::OdIndData {
                rw_direction: rw,
                com_channel: channel,
                address_ctrl: addr_ctrl,
                length: od_data.len() as u8,
                data: od_data.clone(),
            };
            let _ = od_handler.od_ind(&od_ind_data);
            if self.device_operate_state == types::MasterCommand::DeviceOperate {
                const PD_OUT_LEN: u8 = config::process_data::pd_out::config_length_in_bytes();
                const PD_IN_LEN: u8 = config::process_data::pd_in::config_length_in_bytes();
                // In OPERATE mode, invoke PD.ind service indication
                let pd_out_address = extract_address_fctrl!(addr_ctrl);
                let pd_out_len = pd_data.len() as u8;
                let _ = pd_handler.pd_ind(0, 0, pd_data, pd_out_address, pd_out_len);
            }
        }
        todo!("Implement PD.ind service indication values");
        Ok(())
    }

    /// Transition T6: CreateMessage -> Idle
    /// Compile and invoke PL_Transfer.rsp service response (Device sends response message)
    fn execute_t6<T: pl::physical_layer::PhysicalLayerReq>(
        &mut self,
        physical_layer: &mut T,
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
    fn execute_t9<T: pl::physical_layer::PhysicalLayerReq>(
        &mut self,
        physical_layer: &mut T,
    ) -> IoLinkResult<()> {
        physical_layer.stop_timer(pl::physical_layer::Timer::MaxUARTframeTime)?;
        physical_layer.stop_timer(pl::physical_layer::Timer::MaxCycleTime)?;
        Ok(())
    }

    fn calculate_expected_rx_bytes(
        device_mode: DeviceMode,
        mc: utils::frame_fromat::message::MsequenceControl,
        ckt: utils::frame_fromat::message::ChecksumMsequenceType,
    ) -> u8 {
        match device_mode {
            DeviceMode::Startup => {
                if ckt.m_seq_type() == types::MsequenceBaseType::Type0 {
                    if mc.read_write() == types::RwDirection::Read {
                        // Receivable bytes will be 2 bytes.
                        2
                    } else {
                        // Receivable bytes will be 3 bytes including OD byte.
                        3
                    }
                } else {
                    0 // Invalid M-sequence type
                }
            }
            DeviceMode::PreOperate => {
                use config::m_seq_capability::pre_operate_m_sequence;
                use config::on_req_data::pre_operate;
                use utils::frame_fromat::message;
                if ckt.m_seq_type() == pre_operate_m_sequence::m_sequence_base_type() {
                    if mc.read_write() == types::RwDirection::Read {
                        message::HEADER_SIZE as u8
                    } else {
                        message::HEADER_SIZE as u8 + pre_operate::od_length() as u8
                    }
                } else {
                    0 // Invalid M-sequence type
                }
            }
            DeviceMode::Operate => {
                use config::m_seq_capability::operate_m_sequence;
                use config::on_req_data::operate;
                use utils::frame_fromat::message;
                if ckt.m_seq_type() == operate_m_sequence::m_sequence_base_type() {
                    if mc.read_write() == types::RwDirection::Read {
                        message::HEADER_SIZE as u8
                    } else {
                        message::HEADER_SIZE as u8 + operate::od_length() as u8
                    }
                } else {
                    0 // Invalid M-sequence type
                }
            }
        }
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
    /// of the input Process Data. PD validity `Device to Master`.
    pub fn pd_in_status_req(&mut self, valid: types::PdStatus) -> IoLinkResult<()> {
        self.pd_in_valid_status = valid;
        Ok(())
    }

    /// Compile IO-Link message from buffer
    /// See IO-Link v1.1.4 `Annex A` (normative)
    /// Codings, timing constraints, and errors
    /// A.1 General structure and encoding of M-sequences
    fn compile_message(&mut self) -> IoLinkResult<()> {
        let io_link_message = &self.tx_message;
        let tx_buffer = &mut self.buffers.tx_buffer;
        tx_buffer.clear();
        let length = match self.device_operate_state {
            types::MasterCommand::DeviceStartup => {
                message::compile_iolink_startup_frame(tx_buffer, io_link_message)?
            }
            types::MasterCommand::DevicePreOperate => {
                message::compile_iolink_preoperate_frame(tx_buffer, io_link_message)?
            }
            types::MasterCommand::DeviceOperate => {
                message::compile_iolink_operate_frame(tx_buffer, io_link_message)?
            }
            _ => return Err(IoLinkError::InvalidMseqType),
        };
        self.buffers.tx_buffer_len = length as u8;
        todo!("Implement IO-Link message compilation logic");
    }

    /// Parse IO-Link message from buffer
    /// See IO-Link v1.1.4 Section 6.1
    fn parse_message(&mut self) -> IoLinkResult<message::IoLinkMessage> {
        use config::m_seq_capability::{operate_m_sequence, pre_operate_m_sequence};
        use utils::frame_fromat::message;
        use types::{MsequenceType, MsequenceBaseType};
        
        let rx_buffer: &mut Vec<u8, { message::MAX_FRAME_SIZE }> = &mut self.buffers.rx_buffer;
        if message::validate_checksum(rx_buffer.len() as u8, rx_buffer) == false {
            // If checksum is invalid, indicate error
            return Err(IoLinkError::ChecksumError);
        }
        let ckt = message::ChecksumMsequenceType::from_bits(rx_buffer[1]);
        let io_link_message = match self.device_operate_state {
            types::MasterCommand::DeviceStartup => {
                const M_SEQ_BASE_TYPE: MsequenceBaseType = message::get_m_sequence_base_type(MsequenceType::Type0);
                if M_SEQ_BASE_TYPE != ckt.m_seq_type() {
                    return Err(IoLinkError::InvalidMseqType);
                }
                message::parse_iolink_startup_frame(rx_buffer)
            }
            types::MasterCommand::DevicePreOperate => {
                let m_seq_base_type: MsequenceBaseType = message::get_m_sequence_base_type(pre_operate_m_sequence::m_sequence_type());
                if m_seq_base_type != ckt.m_seq_type() {
                    return Err(IoLinkError::InvalidMseqType);
                }
                message::parse_iolink_pre_operate_frame(rx_buffer)
            }
            types::MasterCommand::DeviceOperate => {
                let m_seq_base_type: MsequenceBaseType = message::get_m_sequence_base_type(operate_m_sequence::m_sequence_type());
                if m_seq_base_type != ckt.m_seq_type() {
                    return Err(IoLinkError::InvalidMseqType);
                }
                message::parse_iolink_operate_frame(rx_buffer)
            }
            _ => return Err(IoLinkError::InvalidMseqType),
        };

        io_link_message
    }

    /// Clear buffers
    fn clear_buffers(&mut self) {
        self.buffers.rx_buffer.fill(0);
        self.buffers.rx_buffer_len = 0;
        self.buffers.tx_buffer.fill(0);
        self.buffers.tx_buffer_len = 0;
        self.tx_message = message::IoLinkMessage::default();
    }
}

// Physical layer indication trait implementation would go here
// when the physical layer module is properly defined
impl<'a> pl::physical_layer::PhysicalLayerInd for MessageHandler {
    fn pl_transfer_ind(&mut self, rx_byte: u8) -> IoLinkResult<()> {
        let _ = self.process_event(MessageHandlerEvent::PlTransfer(rx_byte));
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

/// DL indications to other modules
impl DlInd for MessageHandler {
    /// See 7.2.1.14 DL_Mode
    /// The DL uses the DL_Mode service to report to System Management that a certain operating
    /// status has been reached. The parameters of the service primitives are listed in Table 29.
    fn dl_mode_ind(&mut self, mode: DlMode) -> IoLinkResult<()> {
        match mode {
            DlMode::Startup => {
                self.device_mode = DeviceMode::Startup;
            }
            DlMode::PreOperate => {
                self.device_mode = DeviceMode::PreOperate;
            }
            DlMode::Operate => {
                self.device_mode = DeviceMode::Operate;
            }
            DlMode::Inactive => {
                self.device_mode = DeviceMode::Startup;
            }
            _ => {}
        }

        Ok(())
    }

    /// See 7.2.1.5 DL_Write
    /// The DL_Write service is used by System Management to write a Device parameter value to
    /// the Device via the page communication channel. The parameters of the service primitives are
    /// listed in Table 20.
    fn dl_write_ind(&mut self, address: u8, value: u8) -> IoLinkResult<()> {
        Err(IoLinkError::NoImplFound)
    }

    /// 7.2.1.4 DL_Read
    /// The DL_Read service is used by System Management to read a Device parameter value via
    /// the page communication channel. The parameters of the service primitives are listed in Table 19.
    fn dl_read_ind(&mut self, address: u8) -> IoLinkResult<()> {
        Err(IoLinkError::NoImplFound)
    }
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
