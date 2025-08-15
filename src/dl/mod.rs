use crate::{IoLinkResult, pl, system_management};
use crate::{al, storage};

mod command_handler;
mod event_handler;
mod isdu_handler;
mod message_handler;
mod mode_handler;
mod od_handler;
mod pd_handler;

pub use command_handler::DlControlInd;
pub use event_handler::{DlEventReq, DlEventTriggerConf};
pub use isdu_handler::{DlIsduAbort, DlIsduTransportInd, DlIsduTransportRsp, Isdu, MAX_ISDU_LENGTH, IsduService};
pub use mode_handler::DlInd;
pub use od_handler::{DlParamRsp, DlReadParamInd, DlWriteParamInd};
pub use pd_handler::{DlPDInputUpdate, DlPDOutputTransportInd};

pub struct DataLinkLayer<'a> {
    command_handler: command_handler::CommandHandler,
    mode_handler: mode_handler::DlModeHandler,
    event_handler: event_handler::EventHandler<'a>,
    message_handler: message_handler::MessageHandler<'a>,
    pd_handler: pd_handler::ProcessDataHandler,
    isdu_handler: isdu_handler::IsduHandler<'a>,
    od_handler: od_handler::OnRequestDataHandler<'a>,
}

impl<'b> DataLinkLayer<'b> {
    pub fn poll(
        &'b mut self,
        system_management: &mut system_management::SystemManagement,
        physical_layer: &mut pl::physical_layer::PhysicalLayer,
        application_layer: &mut al::ApplicationLayer,
    ) -> IoLinkResult<()> {
        // Command handler poll
        {
            let _ = self
                .command_handler
                .poll(&mut self.message_handler, application_layer);
        }
        
        // Mode handler poll
        {
            let _ = self.mode_handler.poll(
                &mut self.isdu_handler,
                &mut self.event_handler,
                &mut self.command_handler,
                &mut self.od_handler,
                &mut self.pd_handler,
                &mut self.message_handler,
                system_management,
            );
        }
        
        // Event handler poll
        {
            let _ = self.event_handler.poll(&mut self.message_handler);
        }
        
        // PD handler poll
        {
            let _ = self.pd_handler.poll(&mut self.message_handler);
        }
        
        // ISDU handler poll - separate scope to avoid conflicts
        {
            let isdu_handler = &mut self.isdu_handler;
            let _ = isdu_handler.poll(&mut self.message_handler, application_layer);
        }
        
        // Message handler poll - separate scope to avoid conflicts
        {
            let _ = self.message_handler.poll(
                &mut self.event_handler,
                &mut self.isdu_handler,
                &mut self.od_handler,
                &mut self.pd_handler,
                &mut self.mode_handler,
                physical_layer,
            );
        }
        
        // OD handler poll - separate scope to avoid conflicts (moved to the end)
        {
            let _ = self.od_handler.poll(
                &mut self.command_handler,
                &mut self.isdu_handler,
                &mut self.event_handler,
                application_layer,
                system_management,
            );
        }
        
        Ok(())
    }
}

impl<'a> od_handler::DlParamRsp for DataLinkLayer<'a> {
    fn dl_read_param_rsp(&mut self, length: u8, data: &[u8]) -> IoLinkResult<()> {
        self.od_handler
            .dl_read_param_rsp(length, data, &mut self.message_handler)
    }

    fn dl_write_param_rsp(&mut self) -> IoLinkResult<()> {
        // No response is expected in specs
        Ok(())
    }
}

impl<'a> isdu_handler::DlIsduTransportRsp for DataLinkLayer<'a> {
    fn dl_isdu_transport_read_rsp(&mut self, length: u8, data: &[u8]) -> IoLinkResult<()> {
        self.isdu_handler
            .dl_isdu_transport_read_rsp(length, data, &mut self.message_handler)
    }

    fn dl_isdu_transport_write_rsp(&mut self) -> IoLinkResult<()> {
        self.isdu_handler
            .dl_isdu_transport_write_rsp(&mut self.message_handler)
    }

    fn dl_isdu_transport_read_error_rsp(
        &mut self,
        error: u8,
        additional_error: u8,
    ) -> IoLinkResult<()> {
        self.isdu_handler.dl_isdu_transport_read_error_rsp(
            error,
            additional_error,
            &mut self.message_handler,
        )
    }

    fn dl_isdu_transport_write_error_rsp(
        &mut self,
        error: u8,
        additional_error: u8,
    ) -> IoLinkResult<()> {
        self.isdu_handler.dl_isdu_transport_write_error_rsp(
            error,
            additional_error,
            &mut self.message_handler,
        )
    }
}

impl<'a> event_handler::DlEventReq for DataLinkLayer<'a> {
    fn dl_event_req(
        &mut self,
        event_count: u8,
        event_entries: &[storage::event_memory::EventEntry; 6],
    ) -> IoLinkResult<()> {
        self.event_handler.dl_event_req(event_count, event_entries)
    }

    fn dl_event_trigger_req(&mut self) -> IoLinkResult<()> {
        self.event_handler.dl_event_trigger_req()
    }
}

impl<'a> pd_handler::DlPDInputUpdate for DataLinkLayer<'a> {
    fn dl_pd_input_update_req(&mut self, length: u8, input_data: &[u8]) -> IoLinkResult<()> {
        self.pd_handler.dl_pd_input_update_req(length, input_data)
    }
}

impl<'a> Default for DataLinkLayer<'a> {
    fn default() -> Self {
        Self {
            command_handler: command_handler::CommandHandler::new(),
            mode_handler: mode_handler::DlModeHandler::new(),
            event_handler: event_handler::EventHandler::new(),
            message_handler: message_handler::MessageHandler::new(),
            pd_handler: pd_handler::ProcessDataHandler::new(),
            isdu_handler: isdu_handler::IsduHandler::new(),
            od_handler: od_handler::OnRequestDataHandler::new(),
        }
    }
}

impl<'a> pl::physical_layer::PhysicalLayerInd for DataLinkLayer<'a> {
    fn pl_wake_up_ind(&mut self) -> IoLinkResult<()> {
        self.mode_handler.pl_wake_up_ind()
    }

    fn pl_transfer_ind(&mut self, rx_buffer: &mut [u8]) -> IoLinkResult<()> {
        self.message_handler.pl_transfer_ind(rx_buffer)
    }
}
