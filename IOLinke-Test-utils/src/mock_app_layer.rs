use heapless::Vec;
use iolinke_device::{AlEventCnf, ApplicationLayerServicesInd, DlControlCode, DlControlInd};
use iolinke_types::{custom::IoLinkResult, handlers};

use core::result::Result::{Ok, Err};
use core::default::Default;

const PD_OUTPUT_LENGTH: usize =
    iolinke_dev_config::device::process_data::config_pd_out_length_in_bytes() as usize;

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

impl handlers::sm::SystemManagementCnf for MockApplicationLayer {
    /// Handles device communication setup confirmations.
    ///
    /// This method is called when the system management confirms
    /// device communication setup operations.
    ///
    /// # Parameters
    ///
    /// * `result` - Result of the communication setup operation
    ///
    /// # Returns
    ///
    /// - `Ok(())` if confirmation was processed successfully
    /// - `Err(SmError)` if an error occurred
    fn sm_set_device_com_cnf(
        &self,
        __result: handlers::sm::SmResult<()>,
    ) -> handlers::sm::SmResult<()> {
        todo!("Implement device communication setup confirmation");
    }

    /// Handles device communication get confirmations.
    ///
    /// This method is called when the system management confirms
    /// device communication get operations.
    ///
    /// # Parameters
    ///
    /// * `result` - Result containing device communication parameters
    ///
    /// # Returns
    ///
    /// - `Ok(())` if confirmation was processed successfully
    /// - `Err(SmError)` if an error occurred
    fn sm_get_device_com_cnf(
        &self,
        _result: handlers::sm::SmResult<&handlers::sm::DeviceCom>,
    ) -> handlers::sm::SmResult<()> {
        todo!("Implement device communication get confirmation");
    }

    /// Handles device identification setup confirmations.
    ///
    /// This method is called when the system management confirms
    /// device identification setup operations.
    ///
    /// # Parameters
    ///
    /// * `result` - Result of the identification setup operation
    ///
    /// # Returns
    ///
    /// - `Ok(())` if confirmation was processed successfully
    /// - `Err(SmError)` if an error occurred
    fn sm_set_device_ident_cnf(
        &self,
        _result: handlers::sm::SmResult<()>,
    ) -> handlers::sm::SmResult<()> {
        todo!("Implement device identification setup confirmation");
    }

    /// Handles device identification get confirmations.
    ///
    /// This method is called when the system management confirms
    /// device identification get operations.
    ///
    /// # Parameters
    ///
    /// * `result` - Result containing device identification parameters
    ///
    /// # Returns
    ///
    /// - `Ok(())` if confirmation was processed successfully
    /// - `Err(SmError)` if an error occurred
    fn sm_get_device_ident_cnf(
        &self,
        _result: handlers::sm::SmResult<&iolinke_types::page::page1::DeviceIdent>,
    ) -> handlers::sm::SmResult<()> {
        todo!("Implement device identification get confirmation");
    }
    fn sm_set_device_mode_cnf(
        &self,
        _result: handlers::sm::SmResult<()>,
    ) -> handlers::sm::SmResult<()> {
        todo!("Implement device identification setup confirmation");
    }
}
