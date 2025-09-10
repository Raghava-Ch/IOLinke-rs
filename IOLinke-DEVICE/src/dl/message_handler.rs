//! Message Handler
//!
//! This module implements the Message Handler state machine as defined in
//! IO-Link Specification v1.1.4 Section 6.3
use heapless::Vec;
use iolinke_derived_config::device as derived_config;
use iolinke_types::{
    custom::{IoLinkError, IoLinkResult},
    frame::msequence::{PdStatus, RwDirection, TransmissionRate},
    handlers::{
        self,
        message::{MHInfo, MhConfState, MsgHandlerInfo},
        od::OdInd,
    },
};
use iolinke_util::{
    frame_fromat::message::{
        DeviceOperationMode, MAX_RX_FRAME_SIZE, MAX_TX_FRAME_SIZE, MessageBufferError,
        RxMessageBuffer, TxMessageBuffer, calculate_max_uart_frame_time,
    },
    log_state_transition, log_state_transition_error,
};

use crate::{
    dl::{mode_handler, od_handler, pd_handler},
    pl,
};
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
    CreateMessage(RwDirection),
}

/// See Table 47 – State transition tables of the Device message handler
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Transition {
    /// Nothing to transit.
    Tn,
    /// 0 1 –
    T1,
    /// 1 2 Start "MaxUARTframeTime" and "MaxCycleTime" when in OPERATE.
    _T2,
    /// 2 2 Restart timer "MaxUARTframeTime".
    _T3,
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
    NoError(RwDirection),
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

#[derive(Debug, Clone)]
struct Buffers {
    rx_buffer: RxMessageBuffer<{ MAX_RX_FRAME_SIZE }>,
    tx_buffer: TxMessageBuffer<{ MAX_TX_FRAME_SIZE }>,
}

/// Message Handler implementation
#[derive(Debug, Clone)]
pub struct MessageHandler {
    state: MessageHandlerState,
    exec_transition: Transition,
    buffers: Buffers,
    device_operate_state: DeviceOperationMode,
    pd_in_valid_status: PdStatus,

    transmission_rate: TransmissionRate,
    expected_rx_bytes: u8,

    event_flag: bool,
    pd_status: PdStatus,
    rx_frame_operation_state: DeviceOperationMode,
}

impl MessageHandler {
    /// Create a new Message Handler
    pub fn new() -> Self {
        Self {
            state: MessageHandlerState::Inactive,
            exec_transition: Transition::Tn,
            buffers: Buffers {
                rx_buffer: RxMessageBuffer::new(),
                tx_buffer: TxMessageBuffer::new(),
            },
            device_operate_state: DeviceOperationMode::Startup,
            pd_in_valid_status: PdStatus::INVALID,
            transmission_rate: TransmissionRate::default(),
            expected_rx_bytes: 0, // 0 means not set
            event_flag: false,
            pd_status: PdStatus::INVALID,
            rx_frame_operation_state: DeviceOperationMode::Startup,
        }
    }

    /// Process an event
    fn process_event(&mut self, event: MessageHandlerEvent) -> IoLinkResult<()> {
        use MessageHandlerEvent as Event;
        use MessageHandlerState as State;

        let (new_transition, new_state) = match (self.state, event) {
            (State::Inactive, Event::MhConfActive) => (Transition::T1, State::Idle),
            (State::Inactive, Event::MhConfInactive) => (Transition::T11, State::Inactive),
            (State::Idle, Event::PlTransfer) => {
                // This transition is intentionally handled within
                // 'pl_transfer_ind' to meet strict performance and timing requirements.
                // Change the State.
                (Transition::Tn, State::GetMessage)
            }
            (State::Idle, Event::TimerMaxCycle) => (Transition::T10, State::Idle),
            (State::GetMessage, Event::PlTransfer) => {
                // This transition is intentionally handled within
                // 'pl_transfer_ind' to meet strict performance and timing requirements.
                // Change the State.
                (Transition::Tn, State::GetMessage)
            }
            (State::GetMessage, Event::Completed) => (Transition::T4, State::CheckMessage),
            (State::GetMessage, Event::TimerMaxUARTFrame) => (Transition::T9, State::Idle),
            (State::CheckMessage, Event::NoError(rw_req_dir)) => {
                (Transition::T5, State::CreateMessage(rw_req_dir))
            }
            (State::CheckMessage, Event::ChecksumError) => (Transition::T7, State::Idle),
            (State::CheckMessage, Event::TypeError) => (Transition::T8, State::Idle),
            (State::CreateMessage(_), Event::Ready) => (Transition::T6, State::Idle),
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
        od_handler: &mut od_handler::OnRequestDataHandler,
        pd_handler: &mut pd_handler::ProcessDataHandler,
        mode_handler: &mut mode_handler::DlModeHandler,
        physical_layer: &mut T,
    ) -> IoLinkResult<()> {
        match self.exec_transition {
            Transition::Tn => {
                // No transition, remain in current state
            }
            Transition::T1 => {
                self.exec_transition = Transition::Tn;
                self.execute_t1()?;
            }
            Transition::_T2 => {
                self.exec_transition = Transition::Tn;
                // This transition is intentionally handled within
                // 'pl_transfer_ind' to meet strict performance and timing requirements.
            }
            Transition::_T3 => {
                self.exec_transition = Transition::Tn;
                // This transition is intentionally handled within
                // 'pl_transfer_ind' to meet strict performance and timing requirements.
            }
            Transition::T4 => {
                self.exec_transition = Transition::Tn;
                self.execute_t4(physical_layer)?;
            }
            Transition::T5 => {
                self.exec_transition = Transition::Tn;
                self.execute_t5(od_handler, pd_handler)?;
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

        let _ = self.poll_active_states::<T>();
        Ok(())
    }

    fn poll_active_states<T: pl::physical_layer::PhysicalLayerReq>(&mut self) -> IoLinkResult<()> {
        match self.state {
            MessageHandlerState::CheckMessage => {
                self.execute_check_message()?;
            }
            MessageHandlerState::CreateMessage(rw_req_dir) => {
                // Check the response is ready to be sent
                if self
                    .buffers
                    .tx_buffer
                    .is_ready(self.rx_frame_operation_state)
                {
                    let _ = self.execute_create_message(rw_req_dir);
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
        physical_layer: &T,
        rx_byte: u8,
    ) -> IoLinkResult<()> {
        self.buffers.rx_buffer.clear();
        let _ = self.buffers.rx_buffer.push(rx_byte);
        // Start MaxUARTframeTime and MaxCycleTime timers according to the transmission rate
        let max_uart_frame_time = calculate_max_uart_frame_time(self.transmission_rate);
        let _ = physical_layer
            .start_timer(pl::physical_layer::Timer::MaxCycleTime, max_uart_frame_time);
        self.expected_rx_bytes = 0;
        Ok(())
    }

    /// Transition T3: GetMessage -> GetMessage (restart MaxUARTframeTime)
    /// Restart timer "MaxUARTframeTime"
    fn execute_t3<T: pl::physical_layer::PhysicalLayerReq>(
        &mut self,
        physical_layer: &T,
        rx_byte: u8,
    ) -> IoLinkResult<()> {
        let _ = self.buffers.rx_buffer.push(rx_byte);
        let max_uart_frame_time = calculate_max_uart_frame_time(self.transmission_rate);
        let _ = physical_layer.restart_timer(
            pl::physical_layer::Timer::MaxUARTframeTime,
            max_uart_frame_time,
        );
        // Find the number of UART frames to be received using first two bytes of the message
        if self.buffers.rx_buffer.len() == 2 {
            self.expected_rx_bytes = self
                .buffers
                .rx_buffer
                .calculate_expected_rx_bytes(self.device_operate_state);
        }
        if self.expected_rx_bytes == self.buffers.rx_buffer.len() as u8 {
            self.rx_frame_operation_state = self.device_operate_state;
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
        od_handler: &mut od_handler::OnRequestDataHandler,
        pd_handler: &mut pd_handler::ProcessDataHandler,
    ) -> IoLinkResult<()> {
        if self.device_operate_state == DeviceOperationMode::Operate {
            let pd_out_data = match self.buffers.rx_buffer.extract_pd() {
                Ok(pd_out_data) => pd_out_data,
                Err(_) => &[],
            };
            let pd_out_data = match Vec::from_slice(pd_out_data) {
                Ok(pd_out_data) => pd_out_data,
                Err(_) => Vec::new(),
            };
            const PD_OUT_LENGTH: u8 =
                derived_config::process_data::pd_out::config_length_in_bytes() as u8;
            const PD_IN_LENGTH: u8 =
                derived_config::process_data::pd_in::config_length_in_bytes() as u8;
            // In OPERATE mode, invoke PD.ind service indication
            let pd_out_len = pd_out_data.len() as u8;
            if pd_out_len != PD_OUT_LENGTH {
                return Err(IoLinkError::InvalidLength);
            }
            let _ = pd_handler.pd_ind(0, PD_IN_LENGTH, 0, &pd_out_data);
        }
        let od_data_len = match self.rx_frame_operation_state {
            DeviceOperationMode::Startup => 1,
            DeviceOperationMode::PreOperate => {
                derived_config::on_req_data::pre_operate::od_length() as u8
            }
            DeviceOperationMode::Operate => derived_config::on_req_data::operate::od_length() as u8,
        };
        let mc = self
            .buffers
            .rx_buffer
            .extract_mc()
            .map_err(|_| IoLinkError::InvalidData)?;
        let rw = mc.read_write();
        let channel = mc.comm_channel();
        let addr_ctrl = mc.address_fctrl();
        let od_data = match self.buffers.rx_buffer.extract_od_from_write_req(self.rx_frame_operation_state) {
            Ok(od_data) => od_data,
            Err(_) => &[],
        };
        let od_data = match Vec::from_slice(od_data) {
            Ok(od_data) => od_data,
            Err(_) => Vec::new(),
        };
        let od_ind_data = handlers::od::OdIndData {
            rw_direction: rw,
            com_channel: channel,
            address_ctrl: addr_ctrl,
            req_length: od_data_len,
            data: od_data,
        };
        let _ = od_handler.od_ind(&od_ind_data);
        self.rx_frame_operation_state = self.device_operate_state;
        Ok(())
    }

    /// Transition T6: CreateMessage -> Idle
    /// Compile and invoke PL_Transfer.rsp service response (Device sends response message)
    fn execute_t6<T: pl::physical_layer::PhysicalLayerReq>(
        &mut self,
        physical_layer: &mut T,
    ) -> IoLinkResult<()> {
        // Compiled and send response via PL_Transfer.rsp (handled externally)
        physical_layer.pl_transfer_req(self.buffers.tx_buffer.get_as_slice())?;
        self.buffers.tx_buffer.clear();

        Ok(())
    }

    /// Transition T7: CheckMessage -> Idle
    /// No message (e.g., timeout)
    fn execute_t7(&mut self) -> IoLinkResult<()> {
        Ok(())
    }

    /// Transition T8: CheckMessage -> Idle
    /// Indicate error to DL-mode handler via MHInfo (ILLEGAL_MESSAGETYPE)
    fn execute_t8(&mut self, mode_handler: &mut mode_handler::DlModeHandler) -> IoLinkResult<()> {
        // mode_handler.mh_info(MHInfo::IllegalMessagetype);
        // Error indication handled externally
        mode_handler.mh_info(MHInfo::IllegalMessagetype);
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

    /// State {CreateMessage}
    fn execute_create_message(&mut self, rw_req_dir: RwDirection) -> Result<(), IoLinkError> {
        self.buffers.tx_buffer.compile_message_rsp(
            self.rx_frame_operation_state,
            rw_req_dir,
            self.event_flag,
            self.pd_status,
        )?;
        Ok(())
    }

    /// State {CheckMessage}
    fn execute_check_message(&mut self) -> Result<(), IoLinkError> {
        let rw_req_dir = match self.parse_message() {
            Ok(rw_req_dir) => rw_req_dir,
            Err(e) => {
                // Handle parsing error
                match e {
                    IoLinkError::ChecksumError => {
                        let _ = self.process_event(MessageHandlerEvent::ChecksumError);
                        return Err(e);
                    }
                    IoLinkError::InvalidMseqType => {
                        let _ = self.process_event(MessageHandlerEvent::TypeError);
                        return Err(e);
                    }
                    _ => return Err(e),
                }
            }
        };
        self.process_event(MessageHandlerEvent::NoError(rw_req_dir))?;
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
        self.event_flag = flag;
    }

    /// See 7.2.2.2 OD
    /// The OD service is used to set up the On-request Data for the next message to be sent. In
    /// turn, the confirmation of the service contains the data from the receiver. The parameters of
    /// the service primitives are listed in Table 35.
    pub fn od_rsp(&mut self, length: u8, data: &[u8]) -> IoLinkResult<()> {
        self.buffers
            .tx_buffer
            .insert_od(length as usize, &data[..length as usize], self.rx_frame_operation_state)
            .map_err(|_| IoLinkError::InvalidParameter)?;
        Ok(())
    }

    /// See 7.2.2.3 PD
    /// The PD service is used to setup the Process Data to be sent through the process
    /// communication channel. The confirmation of the service contains the data from the receiver.
    /// The parameters of the service primitives are listed in Table 36.
    pub fn pd_rsp(&mut self, length: usize, data: &[u8]) -> IoLinkResult<()> {
        self.buffers
            .tx_buffer
            .insert_pd(&data[..length as usize], self.rx_frame_operation_state)
            .map_err(|_| IoLinkError::InvalidParameter)?;
        self.pd_status = self.pd_in_valid_status;
        Ok(())
    }

    /// 7.2.2.5 PDInStatus
    /// The service PDInStatus sets and signals the validity qualifier
    /// of the input Process Data. PD validity `Device to Master`.
    pub fn pd_in_status_req(&mut self, valid: PdStatus) -> IoLinkResult<()> {
        self.pd_in_valid_status = valid;
        Ok(())
    }
    /// Parse IO-Link message from buffer
    /// See IO-Link v1.1.4 Section 6.1
    fn parse_message(&mut self) -> IoLinkResult<RwDirection> {
        match self.buffers.rx_buffer.valid_req(self.device_operate_state) {
            Ok(rw_req_dir) => Ok(rw_req_dir),
            Err(e) => match e {
                MessageBufferError::InvalidChecksum => {
                    return Err(IoLinkError::ChecksumError);
                }
                MessageBufferError::InvalidMseqType => {
                    return Err(IoLinkError::InvalidMseqType);
                }
                _ => return Err(IoLinkError::InvalidParameter),
            },
        }
    }
}

// Physical layer indication trait implementation would go here
// when the physical layer module is properly defined
impl<'a> pl::physical_layer::PhysicalLayerInd for MessageHandler {
    fn pl_transfer_ind<T: pl::physical_layer::PhysicalLayerReq>(
        &mut self,
        physical_layer: &mut T,
        rx_byte: u8,
    ) -> IoLinkResult<()> {
        use MessageHandlerState as State;
        let current_state = self.state;
        let event = MessageHandlerEvent::PlTransfer;
        let _ = self.process_event(event);
        match current_state {
            State::Idle => {
                self.execute_t2(physical_layer, rx_byte)?;
            }
            State::GetMessage => {
                self.execute_t3(physical_layer, rx_byte)?;
            }
            _ => {
                // Do nothing
                // Other state actvities are handled in 'process_event'.
            }
        }
        Ok(())
    }
}

impl pl::physical_layer::IoLinkTimer for MessageHandler {
    /// Any MasterCommand received by the Device command handler
    /// (see Table 44 and Figure 54, state "CommandHandler_2")
    fn timer_elapsed<T: pl::physical_layer::PhysicalLayerReq>(
        &mut self,
        timer: pl::physical_layer::Timer,
    ) -> IoLinkResult<()> {
        let event = match timer {
            pl::physical_layer::Timer::MaxCycleTime => MessageHandlerEvent::TimerMaxCycle,
            pl::physical_layer::Timer::MaxUARTframeTime => MessageHandlerEvent::TimerMaxUARTFrame,
            _ => return Ok(()),
        };
        let _ = self.process_event(event);
        Ok(())
    }
}

/// DL indications to other modules
impl handlers::mode::DlModeInd for MessageHandler {
    /// See 7.2.1.14 DL_Mode
    /// The DL uses the DL_Mode service to report to System Management that a certain operating
    /// status has been reached. The parameters of the service primitives are listed in Table 29.
    fn dl_mode_ind(&mut self, mode: handlers::mode::DlMode) -> IoLinkResult<()> {
        use handlers::mode::DlMode;
        match mode {
            DlMode::Startup => {
                self.device_operate_state = DeviceOperationMode::Startup;
            }
            DlMode::PreOperate => {
                self.device_operate_state = DeviceOperationMode::PreOperate;
            }
            DlMode::Operate => {
                self.device_operate_state = DeviceOperationMode::Operate;
            }
            DlMode::Inactive => {
                self.device_operate_state = DeviceOperationMode::Startup;
            }
            DlMode::Com1 => {
                self.transmission_rate = TransmissionRate::Com1;
            }
            DlMode::Com2 => {
                self.transmission_rate = TransmissionRate::Com2;
            }
            DlMode::Com3 => {
                self.transmission_rate = TransmissionRate::Com3;
            }
            DlMode::Comlost => {
                self.transmission_rate = TransmissionRate::Com1;
            }
            _ => {}
        }

        Ok(())
    }
}

impl handlers::mode::DlReadWriteInd for MessageHandler {
    /// See 7.2.1.5 DL_Write
    /// The DL_Write service is used by System Management to write a Device parameter value to
    /// the Device via the page communication channel. The parameters of the service primitives are
    /// listed in Table 20.
    fn dl_write_ind(&mut self, _address: u8, _value: u8) -> IoLinkResult<()> {
        Err(IoLinkError::NoImplFound)
    }

    /// 7.2.1.4 DL_Read
    /// The DL_Read service is used by System Management to read a Device parameter value via
    /// the page communication channel. The parameters of the service primitives are listed in Table 19.
    fn dl_read_ind(&mut self, _address: u8) -> IoLinkResult<()> {
        Err(IoLinkError::NoImplFound)
    }
}

impl Default for MessageHandler {
    fn default() -> Self {
        Self::new()
    }
}
