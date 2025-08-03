use crate::{dl, IoLinkResult};

mod application;
mod event_sm;
mod on_request;

pub trait ApplicationLayerInd {
    fn al_read_ind(&mut self, index: u16, sub_index: u8) -> IoLinkResult<Vec<u8, 32>>;

    fn al_write_ind(&mut self, index: u16, sub_index: u8, data: &[u8]) -> IoLinkResult<()>;
    
    fn al_abort_ind(&mut self) -> IoLinkResult<()>;

    fn al_set_input_ind(&mut self,) -> IoLinkResult<()>;

    fn al_pd_cycle_ind(&mut self);

    fn al_get_output_ind(&mut self) -> IoLinkResult<()>;

    fn al_new_output_ind(&mut self, ) -> IoLinkResult<()>;

    fn al_event(&mut self, ) -> IoLinkResult<()>;

    fn al_control(&mut self, control_code: u8) -> IoLinkResult<()>;

}

pub struct ApplicationLayer<'a> {
    event_sm: event_sm::EventStateMachine,
    on_request: on_request::OnRequestHandler<'a>,
    application: application::ApplicationLayerImpl,
}

impl<'a> ApplicationLayer<'a> {
    pub fn poll(&mut self, datalink_layer: &mut dl::DataLinkLayer) -> IoLinkResult<()> {
        self.event_sm.poll(&mut self.application, datalink_layer)?;
        self.on_request.poll(&mut self.application, datalink_layer)?;
        self.application.poll()?;

        Ok(())
    }
}

impl<'a> Default for ApplicationLayer<'a> {
    fn default() -> Self {
        Self {
            event_sm: event_sm::EventStateMachine::new(),
            on_request: on_request::OnRequestHandler::new(),
            application: application::ApplicationLayerImpl::new(),
        }
    }
}