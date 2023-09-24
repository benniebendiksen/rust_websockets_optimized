use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use arbitrage_processing::api::{
    order::{OrderRequest, OrderResponse},
    types::OrderStatus,
    ws::{WsRequest, WsResponse},
};
use serde_json::value::RawValue;
use std::{thread, time::Duration};

mod subscription;

#[derive(Default)]
struct MyWs {}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        thread::sleep(Duration::new(0, 1_000_000));
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                let request = serde_json::from_str::<WsRequest<&RawValue>>(&text).unwrap();
                match request.method.as_str() {
                    "order.place" => {
                        let params =
                            serde_json::from_str::<OrderRequest>(request.params.get()).unwrap();
                        let response = WsResponse {
                            id: request.id,
                            status: 200,
                            error: None,
                            result: OrderResponse {
                                symbol: params.symbol,
                                transact_time: params.timestamp,
                                price: params.price,
                                orig_qty: params.quantity.clone(),
                                executed_qty: params.quantity.clone(),
                                cummulative_quote_qty: None,
                                status: Some(OrderStatus::Filled),
                            },
                        };
                        // println!("sending {response:#?}");
                        ctx.text(serde_json::to_string(&response).unwrap())
                    }
                    method => panic!("invalid method: {method}"),
                }
            }
            _ => (),
        }
    }
}

async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(MyWs::default(), &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let ip = "127.0.0.1";
    let port = 50_000;

    println!("Server is listening to {}:{}", ip, port);

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/stream", web::get().to(subscription::stream))
    })
    .bind((ip, port))?
    .run()
    .await
}
