use serde::{Deserialize, Serialize, de};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize)]
pub enum ProcessDataLength {
    /// Bit-oriented process data length (0–16 bits).
    Bit(u8),
    /// Octet-oriented process data length (0–32 octets).
    Octet(u8),
}

impl std::fmt::Display for ProcessDataLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessDataLength::Bit(n) => write!(f, "Bit({})", n),
            ProcessDataLength::Octet(n) => write!(f, "Octet({})", n),
        }
    }
}

impl<'de> Deserialize<'de> for ProcessDataLength {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        parse_from_value(value).map_err(de::Error::custom)
    }
}

fn parse_from_value(value: Value) -> Result<ProcessDataLength, String> {
    match value {
        Value::String(s) => {
            parse_from_str(&s).ok_or_else(|| format!("invalid process data length literal `{s}`"))
        }
        Value::Number(num) => num
            .as_u64()
            .and_then(|v| u8::try_from(v).ok())
            .map(ProcessDataLength::Octet)
            .ok_or_else(|| "process data length out of range".to_string()),
        Value::Object(map) if map.len() == 1 => {
            let (key, value) = map.into_iter().next().expect("map with len 1");
            let number = value
                .as_u64()
                .ok_or_else(|| "process data length must be numeric".to_string())?;
            let number =
                u8::try_from(number).map_err(|_| "process data length out of range".to_string())?;
            match key.as_str() {
                "Bit" => Ok(ProcessDataLength::Bit(number)),
                "Octet" => Ok(ProcessDataLength::Octet(number)),
                other => Err(format!("unknown process data length variant `{other}`")),
            }
        }
        other => Err(format!(
            "unsupported process data length representation: {other}"
        )),
    }
}

fn parse_from_str(raw: &str) -> Option<ProcessDataLength> {
    let trimmed = raw.trim();
    if let Some(inner) = trimmed
        .strip_prefix("Bit(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        inner.trim().parse::<u8>().ok().map(ProcessDataLength::Bit)
    } else if let Some(inner) = trimmed
        .strip_prefix("Octet(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        inner
            .trim()
            .parse::<u8>()
            .ok()
            .map(ProcessDataLength::Octet)
    } else {
        None
    }
}

