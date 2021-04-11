use crate::analysis::{Analyze, Fetch, Position};

pub trait Execute {
    fn execute(&self, position: &Position);
}

pub struct Watcher<'w, A: Analyze> {
    analyzer: A,
    fetchers: Vec<Box<dyn 'w + Fetch>>,
    executors: Vec<Box<dyn 'w + Execute>>,
}

impl<'w, A: Analyze> Watcher<'w, A> {
    pub fn new(analyzer: A) -> Self {
        Watcher {
            analyzer,
            fetchers: vec![],
            executors: vec![],
        }
    }

    pub fn add_fetcher<F: 'w + Fetch>(&mut self, fetcher: F) {
        self.fetchers.push(Box::new(fetcher));
    }

    pub fn add_executor<E: 'w + Execute>(&mut self, executor: E) {
        self.executors.push(Box::new(executor));
    }

    pub fn watch(&self) {
        for fetcher in &self.fetchers {
            if let Ok(candles) = fetcher.fetch() {
                if let Some(position) = self.analyzer.analyze(&candles) {
                    for e in &self.executors {
                        e.execute(&position);
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
