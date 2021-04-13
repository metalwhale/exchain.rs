use crate::analysis::{Analyze, Fetch, Status};
use std::{
    cell::RefCell,
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap,
    },
    error::Error,
    rc::Rc,
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
    fn execute(&self, position: &Position) -> Result<(), Box<dyn Error>>;
}

struct Actor {
    fetcher: Box<dyn Fetch>,
    pairs: Vec<String>,
    executors: Vec<Box<dyn Execute>>,
}
pub struct Watcher<A: Analyze> {
    analyzer: A,
    actors: HashMap<String, Actor>,
}
impl<A: Analyze> Watcher<A> {
    pub fn new(analyzer: A) -> Self {
        Watcher {
            analyzer,
            actors: HashMap::new(),
        }
    }

    pub fn add_fetcher<F: 'static + Fetch>(
        mut self,
        key: &str,
        fetcher: F,
    ) -> Result<Self, String> {
        match self.actors.insert(
            key.to_string(),
            Actor {
                fetcher: Box::new(fetcher),
                pairs: vec![],
                executors: vec![],
            },
        ) {
            None => Ok(self),
            Some(_) => Err(format!("`{}` key duplicated.", key)),
        }
    }

    pub fn add_pair(mut self, key: &str, pair: &str) -> Result<Self, String> {
        match self
            .actors
            .entry(key.to_string())
            .and_modify(|a| a.pairs.push(pair.to_string()))
        {
            Occupied(_) => Ok(self),
            Vacant(_) => Err(format!("`{}` key not found. Use `add_fetcher` first.", key)),
        }
    }

    pub fn add_executor<E: 'static + Execute>(
        mut self,
        key: &str,
        executor: E,
    ) -> Result<Self, String> {
        match self
            .actors
            .entry(key.to_string())
            .and_modify(|a| a.executors.push(Box::new(executor)))
        {
            Occupied(_) => Ok(self),
            Vacant(_) => Err(format!("`{}` key not found. Use `add_fetcher` first.", key)),
        }
    }

    pub fn watch(&self) -> Result<(), Box<dyn Error>> {
        for Actor {
            fetcher,
            pairs,
            executors,
        } in self.actors.values()
        {
            for pair in pairs {
                let candles = fetcher.fetch(pair)?;
                let status = self.analyzer.analyze(&candles)?;
                let position = Position::new(pair, status);
                for e in executors {
                    e.execute(&position)?;
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
    orders: RefCell<HashMap<String, Order>>,
}
impl Strategy {
    pub fn new(rest: u128, period: u128, amounts: HashMap<String, (f64, f64)>) -> Self {
        Self {
            rest,
            period,
            amounts,
            orders: RefCell::new(HashMap::new()),
        }
    }

    fn execute(&self, position: &Position) -> Result<Option<f64>, Box<dyn Error>> {
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
            Status::Buy | Status::Quit => match self.orders.borrow_mut().entry(pair.to_string()) {
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
        Ok(match self.orders.borrow().get(pair) {
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
    strategy: Rc<Strategy>,
}
impl SlackExecutor {
    pub fn new(strategy: Rc<Strategy>) -> Self {
        SlackExecutor { strategy }
    }
}
impl Execute for SlackExecutor {
    fn execute(&self, position: &Position) -> Result<(), Box<dyn Error>> {
        if let Some(amount) = self.strategy.execute(position)? {
            let status = match position.status {
                Status::Buy => "Buy",
                Status::Quit => "Quit",
                _ => "",
            };
            println!("{} {}", status, amount);
        }
        Ok(())
    }
}
