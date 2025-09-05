use crate::{
    custom::IoLinkResult,
    frame::msequence::{ComChannel, RwDirection},
};
use heapless::Vec;
use iolinke_dev_config::device as dev_config;

pub const OD_LENGTH: usize = dev_config::on_req_data::max_possible_od_length() as usize;

pub trait OdInd {
    /// Invoke OD.ind service with the provided data
    fn od_ind(&mut self, od_ind_data: &OdIndData) -> IoLinkResult<()>;
}

pub trait OdRsp {
    /// Invoke OD.rsp service with the provided data
    fn od_rsp(&mut self, length: u8, data: &[u8]) -> IoLinkResult<()>;
}

pub trait DlWriteParamInd {
    /// See 7.2.1.3 DL_WriteParam
    /// The DL_WriteParam service is used by the AL to write a parameter value to the Device via
    /// the page communication channel. The parameters of the service primitives are listed in Table 18.
    fn dl_write_param_ind(&mut self, index: u8, data: u8) -> IoLinkResult<()>;
}

pub trait DlParamRsp {
    /// See 7.2.1.4 DL_ReadParam.rsp
    fn dl_read_param_rsp(&mut self, length: u8, data: u8) -> IoLinkResult<()>;
    fn dl_write_param_rsp(&mut self) -> IoLinkResult<()>;
}

pub trait DlReadParamInd {
    /// See 7.2.1.2 DL_ReadParam
    /// The DL_ReadParam service is used by the AL to read a parameter value from the Device via
    /// the page communication channel. The parameters of the service primitives are listed in Table 17.
    fn dl_read_param_ind(&mut self, address: u8) -> IoLinkResult<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OdIndData {
    pub rw_direction: RwDirection,
    pub com_channel: ComChannel,
    pub address_ctrl: u8,
    pub req_length: u8,
    pub data: Vec<u8, OD_LENGTH>,
}

impl OdIndData {
    pub fn new() -> Self {
        Self {
            rw_direction: RwDirection::Read,
            com_channel: ComChannel::Page,
            address_ctrl: 0,
            req_length: 0,
            data: Vec::new(),
        }
    }
}

impl Default for OdIndData {
    fn default() -> Self {
        Self::new()
    }
}
