//! Basic IO-Link Device Example
//!
//! This example demonstrates the basic usage of the IO-Link Device Stack.
//! It creates a device instance and runs a simple polling loop.
//!
//! ## Features Demonstrated
//!
//! - Device initialization and configuration
//! - Basic polling loop operation
//! - Error handling and recovery
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example basic
//! ```
//!
//! ## Expected Behavior
//!
//! The device will start in the Idle state and continuously poll all
//! protocol layers. Since this is a basic example without hardware
//! implementation, most operations will return `NoImplFound` errors,
//! which are handled gracefully by continuing the loop.

use iolinke_device::{
    ComChannel, DeviceCom, DeviceIdent, DeviceMode, IoLinkDevice, CycleTimeBuilder,
    MsequenceBaseType, MsequenceCapability, PhysicalLayerInd, ProcessDataInBuilder,
    ProcessDataOutBuilder, RevisionIdBuilder, RwDirection, SioMode, TransmissionRate,
    test_utils::{self, SystemManagementReq, ThreadMessage, take_care_of_poll_nb},
};
use iolinke_macros::direct_parameter_address;
use std::{
    sync::{
        mpsc::{self, Receiver, Sender}, Arc, Mutex
    },
    time::Duration,
};

fn main() {
    // Create a new IO-Link device instance
    let io_link_device: Arc<Mutex<IoLinkDevice>> = Arc::new(Mutex::new(IoLinkDevice::new()));
    let (usr_to_mock_tx, usr_to_mock_rx) = mpsc::channel();
    let (mock_to_usr_tx, mock_to_usr_rx): (Sender<ThreadMessage>, Receiver<ThreadMessage>) = mpsc::channel();
    println!("IO-Link Device started. Press Ctrl+C to stop.");

    let io_link_device_clone = Arc::clone(&io_link_device);
    let io_link_device_clone_poll = Arc::clone(&io_link_device);
    take_care_of_poll_nb(io_link_device_clone_poll, usr_to_mock_rx, mock_to_usr_tx);

        let mseq_cap = test_utils::m_seq_capability::m_sequence_capability_parameter();

        // Set device identification parameters
        // These are required by the IO-Link specification
        let _ = io_link_device_clone
            .lock()
            .unwrap()
            .sm_set_device_com_req(&DeviceCom {
                suppported_sio_mode: SioMode::default(),
                transmission_rate: TransmissionRate::Com3,
                min_cycle_time: CycleTimeBuilder::new()
                    .with_time_base(0b10)
                    .with_multiplier(0b000001)
                    .build(),
                msequence_capability: MsequenceCapability::from(mseq_cap),
                revision_id: RevisionIdBuilder::new()
                    .with_major_rev(1)
                    .with_minor_rev(1)
                    .build(),
                process_data_in: ProcessDataInBuilder::new()
                    .with_length(9)
                    .with_sio(true)
                    .with_byte(true)
                    .build(),
                process_data_out: ProcessDataOutBuilder::new()
                    .with_length(9)
                    .with_byte(true)
                    .build(),
            });

        let _ = io_link_device_clone
            .lock()
            .unwrap()
            .sm_set_device_ident_req(&DeviceIdent {
                vendor_id: [0x12, 0x34],
                device_id: [0x56, 0x78, 0x9A],
                function_id: [0xBC, 0xDE],
            });

        let _ = io_link_device_clone
            .lock()
            .unwrap()
            .sm_set_device_mode_req(DeviceMode::Sio);

        let _ = io_link_device_clone.lock().unwrap().pl_wake_up_ind();
        let _ = io_link_device_clone
            .lock()
            .unwrap()
            .successful_com(TransmissionRate::Com3);

        let mc = test_utils::MsequenceControlBuilder::new()
            .with_read_write(RwDirection::Read)
            .with_comm_channel(ComChannel::Page)
            .with_address_fctrl(direct_parameter_address!(MinCycleTime))
            .build();
        let mut ckt = test_utils::ChecksumMsequenceTypeBuilder::new()
            .with_m_seq_type(MsequenceBaseType::Type0)
            .with_checksum(0)
            .build();
        let mc_bits = mc.into_bits();
        let ckt_bits = ckt.into_bits();

        let mut rx_buffer = Vec::new();
        rx_buffer.push(mc_bits);
        rx_buffer.push(ckt_bits);
        let checksum =
            test_utils::calculate_checksum_for_testing(rx_buffer.len() as u8, &rx_buffer);
        ckt.set_checksum(checksum);
        let tx_buffer_1 = rx_buffer.get_mut(1).unwrap();
        *tx_buffer_1 = ckt.into_bits();
        let _ = usr_to_mock_tx.send(ThreadMessage::RxData(rx_buffer));
        let rx_data = mock_to_usr_rx.recv_timeout(Duration::from_secs(4)).unwrap();
        let rx_data = match rx_data {
            ThreadMessage::TxData(data) => data,
            _ => panic!("Expected RxData"),
        };
        let cks = test_utils::ChecksumStatus::from(*rx_data.get(1).unwrap());
        println!("Checksum: {:?}", cks);
        let min_cycle_time = iolinke_device::CycleTime::from(*rx_data.get(0).unwrap());
        println!("Min cycle time: {:?}", min_cycle_time);
}
