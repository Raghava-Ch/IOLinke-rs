use iolinke_device::{AlEventCnf, ApplicationLayerServicesInd, DlControlCode, DlControlInd};
use iolinke_types::{custom::IoLinkResult};
use heapless::Vec;

const PD_OUTPUT_LENGTH: usize = iolinke_dev_config::device::process_data::config_pd_out_length_in_bytes() as usize;

pub struct MockApplicationLayer {}

impl MockApplicationLayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MockApplicationLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl DlControlInd for MockApplicationLayer {
    fn dl_control_ind(&mut self, _control_code: DlControlCode) -> IoLinkResult<()> {
        Ok(())
    }
}

impl ApplicationLayerServicesInd for MockApplicationLayer {
    fn al_read_ind(&mut self, _index: u16, _sub_index: u8) -> IoLinkResult<()> {
        todo!()
    }

    fn al_write_ind(&mut self, _index: u16, _sub_index: u8, _data: &[u8]) -> IoLinkResult<()> {
        todo!()
    }

    fn al_abort_ind(&mut self) -> IoLinkResult<()> {
        todo!()
    }

    fn al_pd_cycle_ind(&mut self) {
        println!("AL PD Cycle Ind");
    }

    fn al_new_output_ind(&mut self, _pd_out: &Vec<u8, { PD_OUTPUT_LENGTH }>) -> IoLinkResult<()> {
        println!("AL New Output Ind");
        Err(iolinke_types::custom::IoLinkError::NoImplFound)
    }

    fn al_control_ind(&mut self, _control_code: DlControlCode) -> IoLinkResult<()> {
        println!("AL Control Ind");
        Err(iolinke_types::custom::IoLinkError::NoImplFound)
    }
}

impl AlEventCnf for MockApplicationLayer {
    fn al_event_cnf(&mut self) -> IoLinkResult<()> {
        todo!()
    }
}
