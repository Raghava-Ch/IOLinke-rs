// //! Process Data Handler for Application Layer
// //!
// //! This module implements the Process Data Handler state machine as defined in
// //! IO-Link Specification v1.1.4 Section 8.3.4

// use crate::{
//     dl,
//     types::{self, IoLinkError, IoLinkResult},
// };

// pub struct ProcessDataHandler {
// }

// impl ProcessDataHandler {
//     pub fn new() -> Self {
//         Self {
//         }
//     }

// }

// impl Default for ProcessDataHandler {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl dl::DlPDOutputTransportInd for ProcessDataHandler {
//     fn dl_pd_output_transport_ind(&mut self, pd_out: &[u8; types::MAX_PROCESS_DATA_LENGTH]) -> IoLinkResult<()> {
//         todo!()
//     }

//     fn dl_pd_cycle_ind(&mut self) -> IoLinkResult<()> {
//         todo!()
//     }
// }