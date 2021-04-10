mod analysis;
mod execution;

use analysis::{BitfinexFetcher, MacdAnalyzer};
use execution::SlackExecutor;

use crate::execution::Watcher;

fn main() {
    let mut watcher = Watcher::new(MacdAnalyzer::new());
    watcher.add_fetcher(BitfinexFetcher::new("1D", "tBTCUSD"));
    watcher.add_executor(SlackExecutor::new());
    watcher.watch();
}
