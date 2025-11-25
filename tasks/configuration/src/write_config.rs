use crate::config_file::VendorStorageEntry;
use crate::config_struct::ProcessDataLength;

pub struct ConfigurationWriter {
    workspace_path: std::path::PathBuf,
}

const CONFIG_ON_REQ_DATA_FILE_NAME: &str = "on_req_data.rs";
const CONFIG_PROCESS_DATA_FILE_NAME: &str = "process_data.rs";
const CONFIG_VENDOR_SPECIFICS_FILE_NAME: &str = "vendor_specifics.rs";
const CONFIG_TIMINGS_FILE_NAME: &str = "timings.rs";

const CONFIG_FILES_RELATIVE_PATH: &str = "IOLinke-Dev-config/src/device";
const DERIVED_CONFIG_FILES_RELATIVE_PATH: &str = "IOLinke-Derived-config/src/device";

impl ConfigurationWriter {
    pub fn new(workspace_path: std::path::PathBuf) -> Self {
        Self { workspace_path }
    }
    pub fn write_on_req_data_config(
        &self,
        pre_op_od_len: u8,
        op_od_len: u8,
    ) -> std::io::Result<()> {
        let config_file_path = self.device_config_path(CONFIG_ON_REQ_DATA_FILE_NAME);
        write_config_param_to_file(
            &config_file_path,
            "PRE_OP_OD_LEN",
            &pre_op_od_len.to_string(),
        )?;
        write_config_param_to_file(&config_file_path, "OP_OD_LEN", &op_od_len.to_string())?;
        Ok(())
    }

    pub fn write_process_data_config(
        &self,
        pd_in_len: ProcessDataLength,
        pd_out_len: ProcessDataLength,
    ) -> std::io::Result<()> {
        let config_file_path = self.device_config_path(CONFIG_PROCESS_DATA_FILE_NAME);
        write_config_param_to_file(&config_file_path, "OP_PD_IN_LEN", &pd_in_len.to_string())?;
        write_config_param_to_file(&config_file_path, "OP_PD_OUT_LEN", &pd_out_len.to_string())?;
        Ok(())
    }

    pub fn write_timings_config(&self, min_cycle_time: f32) -> std::io::Result<()> {
        let config_file_path = self.device_config_path(CONFIG_TIMINGS_FILE_NAME);
        write_config_param_to_file(
            &config_file_path,
            "MIN_CYCLE_TIME_IN_MS",
            &min_cycle_time.to_string(),
        )
    }

    pub fn write_vendor_specifics_config(
        &self,
        major_revision_id: u8,
        minor_revision_id: u8,
        vendor_id_1: u8,
        vendor_id_2: u8,
        device_id_1: u8,
        device_id_2: u8,
        device_id_3: u8,
        function_id_1: u8,
        function_id_2: u8,
        vendor_name: &str,
        product_name: &str,
    ) -> std::io::Result<()> {
        let config_file_path = self.device_config_path(CONFIG_VENDOR_SPECIFICS_FILE_NAME);
        let hex_fields = [
            ("MAJOR_REVISION_ID", major_revision_id),
            ("MINOR_REVISION_ID", minor_revision_id),
            ("VENDOR_ID_1", vendor_id_1),
            ("VENDOR_ID_2", vendor_id_2),
            ("DEVICE_ID_1", device_id_1),
            ("DEVICE_ID_2", device_id_2),
            ("DEVICE_ID_3", device_id_3),
            ("FUNCTION_ID_1", function_id_1),
            ("FUNCTION_ID_2", function_id_2),
        ];

        for (name, value) in hex_fields {
            write_config_param_to_file(&config_file_path, name, &format!("0x{:02X}", value))?;
        }

        write_config_param_to_file(&config_file_path, "VENDOR_NAME", &quoted(vendor_name))?;
        write_config_param_to_file(&config_file_path, "PRODUCT_NAME", &quoted(product_name))?;

        Ok(())
    }

    pub fn write_vendor_parameter_storage_config(
        &self,
        entries: &[VendorStorageEntry],
    ) -> std::io::Result<()> {
        if entries.is_empty() {
            return self.write_vendor_parameter_block("");
        }

        let indent = "        ";
        let lines: Vec<String> = entries
            .iter()
            .map(|entry| format!("{indent}{}", entry.to_macro_row()))
            .collect();

        let mut block = String::new();
        block.push('\n');
        block.push_str(&lines.join("\n"));
        block.push('\n');
        block.push_str(indent);

        self.write_vendor_parameter_block(&block)
    }

