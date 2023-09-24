#![allow(unused_imports)]
use crate::{
    api::{
        order::{OrderRequest, OrderResponse},
        secret_key::{self, SecretKey},
        subscription::{SubscribeRequest, SubscribeResponse, SubscriptionUpdate},
        types::{OrderStatus, OrderType, Side},
        BinanceRequest, ws::WsResponse,
    },
    client::BinanceClient,
    triangles::Triangle,
};
use ahash::{HashMap, HashSet};
use awc::ws::Frame;
use env_logger::{Builder as EnvLoggerBuilder, TimestampPrecision};
use futures::channel::mpsc::Receiver;
use log::LevelFilter;
use once_cell::sync::Lazy;
use pyo3::{prelude::*, pyclass::IterNextOutput};
use rand::random;
use std::{
    cell::RefCell,
    hash::Hash,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
    thread::JoinHandle,
    // thread,
    time::{Duration, Instant},
};
use tokio::{
    join,
    runtime::{Builder, Runtime},
    select,
    sync::{
        mpsc::{channel, unbounded_channel, Sender, UnboundedReceiver},
        Mutex as TokioMutex, Notify,
    },
    task::{self, LocalSet},
};

#[derive(Debug)]
enum Subscription {
    Subscribe(Triangle),
}

#[pyclass]
pub struct Client {
    threads: Vec<JoinHandle<()>>,
    subscription_senders: Vec<Sender<Subscription>>,
    next_sender: usize,
    results_reciever: Arc<TokioMutex<UnboundedReceiver<OrderResponse>>>,
}

