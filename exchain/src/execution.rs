use crate::analysis::{Analyze, Candle, Fetch, Status};
use reqwest::blocking::Client;
use serde_json::json;
use std::{
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap,
    },
    error::Error,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone)]
pub struct Position {
    timestamp: u128,
    pair: String,
    status: Status,
}
impl Position {
    fn new(pair: &str, status: Status) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            pair: pair.to_string(),
            status,
        }
    }
}

pub trait Execute {
    fn execute(&self, candles: &[Candle], position: &Position) -> Result<(), Box<dyn Error>>;
}

struct Actor {
    fetcher: Box<dyn Fetch + Send + Sync>,
    pairs: Vec<String>,
    executors: Vec<Box<dyn Execute + Send + Sync>>,
}
pub struct Watcher<A: Analyze> {
    analyzer: A,
    actors: Vec<Actor>,
}
impl<A: Analyze> Watcher<A> {
    pub fn new(analyzer: A) -> Self {
        Watcher {
            analyzer,
            actors: vec![],
        }
    }

    pub fn add(
        &mut self,
        fetcher: Box<dyn Fetch + Send + Sync>,
        pairs: Vec<String>,
        executors: Vec<Box<dyn Execute + Send + Sync>>,
    ) {
        self.actors.push(Actor {
            fetcher,
            pairs,
            executors,
        });
    }

    pub fn watch(&self) -> Result<(), Box<dyn Error>> {
        for Actor {
            fetcher,
            pairs,
            executors,
        } in &self.actors
        {
            for pair in pairs {
                let candles = fetcher.fetch(pair)?;
                let status = self.analyzer.analyze(&candles)?;
                let position = Position::new(pair, status);
                for e in executors {
                    e.execute(&candles, &position)?;
                }
            }
        }
        Ok(())
    }
}

struct Order {
    position: Position,
    amount: f64,
}
impl Order {
    fn new(position: &Position, amount: f64) -> Self {
        Self {
            position: position.clone(),
            amount,
        }
    }
}
pub struct Strategy {
    rest: u128,
    period: u128,
    amounts: HashMap<String, (f64, f64)>,
    orders: Mutex<HashMap<String, Order>>,
}
impl Strategy {
    pub fn new(rest: u128, period: u128, amounts: HashMap<String, (f64, f64)>) -> Self {
        Self {
            rest,
            period,
            amounts,
            orders: Mutex::new(HashMap::new()),
        }
    }

    fn execute(&self, position: &Position) -> Result<Option<f64>, String> {
        let Position {
            timestamp,
            pair,
            status,
        } = position;
        let (small_amount, big_amount) = *self.amounts.get(pair).ok_or(format!(
            "Amount for {} pair not found. Declare by using `new`.",
            pair
        ))?;
        match status {
            Status::Buy | Status::Quit => match self.orders.lock().unwrap().entry(pair.to_string())
            {
                Occupied(mut entry) => {
                    let last_position = &entry.get().position;
                    let rest = timestamp - last_position.timestamp;
                    if last_position.status != *status && rest >= self.rest {
                        let amount = match status {
                            Status::Buy => {
                                if rest > self.period {
                                    big_amount
                                } else {
                                    small_amount
                                }
                            }
                            _ => 0.0,
                        };
                        entry.insert(Order::new(position, amount));
                    }
                }
                Vacant(entry) => {
                    let amount = match status {
                        Status::Buy => small_amount,
                        _ => 0.0,
                    };
                    entry.insert(Order::new(position, amount));
                }
            },
            Status::Hold => {}
        };
        Ok(match self.orders.lock().unwrap().get(pair) {
            Some(order) => {
                if order.position.timestamp == position.timestamp {
                    Some(order.amount)
                } else {
                    None
                }
            }
            None => None,
        })
    }
}

pub struct SlackExecutor {
    strategy: Arc<Strategy>,
    webhook_url: String,
}
impl SlackExecutor {
    pub fn new(strategy: Arc<Strategy>, webhook_url: &str) -> Self {
        SlackExecutor {
            strategy,
            webhook_url: webhook_url.to_string(),
        }
    }
}
impl Execute for SlackExecutor {
    fn execute(&self, candles: &[Candle], position: &Position) -> Result<(), Box<dyn Error>> {
        if let (Some(candle), Some(_amount)) = (candles.last(), self.strategy.execute(position)?) {
            let price = candle.get_price();
            let Position { pair, status, .. } = position;
            let status = match status {
                Status::Buy => "Buy",
                Status::Quit => "Quit",
                _ => "",
            };
            let data = json!({
                "text": "",
                "type": "mrkdwn",
                "attachments": [{
                    "mrkdwn_in": ["text"],
                    "color": if status == "Buy" { "good" } else { "" },
                    "text": format!("*{}* _{}_ at {}", status, pair, price),
                    "fallback": format!("{} {}", status, pair),
                }]
            });
            Client::new().post(&self.webhook_url).json(&data).send()?;
        }
        Ok(())
    }
}
