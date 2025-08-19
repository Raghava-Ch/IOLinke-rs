use crate::dl::DlPDOutputTransportInd;
use crate::{dl, storage, system_management, types, IoLinkResult};

mod event_handler;
pub mod od_handler;
mod parameter_manager;
mod pd_handler;
pub mod services;
mod data_storage;

use heapless::Vec;
pub use parameter_manager::DeviceParametersIndex;
pub use parameter_manager::SubIndex;
pub use parameter_manager::DirectParameterPage1SubIndex;
pub use parameter_manager::DataStorageIndexSubIndex;

pub trait ApplicationLayerReadWriteInd {
    fn al_read_ind(&mut self, index: u16, sub_index: u8) -> IoLinkResult<()>;

    fn al_write_ind(&mut self, index: u16, sub_index: u8, data: &[u8]) -> IoLinkResult<()>;

    fn al_abort_ind(&mut self) -> IoLinkResult<()>;
}

pub trait ApplicationLayerProcessDataInd {
    fn al_set_input_ind(&mut self) -> IoLinkResult<()>;

    fn al_pd_cycle_ind(&mut self);

    fn al_get_output_ind(&mut self) -> IoLinkResult<()>;

    fn al_new_output_ind(&mut self) -> IoLinkResult<()>;

    fn al_control(&mut self, control_code: u8) -> IoLinkResult<()>;
}

pub trait ApplicationLayerEventInd {
    fn al_event(&mut self) -> IoLinkResult<()>;
}

pub struct ApplicationLayer {
    event_handler: event_handler::EventHandler,
    od_handler: od_handler::OnRequestDataHandler,
    services: services::ApplicationLayerServices,
    parameter_manager: parameter_manager::ParameterManager,
    data_storage: data_storage::DataStorage,
}

impl ApplicationLayer {
    pub fn poll(&mut self, data_link_layer: &mut dl::DataLinkLayer) -> IoLinkResult<()> {
        self.event_handler
            .poll(&mut self.services, data_link_layer)?;
        self.od_handler.poll(&mut self.services, data_link_layer)?;
        self.parameter_manager
            .poll(&mut self.od_handler, &mut self.data_storage)?;
        self.data_storage.poll(&mut self.event_handler, &mut self.parameter_manager)?;
        Ok(())
    }
}

impl services::AlEventReq for ApplicationLayer {
    fn al_event_req(&mut self, event_count: u8, event_entries: &[storage::event_memory::EventEntry]) -> IoLinkResult<()> {
        self.event_handler.al_event_req(event_count, event_entries)
    }
}

impl system_management::SystemManagementInd for ApplicationLayer {
    fn sm_device_mode_ind(&mut self, mode: types::DeviceMode) -> system_management::SmResult<()> {
        let _ = self.parameter_manager.sm_device_mode_ind(mode);
        let _ = self.data_storage.sm_device_mode_ind(mode);

        Ok(())
    }
}

impl system_management::SystemManagementCnf for ApplicationLayer {
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

impl dl::DlIsduAbort for ApplicationLayer {
    fn dl_isdu_abort(&mut self) -> IoLinkResult<()> {
        self.od_handler.dl_isdu_abort()
    }
}

impl dl::DlIsduTransportInd for ApplicationLayer {
    fn dl_isdu_transport_ind(&mut self, isdu: dl::Isdu) -> IoLinkResult<()> {
        self.od_handler.dl_isdu_transport_ind(isdu)
    }
}

impl dl::DlReadParamInd for ApplicationLayer {
    fn dl_read_param_ind(&mut self, address: u8) -> IoLinkResult<()> {
        self.od_handler.dl_read_param_ind(address)
    }
}

impl dl::DlWriteParamInd for ApplicationLayer {
    fn dl_write_param_ind(&mut self, index: u8, data: u8) -> IoLinkResult<()> {
        self.od_handler.dl_write_param_ind(index, data)
    }
}

impl dl::DlControlInd for ApplicationLayer {
    fn dl_control_ind(&mut self, control_code: types::DlControlCode) -> IoLinkResult<()> {
        self.services.dl_control_ind(control_code)
    }
}

impl Default for ApplicationLayer {
    fn default() -> Self {
        Self {
            event_handler: event_handler::EventHandler::new(),
            od_handler: od_handler::OnRequestDataHandler::new(),
            services: services::ApplicationLayerServices::new(),
            parameter_manager: parameter_manager::ParameterManager::new(),
            data_storage: data_storage::DataStorage::new(),
        }
    }
}

impl ApplicationLayerReadWriteInd for ApplicationLayer {
    fn al_read_ind(&mut self, index: u16, sub_index: u8) -> IoLinkResult<()> {
        self.parameter_manager.al_read_ind(index, sub_index)
    }

    fn al_write_ind(&mut self, index: u16, sub_index: u8, data: &[u8]) -> IoLinkResult<()> {
        self.parameter_manager.al_write_ind(index, sub_index, data)
    }

    fn al_abort_ind(&mut self) -> IoLinkResult<()> {
        self.parameter_manager.al_abort_ind()
    }
}

impl dl::DlPDOutputTransportInd for ApplicationLayer {
    fn dl_pd_output_transport_ind(&mut self, pd_out: &Vec<u8, {dl::PD_OUTPUT_LENGTH}>) -> IoLinkResult<()> {
        todo!()
    }

    fn dl_pd_cycle_ind(&mut self) -> IoLinkResult<()> {
        todo!()
    }
}
