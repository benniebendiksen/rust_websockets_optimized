use std::{
    collections::HashMap,
    hash::{BuildHasher, Hash, Hasher},
};

use crate::api::{subscription::SubscriptionUpdate, types::Side};

#[derive(Debug, Clone)]
pub struct Triangle {
    pub base: String,
    pub quote: String,
    pub alt: String,
}

impl PartialEq for Triangle {
    fn eq(&self, other: &Self) -> bool {
        self.base == other.base && self.quote == other.quote && self.alt == other.alt
    }
}
impl Eq for Triangle {}

impl Hash for Triangle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.base.hash(state);
        self.quote.hash(state);
        self.alt.hash(state);
    }
}

impl Triangle {
    pub fn new(base: String, quote: String, alt: String) -> Self {
        Self { base, quote, alt }
    }
    pub fn crunch<S: BuildHasher>(
        &self,
        updates: &HashMap<String, SubscriptionUpdate, S>,
        bag_amount_usdt: f64,
    ) -> Option<(Order, Order, Order)> {
        use Side::*;

        // TODO: take const threshold as parameter
        #[allow(non_upper_case_globals)]
        const wanted_profit_pct: f64 = 0.0018;
        let Triangle { base, quote, alt } = self;
        let bag_amount_quote = if quote != "USDT" {
            // TODO: convert bag amount usdt to bag amount in quote
            bag_amount_usdt
        } else {
            bag_amount_usdt
        };

        let base_quote_ask = updates
            .get(&format!("{base}{quote}"))?
            .best_ask_price
            .parse::<f64>()
            .unwrap();
        let alt_base_ask = updates
            .get(&format!("{alt}{base}"))?
            .best_ask_price
            .parse::<f64>()
            .unwrap();
        let alt_quote_bid = updates
            .get(&format!("{alt}{quote}"))?
            .best_bid_price
            .parse::<f64>()
            .unwrap();

        let (base_amt, quote_amt, alt_amt, profit_amt);
        base_amt = bag_amount_quote / base_quote_ask;
        alt_amt = base_amt / alt_base_ask;
        quote_amt = alt_amt * alt_quote_bid;
        profit_amt = quote_amt;

        if profit_amt > wanted_profit_pct * bag_amount_quote {
            Some((
                Order {
                    symbol: format!("{alt}{quote}"),
                    amt: alt_amt.to_string(),
                    action: Sell,
                },
                Order {
                    symbol: format!("{alt}{base}"),
                    amt: alt_amt.to_string(),
                    action: Buy,
                },
                Order {
                    symbol: format!("{base}{quote}"),
                    amt: base_amt.to_string(),
                    action: Buy,
                },
            ))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Order {
    pub symbol: String,
    pub amt: String,
    pub action: Side,
}
