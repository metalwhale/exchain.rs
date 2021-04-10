use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize)]
pub struct Candle {
    timestamp: u64,
    open: f64,
    close: f64,
    high: f64,
    low: f64,
    volume: f64,
}

pub enum Position {
    Buy,
    SmallBuy,
    Sell,
    SmallSell,
    Hold,
    Quit,
}

pub trait Fetch {
    fn fetch(&self) -> Result<Vec<Candle>, Box<dyn Error>>;
}

pub trait Analyze {
    fn analyze(&self, candles: &[Candle]) -> Position;
}

pub struct BitfinexFetcher {
    time_frame: String,
    symbol: String,
}
impl BitfinexFetcher {
    pub fn new(time_frame: &str, symbols: &str) -> Self {
        BitfinexFetcher {
            time_frame: time_frame.to_string(),
            symbol: symbols.to_string(),
        }
    }
}
impl Fetch for BitfinexFetcher {
    fn fetch(&self) -> Result<Vec<Candle>, Box<dyn Error>> {
        let response: Vec<Candle> = reqwest::blocking::get(format!(
            "https://api-pub.bitfinex.com/v2/candles/trade:{}:{}/hist",
            self.time_frame, self.symbol
        ))?
        .json()?;
        Ok(response)
    }
}

pub struct MacdAnalyzer {}
impl MacdAnalyzer {
    pub fn new() -> Self {
        MacdAnalyzer {}
    }
}
impl Analyze for MacdAnalyzer {
    fn analyze(&self, candles: &[Candle]) -> Position {
        todo!()
    }
}
