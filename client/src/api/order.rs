use super::{types::*, utils, BinanceOkResponse, BinanceRequest, RequestPayload};
use pyo3::pyclass;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderRequest {
    pub api_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_client_order_id: Option<u32>,
    pub new_order_resp_type: NewOrderRespType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_order_qty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recv_window: Option<u32>,
    pub side: Side,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    pub timestamp: u64,
    #[serde(rename = "type")]
    pub order_type: OrderType,
}

impl OrderRequest {
    pub fn new(
        api_key: &str,
        symbol: &str,
        side: Side,
        order_type: OrderType,
        quantity: Option<String>,
    ) -> Self {
        let timestamp = utils::timestamp();

        OrderRequest {
            symbol: symbol.into(),
            side,
            order_type,
            timestamp,
            quantity,
            new_order_resp_type: NewOrderRespType::Ack,
            api_key: api_key.into(),
            quote_order_qty: Default::default(),
            new_client_order_id: Default::default(),
            recv_window: Default::default(),
            time_in_force: Default::default(),
            price: Default::default(),
            signature: Default::default(),
        }
    }
}

impl BinanceRequest for OrderRequest {
    type Response = OrderResponse;

    const METHOD: &'static str = "order.place";
}

impl RequestPayload for OrderRequest {
    fn set_signature(&mut self, signature: String) {
        self.signature = Some(signature);
    }

    fn has_signature(&self) -> bool {
        self.signature.is_some()
    }

    fn payload(&self) -> String {
        serde_qs::to_string(&self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[pyclass]
pub struct OrderResponse {
    #[pyo3(get)]
    pub symbol: String,
    #[pyo3(get)]
    pub transact_time: u64,
    #[pyo3(get)]
    pub price: Option<String>,
    #[pyo3(get)]
    pub orig_qty: Option<String>,
    #[pyo3(get)]
    pub executed_qty: Option<String>,
    #[pyo3(get)]
    pub cummulative_quote_qty: Option<String>,
    #[pyo3(get)]
    pub status: Option<OrderStatus>,
}

impl OrderResponse {
    pub fn new(symbol: &str, transaction_time: u64, status: OrderStatus) -> Self {
        OrderResponse {
            symbol: symbol.into(),
            transact_time: transaction_time,
            status: Some(status),
            price: Default::default(),
            orig_qty: Default::default(),
            executed_qty: Default::default(),
            cummulative_quote_qty: Default::default(),
        }
    }
}

impl<'de> BinanceOkResponse<'de> for OrderResponse {}
