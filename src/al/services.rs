//! Application Layer API implementation
//!
//! This module implements the Application Layer interface as defined in
//! IO-Link Specification v1.1.4 Section 8.4

use crate::{dl, storage, types::{self, IoLinkResult}};

/// Application Layer trait defining all request/indication methods
/// See IO-Link v1.1.4 Section 8.4
pub trait ApplicationLayerServicesInd {
    fn al_read_ind(&mut self, index: u16, sub_index: u8) -> IoLinkResult<()>;

    fn al_write_ind(&mut self, index: u16, sub_index: u8, data: &[u8]) -> IoLinkResult<()>;

    fn al_abort_ind(&mut self) -> IoLinkResult<()>;

    fn al_pd_cycle_ind(&mut self);

    fn al_new_output_ind(&mut self) -> IoLinkResult<()>;
    
    fn al_control_ind(&mut self, control_code: types::DlControlCode) -> IoLinkResult<()>;
}

pub enum AlRspError {
    Error(u8, u8), // (error code, additional error code)
    StateConflict,
    NoData,
}
pub type AlResult<T> = Result<T, AlRspError>;

pub trait AlReadRsp<'a> {
    fn al_read_rsp(&mut self, result: AlResult<&'a [u8]>) -> IoLinkResult<()>;
}
pub trait AlWriteRsp {
    fn al_write_rsp(&mut self, result: AlResult<()>) -> IoLinkResult<()>;
}

pub trait AlEventReq<'a> {
    fn al_event_req(
        &mut self,
        event_count: u8,
        event_entries: &'a [storage::event_memory::EventEntry],
    ) -> IoLinkResult<()>; 
}
pub trait AlControlReq {
    /// Handle control codes as defined in IO-Link Specification v1.1.4 Section
    fn al_control_req(&mut self, control_code: types::DlControlCode) -> IoLinkResult<()>;
}

pub trait AlSetInputReq {
    fn al_set_input_req(&mut self, pd_data: &[u8]) -> IoLinkResult<()>;
}

pub trait AlSetInputCnf {
    fn al_set_input_cnf(&mut self, result: AlResult<()>) -> IoLinkResult<()>;
}

pub trait AlGetOutputReq {
    fn al_get_output_req(&mut self) -> IoLinkResult<()>;
}

pub trait AlAbortReq {
    fn al_abort_req(&mut self) -> IoLinkResult<()>;
}

pub trait AlGetOutputCnf {
    fn al_get_output_cnf(&mut self, result: AlResult<()>) -> IoLinkResult<()>;
}

pub trait AlEventCnf {
    fn al_event_cnf(&mut self) -> IoLinkResult<()>;
}

pub struct ApplicationLayerServices {
}

impl<'a> ApplicationLayerServices {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl<'a> Default for ApplicationLayerServices {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> dl::DlControlInd for ApplicationLayerServices {
    fn dl_control_ind(&mut self, control_code: types::DlControlCode) -> IoLinkResult<()> {
        todo!()
    }
}

impl<'a> ApplicationLayerServicesInd for ApplicationLayerServices {
    fn al_read_ind(&mut self, index: u16, sub_index: u8) -> IoLinkResult<()> {
        todo!()
    }

    fn al_write_ind(&mut self, index: u16, sub_index: u8, data: &[u8]) -> IoLinkResult<()> {
        todo!()
    }

    fn al_abort_ind(&mut self) -> IoLinkResult<()> {
        todo!()
    }

    fn al_pd_cycle_ind(&mut self) {
        todo!()
    }

    fn al_new_output_ind(&mut self) -> IoLinkResult<()> {
        todo!()
    }

    fn al_control_ind(&mut self, control_code: types::DlControlCode) -> IoLinkResult<()> {
        todo!()
    }
}

impl<'a> AlEventCnf for ApplicationLayerServices {
    fn al_event_cnf(&mut self) -> IoLinkResult<()> {
        todo!()
    }
}