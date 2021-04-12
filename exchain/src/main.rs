mod analysis;
mod execution;

use crate::analysis::{BitfinexFetcher, BitfinexSymbol, MacdAnalyzer};
use crate::execution::{SlackExecutor, Watcher};

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
    Watcher::new(MacdAnalyzer {})
        .add_fetcher(key, BitfinexFetcher::new("1D"))?
        .add_symbol(key, BitfinexSymbol::new("BTCUSD"))?
        .add_executor(key, SlackExecutor::new())
}
