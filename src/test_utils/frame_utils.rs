//! Frame format utilities for testing IO-Link device communication

use std::sync::{Arc, Mutex};
use crate::*;
use crate::test_utils::{self, ChecksumMsequenceType, ChecksumStatus, ThreadMessage};
use crate::types::{ComChannel, MsequenceBaseType, RwDirection};
use crate::utils::frame_fromat::message::{MsequenceControl, MsequenceControlBuilder};

/// Extracts and validates checksum from response
pub fn validate_checksum(response: &[u8], expected_checksum: u8) -> bool {
    if response.len() < 2 {
        return false;
    }

    let received_checksum = test_utils::ChecksumStatus::from(response[1]);
    received_checksum.checksum() == expected_checksum
}

/// Validate the checksum of the device frame
///
/// # Arguments
///
/// * `data` - The data to validate
///
/// # Returns
///
/// True if the checksum is valid, false otherwise
///
pub fn validate_device_frame_checksum(data: &mut Vec<u8>) -> bool {
    let data_len = data.len();
    let cks = ChecksumStatus::from(data[data_len - 1]);
    let mut cleared_checksum_cks = cks.clone();
    cleared_checksum_cks.set_checksum(0);

    let data_last_index = data.get_mut(data_len - 1).unwrap();
    let cleared_checksum_cks_bits = cleared_checksum_cks.into_bits();
    *data_last_index = cleared_checksum_cks_bits;
    let cks_calculated_checksum =
        test_utils::calculate_checksum_for_testing(data_len as u8, data);
    let rec_cks = cks.checksum();
    cks_calculated_checksum == rec_cks
}

/// Sets up the device with basic configuration for testing
pub fn setup_device_configuration(io_link_device: &Arc<Mutex<IoLinkDevice>>) {
    let mseq_cap = config::m_seq_capability::m_sequence_capability_parameter();

    let _ = io_link_device
        .lock()
        .unwrap()
        .sm_set_device_mode_req(DeviceMode::Idle);
    // Set device identification parameters
    let _ = io_link_device
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

    let _ = io_link_device
        .lock()
        .unwrap()
        .sm_set_device_ident_req(&DeviceIdent {
            vendor_id: [0x12, 0x34],
            device_id: [0x56, 0x78, 0x9A],
            function_id: [0xBC, 0xDE],
        });

    let _ = io_link_device
        .lock()
        .unwrap()
        .sm_set_device_mode_req(DeviceMode::Sio);
}

/// Performs the startup sequence for the device
pub fn perform_startup_sequence(io_link_device: &Arc<Mutex<IoLinkDevice>>) {
    let _ = io_link_device.lock().unwrap().pl_wake_up_ind();
    std::thread::sleep(std::time::Duration::from_micros(100));
    let _ = io_link_device
        .lock()
        .unwrap()
        .successful_com(TransmissionRate::Com3);
}

/// Creates a read request message for testing
pub fn create_startup_read_request(address: u8) -> Vec<u8> {
    let mc = test_utils::MsequenceControlBuilder::new()
        .with_read_write(RwDirection::Read)
        .with_comm_channel(ComChannel::Page)
        .with_address_fctrl(address)
        .build();

    let mut ckt: ChecksumMsequenceType = test_utils::ChecksumMsequenceTypeBuilder::new()
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

    rx_buffer
}

/// Creates a read request message for testing
pub fn create_preop_read_request(address: u8) -> Vec<u8> {
    let mc = test_utils::MsequenceControlBuilder::new()
        .with_read_write(RwDirection::Read)
        .with_comm_channel(ComChannel::Page)
        .with_address_fctrl(address)
        .build();

    let mut ckt = test_utils::ChecksumMsequenceTypeBuilder::new()
        .with_m_seq_type(
            config::m_seq_capability::pre_operate_m_sequence::m_sequence_base_type(),
        )
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

    rx_buffer
}

