# Installation

## Python (Recommended)

```bash
pip install forge-agent-sdk
```

Requirements: Python 3.10+ (including 3.14). Works on Windows, macOS, and Linux.

This gives you the `forge` CLI command and the `forge_sdk` Python package.

### Verify

```bash
forge version
python -c "from forge_sdk import get_version; print(get_version())"
```

## Rust — Build from Source

### Prerequisites

- Rust 1.85+
- SQLite (bundled via sqlx)

### Build

```bash
git clone https://github.com/jalajagrawalgenai/HarnessForge.git
cd HarnessForge
cargo build --release
```

### Run the CLI

```bash
cargo run -p forge-cli -- serve     # Start dashboard
cargo run -p forge-cli -- run "Your task here"
cargo run -p forge-cli -- doctor     # System check
```

### Use as a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
forge-sdk = { git = "https://github.com/jalajagrawalgenai/HarnessForge.git" }
forge-harness = { git = "https://github.com/jalajagrawalgenai/HarnessForge.git" }
```

## Python — Build from Source

```bash
git clone https://github.com/jalajagrawalgenai/HarnessForge.git
cd HarnessForge/packages/forge-py
pip install maturin
maturin develop
```
