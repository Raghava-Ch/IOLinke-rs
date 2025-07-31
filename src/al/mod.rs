use crate::IoLinkResult;

pub mod application;
mod event_sm;
mod on_request;

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