/// Creates a read request message for testing
pub fn create_preop_write_isdu_request(flow_control: u8, buffer: &[u8]) -> Vec<u8> {
    let mc = test_utils::MsequenceControlBuilder::new()
        .with_read_write(RwDirection::Write)
        .with_comm_channel(ComChannel::Isdu)
        .with_address_fctrl(flow_control)
        .build();

    let mut ckt = test_utils::ChecksumMsequenceTypeBuilder::new()
        .with_m_seq_type(
            config::m_seq_capability::pre_operate_m_sequence::m_sequence_base_type(),
        )
        .with_checksum(0)
        .build();

    let mc_bits = mc.into_bits();
    let ckt_bits = ckt.into_bits();

    let mut rx_buffer = Vec::new();
    rx_buffer.push(mc_bits);
    rx_buffer.push(ckt_bits);
    rx_buffer.extend_from_slice(buffer);

    let checksum =
        test_utils::calculate_checksum_for_testing(rx_buffer.len() as u8, &rx_buffer);
    ckt.set_checksum(checksum);

    let tx_buffer_1 = rx_buffer.get_mut(1).unwrap();
    *tx_buffer_1 = ckt.into_bits();

    rx_buffer
}

pub fn create_preop_read_start_isdu_request() -> Vec<u8> {
    let flow_control = flow_ctrl!(START);
    let mc = test_utils::MsequenceControlBuilder::new()
        .with_read_write(RwDirection::Read)
        .with_comm_channel(ComChannel::Isdu)
        .with_address_fctrl(flow_control)
        .build();

    let mut ckt = test_utils::ChecksumMsequenceTypeBuilder::new()
        .with_m_seq_type(
            config::m_seq_capability::pre_operate_m_sequence::m_sequence_base_type(),
        )
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

    rx_buffer
}

/// Creates a read request message for testing
pub fn create_preop_read_isdu_segment(flow_control: u8) -> Vec<u8> {
    let mc = test_utils::MsequenceControlBuilder::new()
        .with_read_write(RwDirection::Read)
        .with_comm_channel(ComChannel::Isdu)
        .with_address_fctrl(flow_control)
        .build();

    let mut ckt = test_utils::ChecksumMsequenceTypeBuilder::new()
        .with_m_seq_type(
            config::m_seq_capability::pre_operate_m_sequence::m_sequence_base_type(),
        )
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

    rx_buffer
}

/// Creates a read request message for testing
pub fn create_preop_isdu_idle_request() -> Vec<u8> {
    let flow_control = flow_ctrl!(IDLE_1);
    let mc = test_utils::MsequenceControlBuilder::new()
        .with_read_write(RwDirection::Read)
        .with_comm_channel(ComChannel::Isdu)
        .with_address_fctrl(flow_control)
        .build();

    let mut ckt = test_utils::ChecksumMsequenceTypeBuilder::new()
        .with_m_seq_type(
            config::m_seq_capability::pre_operate_m_sequence::m_sequence_base_type(),
        )
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

    rx_buffer
}

/// Creates a read request message for testing
pub fn create_preop_write_isdu_complete_request() -> Vec<u8> {
    let buffer = &[]; // No OD
    let flow_control = flow_ctrl!(START);
    let mc = test_utils::MsequenceControlBuilder::new()
        .with_read_write(RwDirection::Read)
        .with_comm_channel(ComChannel::Isdu)
        .with_address_fctrl(flow_control)
        .build();

    let mut ckt = test_utils::ChecksumMsequenceTypeBuilder::new()
        .with_m_seq_type(
            config::m_seq_capability::pre_operate_m_sequence::m_sequence_base_type(),
        )
        .with_checksum(0)
        .build();

    let mc_bits = mc.into_bits();
    let ckt_bits = ckt.into_bits();

    let mut rx_buffer = Vec::new();
    rx_buffer.push(mc_bits);
    rx_buffer.push(ckt_bits);
    rx_buffer.extend_from_slice(buffer);

    let checksum =
        test_utils::calculate_checksum_for_testing(rx_buffer.len() as u8, &rx_buffer);
    ckt.set_checksum(checksum);

    let tx_buffer_1 = rx_buffer.get_mut(1).unwrap();
    *tx_buffer_1 = ckt.into_bits();

    rx_buffer
}

/// Creates a read request message for testing
pub fn create_op_read_request(address: u8) -> Vec<u8> {
    const BASE_TYPE: MsequenceBaseType = config::m_seq_capability::operate_m_sequence::m_sequence_base_type();
    let mc = test_utils::MsequenceControlBuilder::new()
        .with_read_write(RwDirection::Read)
        .with_comm_channel(ComChannel::Page)
        .with_address_fctrl(address)
        .build();
        
        let mut ckt = test_utils::ChecksumMsequenceTypeBuilder::new()
        .with_m_seq_type(BASE_TYPE)
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

    rx_buffer
}

