use coinbase_websocket::websocket::client::CoinbaseWebSocketClient;
use coinbase_websocket::models::ticker::Ticker;

use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger
    env_logger::init();

    // Create websocket client 
    let ws_cli = CoinbaseWebSocketClient::new();

    // Register handler for BTC
    ws_cli.register_handler(
        "BTC-USD".to_string(),
        |msg: Ticker| {
            info!(
                "BTC-USD Handler: Current price of is ${}",
                msg.price
            );
        },
    ).await;

    // Register handler for ETH
    ws_cli.register_handler(
	"ETH-USD".to_string(),
	|msg: Ticker| {
	    info!(
		"ETH-USD Handler: Current price is ${}",
		msg.price
	    );
	},
    ).await;

    // Connect and listen to the websocket where the callback
    // handler will be run for each message 
    return ws_cli.connect_and_listen().await;
}
