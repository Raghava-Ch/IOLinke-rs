mod config_file;
mod config_struct;
mod utils;
mod write_config;

const CONFIG_FILE_NAME: &str = "device_config.toon";
const CRATE_NAME: &str = "IOLinke-Dev-config";
fn main() {
    let workspace_root = utils::get_workspace_root();
    let config_file_path = workspace_root.join(CRATE_NAME).join(CONFIG_FILE_NAME);
    let config_content =
        std::fs::read_to_string(&config_file_path).expect("Failed to read config file");

    let parser = config_file::Parser::from_toon_str(&config_content)
        .expect("Failed to deserialize config file");
    parser.validate().expect("Config validation failed");
    println!("Configuration validated successfully");

    let config_writer = write_config::ConfigurationWriter::new(workspace_root);
    config_writer
        .write_on_req_data_config(
            parser.io_device.pre_operate.od_length,
            parser.io_device.operate.od_length,
        )
        .expect("Failed to write on-request data config");

    config_writer
        .write_process_data_config(
            parser.io_device.operate.pd_in_length,
            parser.io_device.operate.pd_out_length,
        )
        .expect("Failed to write process data config");

    config_writer
        .write_timings_config(parser.io_device.timing.min_cycle_time)
        .expect("Failed to write timings config");

    config_writer
        .write_vendor_specifics_config(
            parser.io_device.vendor.major_revision_id,
            parser.io_device.vendor.minor_revision_id,
            parser.io_device.vendor.vendor_id_1,
            parser.io_device.vendor.vendor_id_2,
            parser.io_device.vendor.device_id_1,
            parser.io_device.vendor.device_id_2,
            parser.io_device.vendor.device_id_3,
            parser.io_device.vendor.function_id_1,
            parser.io_device.vendor.function_id_2,
            &parser.io_device.vendor.vendor_name,
            &parser.io_device.vendor.product_name,
        )
        .expect("Failed to write vendor specifics config");

    config_writer
        .write_vendor_parameter_storage_config(&parser.io_device.vendor.storage)
        .expect("Failed to write vendor parameter storage config");
    println!("Configuration written to the stack project");
}
