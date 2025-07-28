use crate::{pl, sm, IoLinkResult};

mod command_handler;
mod mode_handler;
mod event_handler;
mod isdu_handler;
mod message_handler;
mod od_handler;
mod pd_handler;

pub use mode_handler::DlInd;

pub struct DataLinkLayer {
    command: command_handler::CommandHandler,
    dl_mode: mode_handler::DlModeHandler,
    event_handler: event_handler::EventHandler,
    message: message_handler::MessageHandler,
    process_data: pd_handler::ProcessDataHandler,
    isdu: isdu_handler::IsduHandler,
    od: od_handler::OnRequestHandler,
}

impl DataLinkLayer {
    pub fn poll(
        &mut self,
        system_management: &mut sm::SystemManagement,
        physical_layer: &mut pl::physical_layer::PhysicalLayer,
    ) -> IoLinkResult<()> {
        let _ = self.command.poll(&mut self.message);
        let _ = self.dl_mode.poll(
            &mut self.isdu,
            &mut self.event_handler,
            &mut self.command,
            &mut self.on_request,
            &mut self.process_data,
            &mut self.message,
            system_management,
        );
        let _ = self.event_handler.poll();
        let _ = self.event_handler.poll();
        let _ = self.message.poll();
        let _ = self.process_data.poll();

        Ok(())
    }
}

impl Default for DataLinkLayer {
    fn default() -> Self {
        Self {
            command: command_handler::CommandHandler::new(),
            dl_mode: mode_handler::DlModeHandler::new(),
            event_handler: event_handler::EventHandler::new(),
            message: message_handler::MessageHandler::new(),
            process_data: pd_handler::ProcessDataHandler::new(),
            isdu: isdu_handler::IsduHandler::new(),
            od: od_handler::OnRequestHandler::new(),
        }
    }
}

impl pl::physical_layer::PhysicalLayerInd for DataLinkLayer {
    fn pl_wake_up_ind(&mut self) -> IoLinkResult<()> {
        self.dl_mode.pl_wake_up_ind()
    }

    fn pl_transfer_ind(&mut self, rx_buffer: &mut [u8]) -> IoLinkResult<()> {
        self.message.pl_transfer_ind(rx_buffer)
    }
}