/// Creates a write request message for testing
pub fn create_startup_write_request(address: u8, data: u8) -> Vec<u8> {
    let mc = test_utils::MsequenceControlBuilder::new()
        .with_read_write(RwDirection::Write)
        .with_comm_channel(ComChannel::Page)
        .with_address_fctrl(address)
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
    rx_buffer.push(data); // Add the data to write

    let checksum =
        test_utils::calculate_checksum_for_testing(rx_buffer.len() as u8, &rx_buffer);
    ckt.set_checksum(checksum);

    let tx_buffer_1 = rx_buffer.get_mut(1).unwrap();
    *tx_buffer_1 = ckt.into_bits();

    rx_buffer
}

/// Creates a write request message for testing
pub fn create_preop_write_request(address: u8, data: &[u8]) -> Vec<u8> {
    const OD_LENGTH_BYTES: u8 = config::on_req_data::pre_operate::od_length();
    let mc = test_utils::MsequenceControlBuilder::new()
        .with_read_write(RwDirection::Write)
        .with_comm_channel(ComChannel::Page)
        .with_address_fctrl(address)
        .build();

    let mut ckt = test_utils::ChecksumMsequenceTypeBuilder::new()
        .with_m_seq_type(
            config::m_seq_capability::pre_operate_m_sequence::m_sequence_base_type(),
        )
        .with_checksum(0)
        .build();

    let mc_bits = mc.into_bits();
    let ckt_bits = ckt.into_bits();

    let mut rx_buffer = Vec::new();
    rx_buffer.push(mc_bits);
    rx_buffer.push(ckt_bits);
    for index in 0..OD_LENGTH_BYTES as usize {
        if index < data.len() {
            rx_buffer.push(data[index]); // Add the data to write
        } else {
            rx_buffer.push(0);
        }
    }

    let checksum =
        test_utils::calculate_checksum_for_testing(rx_buffer.len() as u8, &rx_buffer);
    ckt.set_checksum(checksum);

    let tx_buffer_1 = rx_buffer.get_mut(1).unwrap();
    *tx_buffer_1 = ckt.into_bits();

    rx_buffer
}

pub fn create_op_write_request(address: u8, data: &[u8]) -> Vec<u8> {
    todo!()
}

/// ISDU frame creation utilities
pub mod isdu_frame {
    use crate::{
        isdu_read_request_index_code, isdu_read_request_index_index_subindex_code,
        isdu_read_request_index_subindex_code, isdu_write_request_index_code,
        isdu_write_request_index_index_subindex_code, isdu_write_request_index_subindex_code,
        utils::{self, frame_fromat::isdu::IsduService},
    };

    // 0x0010 0x00
    pub fn create_isdu_read_request(index: u16, sub_index: Option<u8>) -> Vec<u8> {
        // {I-Service(0x9), Length(0x3), Index, CHKPDU} ^
        // {I-Service(0xA), Length(0x4), Index, Subindex, CHKPDU} ^
        // {I-Service(0xB), Length(0x5), Index, Index, Subindex, CHKPDU}
        let index_1 = (index & 0xFF) as u8;
        let index_2 = (index >> 8) as u8;
        let isdu_request_buffer = if index <= 0xFF && sub_index.is_none() {
            let mut isdu_service = IsduService::new();
            isdu_service.set_i_service(isdu_read_request_index_code!());
            isdu_service.set_length(0x03);
            let mut rx_buffer = Vec::new();
            rx_buffer.push(isdu_service.into_bits());
            rx_buffer.push(index_1);
            rx_buffer.push(0); // CHKPDU
            let checkpdu = utils::frame_fromat::isdu::calculate_checksum(
                rx_buffer.len() as u8,
                &rx_buffer,
            );
            rx_buffer.pop();
            rx_buffer.push(checkpdu);
            rx_buffer
        } else if index <= 0xFF && sub_index.is_some() {
            let mut isdu_service = IsduService::new();
            isdu_service.set_i_service(isdu_read_request_index_subindex_code!());
            isdu_service.set_length(0x04);
            let mut rx_buffer = Vec::new();
            rx_buffer.push(isdu_service.into_bits());
            rx_buffer.push(index_1);
            rx_buffer.push(sub_index.unwrap());
            rx_buffer.push(0); // CHKPDU
            let checkpdu = utils::frame_fromat::isdu::calculate_checksum(
                rx_buffer.len() as u8,
                &rx_buffer,
            );
            rx_buffer.pop();
            rx_buffer.push(checkpdu);
            rx_buffer
        } else if index > 0xFF && sub_index.is_some() {
            let mut isdu_service = IsduService::new();
            isdu_service.set_i_service(isdu_read_request_index_index_subindex_code!());
            isdu_service.set_length(0x05);
            let mut rx_buffer = Vec::new();
            rx_buffer.push(isdu_service.into_bits());
            rx_buffer.push(index_1);
            rx_buffer.push(index_2);
            rx_buffer.push(sub_index.unwrap());
            rx_buffer.push(0); // CHKPDU
            let checkpdu = utils::frame_fromat::isdu::calculate_checksum(
                rx_buffer.len() as u8,
                &rx_buffer,
            );
            rx_buffer.pop();
            rx_buffer.push(checkpdu);
            rx_buffer
        } else {
            panic!("Invalid index or subindex");
        };

        isdu_request_buffer
    }

