use crate::{
    IoLinkError, IoLinkResult, RwDirection, isdu_extended_length_code, isdu_no_service,
    isdu_read_failure_code, isdu_read_request_index_code,
    isdu_read_request_index_index_subindex_code, isdu_read_request_index_subindex_code,
    isdu_read_success_code, isdu_write_failure_code, isdu_write_request_index_code,
    isdu_write_request_index_index_subindex_code, isdu_write_request_index_subindex_code,
    isdu_write_success_code,
};
use bitfields::bitfield;
use heapless::Vec;

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
    pub direction: RwDirection,
}

/// See A.5.2 I-Service
/// Figure A.16 shows the structure of the I-Service octet.
#[bitfield(u8)]
#[derive(Clone, Copy)]
pub struct IsduService {
    /// Transfer length
    #[bits(4)]
    pub length: u8,
    /// I-Service octet
    #[bits(4)]
    pub i_service: u8,
}

pub fn compile_isdu_write_success_response(
    buffer: &mut Vec<u8, MAX_ISDU_LENGTH>,
) -> IoLinkResult<()> {
    let i_service = IsduServiceBuilder::new()
        .with_i_service(isdu_write_success_code!())
        .with_length(2)
        .build();
    buffer
        .push(i_service.into_bits())
        .map_err(|_| IoLinkError::InvalidLength)?;
    buffer.push(0).map_err(|_| IoLinkError::InvalidLength)?;
    let chkpdu = calculate_checksum(2, &buffer[0..2]);
    buffer.pop();
    buffer
        .push(chkpdu)
        .map_err(|_| IoLinkError::InvalidLength)?;
    Ok(())
}

pub fn compile_isdu_read_success_response(
    length: u8,
    data: &[u8],
    buffer: &mut Vec<u8, MAX_ISDU_LENGTH>,
) -> IoLinkResult<()> {
    if (2..=15).contains(&length) {
        // Valid data length range (excluding length byte and checksum)
        let i_service = IsduServiceBuilder::new()
            .with_i_service(isdu_read_success_code!())
            .with_length(length + 1) // +1 for checksum byte
            .build();
        buffer
            .push(i_service.into_bits())
            .map_err(|_| IoLinkError::InvalidLength)?;
        buffer
            .extend_from_slice(&data[..length as usize])
            .map_err(|_| IoLinkError::InvalidLength)?;
        let total_length = 1 + length as usize;
        buffer.push(0).map_err(|_| IoLinkError::InvalidLength)?;
        let chkpdu = calculate_checksum(total_length as u8, &buffer[0..total_length]);
        buffer.pop();
        buffer
            .push(chkpdu)
            .map_err(|_| IoLinkError::InvalidLength)?;
    } else {
        let i_service = IsduServiceBuilder::new()
            .with_i_service(isdu_read_success_code!())
            .with_length(isdu_extended_length_code!())
            .build();
        buffer
            .push(i_service.into_bits())
            .map_err(|_| IoLinkError::InvalidLength)?;
        buffer
            .push(2 + length)
            .map_err(|_| IoLinkError::InvalidLength)?; // Extended length byte
        buffer
            .extend_from_slice(data)
            .map_err(|_| IoLinkError::InvalidLength)?;
        let total_length = 2 + length as usize;
        let chkpdu = calculate_checksum(total_length as u8, &buffer[0..total_length]);
        buffer.pop();
        buffer
            .push(chkpdu)
            .map_err(|_| IoLinkError::InvalidLength)?;
    }
    Ok(())
}

pub fn compile_isdu_read_failure_response(
    error_code: u8,
    additional_error_code: u8,
    buffer: &mut Vec<u8, MAX_ISDU_LENGTH>,
) -> IoLinkResult<()> {
    let i_service = IsduServiceBuilder::new()
        .with_i_service(isdu_read_failure_code!())
        .with_length(4)
        .build();
    buffer
        .push(i_service.into_bits())
        .map_err(|_| IoLinkError::InvalidLength)?;
    buffer
        .push(error_code)
        .map_err(|_| IoLinkError::InvalidLength)?;
    buffer
        .push(additional_error_code)
        .map_err(|_| IoLinkError::InvalidLength)?;
    buffer.push(0).map_err(|_| IoLinkError::InvalidLength)?;
    let chkpdu = calculate_checksum(4, &buffer[0..4]);
    buffer.pop();
    buffer
        .push(chkpdu)
        .map_err(|_| IoLinkError::InvalidLength)?;
    Ok(())
}

pub fn compile_isdu_write_failure_response(
    error_code: u8,
    additional_error_code: u8,
    buffer: &mut Vec<u8, MAX_ISDU_LENGTH>,
) -> IoLinkResult<()> {
    let i_service = IsduServiceBuilder::new()
        .with_i_service(isdu_write_failure_code!())
        .with_length(3)
        .build();
    buffer
        .push(i_service.into_bits())
        .map_err(|_| IoLinkError::InvalidLength)?;
    buffer
        .push(error_code)
        .map_err(|_| IoLinkError::InvalidLength)?;
    buffer
        .push(additional_error_code)
        .map_err(|_| IoLinkError::InvalidLength)?;
    buffer.push(0).map_err(|_| IoLinkError::InvalidLength)?;
    let chkpdu = calculate_checksum(4, &buffer[0..4]);
    buffer.pop();
    buffer
        .push(chkpdu)
        .map_err(|_| IoLinkError::InvalidLength)?;
    Ok(())
}

pub fn compile_isdu_busy_response() -> IoLinkResult<[u8; 1]> {
    let i_service = IsduServiceBuilder::new()
        .with_i_service(0)
        .with_length(1)
        .build();
    let buffer = i_service.into_bits();
    Ok([buffer])
}

