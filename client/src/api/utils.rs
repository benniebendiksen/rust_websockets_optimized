use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn sign_payload(query: &str, secret_key: &[u8]) -> String {
    let mut hmac = Hmac::<Sha256>::new_from_slice(secret_key).unwrap();
    hmac.update(query.as_bytes());
    let digest = hmac.finalize().into_bytes();

    hex::encode(digest)
}

pub fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
