// //! Process Data Handler for Application Layer
// //!
// //! This module implements the Process Data Handler state machine as defined in
// //! IO-Link Specification v1.1.4 Section 8.3.4

use crate::{al::services, dl};
use heapless::Vec;
use iolinke_types::custom::IoLinkResult;
use iolinke_types::handlers::pd::DlPDInputUpdate;

use core::result::Result::Ok;
use core::default::Default;

pub struct ProcessDataHandler {}

impl ProcessDataHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn dl_pd_output_transport_ind<
        ALS: services::ApplicationLayerServicesInd + services::AlEventCnf,
    >(
        &mut self,
        pd_out: &Vec<u8, { dl::PD_OUTPUT_LENGTH }>,
        services: &mut ALS,
    ) -> IoLinkResult<()> {
        services.al_new_output_ind(pd_out)
    }

    fn dl_pd_input_update_req(
        &mut self,
        length: u8,
        input_data: &[u8],
        data_link_layer: &mut dl::DataLinkLayer,
    ) -> IoLinkResult<()> {
        data_link_layer.dl_pd_input_update_req(length, input_data)
    }

    pub fn dl_pd_cycle_ind<ALS: services::ApplicationLayerServicesInd + services::AlEventCnf>(
        &mut self,
        services: &mut ALS,
    ) -> IoLinkResult<()> {
        services.al_pd_cycle_ind();
        Ok(())
    }
}

impl services::AlSetInputReq for ProcessDataHandler {
    fn al_set_input_req(
        &mut self,
        pd_data: &[u8],
        data_link_layer: &mut dl::DataLinkLayer,
    ) -> IoLinkResult<()> {
        self.dl_pd_input_update_req(pd_data.len() as u8, pd_data, data_link_layer)
    }
}

impl Default for ProcessDataHandler {
    fn default() -> Self {
        Self::new()
    }
}
