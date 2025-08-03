use crate::{pl, sm, IoLinkResult};

mod command_handler;
mod mode_handler;
mod event_handler;
mod isdu_handler;
mod message_handler;
mod od_handler;
mod pd_handler;

pub use od_handler::DlWriteParamInd;
pub use od_handler::DlReadParamInd;
pub use mode_handler::DlInd;
pub use isdu_handler::{DlIsduAbort, DlIsduTransportInd, Isdu};
pub use event_handler::DlEventTriggerConf;

pub struct DataLinkLayer {
    command_handler: command_handler::CommandHandler,
    mode_handler: mode_handler::DlModeHandler,
    event_handler: event_handler::EventHandler,
    message_handler: message_handler::MessageHandler,
    pd_handler: pd_handler::ProcessDataHandler,
    isdu_handler: isdu_handler::IsduHandler,
    od_handler: od_handler::OnRequestDataHandler,
}

impl DataLinkLayer {
    pub fn poll(
        &mut self,
        system_management: &mut sm::SystemManagement,
        physical_layer: &mut pl::physical_layer::PhysicalLayer,
    ) -> IoLinkResult<()> {
        let _ = self.command_handler.poll(physical_layer, &mut &mut self.message_handler);
        let _ = self.mode_handler.poll(
            &mut self.isdu_handler,
            &mut self.event_handler,
            &mut self.command_handler,
            &mut self.od_handler,
            &mut self.pd_handler,
            &mut self.message_handler,
            system_management,
        );
        let _ = self.event_handler.poll(&mut self.message_handler);
        let _ = self.message_handler.poll(
            &mut self.event_handler,
            &mut self.isdu_handler,
            &mut self.od_handler,
            &mut self.pd_handler,
            &mut self.mode_handler,
            physical_layer,
        );
        let _ = self.pd_handler.poll();

        Ok(())
    }

    pub fn write_param_rsp(&mut self) -> IoLinkResult<()> {
        todo!("Implement write_param_rsp")
    }

    pub fn read_param_rsp(&mut self) -> IoLinkResult<()> {
        todo!("Implement read_param_rsp")
    }

    pub fn read_isdu_transport(&mut self, index: u8, sub_index: u8, data: &[u8]) -> IoLinkResult<()> {

        todo!("Implement read_isdu_transport");
    }

    pub fn write_isdu_transport(&mut self, error: u8, additional_error: u8) -> IoLinkResult<()> {

        todo!("Implement read_isdu_transport");
    }

    pub fn negative_isdu_transport(&mut self, error: u8, additional_error: u8) -> IoLinkResult<()> {
        todo!("Implement negative_isdu_transport");
    }

    pub fn event_req(&mut self) -> IoLinkResult<()> {
        // self.event_handler.dl_event_req();

        todo!("Implement event_req");
    }
}

impl Default for DataLinkLayer {
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

impl pl::physical_layer::PhysicalLayerInd for DataLinkLayer {
    fn pl_wake_up_ind(&mut self) -> IoLinkResult<()> {
        self.mode_handler.pl_wake_up_ind()
    }

    fn pl_transfer_ind(&mut self, rx_buffer: &mut [u8]) -> IoLinkResult<()> {
        self.message_handler.pl_transfer_ind(rx_buffer)
    }
}
