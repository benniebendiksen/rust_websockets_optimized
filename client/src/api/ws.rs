use super::BinanceRequest;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsRequest<T> {
    pub id: Uuid,
    pub method: String,
    pub params: T,
}

// The new method takes a parameter request of type T, which should implement the BinanceRequest trait.
impl<T: BinanceRequest> WsRequest<T> {
    pub fn new(request: T) -> Self {
        WsRequest {
            id: Uuid::new_v4(),
            method: T::METHOD.into(),
            params: request,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsError {
    pub code: i32,
    pub msg: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsResponseHeader {
    pub id: Uuid,
    pub status: u64,
    pub error: Option<WsError>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
// A Binance failed response contains no result field
//We make result generic, or can wrap it in an Option, so that we can use it for both successful and failed responses
pub struct WsResponse<T> {
    pub id: Uuid,
    pub status: u64,
    pub error: Option<WsError>,
    pub result: T,
}
