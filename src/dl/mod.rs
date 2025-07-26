use crate::{pl::physical_layer::{PhysicalLayer, PhysicalLayerInd}, sm, IoLinkResult};

mod command;
mod dl_mode;
mod event_handler;
mod isdu;
mod message;
mod process_data;
mod on_request;

pub use dl_mode::DlInd;

pub struct DataLinkLayer {
    command: command::CommandHandler,
    dl_mode: dl_mode::DlModeHandler,
    event_handler: event_handler::EventHandler,
    message: message::MessageHandler,
    process_data: process_data::ProcessDataHandler,
}

impl DataLinkLayer {
    pub fn poll(&mut self, system_management: &mut sm::SystemManagement) -> IoLinkResult<()> {
        self.command.poll();
        self.dl_mode.poll(
            &mut self.message,
            system_management);
        self.event_handler.poll();
        self.event_handler.poll();
        self.message.poll(&mut self.dl_mode);
        self.process_data.poll();

        Ok(())
    }
}

impl Default for DataLinkLayer {
    fn default() -> Self {
        Self {
            command: command::CommandHandler::new(),
            dl_mode: dl_mode::DlModeHandler::new(),
            event_handler: event_handler::EventHandler::new(),
            message: message::MessageHandler::new(),
            process_data: process_data::ProcessDataHandler::new(),
        }
    }
}

impl PhysicalLayerInd for DataLinkLayer {
    fn pl_wake_up_ind(&mut self) -> IoLinkResult<()> {
        self.dl_mode.pl_wake_up_ind()
    }

    fn pl_transfer_ind(&mut self, rx_buffer: &mut [u8]) -> IoLinkResult<()> {
        self.message.pl_transfer_ind(rx_buffer)
    }
}