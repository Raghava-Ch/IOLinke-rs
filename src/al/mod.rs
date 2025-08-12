use crate::{IoLinkResult, al, dl, system_management, types};

mod event_handler;
pub mod od_handler;
mod parameter_manager;
mod pd_handler;
pub mod services;

pub trait ApplicationLayerInd {
    fn al_read_ind(&mut self, index: u16, sub_index: u8) -> IoLinkResult<()>;

    fn al_write_ind(&mut self, index: u16, sub_index: u8, data: &[u8]) -> IoLinkResult<()>;

    fn al_abort_ind(&mut self) -> IoLinkResult<()>;

    fn al_set_input_ind(&mut self) -> IoLinkResult<()>;

    fn al_pd_cycle_ind(&mut self);

    fn al_get_output_ind(&mut self) -> IoLinkResult<()>;

    fn al_new_output_ind(&mut self) -> IoLinkResult<()>;

    fn al_event(&mut self) -> IoLinkResult<()>;

    fn al_control(&mut self, control_code: u8) -> IoLinkResult<()>;
}

pub struct ApplicationLayer<'a> {
    event_handler: event_handler::EventHandler<'a>,
    od_handler: od_handler::OnRequestDataHandler<'a>,
    services: services::ApplicationLayerServices,
    parameter_manager: parameter_manager::ParameterManager,
}

impl<'a> ApplicationLayer<'a> {
    pub fn poll(&mut self, data_link_layer: &mut dl::DataLinkLayer) -> IoLinkResult<()> {
        self.event_handler
            .poll(&mut self.services, data_link_layer)?;
        self.od_handler.poll(&mut self.services, data_link_layer)?;

        Ok(())
    }
}

impl<'a> system_management::SystemManagementInd for ApplicationLayer<'a> {
    fn sm_device_mode_ind(&mut self, mode: types::DeviceMode) -> system_management::SmResult<()> {
        self.parameter_manager.sm_device_mode_ind(mode)
    }
}

impl<'a> system_management::SystemManagementCnf for ApplicationLayer<'a> {
    fn sm_set_device_com_cnf(
        &self,
        result: system_management::SmResult<()>,
    ) -> system_management::SmResult<()> {
        todo!()
    }
    fn sm_get_device_com_cnf(
        &self,
        result: system_management::SmResult<&system_management::DeviceCom>,
    ) -> system_management::SmResult<()> {
        todo!()
    }
    fn sm_set_device_ident_cnf(
        &self,
        result: system_management::SmResult<()>,
    ) -> system_management::SmResult<()> {
        todo!()
    }
    fn sm_get_device_ident_cnf(
        &self,
        result: system_management::SmResult<&system_management::DeviceIdent>,
    ) -> system_management::SmResult<()> {
        todo!()
    }
    fn sm_set_device_mode_cnf(
        &self,
        result: system_management::SmResult<()>,
    ) -> system_management::SmResult<()> {
        todo!()
    }
}

impl<'a> dl::DlIsduAbort for ApplicationLayer<'a> {
    fn dl_isdu_abort(&mut self) -> IoLinkResult<()> {
        self.od_handler.dl_isdu_abort()
    }
}

impl<'a> dl::DlIsduTransportInd for ApplicationLayer<'a> {
    fn dl_isdu_transport_ind(&mut self, isdu: dl::Isdu) -> IoLinkResult<()> {
        self.od_handler.dl_isdu_transport_ind(isdu)
    }
}

impl<'a> dl::DlReadParamInd for ApplicationLayer<'a> {
    fn dl_read_param_ind(&mut self, address: u8) -> IoLinkResult<()> {
        self.od_handler.dl_read_param_ind(address)
    }
}

impl<'a> dl::DlWriteParamInd for ApplicationLayer<'a> {
    fn dl_write_param_ind(&mut self, index: u8, data: u8) -> IoLinkResult<()> {
        self.od_handler.dl_write_param_ind(index, data)
    }
}

impl<'a> dl::DlControlInd for ApplicationLayer<'a> {
    fn dl_control_ind(&mut self, control_code: types::DlControlCode) -> IoLinkResult<()> {
        self.services.dl_control_ind(control_code)
    }
}

impl<'a> Default for ApplicationLayer<'a> {
    fn default() -> Self {
        Self {
            event_handler: event_handler::EventHandler::new(),
            od_handler: od_handler::OnRequestDataHandler::new(),
            services: services::ApplicationLayerServices::new(),
            parameter_manager: parameter_manager::ParameterManager::new(),
        }
    }
}
