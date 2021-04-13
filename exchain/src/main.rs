mod analysis;
mod execution;

use crate::analysis::{BitfinexFetcher, MacdAnalyzer};
use crate::execution::{SlackExecutor, Strategy, Watcher};
use std::{array::IntoIter, rc::Rc};

fn main() {
    match make_watcher("bitfinex") {
        Ok(watcher) => {
            if let Err(e) = watcher.watch() {
                println!("{}", e);
            }
        }
        Err(e) => println!("{}", e),
    }
}

fn make_watcher(key: &str) -> Result<Watcher<MacdAnalyzer>, String> {
    const ONE_DAY: u128 = 86400000;
    let strategy = Rc::new(Strategy::new(
        ONE_DAY / 2,
        ONE_DAY * 6,
        IntoIter::new([("BTCUSD".to_string(), (0.5, 1.0))]).collect(),
    ));
    Watcher::new(MacdAnalyzer::new())
        .add_fetcher(key, BitfinexFetcher::new("1D"))?
        .add_pair(key, "BTCUSD")?
        .add_executor(key, SlackExecutor::new(strategy))
}
