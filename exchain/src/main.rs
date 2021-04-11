mod analysis;
mod execution;

use crate::analysis::{BitfinexFetcher, BitfinexSymbol, MacdAnalyzer};
use crate::execution::{SlackExecutor, Watcher};

fn main() {
    let mut watcher = Watcher::new(MacdAnalyzer {});
    watcher.add_symbol(BitfinexSymbol::new("BTCUSD"));
    watcher.add_fetcher("bitfinex", BitfinexFetcher::new("1D"));
    watcher.add_executor("bitfinex", SlackExecutor::new());
    watcher.watch();
}
