//! Application Layer API implementation
//!
//! This module implements the Application Layer interface as defined in
//! IO-Link Specification v1.1.4 Section 8.4

use iolinke_types::custom::{IoLinkError, IoLinkResult};
use iolinke_types::handlers;

/// Application Layer trait defining all request/indication methods
/// See IO-Link v1.1.4 Section 8.4
pub trait ApplicationLayerServicesInd {
    fn al_read_ind(&mut self, index: u16, sub_index: u8) -> IoLinkResult<()> {
        Err(IoLinkError::NoImplFound)
    }

    fn al_write_ind(&mut self, index: u16, sub_index: u8, data: &[u8]) -> IoLinkResult<()> {
        Err(IoLinkError::NoImplFound)
    }

    fn al_abort_ind(&mut self) -> IoLinkResult<()> {
        Err(IoLinkError::NoImplFound)
    }

    fn al_pd_cycle_ind(&mut self);

    fn al_new_output_ind(&mut self) -> IoLinkResult<()> {
        Err(IoLinkError::NoImplFound)
    }

    fn al_control_ind(
        &mut self,
        control_code: handlers::command::DlControlCode,
    ) -> IoLinkResult<()> {
        Err(IoLinkError::NoImplFound)
    }
}

pub enum AlRspError {
    Error(u8, u8), // (error code, additional error code)
    StateConflict,
    NoData,
}
pub type AlResult<T> = Result<T, AlRspError>;

pub trait AlReadRsp {
    fn al_read_rsp(&mut self, result: AlResult<(u8, &[u8])>) -> IoLinkResult<()>;
}
pub trait AlWriteRsp {
    fn al_write_rsp(&mut self, result: AlResult<()>) -> IoLinkResult<()>;
}

pub trait AlEventReq {
    fn al_event_req(
        &mut self,
        event_count: u8,
        event_entries: &[handlers::event::EventEntry],
    ) -> IoLinkResult<()>;
}
pub trait AlControlReq {
    /// Handle control codes as defined in IO-Link Specification v1.1.4 Section
    fn al_control_req(
        &mut self,
        control_code: handlers::command::DlControlCode,
    ) -> IoLinkResult<()>;
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

pub struct ApplicationLayerServices {}

impl ApplicationLayerServices {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ApplicationLayerServices {
    fn default() -> Self {
        Self::new()
    }
}

impl handlers::command::DlControlInd for ApplicationLayerServices {
    fn dl_control_ind(
        &mut self,
        _control_code: handlers::command::DlControlCode,
    ) -> IoLinkResult<()> {
        Ok(())
    }
}

impl ApplicationLayerServicesInd for ApplicationLayerServices {
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
        todo!()
    }

    fn al_new_output_ind(&mut self) -> IoLinkResult<()> {
        todo!()
    }

    fn al_control_ind(
        &mut self,
        _control_code: handlers::command::DlControlCode,
    ) -> IoLinkResult<()> {
        todo!()
    }
}

impl AlEventCnf for ApplicationLayerServices {
    fn al_event_cnf(&mut self) -> IoLinkResult<()> {
        todo!()
    }
}