impl ProcessDataLength {
    fn validate(self, key: &str) -> io::Result<Self> {
        match self {
            ProcessDataLength::Bit(bits) if bits <= 16 => Ok(self),
            ProcessDataLength::Octet(octets) if octets <= 32 => Ok(self),
            ProcessDataLength::Bit(_) | ProcessDataLength::Octet(_) => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Invalid `{key}` value; see IO-Link Spec v1.1.4, Section B.1.6."),
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ConfigStruct {
    /// PREOPERATE OD length configuration accepted length values are 1, 2, 8, 32.
    PRE_OP_OD_LEN: u8,
    /// OPERATE mode on-request data length limited to 1, 2, 8, or 32.
    OP_OD_LEN: u8,
    /// Protocol major revision identifier nibble.
    MAJOR_REVISION_ID: u8,
    /// Protocol minor revision identifier nibble.
    MINOR_REVISION_ID: u8,
    /// Vendor identifier first octet.
    VENDOR_ID_1: u8,
    /// Vendor identifier second octet.
    VENDOR_ID_2: u8,
    /// Device identifier first octet.
    DEVICE_ID_1: u8,
    /// Device identifier second octet.
    DEVICE_ID_2: u8,
    /// Device identifier third octet.
    DEVICE_ID_3: u8,
    /// Function identifier first octet.
    FUNCTION_ID_1: u8,
    /// Function identifier second octet.
    FUNCTION_ID_2: u8,
    /// Human-readable vendor name (≤64 chars).
    VENDOR_NAME: String,
    /// Human-readable product name (≤64 chars).
    PRODUCT_NAME: String,
    /// Configured minimum cycle time value (ms).
    MIN_CYCLE_TIME_IN_MS: f32,
    /// Process data input length descriptor.
    OP_PD_IN_LEN: ProcessDataLength,
    /// Process data output length descriptor.
    OP_PD_OUT_LEN: ProcessDataLength,
}

impl Default for ConfigStruct {
    fn default() -> Self {
        Self {
            PRE_OP_OD_LEN: 1,
            OP_OD_LEN: 1,
            MAJOR_REVISION_ID: 0,
            MINOR_REVISION_ID: 0,
            VENDOR_ID_1: 0,
            VENDOR_ID_2: 0,
            DEVICE_ID_1: 1,
            DEVICE_ID_2: 0,
            DEVICE_ID_3: 0,
            FUNCTION_ID_1: 0,
            FUNCTION_ID_2: 0,
            VENDOR_NAME: String::from("IOLinke"),
            PRODUCT_NAME: String::from("IOLinke"),
            MIN_CYCLE_TIME_IN_MS: 0.4,
            OP_PD_IN_LEN: ProcessDataLength::Octet(0),
            OP_PD_OUT_LEN: ProcessDataLength::Octet(0),
        }
    }
}

impl ConfigStruct {
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let parameters = parse_parameters(&content)?;
        Self::from_parameters(parameters)
    }

    pub fn from_parameters(mut parameters: BTreeMap<String, String>) -> io::Result<Self> {
        let mut cfg = Self::default();

        cfg.set_pre_op_od_len(parse_u8(
            &take_parameter(&mut parameters, "PRE_OP_OD_LEN")?,
            "PRE_OP_OD_LEN",
        )?)?;
        cfg.set_op_od_len(parse_u8(
            &take_parameter(&mut parameters, "OP_OD_LEN")?,
            "OP_OD_LEN",
        )?)?;
        cfg.set_major_revision_id(parse_u8(
            &take_parameter(&mut parameters, "MAJOR_REVISION_ID")?,
            "MAJOR_REVISION_ID",
        )?)?;
        cfg.set_minor_revision_id(parse_u8(
            &take_parameter(&mut parameters, "MINOR_REVISION_ID")?,
            "MINOR_REVISION_ID",
        )?)?;

        let vendor_id = [
            parse_u8(
                &take_parameter(&mut parameters, "VENDOR_ID_1")?,
                "VENDOR_ID_1",
            )?,
            parse_u8(
                &take_parameter(&mut parameters, "VENDOR_ID_2")?,
                "VENDOR_ID_2",
            )?,
        ];
        cfg.set_vendor_id(vendor_id);

        let device_id = [
            parse_u8(
                &take_parameter(&mut parameters, "DEVICE_ID_1")?,
                "DEVICE_ID_1",
            )?,
            parse_u8(
                &take_parameter(&mut parameters, "DEVICE_ID_2")?,
                "DEVICE_ID_2",
            )?,
            parse_u8(
                &take_parameter(&mut parameters, "DEVICE_ID_3")?,
                "DEVICE_ID_3",
            )?,
        ];
        cfg.set_device_id(device_id)?;

        let function_id = [
            parse_u8(
                &take_parameter(&mut parameters, "FUNCTION_ID_1")?,
                "FUNCTION_ID_1",
            )?,
            parse_u8(
                &take_parameter(&mut parameters, "FUNCTION_ID_2")?,
                "FUNCTION_ID_2",
            )?,
        ];
        cfg.set_function_id(function_id);

        cfg.set_vendor_name(parse_string(
            &take_parameter(&mut parameters, "VENDOR_NAME")?,
            "VENDOR_NAME",
        )?)?;
        cfg.set_product_name(parse_string(
            &take_parameter(&mut parameters, "PRODUCT_NAME")?,
            "PRODUCT_NAME",
        )?)?;
        cfg.set_min_cycle_time_in_ms(parse_f32(
            &take_parameter(&mut parameters, "MIN_CYCLE_TIME_IN_MS")?,
            "MIN_CYCLE_TIME_IN_MS",
        )?)?;

        cfg.set_op_pd_in_len(parse_process_data_length(
            &take_parameter(&mut parameters, "OP_PD_IN_LEN")?,
            "OP_PD_IN_LEN",
        )?)?;
        cfg.set_op_pd_out_len(parse_process_data_length(
            &take_parameter(&mut parameters, "OP_PD_OUT_LEN")?,
            "OP_PD_OUT_LEN",
        )?)?;

        Ok(cfg)
    }

    pub fn pre_op_od_len(&self) -> u8 {
        self.PRE_OP_OD_LEN
    }

    pub fn set_pre_op_od_len(&mut self, value: u8) -> io::Result<()> {
        self.PRE_OP_OD_LEN = validate_pre_op_od_len(value)?;
        Ok(())
    }

    pub fn op_od_len(&self) -> u8 {
        self.OP_OD_LEN
    }

    pub fn set_op_od_len(&mut self, value: u8) -> io::Result<()> {
        self.OP_OD_LEN = validate_op_od_len(value)?;
        Ok(())
    }

    pub fn major_revision_id(&self) -> u8 {
        self.MAJOR_REVISION_ID
    }

    pub fn set_major_revision_id(&mut self, value: u8) -> io::Result<()> {
        self.MAJOR_REVISION_ID = validate_revision_nibble(value, "MAJOR_REVISION_ID")?;
        Ok(())
    }

    pub fn minor_revision_id(&self) -> u8 {
        self.MINOR_REVISION_ID
    }

    pub fn set_minor_revision_id(&mut self, value: u8) -> io::Result<()> {
        self.MINOR_REVISION_ID = validate_revision_nibble(value, "MINOR_REVISION_ID")?;
        Ok(())
    }

    pub fn vendor_id(&self) -> [u8; 2] {
        [self.VENDOR_ID_1, self.VENDOR_ID_2]
    }

    pub fn set_vendor_id(&mut self, octets: [u8; 2]) {
        (self.VENDOR_ID_1, self.VENDOR_ID_2) = (octets[0], octets[1]);
    }

    pub fn device_id(&self) -> [u8; 3] {
        [self.DEVICE_ID_1, self.DEVICE_ID_2, self.DEVICE_ID_3]
    }

    pub fn set_device_id(&mut self, octets: [u8; 3]) -> io::Result<()> {
        validate_device_id(octets)?;
        (self.DEVICE_ID_1, self.DEVICE_ID_2, self.DEVICE_ID_3) = (octets[0], octets[1], octets[2]);
        Ok(())
    }

    pub fn function_id(&self) -> [u8; 2] {
        [self.FUNCTION_ID_1, self.FUNCTION_ID_2]
    }

    pub fn set_function_id(&mut self, octets: [u8; 2]) {
        (self.FUNCTION_ID_1, self.FUNCTION_ID_2) = (octets[0], octets[1]);
    }

    pub fn vendor_name(&self) -> &str {
        &self.VENDOR_NAME
    }

    pub fn set_vendor_name<S: Into<String>>(&mut self, value: S) -> io::Result<()> {
        self.VENDOR_NAME = validate_name(value.into(), "VENDOR_NAME")?;
        Ok(())
    }

    pub fn product_name(&self) -> &str {
        &self.PRODUCT_NAME
    }

    pub fn set_product_name<S: Into<String>>(&mut self, value: S) -> io::Result<()> {
        self.PRODUCT_NAME = validate_name(value.into(), "PRODUCT_NAME")?;
        Ok(())
    }

    pub fn min_cycle_time_in_ms(&self) -> f32 {
        self.MIN_CYCLE_TIME_IN_MS
    }

    pub fn set_min_cycle_time_in_ms(&mut self, value: f32) -> io::Result<()> {
        self.MIN_CYCLE_TIME_IN_MS = validate_min_cycle_time(value)?;
        Ok(())
    }

    pub fn op_pd_in_len(&self) -> ProcessDataLength {
        self.OP_PD_IN_LEN
    }

    pub fn set_op_pd_in_len(&mut self, value: ProcessDataLength) -> io::Result<()> {
        self.OP_PD_IN_LEN = value.validate("OP_PD_IN_LEN")?;
        Ok(())
    }

    pub fn op_pd_out_len(&self) -> ProcessDataLength {
        self.OP_PD_OUT_LEN
    }

    pub fn set_op_pd_out_len(&mut self, value: ProcessDataLength) -> io::Result<()> {
        self.OP_PD_OUT_LEN = value.validate("OP_PD_OUT_LEN")?;
        Ok(())
    }
}

fn parse_parameters(content: &str) -> io::Result<BTreeMap<String, String>> {
    let mut parameters = BTreeMap::new();
    let mut cursor = content;
    while let Some(start_idx) = cursor.find("/*CONFIG:") {
        cursor = &cursor[start_idx + "/*CONFIG:".len()..];
        let end_name = cursor
            .find("*/")
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "unterminated CONFIG header"))?;
        let key = cursor[..end_name].trim().to_string();
        cursor = &cursor[end_name + 2..];
        let end_block = cursor
            .find("/*ENDCONFIG*/")
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "missing ENDCONFIG marker"))?;
        let value = cursor[..end_block].trim().to_string();
        cursor = &cursor[end_block + "/*ENDCONFIG*/".len()..];
        if !key.is_empty() {
            parameters.insert(key, value);
        }
    }
    Ok(parameters)
}

