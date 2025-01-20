use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use std::collections::HashMap;
use std::sync::Arc;
use std::cell::Cell;
use tokio::sync::RwLock;


use crate::models::ticker::{WebSocketMessage, Ticker};

const WS_URL: &str = "wss://advanced-trade-ws.coinbase.com";


/// Interface to coinbase 'advanced trade' websocket
pub struct CoinbaseWebSocketClient {
    // Store handlers in a thread-safe map that can be shared across async tasks
    handlers: Arc<RwLock<HashMap<String, Box<dyn Fn(Ticker)>>>>,
    last_seq_num: Cell<u64>,
}


impl CoinbaseWebSocketClient
{
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
	    last_seq_num: 0.into(),
        }
    }

    /// Method to register a handler for a specific product ID
    ///
    /// When a new message for that particular `product_id` is received the
    /// associated handler method will be called
    pub async fn register_handler(&self, product_id: String, handler: Box<dyn Fn(Ticker)>) {
	info!("Adding handler for product ID {}", product_id);
        let mut handlers = self.handlers.write().await;
        handlers.insert(product_id, handler);
    }

    fn check_seq_num(&self, msg: &WebSocketMessage) -> bool {
	// Handle first message
	if msg.sequence_num == 0 && self.last_seq_num.get() == 0 {
	    return true;
	}

	// Check that the sequence has increased
	if msg.sequence_num < self.last_seq_num.get() {
	    error!("Received sequence number less than previous! Dropping messages somehow");
	    return false;
	}

	// Warn if a message has been dropped somehow
	if msg.sequence_num != self.last_seq_num.get() + 1 {
	    warn!("Last sequence number received was {} but got {}. \
		   Dropping messages somehow",
		  msg.sequence_num,
		  self.last_seq_num.get());
	}

	self.last_seq_num.set(msg.sequence_num);

	return true;
    }

    /// Dispatch based on the message received
    async fn dispatch_message(&self, message: WebSocketMessage) {
	// Ensure sequence numbering is correct
	if !self.check_seq_num(&message) {
	    return;
	}
	
        // Get a read lock on the handlers map
        let handlers = self.handlers.read().await;
        
        // For each ticker in the message
        for event in &message.events {
            for ticker in &event.tickers {
                // If we have a handler for this product ID, call it
                if let Some(handler) = handlers.get(&ticker.product_id) {
                    handler(ticker.clone());
                } else {
		    panic!("Got a product ID {} but don't have a handler for that",
			   ticker.product_id);
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
