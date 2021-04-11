use crate::analysis::{Analyze, Fetch, Status, Symbol};
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Position<'p> {
    timestamp: u128,
    symbol: &'p dyn Symbol,
    status: Status,
}

pub trait Execute {
    fn execute(&self, position: &Position);
}

struct Actor {
    fetcher: Box<dyn Fetch>,
    executors: Vec<Box<dyn Execute>>,
}
pub struct Watcher<A: Analyze> {
    analyzer: A,
    symbols: Vec<Box<dyn Symbol>>,
    actors: HashMap<String, Actor>,
}
impl<A: Analyze> Watcher<A> {
    pub fn new(analyzer: A) -> Self {
        Watcher {
            analyzer,
            symbols: vec![],
            actors: HashMap::new(),
        }
    }

    pub fn add_symbol<S: 'static + Symbol>(&mut self, symbol: S) {
        self.symbols.push(Box::new(symbol));
    }

    pub fn add_fetcher<F: 'static + Fetch>(&mut self, key: &str, fetcher: F) {
        self.actors.insert(
            key.to_string(),
            Actor {
                fetcher: Box::new(fetcher),
                executors: vec![],
            },
        );
    }

    pub fn add_executor<E: 'static + Execute>(&mut self, key: &str, executor: E) {
        self.actors
            .entry(key.to_string())
            .and_modify(|a| a.executors.push(Box::new(executor)));
    }

    pub fn watch(&self) {
        for symbol in &self.symbols {
            let symbol = &**symbol;
            for Actor { fetcher, executors } in self.actors.values() {
                if let Ok(candles) = fetcher.fetch(symbol) {
                    if let Some(status) = self.analyzer.analyze(&candles) {
                        let position = Position {
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis(),
                            symbol,
                            status,
                        };
                        for e in executors {
                            e.execute(&position);
                        }
                    }
                }
            }
        }
    }
}

pub struct SlackExecutor {}
impl SlackExecutor {
    pub fn new() -> Self {
        SlackExecutor {}
    }
}
impl Execute for SlackExecutor {
    fn execute(&self, position: &Position) {}
}
