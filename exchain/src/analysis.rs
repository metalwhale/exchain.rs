use serde::Deserialize;
use std::error::Error;

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Candle {
    timestamp: u128,
    open: f64,
    close: f64,
    high: f64,
    low: f64,
    volume: f64,
}

pub trait Symbol {
    fn symbolize(&self) -> String;
}

pub enum Status {
    Buy,
    Hold,
    Quit,
}

pub trait Fetch {
    fn fetch(&self, symbol: &dyn Symbol) -> Result<Vec<Candle>, Box<dyn Error>>;
}

pub trait Analyze {
    fn analyze(&self, candles: &[Candle]) -> Option<Status>;
}

pub struct BitfinexSymbol {
    pair: String,
}
impl BitfinexSymbol {
    pub fn new(pair: &str) -> Self {
        Self {
            pair: pair.to_string(),
        }
    }
}
impl Symbol for BitfinexSymbol {
    fn symbolize(&self) -> String {
        format!("t{}", self.pair)
    }
}

pub struct BitfinexFetcher {
    time_frame: String,
}
impl BitfinexFetcher {
    pub fn new(time_frame: &str) -> Self {
        Self {
            time_frame: time_frame.to_string(),
        }
    }
}
impl Fetch for BitfinexFetcher {
    fn fetch(&self, symbol: &dyn Symbol) -> Result<Vec<Candle>, Box<dyn Error>> {
        const LIMIT: usize = 240;
        let mut response: Vec<Candle> = reqwest::blocking::get(format!(
            "https://api-pub.bitfinex.com/v2/candles/trade:{}:{}/hist?limit={}",
            self.time_frame,
            symbol.symbolize(),
            LIMIT
        ))?
        .json()?;
        response.reverse();
        Ok(response)
    }
}

struct MacdHistogram {
    macd: f64,
    signal: f64,
}
pub struct MacdAnalyzer {}
impl MacdAnalyzer {
    fn calculate_histograms(&self, prices: &[f64]) -> Vec<MacdHistogram> {
        const FAST_PERIOD: usize = 12;
        const SLOW_PERIOD: usize = 26;
        const SIGNAL_PERIOD: usize = 9;
        let fast_multiplier = 2.0 / (FAST_PERIOD + 1) as f64;
        let slow_multiplier = 2.0 / (SLOW_PERIOD + 1) as f64;
        let signal_multiplier = 2.0 / (SIGNAL_PERIOD + 1) as f64;
        let mut fast: f64 = prices[FAST_PERIOD - 1..=SLOW_PERIOD - 1].iter().sum();
        let mut slow: f64 = prices[0..=SLOW_PERIOD - 1].iter().sum();
        let mut macds = vec![];
        for price in &prices[SLOW_PERIOD..] {
            fast += (price - fast) * fast_multiplier;
            slow += (price - slow) * slow_multiplier;
            macds.push(fast - slow);
        }
        let mut signal = macds[0..=SIGNAL_PERIOD - 1].iter().sum();
        let mut macd_histograms = vec![];
        for macd in macds.drain(SIGNAL_PERIOD..) {
            signal += (macd - signal) * signal_multiplier;
            macd_histograms.push(MacdHistogram { macd, signal });
        }
        macd_histograms
    }
}
impl Analyze for MacdAnalyzer {
    fn analyze(&self, candles: &[Candle]) -> Option<Status> {
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let mut histograms = self.calculate_histograms(&closes);
        let MacdHistogram {
            macd: last_macd,
            signal: last_signal,
        } = histograms.pop()?;
        Some(match last_macd - last_signal {
            d if d > 0.0 => {
                let MacdHistogram {
                    macd: second_last_macd,
                    signal: second_last_signal,
                } = histograms.pop()?;
                match second_last_macd - second_last_signal {
                    d if d <= 0.0 => Status::Buy,
                    _ => Status::Hold,
                }
            }
            _ => Status::Quit,
        })
    }
}
