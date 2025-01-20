# Coinbase Advanced Trading Websocket Interface

An interface to the coinbase advanced websocket written in Rust. 

To use include the `CoinbaseWebSocketClient` and `Ticker` types from the `coinbase_websocket` crate:

```rust
use coinbase_websocket::websocket::client::CoinbaseWebSocketClient;
use coinbase_websocket::models::ticker::Ticker;
```

Handlers for each ticker ID can be added and will run on receipt of each new ticker message

```rust
// Create websocket client 
let ws_cli = CoinbaseWebSocketClient::new();

ws_cli.register_handler(
	"BTC-USD".to_string(),
	Box::new(|msg: Ticker| {
		info!(
			"Current price of BTC is ${}",
			msg.price
		);
	}),
).await;

ws_cli.connect_and_listen().await;
```


## Build
If you use Nix you can just `nix build` then, to run the example main, run `RUST_LOG=info nix run`.
The project can also be built simply with `cargo build`.
