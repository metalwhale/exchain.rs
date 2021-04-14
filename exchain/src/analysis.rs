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
impl Candle {
    pub fn get_price(&self) -> f64 {
        self.close
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Status {
    Buy,
    Hold,
    Quit,
}

pub trait Fetch {
    fn fetch(&self, pair: &str) -> Result<Vec<Candle>, Box<dyn Error>>;
}

pub trait Analyze {
    fn analyze(&self, candles: &[Candle]) -> Result<Status, Box<dyn Error>>;
}

pub struct BitfinexFetcher {
    time_frame: String,
    limit: usize,
}
impl BitfinexFetcher {
    pub fn new(time_frame: &str, limit: usize) -> Self {
        Self {
            time_frame: time_frame.to_string(),
            limit,
        }
    }
}
impl Fetch for BitfinexFetcher {
    fn fetch(&self, pair: &str) -> Result<Vec<Candle>, Box<dyn Error>> {
        let mut response: Vec<Candle> = reqwest::blocking::get(format!(
            "https://api-pub.bitfinex.com/v2/candles/trade:{}:{}/hist?limit={}",
            self.time_frame,
            format!("t{}", pair),
            self.limit
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
pub struct MacdAnalyzer {
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
}
impl MacdAnalyzer {
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Self {
        Self {
            fast_period,
            slow_period,
            signal_period,
        }
    }

    fn calculate_histograms(&self, prices: &[f64]) -> Vec<MacdHistogram> {
        let fast_multiplier = 2.0 / (self.fast_period + 1) as f64;
        let slow_multiplier = 2.0 / (self.slow_period + 1) as f64;
        let signal_multiplier = 2.0 / (self.signal_period + 1) as f64;
        let mut fast: f64 = prices[self.fast_period - 1..=self.slow_period - 1]
            .iter()
            .sum();
        let mut slow: f64 = prices[0..=self.slow_period - 1].iter().sum();
        let mut macds = vec![];
        for price in &prices[self.slow_period..] {
            fast += (price - fast) * fast_multiplier;
            slow += (price - slow) * slow_multiplier;
            macds.push(fast - slow);
        }
        let mut signal = macds[0..=self.signal_period - 1].iter().sum();
        let mut macd_histograms = vec![];
        for macd in macds.drain(self.signal_period..) {
            signal += (macd - signal) * signal_multiplier;
            macd_histograms.push(MacdHistogram { macd, signal });
        }
        macd_histograms
    }
}
impl Analyze for MacdAnalyzer {
    fn analyze(&self, candles: &[Candle]) -> Result<Status, Box<dyn Error>> {
        const ERROR: &str = "Not enough candles.";
        let prices = candles.iter().map(|c| c.get_price()).collect::<Vec<_>>();
        let mut histograms = self.calculate_histograms(&prices);
        let MacdHistogram {
            macd: last_macd,
            signal: last_signal,
        } = histograms.pop().ok_or(ERROR)?;
        Ok(match last_macd - last_signal {
            d if d > 0.0 => {
                let MacdHistogram {
                    macd: second_last_macd,
                    signal: second_last_signal,
                } = histograms.pop().ok_or(ERROR)?;
                match second_last_macd - second_last_signal {
                    d if d <= 0.0 => Status::Buy,
                    _ => Status::Hold,
                }
            }
            _ => Status::Quit,
        })
    }
}
