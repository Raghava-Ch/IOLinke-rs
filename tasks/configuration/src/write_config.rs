use crate::config_struct::ProcessDataLength;

pub struct ConfigurationWriter {
    workspace_path: std::path::PathBuf,
}

const CONFIG_ON_REQ_DATA_FILE_NAME: &str = "on_req_data.rs";
const CONFIG_PROCESS_DATA_FILE_NAME: &str = "process_data.rs";
const CONFIG_VENDOR_SPECIFICS_FILE_NAME: &str = "vendor_specifics.rs";
const CONFIG_TIMINGS_FILE_NAME: &str = "timings.rs";

const CONFIG_FILES_RELATIVE_PATH: &str = "IOLinke-Dev-config/src/device";

impl ConfigurationWriter {
    pub fn new(workspace_path: std::path::PathBuf) -> Self {
        Self { workspace_path }
    }
    pub fn write_on_req_data_config(
        &self,
        pre_op_od_len: u8,
        op_od_len: u8,
    ) -> std::io::Result<()> {
        let config_file_path = self
            .workspace_path
            .join(CONFIG_FILES_RELATIVE_PATH)
            .join(CONFIG_ON_REQ_DATA_FILE_NAME);
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
        let config_file_path = self
            .workspace_path
            .join(CONFIG_FILES_RELATIVE_PATH)
            .join(CONFIG_PROCESS_DATA_FILE_NAME);
        write_config_param_to_file(&config_file_path, "OP_PD_IN_LEN", &pd_in_len.to_string())?;
        write_config_param_to_file(&config_file_path, "OP_PD_OUT_LEN", &pd_out_len.to_string())?;
        Ok(())
    }

    pub fn write_timings_config(&self, min_cycle_time: f32) -> std::io::Result<()> {
        let config_file_path = self
            .workspace_path
            .join(CONFIG_FILES_RELATIVE_PATH)
            .join(CONFIG_TIMINGS_FILE_NAME);
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
        vendor_name: String,
        product_name: String,
    ) -> std::io::Result<()> {
        let config_file_path = self
            .workspace_path
            .join(CONFIG_FILES_RELATIVE_PATH)
            .join(CONFIG_VENDOR_SPECIFICS_FILE_NAME);
        write_config_param_to_file(
            &config_file_path,
            "MAJOR_REVISION_ID",
            &format!("0x{:02X}", major_revision_id),
        )?;
        write_config_param_to_file(
            &config_file_path,
            "MINOR_REVISION_ID",
            &format!("0x{:02X}", minor_revision_id),
        )?;

        write_config_param_to_file(&config_file_path, "VENDOR_ID_1", &format!("0x{:02X}", vendor_id_1))?;
        write_config_param_to_file(&config_file_path, "VENDOR_ID_2", &format!("0x{:02X}", vendor_id_2))?;

        write_config_param_to_file(&config_file_path, "DEVICE_ID_1", &format!("0x{:02X}", device_id_1))?;
        write_config_param_to_file(&config_file_path, "DEVICE_ID_2", &format!("0x{:02X}", device_id_2))?;
        write_config_param_to_file(&config_file_path, "DEVICE_ID_3", &format!("0x{:02X}", device_id_3))?;

        write_config_param_to_file(
            &config_file_path,
            "FUNCTION_ID_1",
            &format!("0x{:02X}", function_id_1),
        )?;
        write_config_param_to_file(
            &config_file_path,
            "FUNCTION_ID_2",
            &format!("0x{:02X}", function_id_2),
        )?;

        write_config_param_to_file(&config_file_path, "VENDOR_ID_1", &format!("0x{:02X}", vendor_id_1))?;
        write_config_param_to_file(&config_file_path, "VENDOR_ID_2", &format!("0x{:02X}", vendor_id_2))?;

        write_config_param_to_file(&config_file_path, "DEVICE_ID_1", &format!("0x{:02X}", device_id_1))?;
        write_config_param_to_file(&config_file_path, "DEVICE_ID_2", &format!("0x{:02X}", device_id_2))?;
        write_config_param_to_file(&config_file_path, "DEVICE_ID_3", &format!("0x{:02X}", device_id_3))?;

        write_config_param_to_file(
            &config_file_path,
            "FUNCTION_ID_1",
            &format!("0x{:02X}", function_id_1),
        )?;
        write_config_param_to_file(
            &config_file_path,
            "FUNCTION_ID_2",
            &format!("0x{:02X}", function_id_2),
        )?;

        write_config_param_to_file(&config_file_path, "VENDOR_NAME", &format!("\"{vendor_name}\""))?;

        write_config_param_to_file(&config_file_path, "PRODUCT_NAME", &format!("\"{product_name}\""))?;

        Ok(())
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
    let leading_len = between
        .char_indices()
        .find(|&(_, ch)| !ch.is_whitespace())
        .map(|(idx, _)| idx)
        .unwrap_or(between.len());
    let trailing_len = between
        .char_indices()
        .rev()
        .find(|&(_, ch)| !ch.is_whitespace())
        .map(|(idx, ch)| between.len() - idx - ch.len_utf8())
        .unwrap_or(0);

    let leading = &between[..leading_len];
    let trailing = &between[between.len() - trailing_len..];
    let new_between = format!("{leading}{param_value}{trailing}");

    if between == new_between {
        return Ok(());
    }

    let mut updated = String::with_capacity(content.len() - between.len() + new_between.len());
    updated.push_str(&content[..value_start]);
    updated.push_str(&new_between);
    updated.push_str(&content[value_end..]);

    fs::write(file_path, updated)
}
