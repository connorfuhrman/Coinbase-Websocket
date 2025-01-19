use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;


use crate::models::ticker::{WebSocketMessage, Ticker};

const WS_URL: &str = "wss://advanced-trade-ws.coinbase.com";

// Type alias for our handler function
pub type MessageHandler = Arc<dyn Fn(Ticker) + Send + Sync>;

/// Interface to coinbase 'advanced trade' websocket
pub struct CoinbaseWebSocketClient {
    // Store handlers in a thread-safe map that can be shared across async tasks
    handlers: Arc<RwLock<HashMap<String, MessageHandler>>>,
}


impl CoinbaseWebSocketClient {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Method to register a handler for a specific product ID
    pub async fn register_handler<F>(&self, product_id: String, handler: F)
    where
        F: Fn(Ticker) + Send + Sync + 'static,
    {
	info!("Adding handler for product ID {}", product_id);
        let mut handlers = self.handlers.write().await;
        handlers.insert(product_id, Arc::new(handler));
    }

    async fn dispatch_message(
        &self,
        message: WebSocketMessage,
    ) {
        // Get a read lock on the handlers map
        let handlers = self.handlers.read().await;
        
        // For each ticker in the message
        for event in &message.events {
            for ticker in &event.tickers {
                // If we have a handler for this product ID, call it
                if let Some(handler) = handlers.get(&ticker.product_id) {
                    handler(ticker.clone());
                }
            }
        }
    }


    pub async fn connect_and_listen(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Connecting to coinbase websocket");
        
        // Create the subscription message
        let handlers = self.handlers.read().await;
        let product_ids: Vec<String> = handlers.keys().cloned().collect();
        
        // Ensure we have at least one handler registered
        if product_ids.is_empty() {
            return Err("No handlers registered. \
			Please register at least one handler before connecting.".into());
        }
        
        debug!("Found handlers for products: {:?}", product_ids);
	
        let subscribe_msg = json!({
            "type": "subscribe",
            "product_ids": product_ids,
            "channel": "ticker"
        });
        info!("Subscribing to {}", subscribe_msg);

	// Connect to the WebSocket server
        debug!("Connecting to websocket {}", WS_URL);
        let (ws_stream, _) = connect_async(WS_URL).await?;
        info!("Connected to Coinbase WebSocket server");
        
        // Split the WebSocket stream into sender and receiver
        let (mut write, mut read) = ws_stream.split();
        
        // Send the subscription message
        write.send(Message::Text(subscribe_msg.to_string())).await?;
        
        // Handle incoming messages
        while let Some(message_result) = read.next().await {
            match message_result {
                Ok(message) => match message {
                    Message::Text(text) => {
			debug!("Received JSON message from websocket {}", text);
                        // Parse the JSON message
                        match serde_json::from_str::<WebSocketMessage>(&text) {
                            Ok(data) => self.dispatch_message(data).await,
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
                    return Err(e.into())
                }
            }
        }
        
        Ok(())
    }
}