fn take_parameter(map: &mut BTreeMap<String, String>, key: &str) -> io::Result<String> {
    map.remove(key).ok_or_else(|| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!("CONFIG parameter `{key}` not found"),
        )
    })
}

fn parse_u8(raw: &str, key: &str) -> io::Result<u8> {
    let mut cleaned = raw.trim().trim_end_matches(',').trim();
    if let Some(stripped) = cleaned.strip_suffix("u8") {
        cleaned = stripped.trim();
    }
    if let Some(stripped) = cleaned.strip_suffix("U8") {
        cleaned = stripped.trim();
    }
    let value = if let Some(hex) = cleaned
        .strip_prefix("0x")
        .or_else(|| cleaned.strip_prefix("0X"))
    {
        u8::from_str_radix(hex, 16).map_err(|_| invalid_value(key, raw))
    } else {
        cleaned.parse::<u8>().map_err(|_| invalid_value(key, raw))
    }?;
    Ok(value)
}

fn parse_f32(raw: &str, key: &str) -> io::Result<f32> {
    let cleaned = raw.trim().trim_end_matches(',').trim();
    cleaned.parse::<f32>().map_err(|_| invalid_value(key, raw))
}

fn parse_string(raw: &str, _key: &str) -> io::Result<String> {
    let mut cleaned = raw.trim().trim_end_matches(',').trim();
    if let (Some(stripped), true) = (cleaned.strip_prefix('"'), cleaned.ends_with('"')) {
        cleaned = &stripped[..stripped.len() - 1];
    }
    Ok(cleaned.to_string())
}