pub fn compile_isdu_no_service_response() -> IoLinkResult<[u8; 1]> {
    let i_service = IsduServiceBuilder::new()
        .with_i_service(isdu_no_service!())
        .with_length(0)
        .build();
    let buffer = i_service.into_bits();
    Ok([buffer])
}

pub fn parse_isdu_request(
    buffer: &Vec<u8, MAX_ISDU_LENGTH>,
) -> IoLinkResult<(
    IsduService,
    u16,
    u8,
    /* Range */ Option<(usize /* Start */, usize /* End */)>,
)> {
    if buffer.len() < 3 {
        return Err(IoLinkError::InvalidParameter);
    }
    println!("buffer: {:?}", buffer);
    if calculate_checksum(buffer.len() as u8, buffer) != 0 {
        // Invalid checksum
        return Err(IoLinkError::ChecksumError);
    }
    let i_service: IsduService = IsduService::from_bits(buffer[0]);
    match i_service.i_service() {
        isdu_read_request_index_code!() => {
            let (i_service, index, sub_index) = parse_read_request_with_index(buffer)?;
            Ok((i_service, index, sub_index, None))
        }
        isdu_read_request_index_subindex_code!() => {
            let (i_service, index, sub_index) = parse_read_request_with_index_subindex(buffer)?;
            Ok((i_service, index, sub_index, None))
        }
        isdu_read_request_index_index_subindex_code!() => {
            let (i_service, index, sub_index) =
                parse_read_request_with_index_index_subindex(buffer)?;
            Ok((i_service, index, sub_index, None))
        }
        isdu_write_request_index_code!() => {
            let (i_service, index, sub_index, range) = parse_write_request_with_index(buffer)?;
            Ok((i_service, index, sub_index, Some(range)))
        }
        isdu_write_request_index_subindex_code!() => {
            let (i_service, index, sub_index, range) =
                parse_write_request_with_index_subindex(buffer)?;
            Ok((i_service, index, sub_index, Some(range)))
        }
        isdu_write_request_index_index_subindex_code!() => {
            let (i_service, index, sub_index, range) =
                parse_write_request_with_index_index_subindex(buffer)?;
            Ok((i_service, index, sub_index, Some(range)))
        }
        _ => Err(IoLinkError::InvalidParameter),
    }
}

pub fn parse_read_request_with_index(buffer: &Vec<u8, MAX_ISDU_LENGTH>) -> IoLinkResult<(IsduService, u16, u8)> {
    let i_service: IsduService = IsduService::from_bits(buffer[0]);
    if i_service.i_service() != isdu_read_request_index_code!() {
        return Err(IoLinkError::InvalidParameter);
    }
    let index = buffer[1];
    Ok((i_service, index as u16, 0))
}

pub fn parse_read_request_with_index_subindex(
    buffer: &Vec<u8, MAX_ISDU_LENGTH>,
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
    buffer: &Vec<u8, MAX_ISDU_LENGTH>,
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
    buffer: &Vec<u8, MAX_ISDU_LENGTH>,
) -> IoLinkResult<(
    IsduService,
    u16,
    u8,
    /* Range */ (usize /* Start */, usize /* End */),
)> {
    let i_service: IsduService = IsduService::from_bits(buffer[0]);
    if i_service.i_service() != isdu_write_request_index_code!() {
        return Err(IoLinkError::InvalidParameter);
    }
    let length = i_service.length();
    if !(2..=15).contains(&length) {
        return Err(IoLinkError::InvalidData);
    }
    let index = buffer[1];
    Ok((i_service, index as u16, 0, (2, 3 - length as usize)))
}

pub fn parse_write_request_with_index_subindex(
    buffer: &Vec<u8, MAX_ISDU_LENGTH>,
) -> IoLinkResult<(
    IsduService,
    u16, // Index
    u8,  // Subindex
    (usize /* Start */, usize /* End */), // Range
)> {
    let isdu_service_bits = buffer.get(0).ok_or(IoLinkError::InvalidParameter)?;
    let i_service: IsduService = IsduService::from_bits(*isdu_service_bits);
    let mut length = i_service.length();
    if length > 15 {
        return Err(IoLinkError::InvalidLength)
    }
    if length == 1 {
        length = *buffer.get(1).ok_or(IoLinkError::InvalidParameter)?;
        let index = *buffer.get(2).ok_or(IoLinkError::InvalidParameter)?;
        let subindex = *buffer.get(3).ok_or(IoLinkError::InvalidParameter)?;
        return Ok((i_service, index as u16, subindex, (3, 3 + length as usize)))
    }
    if !(2..=15).contains(&length) {
        let index = *buffer.get(1).ok_or(IoLinkError::InvalidParameter)?;
        let subindex = *buffer.get(2).ok_or(IoLinkError::InvalidParameter)?;
        return Ok((i_service, index as u16, subindex, (3, 3 + length as usize)))
    }
    return Err(IoLinkError::InvalidData);
}

pub fn parse_write_request_with_index_index_subindex(
    buffer: &Vec<u8, MAX_ISDU_LENGTH>,
) -> IoLinkResult<(
    IsduService,
    u16, // Index
    u8,  // Subindex
    (usize /* Start */, usize /* End */), // Range
)> {
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
    Ok((i_service, index, subindex, (5, 5 + length as usize)))
}

pub fn calculate_checksum(length: u8, data: &[u8]) -> u8 {
    let mut checkpdu = 0;
    for byte in data.iter().take(length as usize) {
        checkpdu ^= byte;
    }
    checkpdu
}
