use crate::{
    IoLinkError, IoLinkResult, isdu_busy, isdu_extended_length_code, isdu_no_service,
    isdu_read_failure_code, isdu_read_request_index_code,
    isdu_read_request_index_index_subindex_code, isdu_read_request_index_subindex_code,
    isdu_read_success_code, isdu_write_failure_code, isdu_write_request_index_code,
    isdu_write_request_index_index_subindex_code, isdu_write_request_index_subindex_code,
    isdu_write_success_code,
};
use heapless::Vec;
use bitfields::bitfield;

pub const MAX_ISDU_LENGTH: usize = 238;

/// ISDU (Index Service Data Unit) structure
/// See IO-Link v1.1.4 Section 8.4.3
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Isdu {
    /// Parameter index
    pub index: u16,
    /// Sub-index
    pub sub_index: u8,
    /// Data payload
    pub data: Vec<u8, MAX_ISDU_LENGTH>,
    /// Read/Write operation flag
    pub direction: bool,
}

/// See A.5.2 I-Service
/// Figure A.16 shows the structure of the I-Service octet.
#[bitfield(u8)]
#[derive(Clone, Copy)]
pub struct IsduService {
    /// I-Service octet
    #[bits(4)]
    pub i_service: u8,
    /// Transfer length
    #[bits(4)]
    pub length: u8,
}

pub fn compile_isdu_write_success_response(buffer: &mut [u8]) -> IoLinkResult<()> {
    let i_service = IsduServiceBuilder::new()
        .with_i_service(isdu_write_success_code!())
        .with_length(2)
        .build();
    buffer[0] = i_service.into_bits();
    buffer[1] = 0;
    let chkpdu = calculate_checksum(2, &buffer[0..2]);
    buffer[1] = chkpdu;
    Ok(())
}

pub fn compile_isdu_read_success_response(
    length: u8,
    data: &[u8],
    buffer: &mut [u8],
) -> IoLinkResult<()> {
    if (1..=15).contains(&length) {
        // Valid data length range (excluding length byte and checksum)
        let i_service = IsduServiceBuilder::new()
            .with_i_service(isdu_read_success_code!())
            .with_length(length + 2) // +2 for length byte and checksum
            .build();
        buffer[0] = i_service.into_bits();
        buffer[1..1 + length as usize].copy_from_slice(&data[..length as usize]);
        let total_length = 1 + length as usize;
        buffer[total_length] = 0;
        let chkpdu = calculate_checksum(total_length as u8, &buffer[0..total_length]);
        buffer[total_length] = chkpdu;
    } else {
        let i_service = IsduServiceBuilder::new()
            .with_i_service(isdu_read_success_code!())
            .with_length(isdu_extended_length_code!())
            .build();
        buffer[0] = i_service.into_bits();
        buffer[1] = 2 + length; // Extended length byte
        buffer[2..2 + length as usize].copy_from_slice(&data[..length as usize]);
        let total_length = 2 + length as usize;
        let chkpdu = calculate_checksum(total_length as u8, &buffer[0..total_length]);
        buffer[total_length] = chkpdu;
    }
    Ok(())
}

pub fn compile_isdu_read_failure_response(
    error_code: u8,
    additional_error_code: u8,
    buffer: &mut [u8],
) {
    let i_service = IsduServiceBuilder::new()
        .with_i_service(isdu_read_failure_code!())
        .with_length(4)
        .build();
    buffer[0] = i_service.into_bits();
    buffer[1] = error_code;
    buffer[2] = additional_error_code;
    buffer[3] = 0;
    let chkpdu = calculate_checksum(4, &buffer[0..4]);
    buffer[3] = chkpdu;
}

pub fn compile_isdu_write_failure_response(
    error_code: u8,
    additional_error_code: u8,
    buffer: &mut [u8],
) -> IoLinkResult<()> {
    let i_service = IsduServiceBuilder::new()
        .with_i_service(isdu_write_failure_code!())
        .with_length(3)
        .build();
    buffer[0] = i_service.into_bits();
    buffer[1] = error_code;
    buffer[2] = additional_error_code;
    buffer[3] = 0;
    let chkpdu = calculate_checksum(4, &buffer[0..4]);
    buffer[3] = chkpdu;
    Ok(())
}

pub fn compile_isdu_busy_failure_response() -> IoLinkResult<[u8; 1]> {
    let i_service = IsduServiceBuilder::new()
        .with_length(isdu_busy!())
        .build();
    let buffer = i_service.into_bits();
    Ok([buffer])
}

pub fn compile_isdu_no_service_response() -> IoLinkResult<[u8; 1]> {
    let i_service = IsduServiceBuilder::new()
        .with_i_service(0)
        .with_length(isdu_no_service!())
        .build();
    let buffer = i_service.into_bits();
    Ok([buffer])
}