fn parse_process_data_length(raw: &str, key: &str) -> io::Result<ProcessDataLength> {
    let cleaned = raw.trim().trim_end_matches(',').trim();
    if let Some(inner) = cleaned
        .strip_prefix("Bit(")
        .and_then(|s| s.strip_suffix(')'))
    {
        return Ok(ProcessDataLength::Bit(
            inner
                .trim()
                .parse::<u8>()
                .map_err(|_| invalid_value(key, raw))?,
        ));
    }
    if let Some(inner) = cleaned
        .strip_prefix("Octet(")
        .and_then(|s| s.strip_suffix(')'))
    {
        return Ok(ProcessDataLength::Octet(
            inner
                .trim()
                .parse::<u8>()
                .map_err(|_| invalid_value(key, raw))?,
        ));
    }
    Err(invalid_value(key, raw))
}

fn validate_pre_op_od_len(value: u8) -> io::Result<u8> {
    match value {
        1 | 2 | 8 | 32 => Ok(value),
        _ => Err(io::Error::new(
            ErrorKind::InvalidData,
            "PRE_OP_OD_LEN must be 1, 2, 8, or 32 octets (IO-Link Spec Table A.8).",
        )),
    }
}

fn validate_op_od_len(value: u8) -> io::Result<u8> {
    match value {
        1 | 2 | 8 | 32 => Ok(value),
        _ => Err(io::Error::new(
            ErrorKind::InvalidData,
            "OP_OD_LEN must be 1, 2, 8, or 32 octets (IO-Link Spec Table A.10).",
        )),
    }
}

fn validate_revision_nibble(value: u8, key: &str) -> io::Result<u8> {
    if value <= 0x0F {
        Ok(value)
    } else {
        Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("{key} must be a 4-bit value (0x0–0xF)."),
        ))
    }
}

fn validate_device_id(octets: [u8; 3]) -> io::Result<()> {
    if octets.iter().all(|&b| b == 0) {
        Err(io::Error::new(
            ErrorKind::InvalidData,
            "DEVICE_ID must not be all zeros (IO-Link Spec B.1.9).",
        ))
    } else {
        Ok(())
    }
}

fn validate_name(name: String, key: &str) -> io::Result<String> {
    if name.is_empty() || name.len() > 64 {
        Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("{key} must be 1–64 characters (IO-Link Spec B.2.8/B.2.10)."),
        ))
    } else {
        Ok(name)
    }
}

fn validate_min_cycle_time(value: f32) -> io::Result<f32> {
    let valid = (0.4..=6.3).contains(&value)
        || (6.4..=31.6).contains(&value)
        || (32.0..=132.8).contains(&value);
    if valid {
        Ok(value)
    } else {
        Err(io::Error::new(
            ErrorKind::InvalidData,
            "MIN_CYCLE_TIME_IN_MS must be within 0.4–6.3, 6.4–31.6, or 32.0–132.8 ms (IO-Link Spec Table B.3).",
        ))
    }
}

fn invalid_value(key: &str, raw: &str) -> io::Error {
    io::Error::new(
        ErrorKind::InvalidData,
        format!("Unable to parse `{key}` from `{raw}`."),
    )
}
