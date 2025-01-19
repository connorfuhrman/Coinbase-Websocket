use coinbase_websocket::websocket::client::CoinbaseWebSocketClient;
use coinbase_websocket::models::ticker::Ticker;

use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger
    env_logger::init();

    let ws_cli = CoinbaseWebSocketClient::new();
    
    ws_cli.register_handler(
        "BTC-USD".to_string(),
        |msg: Ticker| {	    
            info!(
                "BTC-USD Handler: Current price of {} is {}",
		msg.ticker_type,
                msg.price
            );
        },
    ).await;

    return ws_cli.connect_and_listen().await;
}
