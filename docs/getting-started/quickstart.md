# Quickstart

## Install

```bash
cargo install forge-sdk
```

## 5-Minute Setup

```rust
use forge_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), ForgeError> {
    let harness = Harness::builder()
        .dry_run(false)
        .build()?;

    println!("Forge harness ready!");
    Ok(())
}
```

## Run Your First Agent

Wrap any existing agent with `AgentAdapter` and run it inside the harness.
