//! Forge Server entry point.
//!
//! Start with: cargo run -p forge-server
//! Or via CLI:   forge serve

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    forge_server::run_server(3000).await;
}