pub fn parse_isdu_write_request(buffer: &[u8]) -> IoLinkResult<(IsduService, u16, u8, &[u8])> {
    if buffer.len() < 3 {
        return Err(IoLinkError::InvalidParameter);
    }
    if calculate_checksum(buffer.len() as u8, buffer) != 0 {
        // Invalid checksum
        return Err(IoLinkError::ChecksumError);
    }
    let i_service: IsduService = IsduService::from_bits(buffer[0]);
    if i_service.i_service() != isdu_write_request_index_code!() {
        parse_write_request_with_index(buffer)
    } else if i_service.i_service() == isdu_write_request_index_subindex_code!() {
        parse_write_request_with_index_subindex(buffer)
    } else if i_service.i_service() == isdu_write_request_index_index_subindex_code!() {
        parse_write_request_with_index_index_subindex(buffer)
    } else {
        return Err(IoLinkError::InvalidData);
    }
}

pub fn parse_isdu_read_request(buffer: &[u8]) -> IoLinkResult<(IsduService, u16, u8)> {
    if buffer.len() < 3 {
        return Err(IoLinkError::InvalidParameter);
    }
    if calculate_checksum(buffer.len() as u8, buffer) != 0 {
        // Invalid checksum
        return Err(IoLinkError::ChecksumError);
    }
    let i_service: IsduService = IsduService::from_bits(buffer[0]);
    if i_service.i_service() != isdu_read_request_index_code!() {
        parse_read_request_with_index(buffer)
    } else if i_service.i_service() == isdu_read_request_index_subindex_code!() {
        parse_read_request_with_index_subindex(buffer)
    } else if i_service.i_service() == isdu_read_request_index_index_subindex_code!() {
        parse_read_request_with_index_index_subindex(buffer)
    } else {
        return Err(IoLinkError::InvalidParameter);
    }
}

pub fn parse_read_request_with_index(buffer: &[u8]) -> IoLinkResult<(IsduService, u16, u8)> {
    let i_service: IsduService = IsduService::from_bits(buffer[0]);
    if i_service.i_service() != isdu_read_request_index_code!() {
        return Err(IoLinkError::InvalidParameter);
    }
    let index = buffer[1];
    Ok((i_service, index as u16, 0))
}

pub fn parse_read_request_with_index_subindex(
    buffer: &[u8],
) -> IoLinkResult<(IsduService, u16, u8)> {
    let i_service: IsduService = IsduService::from_bits(buffer[0]);
    if i_service.i_service() != isdu_read_request_index_subindex_code!() {
        return Err(IoLinkError::InvalidParameter);
    }
    let index = buffer[1];
    let subindex = buffer[2];
    Ok((i_service, index as u16, subindex))
}

pub fn parse_read_request_with_index_index_subindex(
    buffer: &[u8],
) -> IoLinkResult<(IsduService, u16, u8)> {
    let i_service: IsduService = IsduService::from_bits(buffer[0]);
    if i_service.i_service() != isdu_read_request_index_index_subindex_code!() {
        return Err(IoLinkError::InvalidParameter);
    }
    let index = u16::from_le_bytes([buffer[1], buffer[2]]);
    let subindex = buffer[3];
    Ok((i_service, index, subindex))
}

pub fn parse_write_request_with_index(
    buffer: &[u8],
) -> IoLinkResult<(IsduService, u16, u8, &[u8])> {
    let i_service: IsduService = IsduService::from_bits(buffer[0]);
    if i_service.i_service() != isdu_write_request_index_code!() {
        return Err(IoLinkError::InvalidParameter);
    }
    let length = i_service.length();
    if !(2..=15).contains(&length) {
        return Err(IoLinkError::InvalidData);
    }
    let index = buffer[1];
    Ok((
        i_service,
        index as u16,
        0,
        &buffer[2..(3 - length as usize)],
    ))
}

pub fn parse_write_request_with_index_subindex(
    buffer: &[u8],
) -> IoLinkResult<(IsduService, u16, u8, &[u8])> {
    let i_service: IsduService = IsduService::from_bits(buffer[0]);
    if i_service.i_service() != isdu_write_request_index_subindex_code!() {
        return Err(IoLinkError::InvalidParameter);
    }
    let length = i_service.length();
    if !(2..=15).contains(&length) {
        return Err(IoLinkError::InvalidData);
    }
    let index = buffer[1];
    let subindex = buffer[2];
    let data = &buffer[3..(3 + length as usize)];
    Ok((i_service, index as u16, subindex, data))
}

pub fn parse_write_request_with_index_index_subindex(
    buffer: &[u8],
) -> IoLinkResult<(IsduService, u16, u8, &[u8])> {
    let i_service: IsduService = IsduService::from_bits(buffer[0]);
    if i_service.i_service() != isdu_write_request_index_index_subindex_code!() {
        return Err(IoLinkError::InvalidParameter);
    }
    if i_service.length() != 1 {
        return Err(IoLinkError::InvalidData);
    }
    let length = buffer[1];
    if !(17..=238).contains(&length) {
        return Err(IoLinkError::InvalidData);
    }
    let index = u16::from_le_bytes([buffer[2], buffer[3]]);
    let subindex = buffer[4];
    let data = &buffer[5..(5 + length as usize)];
    Ok((i_service, index, subindex, data))
}

pub fn calculate_checksum(length: u8, data: &[u8]) -> u8 {
    let mut checkpdu = 0;
    for byte in data.iter().take(length as usize) {
        checkpdu ^= byte;
    }
    checkpdu
}
