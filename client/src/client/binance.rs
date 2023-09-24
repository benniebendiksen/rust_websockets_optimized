use crate::api::{utils, ws::WsResponseHeader, BinanceRequest, ProcessedRequest};
use actix_codec::Framed;
use actix_web::web::Bytes;
use ahash::AHashMap as HashMap;
use awc::{
    error::WsProtocolError,
    ws::{Codec, Frame},
    BoxedSocket,
};
use futures::SinkExt;
use futures_util::StreamExt;
use std::time::Duration;
use uuid::Uuid;

pub type Connection = Framed<BoxedSocket, Codec>;

pub struct BinanceClient {
    pending: usize,
    prepared: usize,
    connection: Option<Connection>,
    url: String,
    requests: HashMap<Uuid, &'static str>,
    // buffer: HashMap<Uuid, (Option<&'static str>, Option<Result<Frame, WsProtocolError>>)>,
}

impl BinanceClient {
    pub fn new(url: String) -> Self {
        BinanceClient {
            pending: 0,
            prepared: 0,
            connection: None,
            url,
            requests: HashMap::default(),
            // buffer: HashMap::default(),
        }
    }

    // /// Set whether to operate in bounded mode, that is, to expect unsolicited messages
    // pub fn set_bounded(&mut self, bounded: bool)

    fn get_connection(&mut self) -> &mut Connection {
        self.connection.as_mut().expect("No connection")
    }

    pub async fn connect(&mut self, timeout: Duration) {
        let client = awc::Client::builder()
            .timeout(timeout)
            .max_http_version(awc::http::Version::HTTP_11)
            .finish();

        let (_resp, connection) = client.ws(&self.url).connect().await.unwrap();
        self.connection = Some(connection);

        log::info!("Client connected");
    }

    pub async fn disconnect(self) -> Result<(), WsProtocolError> {
        if let Some(mut connection) = self.connection {
            return connection.close().await;
        }

        log::info!("Client disconnected");

        Ok(())
    }

    pub async fn ping(&mut self) -> Result<(), WsProtocolError> {
        let timestamp = utils::timestamp();
        let ping = Bytes::copy_from_slice(timestamp.to_string().as_bytes());
        let connection = self.get_connection();
        connection.send(awc::ws::Message::Ping(ping)).await
    }

    pub async fn pong(&mut self, bytes: &[u8]) -> Result<(), WsProtocolError> {
        let pong = Bytes::copy_from_slice(bytes);
        let connection = self.get_connection();
        connection.send(awc::ws::Message::Pong(pong)).await
    }

    pub async fn send<T>(&mut self, request: ProcessedRequest<T>) -> Result<Uuid, WsProtocolError>
    where
        T: BinanceRequest,
    {
        let connection = self.get_connection();
        connection
            .send(awc::ws::Message::Text(request.text.into()))
            .await?;

        log::info!(
            "Send {} request with id {} ({} prepared)",
            T::METHOD,
            request.id,
            self.prepared
        );

        self.requests.insert(request.id, T::METHOD);
        self.pending += 1 + self.prepared;
        self.prepared = 0;

        Ok(request.id)
    }

    pub async fn feed<T>(&mut self, request: ProcessedRequest<T>) -> Result<Uuid, WsProtocolError>
    where
        T: BinanceRequest,
    {
        let connection = self.get_connection();
        connection
            .feed(awc::ws::Message::Text(request.text.into()))
            .await?;

        log::info!("Prepare {} request with id {}", T::METHOD, request.id);

        self.requests.insert(request.id, T::METHOD);
        self.prepared += 1;

        Ok(request.id)
    }

    pub async fn flush(&mut self) -> Result<(), WsProtocolError> {
        let connection = self.get_connection();
        let result = connection.flush().await;

        log::info!("Flush {} requests", self.prepared);
        self.pending += self.prepared;
        self.prepared = 0;

        result
    }

    pub fn pending_amount(&self) -> usize {
        self.pending
    }

    pub fn pending(&self) -> bool {
        self.pending != 0
    }

    async fn next_inner(
        &mut self,
    ) -> Result<Option<(Option<Uuid>, Option<&'static str>, Bytes)>, WsProtocolError>
// (
    //     Option<Uuid>,
    //     (Option<&'static str>, Option<Result<Bytes, WsProtocolError>>),
    // )
    {
        loop {
            let connection = self.get_connection();
            let response = connection.next().await;
            match response {
                Some(Ok(Frame::Ping(bytes))) => {
                    let result = self.pong(&bytes).await;
                    if let Err(err) = result {
                        return Err(err);
                    }
                    log::info!("Pong({:?})", bytes);
                }
                Some(Ok(Frame::Text(response_bytes))) => {
                    let response = String::from_utf8(response_bytes.to_vec()).unwrap();
                    let id_method: Option<(Uuid, Option<&str>)> =
                        serde_json::from_str::<WsResponseHeader>(&response)
                            .ok()
                            .map(|header| (header.id, self.requests.remove(&header.id)));
                    let id = id_method.map(|p| p.0);
                    let method = id_method.and_then(|p| p.1);

                    match (id, method) {
                        (Some(id), Some(method)) => {
                            log::info!("Received {method} response with id {id}")
                        }
                        (Some(id), None) => log::info!("Received response with id {id}"),
                        (None, None) => log::info!("Recieved response"),
                        (None, Some(_)) => unreachable!(),
                    };

                    // only decrement for messages with an id
                    if let Some(_) = id {
                        self.pending -= 1;
                    }
                    return Ok(Some((id, method, response_bytes)));
                }
                None => {
                    log::warn!("Server disconnected");
                    self.pending = 0;
                    return Ok(None);
                }
                // TODO: this is bad error handling
                _ => unimplemented!("{response:?}"),
            }
        }
    }

    pub async fn next(
        &mut self,
    ) -> Result<Option<(Option<Uuid>, Option<&'static str>, Bytes)>, WsProtocolError> {
        self.next_inner().await
    }

    // pub async fn response_to(
    //     &mut self,
    //     id: Uuid,
    // ) -> (Option<&'static str>, Option<Result<Frame, WsProtocolError>>) {
    //     if let Some(resp) = self.buffer.remove(&id) {
    //         resp
    //     } else {
    //         loop {
    //             let (id2, resp) = self.next_inner().await;
    //             if let Some(id2) = id2 {
    //                 if id == id2 {
    //                     return resp;
    //                 } else {
    //                     self.buffer.insert(id2, resp);
    //                 }
    //             }
    //         }
    //     }
    // }
}