    fn write_vendor_parameter_block(&self, block: &str) -> std::io::Result<()> {
        let derived_path = self
            .workspace_path
            .join(DERIVED_CONFIG_FILES_RELATIVE_PATH)
            .join(CONFIG_VENDOR_SPECIFICS_FILE_NAME);
        write_config_param_to_file(&derived_path, "VENDOR_PARAMS", block)
    }

    fn device_config_path(&self, file_name: &str) -> std::path::PathBuf {
        self.workspace_path
            .join(CONFIG_FILES_RELATIVE_PATH)
            .join(file_name)
    }
}

fn write_config_param_to_file(
    file_path: &std::path::Path,
    param_name: &str,
    param_value: &str,
) -> std::io::Result<()> {
    use std::fs;
    use std::io::{self, ErrorKind};

    let content = fs::read_to_string(file_path)?;
    let start_marker = format!("/*CONFIG:{param_name}*/");
    let end_marker = "/*ENDCONFIG*/";

    let start = content.find(&start_marker).ok_or_else(|| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "CONFIG marker `{}` not found in {:?}",
                param_name, file_path
            ),
        )
    })?;

    let value_start = start + start_marker.len();
    let trailing_slice = &content[value_start..];
    let end_rel = trailing_slice.find(end_marker).ok_or_else(|| {
        io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "`/*ENDCONFIG*/` marker missing for `{}` in {:?}",
                param_name, file_path
            ),
        )
    })?;
    let value_end = value_start + end_rel;

    let between = &content[value_start..value_end];
    let trimmed = between.trim();

    let new_between = if trimmed.is_empty() {
        param_value.to_string()
    } else {
        let start_idx = between
            .char_indices()
            .find(|&(_, ch)| !ch.is_whitespace())
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        let end_idx = between
            .char_indices()
            .rev()
            .find(|&(_, ch)| !ch.is_whitespace())
            .map(|(idx, ch)| idx + ch.len_utf8())
            .unwrap_or(start_idx);

        let leading = &between[..start_idx];
        let trailing = &between[end_idx..];
        let existing = &between[start_idx..end_idx];
        if existing.contains('\n') || param_value.contains('\n') {
            param_value.to_string()
        } else {
            let merged = merge_literal(existing, param_value);
            format!("{leading}{merged}{trailing}")
        }
    };

    if between == new_between {
        return Ok(());
    }

    let mut updated = String::with_capacity(content.len() - between.len() + new_between.len());
    updated.push_str(&content[..value_start]);
    updated.push_str(&new_between);
    updated.push_str(&content[value_end..]);

    fs::write(file_path, updated)
}

fn quoted(value: &str) -> String {
    format!("\"{value}\"")
}

fn merge_literal(existing: &str, replacement: &str) -> String {
    if existing.is_empty()
        || existing.starts_with('"')
        || existing.starts_with('\'')
        || existing.contains('(')
    {
        return replacement.to_string();
    }

    let mut core = existing;
    let mut trailing_punct = String::new();

    if let Some(stripped) = core.strip_suffix(',') {
        core = stripped;
        trailing_punct.push(',');
    }
    if let Some(stripped) = core.strip_suffix(';') {
        core = stripped;
        trailing_punct.insert(0, ';');
    }

    const TYPE_SUFFIXES: [&str; 18] = [
        "u8", "u16", "u32", "u64", "usize", "i8", "i16", "i32", "i64", "isize", "f32", "f64", "U8",
        "U16", "U32", "U64", "USIZE", "ISIZE",
    ];

    let mut type_suffix = "";
    for suffix in TYPE_SUFFIXES.iter() {
        if core.ends_with(suffix) {
            type_suffix = suffix;
            core = &core[..core.len() - suffix.len()];
            break;
        }
    }

    // If removing a suffix consumed entire literal, fall back to replacement as-is
    if core.is_empty() {
        return format!("{replacement}{type_suffix}{trailing_punct}");
    }

    format!("{replacement}{type_suffix}{trailing_punct}")
}
