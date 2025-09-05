use heapless::Vec;
use iolinke_types::{
    custom::{IoLinkError, IoLinkResult},
    frame::isdu::{IsduIServiceCode, IsduLengthCode, IsduService},
    handlers::isdu::MAX_ISDU_LENGTH,
};

pub const fn compile_isdu_write_success_response() -> [u8; 3] {
    let mut buffer = [0; 3];
    let mut i_service = IsduService::new();
    i_service.set_i_service(IsduIServiceCode::WriteSuccess);
    i_service.set_length(2);
    buffer[0] = i_service.into_bits();
    buffer[1] = 0;
    let chkpdu = calculate_checksum(2, &buffer);
    buffer[2] = chkpdu;
    buffer
}

pub fn compile_isdu_read_success_response(
    length: u8,
    data: &[u8],
    buffer: &mut Vec<u8, MAX_ISDU_LENGTH>,
) -> IoLinkResult<()> {
    if (2..=15).contains(&length) {
        // Valid data length range (excluding length byte and checksum)
        let mut i_service = IsduService::new();
        i_service.set_i_service(IsduIServiceCode::ReadSuccess);
        i_service.set_length(length + 2); // +2 for iservice and for checksum byte
        buffer
            .push(i_service.into_bits())
            .map_err(|_| IoLinkError::InvalidLength)?;
        buffer
            .extend_from_slice(&data[..length as usize])
            .map_err(|_| IoLinkError::InvalidLength)?;
        let total_length = 2 + length as usize;
        buffer.push(0).map_err(|_| IoLinkError::InvalidLength)?;
        let chkpdu = calculate_checksum(total_length as u8, &buffer[0..total_length]);
        buffer.pop();
        buffer
            .push(chkpdu)
            .map_err(|_| IoLinkError::InvalidLength)?;
    } else {
        let mut i_service = IsduService::new();
        i_service.set_i_service(IsduIServiceCode::ReadSuccess);
        i_service.set_length(IsduLengthCode::Extended.into());
        buffer
            .push(i_service.into_bits())
            .map_err(|_| IoLinkError::InvalidLength)?;
        buffer
            .push(3 + length) // isdu service byte + Length byte + checksum byte
            .map_err(|_| IoLinkError::InvalidLength)?; // Extended length byte
        buffer
            .extend_from_slice(&data[..length as usize])
            .map_err(|_| IoLinkError::InvalidLength)?;
        buffer.push(0).map_err(|_| IoLinkError::InvalidLength)?; // Placeholder for checksum, Initially 0
        let total_length = 3 + length as usize;
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
    let mut i_service = IsduService::new();
    i_service.set_i_service(IsduIServiceCode::ReadFailure);
    i_service.set_length(4); // +2 for iservice and for checksum byte + 2 for error code and additional error code
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
    let mut i_service = IsduService::new();
    i_service.set_i_service(IsduIServiceCode::WriteFailure);
    i_service.set_length(4); // +2 for iservice and for checksum byte + 2 for error code and additional error code
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

pub const fn compile_isdu_busy_response() -> [u8; 1] {
    let mut i_service = IsduService::new();
    i_service.set_i_service(IsduIServiceCode::NoService);
    i_service.set_length(1);
    let buffer = i_service.into_bits();
    [buffer]
}

pub const fn compile_isdu_no_service_response() -> [u8; 1] {
    let mut i_service = IsduService::new();
    i_service.set_i_service(IsduIServiceCode::NoService);
    i_service.set_length(1);
    let buffer = i_service.into_bits();
    [buffer]
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
        IsduIServiceCode::ReadRequestIndex => {
            let (i_service, index, sub_index) = parse_read_request_with_index(buffer)?;
            Ok((i_service, index, sub_index, None))
        }
        IsduIServiceCode::ReadRequestIndexSubindex => {
            let (i_service, index, sub_index) = parse_read_request_with_index_subindex(buffer)?;
            Ok((i_service, index, sub_index, None))
        }
        IsduIServiceCode::ReadRequestIndexIndexSubindex => {
            let (i_service, index, sub_index) =
                parse_read_request_with_index_index_subindex(buffer)?;
            Ok((i_service, index, sub_index, None))
        }
        IsduIServiceCode::WriteRequestIndex => {
            let (i_service, index, sub_index, range) = parse_write_request_with_index(buffer)?;
            Ok((i_service, index, sub_index, Some(range)))
        }
        IsduIServiceCode::WriteRequestIndexSubindex => {
            let (i_service, index, sub_index, range) =
                parse_write_request_with_index_subindex(buffer)?;
            Ok((i_service, index, sub_index, Some(range)))
        }
        IsduIServiceCode::WriteRequestIndexIndexSubindex => {
            let (i_service, index, sub_index, range) =
                parse_write_request_with_index_index_subindex(buffer)?;
            Ok((i_service, index, sub_index, Some(range)))
        }
        _ => Err(IoLinkError::InvalidParameter),
    }
}

pub fn parse_read_request_with_index(
    buffer: &Vec<u8, MAX_ISDU_LENGTH>,
) -> IoLinkResult<(IsduService, u16, u8)> {
    let i_service: IsduService = IsduService::from_bits(buffer[0]);
    if i_service.i_service() != IsduIServiceCode::ReadRequestIndex {
        return Err(IoLinkError::InvalidParameter);
    }
    let index = buffer[1];
    Ok((i_service, index as u16, 0))
}

pub fn parse_read_request_with_index_subindex(
    buffer: &Vec<u8, MAX_ISDU_LENGTH>,
) -> IoLinkResult<(IsduService, u16, u8)> {
    let i_service: IsduService = IsduService::from_bits(buffer[0]);
    if i_service.i_service() != IsduIServiceCode::ReadRequestIndexSubindex {
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
    if i_service.i_service() != IsduIServiceCode::ReadRequestIndexIndexSubindex {
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
    if i_service.i_service() != IsduIServiceCode::WriteRequestIndex {
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
    u16,                                  // Index
    u8,                                   // Subindex
    (usize /* Start */, usize /* End */), // Range
)> {
    let isdu_service_bits = buffer.get(0).ok_or(IoLinkError::InvalidParameter)?;
    let i_service: IsduService = IsduService::from_bits(*isdu_service_bits);
    let mut length = i_service.length();
    if length > 15 {
        return Err(IoLinkError::InvalidLength);
    }
    if length == 1 {
        length = *buffer.get(1).ok_or(IoLinkError::InvalidParameter)?;
        let index = *buffer.get(2).ok_or(IoLinkError::InvalidParameter)?;
        let subindex = *buffer.get(3).ok_or(IoLinkError::InvalidParameter)?;
        return Ok((i_service, index as u16, subindex, (4, 4 + length as usize)));
    }
    if !(2..=15).contains(&length) {
        let index = *buffer.get(1).ok_or(IoLinkError::InvalidParameter)?;
        let subindex = *buffer.get(2).ok_or(IoLinkError::InvalidParameter)?;
        return Ok((i_service, index as u16, subindex, (3, 3 + length as usize)));
    }
    return Err(IoLinkError::InvalidData);
}

pub fn parse_write_request_with_index_index_subindex(
    buffer: &Vec<u8, MAX_ISDU_LENGTH>,
) -> IoLinkResult<(
    IsduService,
    u16,                                  // Index
    u8,                                   // Subindex
    (usize /* Start */, usize /* End */), // Range
)> {
    let i_service: IsduService = IsduService::from_bits(buffer[0]);
    if i_service.i_service() != IsduIServiceCode::WriteRequestIndexIndexSubindex {
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

pub const fn calculate_checksum(length: u8, data: &[u8]) -> u8 {
    let mut checkpdu = 0;
    let mut i = 0;
    while i < length as usize {
        // Avoid out-of-bounds access
        if i < data.len() {
            checkpdu ^= data[i];
        }
        i += 1;
    }
    checkpdu
}
