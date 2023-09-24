use super::{utils, RequestPayload};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[repr(transparent)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecretKey(String);

impl SecretKey {
    pub fn new(key: &str) -> Self {
        SecretKey(key.into())
    }

    pub fn sign<T: RequestPayload>(&self, request: &mut T) {
        let payload = request.payload();
        let signature = utils::sign_payload(&payload, self.as_bytes());
        request.set_signature(signature);
    }
}

impl<T> From<T> for SecretKey
where
    String: From<T>,
{
    fn from(value: T) -> Self {
        Self(String::from(value))
    }
}

impl Deref for SecretKey {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SecretKey {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::api::{
        order::OrderRequest,
        secret_key::SecretKey,
        types::{NewOrderRespType, OrderType, Side, TimeInForce},
    };

    #[test]
    fn signature() {
        let secret_key =
            SecretKey::new("NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j");

        let api_key = "vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A";
        let mut order_request = OrderRequest::new(
            api_key,
            "BTCUSDT",
            Side::Sell,
            OrderType::Limit,
            Some("0.01000000".into()),
        );

        order_request.new_order_resp_type = NewOrderRespType::Ack;
        order_request.price = Some("52000.00".into());
        order_request.recv_window = Some(100);
        order_request.time_in_force = Some(TimeInForce::Gtc);
        order_request.timestamp = 1645423376532;

        assert_eq!(order_request.signature, None);
        secret_key.sign(&mut order_request);
        assert_eq!(
            order_request.signature.unwrap(),
            "cc15477742bd704c29492d96c7ead9414dfd8e0ec4a00f947bb5bb454ddbd08a"
        );
    }
}
