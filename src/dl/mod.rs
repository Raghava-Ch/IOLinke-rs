use crate::IoLinkResult;

mod command;
mod dl_mode;
mod event_handler;
mod isdu;
mod message;
mod process_data;

pub struct DataLinkLayer {
    command: command::CommandHandler,
    dl_mode: dl_mode::DlModeHandler,
    event_handler: event_handler::EventHandler,
    message: message::MessageHandler,
    process_data: process_data::ProcessDataHandler,
}

impl DataLinkLayer {
    pub fn poll(&mut self) -> IoLinkResult<()> {
        self.command.poll();
        self.dl_mode.poll();
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