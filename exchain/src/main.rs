mod analysis;
mod execution;

use crate::analysis::{BitfinexFetcher, MacdAnalyzer};
use crate::execution::{SlackExecutor, Strategy, Watcher};
use clokwerk::{Scheduler, TimeUnits};
use serde::Deserialize;
use std::{env, error::Error, fs, sync::Arc, thread, time::Duration};

#[derive(Deserialize)]
struct Config {
    interval: u32,
    rest: u64,
    period: u64,
    pairs: Vec<(String, f64, f64)>,
    time_frame: String,
    slack_webhook_url: String,
}

fn read_config() -> Result<Config, Box<dyn Error>> {
    let args = env::args().collect::<Vec<_>>();
    let config_path = args.get(1).ok_or("No arguments passed.")?;
    let config_content = fs::read_to_string(config_path)?;
    Ok(toml::from_str(&config_content)?)
}

fn make_scheduler() -> Result<Scheduler, Box<dyn Error>> {
    const KEY: &str = "bitfinex";
    let Config {
        interval,
        rest,
        period,
        pairs,
        time_frame,
        slack_webhook_url,
    } = read_config()?;
    let strategy = Arc::new(Strategy::new(
        rest.into(),
        period.into(),
        pairs
            .iter()
            .map(|(p, s, b)| (p.to_string(), (*s, *b)))
            .collect(),
    ));
    let mut watcher =
        Watcher::new(MacdAnalyzer::new()).add_fetcher(KEY, BitfinexFetcher::new(&time_frame))?;
    for (pair, _, _) in &pairs {
        watcher = watcher.add_pair(KEY, pair)?;
    }
    watcher = watcher.add_executor(KEY, SlackExecutor::new(strategy, &slack_webhook_url))?;
    let mut scheduler = Scheduler::new();
    scheduler.every(interval.minutes()).run(move || {
        if let Err(error) = watcher.watch() {
            println!("{}", error);
        }
    });
    Ok(scheduler)
}

fn main() {
    match make_scheduler() {
        Ok(mut scheduler) => loop {
            scheduler.run_pending();
            thread::sleep(Duration::from_millis(100));
        },
        Err(error) => {
            println!("{}", error);
        }
    }
}
