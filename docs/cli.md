# CLI Reference

Forge provides a CLI via both the Python package (`pip install forge-agent-sdk`) and the Rust crate (`cargo run -p forge-cli`).

## Python CLI (`forge`)

Available after `pip install forge-agent-sdk`:

| Command | Description |
|---|---|
| `forge serve [port]` | Start the dashboard server (default port 3000) |
| `forge version` | Show version |
| `forge help` | Show help |

## Rust CLI (`cargo run -p forge-cli`)

Available when building from source. Includes all Python CLI commands plus:

| Command | Description |
|---|---|
| `forge serve` | Start dashboard + API server |
| `forge run <task>` | Run agent through harness |
| `forge run --dry-run <task>` | Observe and detect, no intervention |
| `forge run --preset claude-code <task>` | Run with specific preset |
| `forge init [--name <name>]` | Scaffold new Forge project |
| `forge doctor` | Check system dependencies |
| `forge explain <session-id>` | Human-readable audit report |
| `forge watch [session-id]` | Live TUI session viewer |
| `forge bench [--suite <name>]` | Run benchmark suite |
| `forge improve [--agent-type <type>]` | Run meta-harness improvement |
| `forge validate [--config <path>]` | Validate harness config |
| `forge test [--tasks <name>]` | Run agent test suite |
| `forge diff <a> <b>` | Compare sessions or harness versions |
| `forge config [--key <k>] [--value <v>]` | View or edit forge.toml |
| `forge plugin search|install|list` | Manage community plugins |
| `forge export <id> [--format <fmt>]` | Export session data |
| `forge completion <shell>` | Generate shell completions |

!!! note
    Some Rust CLI commands are early-stage and may print placeholder output. The core commands (`serve`, `run`, `init`, `doctor`) are fully functional.
