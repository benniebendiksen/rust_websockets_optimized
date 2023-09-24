use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::ws::WsRequest;
//submodules
pub mod order;
pub mod secret_key;
pub mod subscription;
pub mod types;
pub mod utils;
pub mod ws;

pub struct ProcessedRequest<T: ?Sized> {
    pub(crate) id: Uuid,
    pub(crate) text: String,
    pub(crate) marker: PhantomData<T>,
}

pub trait BinanceRequest: Serialize {
    type Response: for<'de> BinanceOkResponse<'de>;

    const METHOD: &'static str;

    fn preprocess(self) -> Result<ProcessedRequest<Self>, serde_json::Error>
    where
        Self: Sized,
    {
        let ws_request = WsRequest::new(self);
        let text = serde_json::to_string(&ws_request)?;
        Ok(ProcessedRequest {
            id: ws_request.id,
            text,
            marker: PhantomData,
        })
    }
}

//BinanceRequest is implemented by any reference to a type that
//already implements the BinanceRequest trait
impl<T> BinanceRequest for &T
where
    //T must implement the 'BinanceRequest' trait in order for the
    //impl block to be valid.
    T: BinanceRequest,
{
    type Response = T::Response;

    const METHOD: &'static str = T::METHOD;
}
// RequestPayload is a subtrait of BinanceRequest so whatever
// implements RequestPayload must implement BinanceRequest
pub trait RequestPayload: BinanceRequest {
    fn set_signature(&mut self, signature: String);

    fn has_signature(&self) -> bool;

    fn payload(&self) -> String;
}

pub trait BinanceOkResponse<'de>: Deserialize<'de> {}
