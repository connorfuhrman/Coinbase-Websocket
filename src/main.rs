// This line declares that we're using the async runtime
// The #[tokio::main] attribute transforms our async main function into a regular main function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // println! is a macro (note the ! at the end) that prints text to the console
    println!("Hello, World!");

    // This demonstrates string formatting in Rust
    let name = "Rustacean";
    // println!("Welcome to the world of Rust, {}!", name);
    println!("Hello world {}", name);

    // Since we're using the Result type, we need to return Ok(())
    // () is the "unit type" in Rust, similar to void in other languages
    Ok(())
}
