use crate::IoLinkResult;

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

pub struct ApplicationLayer {
    event_sm: event_sm::EventStateMachine,
    on_request: on_request::OnRequestHandler,
    application: application::ApplicationLayerImpl,
}

impl ApplicationLayer {
    pub fn poll(&mut self) -> IoLinkResult<()> {
        self.event_sm.poll();
        self.on_request.poll();
        self.application.poll();

        Ok(())
    }
}

impl Default for ApplicationLayer {
    fn default() -> Self {
        Self {
            event_sm: event_sm::EventStateMachine::new(),
            on_request: on_request::OnRequestHandler::new(),
            application: application::ApplicationLayerImpl::new(),
        }
    }
}