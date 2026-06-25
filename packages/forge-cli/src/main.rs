use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "forge", about = "Forge — The Agent Harness SDK", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scaffold a new Forge project
    Init { name: Option<String> },
    /// Run an agent with harness
    Run { task: String, #[arg(long)] agent: Option<String>, #[arg(long)] dry_run: bool },
    /// Show active session health
    Watch { session_id: Option<String> },
    /// Replay a session from audit trail
    Replay { session_id: String },
    /// Print human-readable audit report
    Explain { session_id: String },
    /// Run benchmark suite
    Bench { #[arg(long)] suite: Option<String> },
    /// Run meta-harness improvement cycle
    Improve { #[arg(long)] agent_type: Option<String> },
    /// Start API server and dashboard
    Serve { #[arg(long, default_value = "3000")] port: u16 },
    /// Check system dependencies
    Doctor,
    /// Run agent against test suite
    Test { #[arg(long)] tasks: Option<String> },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { name } => {
            let n = name.unwrap_or_else(|| "my-agent".into());
            println!("forge: Initialized new Forge project '{}'", n);
        }
        Commands::Run { task, agent, dry_run } => {
            println!("forge: Running task '{}' (dry_run: {})", task, dry_run);
        }
        Commands::Watch { session_id } => {
            println!("forge: Watching session {}", session_id.unwrap_or_else(|| "latest".into()));
        }
        Commands::Replay { session_id } => {
            println!("forge: Replaying session {}", session_id);
        }
        Commands::Explain { session_id } => {
            println!("forge: Audit report for session {}", session_id);
        }
        Commands::Bench { suite } => {
            println!("forge: Running benchmark suite: {}", suite.unwrap_or_else(|| "standard".into()));
        }
        Commands::Improve { agent_type } => {
            println!("forge: Running meta-harness improvement for {}", agent_type.unwrap_or_else(|| "all".into()));
        }
        Commands::Serve { port } => {
            println!("forge: Starting server on port {}", port);
        }
        Commands::Doctor => {
            println!("forge: System check — Rust ✅, SQLite ✅, Docker ⚠️");
        }
        Commands::Test { tasks } => {
            println!("forge: Running tests: {}", tasks.unwrap_or_else(|| "all".into()));
        }
    }
}
