use crate::*;
use moonlight::serde::{de::DeserializeOwned, Serialize};
use moonlight::{serde_json, AuthToken, CorId, SessionId};
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use web_sys::{Request, RequestInit, Response};

mod sse;
use sse::SSE;

// ------ Connection ------

pub struct Connection<UMsg, DMsg> {
    session_id: SessionId,
    _sse: SSE,
    auth_token_getter: Option<Box<dyn Fn() -> Option<AuthToken> + Send + Sync>>,
    msg_types: PhantomData<(UMsg, DMsg)>,
}

impl<UMsg: Serialize, DMsg: DeserializeOwned> Connection<UMsg, DMsg> {
    pub fn new(down_msg_handler: impl FnOnce(DMsg, CorId) + Clone + Send + Sync + 'static) -> Self {
        let session_id = SessionId::new();
        Self {
            session_id,
            _sse: SSE::new(session_id, down_msg_handler),
            auth_token_getter: None,
            msg_types: PhantomData,
        }
    }

    pub fn auth_token_getter<IAT>(
        mut self,
        getter: impl FnOnce() -> IAT + Clone + Send + Sync + 'static,
    ) -> Self
    where
        IAT: Into<Option<AuthToken>>,
    {
        let getter = move || (getter.clone())().into();
        self.auth_token_getter = Some(Box::new(getter));
        self
    }

    pub async fn send_up_msg(&self, up_msg: UMsg) -> Result<CorId, SendUpMsgError> {
        // ---- RequestInit ----
        #[cfg(feature = "serde-lite")]
        let body = serde_json::to_string(&up_msg.serialize().unwrap_throw()).unwrap_throw();
        #[cfg(feature = "serde")]
        let body = serde_json::to_string(&up_msg).unwrap_throw();

        let mut request_init = RequestInit::new();
        request_init.method("POST").body(Some(&JsValue::from(body)));

        // ---- Request ----
        let request =
            Request::new_with_str_and_init("/_api/up_msg_handler", &request_init).unwrap_throw();

        // ---- Headers ----
        let cor_id = CorId::new();
        let headers = request.headers();
        headers
            .set("X-Correlation-ID", &cor_id.to_string())
            .unwrap_throw();
        headers
            .set("X-Session-ID", &self.session_id.to_string())
            .unwrap_throw();

        let auth_token = self
            .auth_token_getter
            .as_ref()
            .and_then(|auth_token_getter| auth_token_getter());
        if let Some(auth_token) = auth_token {
            headers
                .set("X-Auth-Token", auth_token.as_str())
                .unwrap_throw();
        }

        // ---- Response ----
        let response = JsFuture::from(window().fetch_with_request(&request))
            .await
            .map_err(|error| SendUpMsgError::RequestFailed(error))?
            .unchecked_into::<Response>();

        if response.ok() {
            return Ok(cor_id);
        }
        Err(SendUpMsgError::ResponseIsNot2xx)
    }
}

// ------ SendUpMsgError ------

#[derive(Debug)]
pub enum SendUpMsgError {
    RequestFailed(JsValue),
    ResponseIsNot2xx,
}

impl fmt::Display for SendUpMsgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SendUpMsgError::RequestFailed(error) => {
                write!(f, "request failed: {:?}", error)
            }
            SendUpMsgError::ResponseIsNot2xx => {
                write!(f, "response status is not 2xx")
            }
        }
    }
}

impl Error for SendUpMsgError {}
