use std::{collections::HashMap, time::Duration};

use actix::{Actor, ActorContext, AsyncContext, SpawnHandle, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws::{self, WsResponseBuilder};
use arbitrage_processing::api::{
    subscription::{SubscribeRequest, SubscribeResponse, SubscriptionUpdate},
    ws::WsRequest,
};
use rand::{distributions::Uniform, prelude::Distribution, rngs::StdRng, SeedableRng};
use serde_json::value::RawValue;

struct Subscriptions {
    rng: StdRng,
    subscriptions: HashMap<String, SpawnHandle>,
}
impl Default for Subscriptions {
    fn default() -> Self {
        Self {
            rng: StdRng::from_entropy(),
            subscriptions: Default::default(),
        }
    }
}

impl Drop for Subscriptions {
    fn drop(&mut self) {
        println!("killing subscriptions")
    }
}

impl Actor for Subscriptions {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("starting");
        ctx.run_interval(Duration::from_secs(5), |_act, ctx| {
            ctx.ping(b"PING");
        });
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> actix::Running {
        println!("stopping subscriptions");
        // TODO: this is bad dont do this
        actix::Running::Continue
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Subscriptions {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // thread::sleep(Duration::new(0, 1_000_000));
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                let request = serde_json::from_str::<WsRequest<&RawValue>>(&text).unwrap();
                match request.method.as_str() {
                    "SUBSCRIBE" => {
                        let params =
                            serde_json::from_str::<SubscribeRequest>(request.params.get()).unwrap();
                        let response = SubscribeResponse;
                        println!("subscribing to {params:?}");
                        self.subscriptions
                            .extend(params.symbols.into_iter().map(|symbol| {
                                if let Some((symbol, _ty)) = symbol.split_once('@') {
                                    (
                                        symbol.to_string(),
                                        ctx.run_interval(
                                            Duration::from_millis(
                                                Uniform::new_inclusive(400, 600)
                                                    .sample(&mut self.rng),
                                            ),
                                            coin_ticker(symbol.to_string()),
                                        ),
                                    )
                                } else {
                                    todo!("error handling");
                                }
                            }));
                        ctx.text(serde_json::to_string(&response).unwrap());
                    }
                    method => panic!("invalid method: {method}"),
                }
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => (),
        }
    }
}

pub async fn stream(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (addr, response) =
        WsResponseBuilder::new(Subscriptions::default(), &req, stream).start_with_addr()?;
    // leak the actor handle, allowing it to persist
    std::mem::forget(addr);
    Ok(response)
}

fn coin_ticker(
    symbol: String,
) -> impl FnMut(&mut Subscriptions, &mut ws::WebsocketContext<Subscriptions>) + 'static {
    move |act, ctx| {
        let qty_dist = Uniform::new(10., 200.);
        let bid_dist = Uniform::new(25., 50.);
        let ask_diff_dist = Uniform::new(0., 0.1);

        let bid = bid_dist.sample(&mut act.rng);
        let ask = bid + ask_diff_dist.sample(&mut act.rng);
        let resp = SubscriptionUpdate {
            update_id: 0,
            symbol: symbol.clone(),
            best_bid_price: format!("{:.8}", bid),
            best_bid_qty: format!("{:.8}", qty_dist.sample(&mut act.rng)),
            best_ask_price: format!("{:.8}", ask),
            best_ask_qty: format!("{:.8}", qty_dist.sample(&mut act.rng)),
        };
        // println!("sending {resp:#?}");
        ctx.text(serde_json::to_string_pretty(&resp).unwrap());
    }
}
