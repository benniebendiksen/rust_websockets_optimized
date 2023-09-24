use serde::{Deserialize, Serialize};

use crate::api::{BinanceOkResponse, BinanceRequest};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(transparent)]
pub struct SubscribeRequest {
    pub symbols: Vec<String>,
}

impl SubscribeRequest {
    pub fn new(symbols: Vec<String>) -> Self {
        Self { symbols }
    }
}

impl BinanceRequest for SubscribeRequest {
    type Response = SubscribeResponse;

    const METHOD: &'static str = "SUBSCRIBE";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubscribeResponse;

impl<'de> BinanceOkResponse<'de> for SubscribeResponse {}

// TODO: prices and qtys as floats with parsing to/from string

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubscriptionUpdate {
    #[serde(rename = "u")]
    pub update_id: u32,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "b")]
    pub best_bid_price: String,
    #[serde(rename = "B")]
    pub best_bid_qty: String,
    #[serde(rename = "a")]
    pub best_ask_price: String,
    #[serde(rename = "A")]
    pub best_ask_qty: String,
}
