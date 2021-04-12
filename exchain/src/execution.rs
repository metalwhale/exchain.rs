use crate::analysis::{Analyze, Fetch, Status, Symbol};
use std::{
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap,
    },
    error::Error,
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
    symbols: Vec<Box<dyn Symbol>>,
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
                symbols: vec![],
                executors: vec![],
            },
        ) {
            None => Ok(self),
            Some(_) => Err(format!("`{}` key duplicated.", key)),
        }
    }

    pub fn add_symbol<S: 'static + Symbol>(mut self, key: &str, symbol: S) -> Result<Self, String> {
        match self
            .actors
            .entry(key.to_string())
            .and_modify(|a| a.symbols.push(Box::new(symbol)))
        {
            Occupied(_) => Ok(self),
            Vacant(_) => Err(format!("No `{}` key found. Use `add_fetcher` first.", key)),
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
            Vacant(_) => Err(format!("No `{}` key found. Use `add_fetcher` first.", key)),
        }
    }

    pub fn watch(&self) -> Result<(), Box<dyn Error>> {
        for Actor {
            fetcher,
            symbols,
            executors,
        } in self.actors.values()
        {
            for symbol in symbols {
                let symbol = &**symbol;
                let candles = fetcher.fetch(symbol)?;
                let status = self.analyzer.analyze(&candles)?;
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
        Ok(())
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
