use crate::analysis::{Analyze, Fetch, Status};
use std::{
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap,
    },
    error::Error,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Position {
    timestamp: u128,
    pair: String,
    status: Status,
}
impl Position {
    fn new(time: SystemTime, pair: &str, status: Status) -> Self {
        Self {
            timestamp: time.duration_since(UNIX_EPOCH).unwrap().as_millis(),
            pair: pair.to_string(),
            status,
        }
    }
}

pub trait Execute {
    fn execute(&self, position: &Position);
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
            pairs,
            executors,
        } in self.actors.values()
        {
            for pair in pairs {
                let candles = fetcher.fetch(pair)?;
                let status = self.analyzer.analyze(&candles)?;
                let position = Position::new(SystemTime::now(), pair, status);
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
