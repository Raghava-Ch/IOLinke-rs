use crate::custom::IoLinkResult;
use heapless::Vec;
use iolinke_dev_config::device as dev_config;

pub const PD_INPUT_LENGTH: usize =
    dev_config::process_data::config_pd_in_length_in_bytes() as usize;
pub const PD_OUTPUT_LENGTH: usize =
    dev_config::process_data::config_pd_out_length_in_bytes() as usize;

pub trait DlPDInputUpdate {
    fn dl_pd_input_update_req(&mut self, length: u8, input_data: &[u8]) -> IoLinkResult<()>;
}

pub trait DlPDOutputTransportInd {
    fn dl_pd_output_transport_ind(
        &mut self,
        pd_out: &Vec<u8, PD_OUTPUT_LENGTH>,
    ) -> IoLinkResult<()>;
    fn dl_pd_cycle_ind(&mut self) -> IoLinkResult<()>;
}

/// Process data input/output structure
/// See IO-Link v1.1.4 Section 8.4.2
#[derive(Debug, Default, Clone)]
pub struct ProcessData {
    /// Input data from device
    pub _input: Vec<u8, PD_INPUT_LENGTH>,
    pub _input_length: u8,
    /// Output data to device
    pub _output: Vec<u8, PD_OUTPUT_LENGTH>,
    pub _output_length: u8,
    /// Data validity flag
    pub _valid: bool,
}

/// All the Process Data Handler configuration states used
/// See Figure 47 – State machine of the Device Process Data handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdConfState {
    /// (Handler changed to the ACTIVE state)
    Active,
    /// (Handler changed to the INACTIVE state)
    Inactive,
}
