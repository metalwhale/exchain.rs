use crate::analysis::{Analyze, Fetch, Position};

pub trait Execute {
    fn execute(&self, position: &Position);
}

pub struct Watcher<'f, 'e, A: Analyze> {
    fetchers: Vec<Box<dyn 'f + Fetch>>,
    analyzer: A,
    executors: Vec<Box<dyn 'e + Execute>>,
}

impl<'f, 'e, A: Analyze> Watcher<'f, 'e, A> {
    pub fn new(analyzer: A) -> Self {
        Watcher {
            fetchers: vec![],
            analyzer,
            executors: vec![],
        }
    }

    pub fn add_fetcher<F: 'f + Fetch>(&mut self, fetcher: F) {
        self.fetchers.push(Box::new(fetcher));
    }

    pub fn add_executor<E: 'e + Execute>(&mut self, executor: E) {
        self.executors.push(Box::new(executor));
    }

    pub fn watch(&self) {
        for fetcher in &self.fetchers {
            if let Ok(candles) = fetcher.fetch() {
                let position = self.analyzer.analyze(&candles);
                for e in &self.executors {
                    e.execute(&position);
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
