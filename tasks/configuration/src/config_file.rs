use crate::config_struct::ProcessDataLength;
use serde::{Deserialize, Serialize};
use std::io::{self, ErrorKind};

/// Root configuration wrapper corresponding to the `IODevice` entry in `device_config.toon`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parser {
    #[serde(rename = "IODevice")]
    pub io_device: IODevice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IODevice {
    #[serde(rename = "PreOperate")]
    pub pre_operate: PreOperate,
    #[serde(rename = "Operate")]
    pub operate: Operate,
    #[serde(rename = "Timing")]
    pub timing: Timing,
    #[serde(rename = "Vendor")]
    pub vendor: Vendor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreOperate {
    #[serde(rename = "OdLength", deserialize_with = "deserialize_u8")]
    pub od_length: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operate {
    #[serde(rename = "OdLength", deserialize_with = "deserialize_u8")]
    pub od_length: u8,
    #[serde(rename = "PdInLength")]
    pub pd_in_length: ProcessDataLength,
    #[serde(rename = "PdOutLength")]
    pub pd_out_length: ProcessDataLength,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timing {
    #[serde(rename = "MinCycleTime")]
    pub min_cycle_time: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vendor {
    #[serde(rename = "MajorRevisionID", deserialize_with = "deserialize_u8")]
    pub major_revision_id: u8,
    #[serde(rename = "MinorRevisionID", deserialize_with = "deserialize_u8")]
    pub minor_revision_id: u8,
    #[serde(rename = "VendorID1", deserialize_with = "deserialize_u8")]
    pub vendor_id_1: u8,
    #[serde(rename = "VendorID2", deserialize_with = "deserialize_u8")]
    pub vendor_id_2: u8,
    #[serde(rename = "DeviceID1", deserialize_with = "deserialize_u8")]
    pub device_id_1: u8,
    #[serde(rename = "DeviceID2", deserialize_with = "deserialize_u8")]
    pub device_id_2: u8,
    #[serde(rename = "DeviceID3", deserialize_with = "deserialize_u8")]
    pub device_id_3: u8,
    #[serde(rename = "FunctionID1", deserialize_with = "deserialize_u8")]
    pub function_id_1: u8,
    #[serde(rename = "FunctionID2", deserialize_with = "deserialize_u8")]
    pub function_id_2: u8,
    #[serde(rename = "VendorName")]
    pub vendor_name: String,
    #[serde(rename = "ProductName")]
    pub product_name: String,
    #[serde(rename = "Storage", default)]
    pub storage: Vec<VendorStorageEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorStorageEntry {
    #[serde(rename = "Index: i")]
    pub index: String,
    #[serde(rename = "Subindex: i")]
    pub subindex: String,
    #[serde(rename = "Length: i")]
    pub length: u8,
    #[serde(rename = "IndexRange")]
    pub index_range: String,
    #[serde(rename = "Access")]
    pub access: String,
    #[serde(rename = "Type")]
    pub data_type: String,
    #[serde(rename = "DefaultValue")]
    pub default_value: String,
}

impl VendorStorageEntry {
    pub fn to_macro_row(&self) -> String {
        let index = self.index.trim();
        let subindex = self.subindex.trim();
        let range = self.index_range.trim();
        let access = self.access.trim();
        let data_type = self.data_type.trim();
        let default_expr = self.formatted_default_value();
        format!(
            "(            /* Index */ {index},         /* Subindex */ {subindex},               /* Length */ {},        /* IndexRange */ {range},        /* Access */ {access},     /* Type */ {data_type},  /* DefaultValue */ {default_expr}),",
            self.length
        )
    }

    fn formatted_default_value(&self) -> String {
        let raw = self.default_value.trim();
        if raw.is_empty() {
            return String::from("&[]");
        }

        match self.data_type.trim() {
            "StringT" => {
                if raw.starts_with("b\"") || raw.starts_with("B\"") {
                    raw.to_string()
                } else if raw.starts_with('"') && raw.ends_with('"') {
                    format!("b{}", raw)
                } else {
                    let sanitized = raw.trim_matches('"');
                    format!("b\"{}\"", sanitized)
                }
            }
            _ => {
                let value = raw.trim_matches('"');
                if value.starts_with('&') {
                    value.to_string()
                } else if value.starts_with('[') && value.ends_with(']') {
                    format!("&{value}")
                } else {
                    format!("&[{value}]")
                }
            }
        }
    }
}

impl Parser {
    pub fn from_toon_str(input: &str) -> io::Result<Self> {
        let mut sanitized = String::with_capacity(input.len());
        for line in input.lines().filter(|line| !line.trim().is_empty()) {
            sanitized.push_str(&normalize_line(line));
            sanitized.push('\n');
        }

        let value = toon_rust::decode(&sanitized, None)
            .map_err(|err| io::Error::new(ErrorKind::InvalidData, err.to_string()))?;
        serde_json::from_value(value)
            .map_err(|err| io::Error::new(ErrorKind::InvalidData, err.to_string()))
    }

    pub fn validate(&self) -> io::Result<()> {
        self.io_device.validate()
    }
}

impl IODevice {
    pub fn validate(&self) -> io::Result<()> {
        self.pre_operate.validate()?;
        self.operate.validate()?;
        self.timing.validate()?;
        self.vendor.validate()
    }
}

impl PreOperate {
    pub fn validate(&self) -> io::Result<()> {
        validate_od_length(self.od_length, "IODevice.PreOperate.OdLength")
    }
}

impl Operate {
    pub fn validate(&self) -> io::Result<()> {
        validate_od_length(self.od_length, "IODevice.Operate.OdLength")?;
        validate_process_data_length(self.pd_in_length, "IODevice.Operate.PdInLength")?;
        validate_process_data_length(self.pd_out_length, "IODevice.Operate.PdOutLength")
    }
}

impl Timing {
    pub fn validate(&self) -> io::Result<()> {
        validate_min_cycle_time(self.min_cycle_time, "IODevice.Timing.MinCycleTime")
    }
}

impl Vendor {
    pub fn validate(&self) -> io::Result<()> {
        validate_revision_nibble(self.major_revision_id, "IODevice.Vendor.MajorRevisionID")?;
        validate_revision_nibble(self.minor_revision_id, "IODevice.Vendor.MinorRevisionID")?;
        validate_device_id(
            [self.device_id_1, self.device_id_2, self.device_id_3],
            "IODevice.Vendor.DeviceID",
        )?;
        validate_name(&self.vendor_name, "IODevice.Vendor.VendorName")?;
        validate_name(&self.product_name, "IODevice.Vendor.ProductName")
    }
}

fn deserialize_u8<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error as _;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum U8Repr<'a> {
        Num(u8),
        Str(&'a str),
    }

    match U8Repr::deserialize(deserializer)? {
        U8Repr::Num(value) => Ok(value),
        U8Repr::Str(raw) => {
            let trimmed = raw.trim();
            if let Some(hex) = trimmed
                .strip_prefix("0x")
                .or_else(|| trimmed.strip_prefix("0X"))
            {
                u8::from_str_radix(hex, 16)
                    .map_err(|_| D::Error::custom("invalid hexadecimal u8 literal"))
            } else {
                trimmed
                    .parse::<u8>()
                    .map_err(|_| D::Error::custom("invalid decimal u8 literal"))
            }
        }
    }
}

fn validate_od_length(value: u8, path: &str) -> io::Result<()> {
    matches!(value, 1 | 2 | 8 | 32)
        .then_some(())
        .ok_or_else(|| {
            invalid_data(
                path,
                "Allowed values are 1, 2, 8, 32 (IO-Link Tables A.8/A.10).",
            )
        })
}

fn validate_process_data_length(length: ProcessDataLength, path: &str) -> io::Result<()> {
    use ProcessDataLength::*;
    match length {
        Bit(bits) if bits <= 16 => Ok(()),
        Octet(octets) if octets <= 32 => Ok(()),
        _ => Err(invalid_data(
            path,
            "Process data length out of range (bit ≤16, octet ≤32; IO-Link Table B.6).",
        )),
    }
}

fn validate_revision_nibble(value: u8, path: &str) -> io::Result<()> {
    (value <= 0x0F).then_some(()).ok_or_else(|| {
        invalid_data(
            path,
            "Revision nibble must be within 0x0–0xF (IO-Link B.1.4).",
        )
    })
}

fn validate_device_id(octets: [u8; 3], path: &str) -> io::Result<()> {
    (!octets.iter().all(|&octet| octet == 0))
        .then_some(())
        .ok_or_else(|| invalid_data(path, "DeviceID must not be all zeros (IO-Link B.1.9)."))
}

fn validate_name(value: &str, path: &str) -> io::Result<()> {
    let len = value.len();
    (len != 0 && len <= 64)
        .then_some(())
        .ok_or_else(|| invalid_data(path, "Name must be 1–64 characters (IO-Link B.2.8/B.2.10)."))
}

fn validate_min_cycle_time(value: f32, path: &str) -> io::Result<()> {
    let valid = (0.4..=6.3).contains(&value)
        || (6.4..=31.6).contains(&value)
        || (32.0..=132.8).contains(&value);
    valid.then_some(()).ok_or_else(|| {
        invalid_data(
            path,
            "MinCycleTime must follow Table B.3 ranges (0.4–132.8 ms).",
        )
    })
}

fn invalid_data(path: &str, msg: &str) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, format!("{path}: {msg}"))
}

fn normalize_line(line: &str) -> String {
    let trimmed = line.trim_end();
    if let Some(colon_idx) = trimmed.find(':') {
        let (head, tail) = trimmed.split_at(colon_idx + 1);
        let value = tail.trim();
        if let Some(hex) = value
            .strip_prefix("0x")
            .or_else(|| value.strip_prefix("0X"))
        {
            if hex.chars().all(|c| c.is_ascii_hexdigit()) {
                if let Ok(number) = u8::from_str_radix(hex, 16) {
                    return format!("{head} {number}");
                }
            }
        }
        format!("{head} {value}")
    } else {
        trimmed.to_owned()
    }
}
