use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use serde::{Deserialize, Serialize};
// use chrono::{DateTime, Utc};

use log::{info, debug, warn, error};

/// Represents the complete WebSocket message from Coinbase
#[derive(Debug, Serialize, Deserialize)]
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
    pub timestamp: String // DateTime<Utc>,
}

/// Represents an event within the WebSocket message
#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    /// Array of ticker updates
    pub tickers: Vec<Ticker>,
    /// Event type (e.g., "update")
    #[serde(rename = "type")]
    pub event_type: String,
}

/// Contains the actual ticker data for a cryptocurrency
#[derive(Debug, Serialize, Deserialize)]
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

// The URL for Coinbase's WebSocket API
const WS_URL: &str = "wss://advanced-trade-ws.coinbase.com";

pub fn handle_ws_msg(msg: WebSocketMessage) {
    debug!("Got new message from ws @ {}", msg.timestamp);
    info!("Current value of BTC-USD is {}", msg.events[0].tickers[0].price);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    info!("Connecting to coinbase owebsocket");
    
    // Connect to the WebSocket server
    debug!("Connecting to websocket {}", WS_URL);
    let (ws_stream, _) = connect_async(WS_URL).await?;
    info!("Connected to Coinbase WebSocket server");
    
    // Split the WebSocket stream into sender and receiver
    let (mut write, mut read) = ws_stream.split();
    
    // Create the subscription message
    let subscribe_msg = json!({
        "type": "subscribe",
        "product_ids": ["BTC-USD"],
        "channel": "ticker"
    });
    debug!("Subscribing to {}", subscribe_msg);
    
    // Send the subscription message
    write.send(Message::Text(subscribe_msg.to_string())).await?;
    
    // Handle incoming messages
    while let Some(message_result) = read.next().await {
        match message_result {
            Ok(message) => {
                match message {
                    Message::Text(text) => {
                        // Parse the JSON message
                        match serde_json::from_str::<WebSocketMessage>(&text) {
                            // Ok(data) => debug!("Received message: {}", data),
			    Ok(data) => handle_ws_msg(data),
                            Err(e) => error!("Failed to parse message '{}': {}", text, e),
                        }
                    },
                    Message::Close(_) => {
                        warn!("Connection closed by server");
                        break;
                    },
                    _ => {
                        // Handle other message types (Binary, Ping, Pong, etc.)
                        debug!("Received non-text message: {:?}", message);
                    }
                }
            },
            Err(e) => {
                error!("Error receiving message: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}
