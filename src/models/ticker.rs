use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the complete WebSocket message from Coinbase
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebSocketMessage {
    /// The channel name (e.g., "ticker")
    pub channel: String,
    /// Optional client identifier
    #[serde(default)]
    pub client_id: String,
    /// Array of events in this message
    pub events: Vec<Event>,
    /// Sequence number for message ordering
    pub sequence_num: u64,
    /// Timestamp of the message
    pub timestamp: String
}

/// Represents different types of events within the WebSocket message
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Event {
    // Variant for ticker updates (snapshot or update)
    TickerEvent {
        #[serde(rename = "type")]
        event_type: String,
        tickers: Vec<Ticker>,
    },
    // Variant for subscription acknowledgments
    SubscriptionEvent {
        subscriptions: HashMap<String, Vec<String>>,
    },
}

/// Contains the actual ticker data for a cryptocurrency
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ticker {
    /// Best asking price in USD
    pub best_ask: String,
    /// Quantity available at best ask price
    pub best_ask_quantity: String,
    /// Best bid price in USD
    pub best_bid: String,
    /// Quantity available at best bid price
    pub best_bid_quantity: String,
    /// 24-hour high price
    pub high_24_h: String,
    /// 52-week high price
    pub high_52_w: String,
    /// 24-hour low price
    pub low_24_h: String,
    /// 52-week low price
    pub low_52_w: String,
    /// Current price
    pub price: String,
    /// 24-hour price percent change
    pub price_percent_chg_24_h: String,
    /// Product identifier (e.g., "BTC-USD")
    pub product_id: String,
    /// Type of ticker update
    #[serde(rename = "type")]
    pub ticker_type: String,
    /// 24-hour trading volume
    pub volume_24_h: String,
}