    // 0x0010 0x00
    pub fn create_isdu_write_request(
        index: u16,
        sub_index: Option<u8>,
        data: &[u8],
    ) -> Vec<u8> {
        // {I-Service(0x9), Length(0x3), Index, CHKPDU} ^
        // {I-Service(0xA), Length(0x4), Index, Subindex, CHKPDU} ^
        // {I-Service(0xB), Length(0x5), Index, Index, Subindex, CHKPDU}
        let index_1 = (index & 0xFF) as u8;
        let index_2 = (index >> 8) as u8;
        let isdu_request_buffer = if index <= 0xFF && sub_index.is_none() {
            let mut isdu_service = IsduService::new();
            isdu_service.set_i_service(isdu_write_request_index_code!());
            isdu_service.set_length(0x03);
            let mut rx_buffer = Vec::new();
            rx_buffer.push(isdu_service.into_bits());
            rx_buffer.push(index_1);
            rx_buffer.extend_from_slice(data);
            rx_buffer.push(0); // CHKPDU
            let checkpdu = utils::frame_fromat::isdu::calculate_checksum(
                rx_buffer.len() as u8,
                &rx_buffer,
            );
            rx_buffer.pop();
            rx_buffer.push(checkpdu);
            rx_buffer
        } else if index <= 0xFF && sub_index.is_some() {
            let isdu_service_length = if data.len() > 15 {
                1
            } else {
                data.len() as u8
            };
            let mut isdu_service = IsduService::new();
            isdu_service.set_i_service(isdu_write_request_index_subindex_code!());
            isdu_service.set_length(isdu_service_length);
            let isdu_service_ext_length = if data.len() > 15 {
                Some(data.len() as u8)
            } else {
                None
            };
            let mut rx_buffer = Vec::new();
            rx_buffer.push(isdu_service.into_bits());
            if isdu_service_ext_length.is_some() {
                rx_buffer.push(isdu_service_ext_length.unwrap());
            }
            rx_buffer.push(index_1);
            rx_buffer.push(sub_index.unwrap());
            rx_buffer.extend_from_slice(data);
            rx_buffer.push(0); // CHKPDU
            let checkpdu = utils::frame_fromat::isdu::calculate_checksum(
                rx_buffer.len() as u8,
                &rx_buffer,
            );
            rx_buffer.pop();
            rx_buffer.push(checkpdu);
            rx_buffer
        } else if index > 0xFF && sub_index.is_some() {
            let mut isdu_service = IsduService::new();
            isdu_service.set_i_service(isdu_write_request_index_index_subindex_code!());
            isdu_service.set_length(0x05);
            let mut rx_buffer = Vec::new();
            rx_buffer.push(isdu_service.into_bits());
            rx_buffer.push(index_1);
            rx_buffer.push(index_2);
            rx_buffer.push(sub_index.unwrap());
            rx_buffer.extend_from_slice(data);
            rx_buffer.push(0); // CHKPDU
            let checkpdu = utils::frame_fromat::isdu::calculate_checksum(
                rx_buffer.len() as u8,
                &rx_buffer,
            );
            rx_buffer.pop();
            rx_buffer.push(checkpdu);
            rx_buffer
        } else {
            panic!("Invalid index or subindex");
        };

        isdu_request_buffer
    }
}
