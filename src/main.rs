use coinbase_websocket::websocket::client::CoinbaseWebSocketClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger
    env_logger::init();
    
    // Connect and start listening for messages
    CoinbaseWebSocketClient::connect_and_listen().await
}
