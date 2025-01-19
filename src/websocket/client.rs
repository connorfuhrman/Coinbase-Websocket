use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::models::ticker::WebSocketMessage;

pub const WS_URL: &str = "wss://advanced-trade-ws.coinbase.com";

/// Interface to coinbase 'advanced trade' websocket
pub struct CoinbaseWebSocketClient;

impl CoinbaseWebSocketClient {
    pub fn handle_ws_msg(msg: WebSocketMessage) {
        debug!("Got new message from ws @ {}", msg.timestamp);
        info!(
            "Current value of BTC-USD is {}",
            msg.events[0].tickers[0].price
        );
    }

    pub async fn connect_and_listen() -> Result<(), Box<dyn std::error::Error>> {
        info!("Connecting to coinbase websocket");
        
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
                Ok(message) => match message {
                    Message::Text(text) => {
                        // Parse the JSON message
                        match serde_json::from_str::<WebSocketMessage>(&text) {
                            Ok(data) => Self::handle_ws_msg(data),
                            Err(e) => error!("Failed to parse message '{}': {}", text, e),
                        }
                    }
                    Message::Close(_) => {
                        warn!("Connection closed by server");
                        break;
                    }
                    _ => {
                        debug!("Received non-text message: {:?}", message);
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
}