#[pymethods]
impl Client {
    #[new]
    pub fn new(
        url: String,
        streams_url: String,
        api_key: String,
        secret_key: &str,
        thread_num: usize,
        timeout: Option<u64>,
    ) -> Self {
        let secret_key = SecretKey::new(secret_key);
        let (results_sender, results_reciever) = unbounded_channel::<OrderResponse>();

        let (subscription_senders, threads) = (0..thread_num)
            .map(|i| {
                // TODO: break this function into small pieces
                let (subscriptions_sender, mut subscriptions_reciever) = channel(100);
                let results_sender = results_sender.clone();
                let api_key = api_key.clone();
                let secret_key = secret_key.clone();
                let url = url.clone();
                let streams_url = streams_url.clone();
                let handle = thread::Builder::new()
                    .name(format!("worker {i}"))
                    .spawn(move || {
                        let runtime = Builder::new_current_thread().enable_all().build().unwrap();
                        let mut client = BinanceClient::new(url);
                        let mut ticker = BinanceClient::new(streams_url);
                        let updates: Rc<RefCell<HashMap<String, SubscriptionUpdate>>> = Default::default();
                        let update_notify = Rc::new(Notify::new());
                        let triangles: Rc<RefCell<HashSet<Triangle>>> = Default::default();
                        let local = LocalSet::new();
                        local.block_on(&runtime, async move {
                            join!(
                                client.connect(Duration::new(timeout.unwrap_or(5), 0)),
                                ticker.connect(Duration::new(timeout.unwrap_or(5), 0)),
                            );
                            let _ticker = {
                                let updates = updates.clone();
                                let triangles = triangles.clone();
                                let update_notify = update_notify.clone();
                                task::spawn_local(async move {
                                // println!("[{i}]: starting update_subs");
                                loop {
                                    // println!("[{i}]: polling");
                                    select! {
                                        Some(sub) = subscriptions_reciever.recv() => {
                                            println!("[{i}]: recieved {sub:?}");
                                            match sub {
                                                Subscription::Subscribe(tri) => {
                                                    triangles.borrow_mut().insert(tri.clone());
                                                    let Triangle {base, quote, alt, ..} = tri;
                                                    let mut subs = vec![
                                                        format!("{base}{quote}@bookTicker"),
                                                        format!("{alt}{base}@bookTicker"),
                                                        format!("{alt}{quote}@bookTicker")
                                                    ];
                                                    if quote != "USDT" {
                                                        // TODO: avoid duplicating requests?
                                                        subs.push(format!("{quote}USDT@bookTicker"));
                                                    }
                                                    let request = SubscribeRequest::new(subs);
                                                    let request = request.preprocess().unwrap();
                                                    ticker.send(request).await.unwrap();
                                                    // println!("[{i}]: sent subscribe");
                                                }
                                            }

                                        }
                                        Ok(Some((_ ,_, response))) = ticker.next() => {
                                            let update = serde_json::from_slice::<SubscriptionUpdate>(&response);
                                            if let Ok(update) = update {
                                                let mut updates = updates.borrow_mut();
                                                updates.insert(update.symbol.clone(), update);
                                                // println!("[{i}]: updates: {updates:#?}",);
                                                update_notify.notify_one();
                                            }
                                        }
                                        else => break,
                                        // else => {println!("[{i}]: something happened")},
                                    }
                                }
                                // println!("[{i}]: giving up")
                                })
                            };
                            loop {
                                // TODO: crunch more triangles if we stopped halfway through
                                update_notify.notified().await;

                                // println!("[{i}: checking tris]");
                                let tri = {
                                    let updates = updates.borrow();
                                    // TODO: bag amount handling
                                    triangles.borrow().iter().find_map(|tri| tri.crunch(&updates, 100.))
                                };

                                if let Some((leg1, leg2, leg3)) = tri {
                                    // println!("[{i}]: Found triangle opportunity: {} -- {} -- {}", leg1.symbol, leg2.symbol, leg3.symbol);
                                    let mut order_request = OrderRequest::new(
                                        &api_key,
                                        &leg1.symbol,
                                        leg1.action,
                                        OrderType::Market,
                                        Some(leg1.amt),
                                    );
                                    let mut order_request_2 = OrderRequest::new(
                                        &api_key,
                                        &leg2.symbol,
                                        leg2.action,
                                        OrderType::Market,
                                        Some(leg2.amt),
                                    );

                                    let mut order_request_3 = OrderRequest::new(
                                        &api_key,
                                        &leg3.symbol,
                                        leg3.action,
                                        OrderType::Market,
                                        Some(leg3.amt),
                                    );

                                    secret_key.sign(&mut order_request);
                                    
                                    let order_request = order_request.preprocess().unwrap();
                                    client.send(order_request).await.unwrap();

                                    secret_key.sign(&mut order_request_2);
                                    secret_key.sign(&mut order_request_3);
                                    let order_request_2 = order_request_2.preprocess().unwrap();
                                    let order_request_3 = order_request_3.preprocess().unwrap();

                                    let (_id, _method, response) =
                                        client.next().await.unwrap().unwrap();

                                    // println!("bytes: {response:?}");
                                    let response = serde_json::from_slice::<WsResponse<OrderResponse>>(&response);
                                    
                                    // conditionally send the next two orders
                                    if let Ok(
                                        WsResponse {
                                            result:
                                            OrderResponse {
                                                status: Some(OrderStatus::Filled),
                                                ..
                                            },
                                            ..
                                        }
                                    ) = response
                                    {
                                        // alt options:
                                        // ```
                                        // if let Ok(response) = response {
                                        //     if let Some(OrderStatus::Filled) = response.status {
                                        // ```
                                        // ```
                                        // if let Ok(response) = response {
                                        //     if matches!(response.status, Some(OrderStatus::Filled)) {
                                        // ```
                                        println!("sending next two orders");

                                        client.feed(order_request_2).await.unwrap();
                                        client.feed(order_request_3).await.unwrap();
                                        client.flush().await.unwrap();

                                        if let Ok(response) = &response {
                                            // print!("sending {response:?}");
                                            results_sender.send(response.result.clone()).unwrap();
                                        }

                                        let (_, _, response2) = client.next().await.unwrap().unwrap();
                                        let (_, _, response3) = client.next().await.unwrap().unwrap();

                                        let response2 = serde_json::from_slice::<WsResponse<OrderResponse>>(&response2);
                                        let response3 = serde_json::from_slice::<WsResponse<OrderResponse>>(&response3);

                                        if let Ok(response) = &response2 {
                                            // print!("sending {response:?}");
                                            results_sender.send(response.result.clone()).unwrap();
                                        }
                                        if let Ok(response) = &response3 {
                                            // print!("sending {response:?}");
                                            results_sender.send(response.result.clone()).unwrap();
                                        }

                                    }
                                }
                            }
                            // ticker.await.unwrap();
                        });
                    })
                    .unwrap();
                (subscriptions_sender, handle)
            })
            .unzip();
        Self {
            threads,
            subscription_senders,
            next_sender: 0,
            // url,
            // api_key,
            // secret_key,
            results_reciever: Arc::new(TokioMutex::new(results_reciever)),
        }
    }
    // pub fn update_tris(&self, map: HashMap<String, HashMap<String, i32>>) {}
    pub fn get_result<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let reciever = self.results_reciever.clone();
        pyo3_asyncio::tokio::future_into_py(
            py,
            async move {
                Ok(reciever.lock().await.recv().await)
            },
        )
    }
    pub fn subscribe(&mut self, triangles: Vec<(String, String, String)>) {
        // TODO: proper python conversion traits
        for(base, quote, alt) in triangles {
            self.subscription_senders[self.next_sender]
                .try_send(Subscription::Subscribe(Triangle::new(
                    base,
                    quote,
                    alt,
                )))
                .expect("couldn't send subscription");
            self.next_sender = (self.next_sender + 1) % self.subscription_senders.len();
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.subscription_senders.clear();
        for thread in self.threads.drain(..) {
            thread.join().unwrap();
        }
    }
}

#[pymodule]
fn arbitrage_processing(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Client>()?;

    Ok(())
}
