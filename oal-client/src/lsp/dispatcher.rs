use super::state::GlobalState;
use lsp_server::{ExtractError, Notification, Request, RequestId};

pub struct RequestDispatcher<'a> {
    state: &'a mut GlobalState,
    req: Option<Request>,
}

impl<'a> RequestDispatcher<'a> {
    pub fn new(state: &'a mut GlobalState, req: Request) -> Self {
        RequestDispatcher {
            state,
            req: Some(req),
        }
    }

    pub fn on<N>(
        &mut self,
        hook: fn(&mut GlobalState, RequestId, N::Params) -> anyhow::Result<()>,
    ) -> anyhow::Result<&mut Self>
    where
        N: lsp_types::request::Request,
        N::Params: serde::de::DeserializeOwned,
    {
        let req = match self.req.take() {
            Some(r) => r,
            None => return Ok(self),
        };
        let (id, params) = match req.extract::<N::Params>(N::METHOD) {
            Ok(p) => p,
            Err(err @ ExtractError::JsonError { .. }) => return Err(anyhow::Error::from(err)),
            Err(ExtractError::MethodMismatch(req)) => {
                self.req = Some(req);
                return Ok(self);
            }
        };
        hook(self.state, id, params)?;
        Ok(self)
    }
}

pub struct NotificationDispatcher<'a> {
    state: &'a mut GlobalState,
    not: Option<Notification>,
}

impl<'a> NotificationDispatcher<'a> {
    pub fn new(state: &'a mut GlobalState, not: Notification) -> Self {
        NotificationDispatcher {
            state,
            not: Some(not),
        }
    }

    pub fn on<N>(
        &mut self,
        hook: fn(&mut GlobalState, N::Params) -> anyhow::Result<()>,
    ) -> anyhow::Result<&mut Self>
    where
        N: lsp_types::notification::Notification,
        N::Params: serde::de::DeserializeOwned,
    {
        let not = match self.not.take() {
            Some(n) => n,
            None => return Ok(self),
        };
        let params = match not.extract::<N::Params>(N::METHOD) {
            Ok(p) => p,
            Err(err @ ExtractError::JsonError { .. }) => return Err(anyhow::Error::from(err)),
            Err(ExtractError::MethodMismatch(not)) => {
                self.not = Some(not);
                return Ok(self);
            }
        };
        hook(self.state, params)?;
        Ok(self)
    }
}
