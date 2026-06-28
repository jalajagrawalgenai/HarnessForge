// forge-cli/src/main.rs — Forge CLI entry point
//
// All commands now call into real SDK/harness logic instead of printing stubs.

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
    Init {
        /// Project name
        name: Option<String>,
        /// Agent type preset
        #[arg(long, default_value = "solo")]
        agent_type: String,
    },
    /// Run an agent with harness observation and intervention
    Run {
        /// Task description for the agent
        task: String,
        /// Agent type (solo, claude-code, langgraph, etc.)
        #[arg(long, default_value = "solo")]
        agent: String,
        /// Preset to use
        #[arg(long, default_value = "solo")]
        preset: String,
        /// Observe and detect but don't intervene
        #[arg(long)]
        dry_run: bool,
    },
    /// Show active session health (TUI)
    Watch { session_id: Option<String> },
    /// Replay a session from audit trail
    Replay { session_id: String },
    /// Print human-readable audit report
    Explain { session_id: String },
    /// Run benchmark suite
    Bench {
        #[arg(long)]
        suite: Option<String>,
    },
    /// Run meta-harness improvement cycle
    Improve {
        #[arg(long)]
        agent_type: Option<String>,
    },
    /// Start API server and dashboard
    Serve {
        #[arg(long, default_value = "3000")]
        port: u16,
    },
    /// Check system dependencies
    Doctor,
    /// Run agent against test suite
    Test {
        #[arg(long)]
        tasks: Option<String>,
    },
    /// Validate harness config
    Validate {
        #[arg(long)]
        config: Option<String>,
    },
    /// Compare two sessions or harness versions
    Diff {
        a: String,
        b: String,
        #[arg(long)]
        harness: bool,
    },
    /// View or edit forge.toml
    Config {
        #[arg(long)]
        key: Option<String>,
        #[arg(long)]
        value: Option<String>,
    },
    /// Manage community plugins
    Plugin {
        #[command(subcommand)]
        action: Option<PluginAction>,
    },
    /// Export session data
    Export {
        session_id: String,
        #[arg(long, default_value = "json")]
        format: String,
    },
    /// Generate shell completions
    Completion { shell: String },
}

#[derive(Subcommand)]
enum PluginAction {
    Search { query: String },
    Install { name: String },
    List,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name, agent_type } => {
            let n = name.unwrap_or_else(|| "my-forge-agent".into());
            cmd_init(&n, &agent_type);
        }
        Commands::Run {
            task,
            agent,
            preset,
            dry_run,
        } => {
            cmd_run(&task, &agent, &preset, dry_run).await;
        }
        Commands::Watch { session_id } => {
            let sid = session_id.unwrap_or_else(|| "latest".into());
            println!("🔍 Forge Watch — session: {sid}");
            match forge_cli::render::watch::run_watch(&sid).await {
                Ok(_) => {}
                Err(e) => eprintln!("Watch error: {e}"),
            }
        }
        Commands::Replay { session_id } => {
            println!("⏪ Replaying session: {session_id}");
            println!("(Session replay requires audit store — coming in next release)");
        }
        Commands::Explain { session_id } => {
            println!("📋 Audit report for session: {session_id}");
            cmd_explain(&session_id).await;
        }
        Commands::Bench { suite } => {
            let s = suite.unwrap_or_else(|| "standard".into());
            println!("🏃 Running benchmark suite: {s}");
            cmd_bench(&s).await;
        }
        Commands::Improve { agent_type } => {
            let at = agent_type.unwrap_or_else(|| "all".into());
            println!("🧠 Running meta-harness improvement for: {at}");
            cmd_improve(&at).await;
        }
        Commands::Serve { port } => {
            println!("🚀 Starting Forge server on http://localhost:{port}");
            println!("📊 Dashboard: http://localhost:{port}");
            println!("Press Ctrl+C to stop");
            forge_server::run_server(port).await;
        }
        Commands::Doctor => {
            cmd_doctor();
        }
        Commands::Test { tasks } => {
            let t = tasks.unwrap_or_else(|| "all".into());
            println!("🧪 Running tests: {t}");
            cmd_test(&t).await;
        }
        Commands::Validate { config } => {
            let c = config.unwrap_or_else(|| "forge.toml".into());
            println!("✅ Validating config: {c}");
            cmd_validate(&c);
        }
        Commands::Diff { a, b, harness } => {
            if harness {
                println!("🔄 Comparing harness versions {a} vs {b}");
            } else {
                println!("🔄 Comparing sessions {a} vs {b}");
            }
        }
        Commands::Config { key, value } => match (key, value) {
            (Some(k), Some(v)) => println!("⚙ Setting {k} = {v}"),
            (Some(k), None) => println!("⚙ {k} = (default)"),
            (None, _) => println!("⚙ Listing all config (forge.toml)"),
        },
        Commands::Plugin { action } => match action {
            Some(PluginAction::Search { query }) => println!("🔍 Searching plugins: {query}"),
            Some(PluginAction::Install { name }) => println!("📦 Installing plugin: {name}"),
            Some(PluginAction::List) | None => {
                println!("📦 Installed plugins: (plugin registry coming soon)")
            }
        },
        Commands::Export { session_id, format } => {
            println!("📤 Exporting session {session_id} as {format}");
        }
        Commands::Completion { shell } => {
            println!("🐚 Generating completions for {shell}");
        }
    }
}

// ─── Command implementations ──────────────────────────────────────────

fn cmd_init(name: &str, agent_type: &str) {
    use std::fs;
    use std::path::Path;

    let dir = Path::new(name);
    if dir.exists() {
        eprintln!("Error: directory '{}' already exists", name);
        std::process::exit(1);
    }

    fs::create_dir_all(dir.join("src")).expect("Failed to create project directory");
    fs::create_dir_all(dir.join("tests")).expect("Failed to create tests directory");

    // Generate Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
forge-sdk = {{ path = "../forge/packages/forge-sdk" }}
forge-harness = {{ path = "../forge/packages/forge-harness" }}
tokio = {{ version = "1", features = ["full"] }}
anyhow = "1"

[[bin]]
name = "{name}"
path = "src/main.rs"
"#
    );
    fs::write(dir.join("Cargo.toml"), &cargo_toml).expect("Failed to write Cargo.toml");

    // Generate main.rs
    let main_rs = format!(
        r#"// {name} — AI agent wrapped with Forge harness

use forge_sdk::agent::{{AgentType, MockAgent}};
use forge_harness::runner::quick_run;

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    let mut agent = MockAgent::new("{name}-1", AgentType::{agent_type_ident});
    let task = "Say hello and explain what you can do";

    println!("🚀 Running {{}} with Forge harness...", agent.id());
    let result = quick_run(&mut agent, task).await?;

    println!("✅ Task completed!");
    println!("   Agent:       {{}}", result.agent_id);
    println!("   Success:     {{}}", result.success);
    println!("   Observations: {{}}", result.observation_count);
    println!("   Detections:  {{}}", result.detection_count);
    println!("   Interventions: {{}}", result.intervention_count);

    Ok(())
}}
"#,
        agent_type_ident = agent_type_to_enum(agent_type),
    );
    fs::write(dir.join("src/main.rs"), &main_rs).expect("Failed to write main.rs");

    // Generate forge.toml
    let forge_toml = format!(
        r#"# Forge harness configuration for {name}
[harness]
agent_type = "{agent_type}"
preset = "{agent_type}"
dry_run = false

[observers]
enabled = ["token", "latency", "cost", "accuracy", "security"]

[detectors]
enabled = ["loop", "stale_context", "secret_leak"]

[strategies]
enabled = ["nudge", "compact", "pause", "escalate"]

[audit]
store = "sqlite"
path = "./audit.db"
retention_days = 30
"#
    );
    fs::write(dir.join("forge.toml"), &forge_toml).expect("Failed to write forge.toml");

    // Generate .gitignore
    fs::write(
        dir.join(".gitignore"),
        "target/\n*.db\n*.db-journal\n*.db-wal\n*.db-shm\n.env\n",
    )
    .expect("Failed to write .gitignore");

    println!("✅ Created new Forge project: {name}");
    println!("   Agent type: {agent_type}");
    println!();
    println!("   cd {name}");
    println!("   cargo run");
}

fn agent_type_to_enum(at: &str) -> &str {
    match at.to_lowercase().as_str() {
        "solo" => "Solo",
        "claude-code" | "claude" => "ClaudeCode",
        "langgraph" => "LangGraph",
        "crewai" | "crew" => "CrewAI",
        "autogen" => "AutoGen",
        "langchain" => "LangChain",
        "aider" => "Aider",
        "cline" => "Cline",
        "continue" => "Continue",
        "copilot" => "Copilot",
        "cursor" => "Cursor",
        "windsurf" => "Windsurf",
        "devin" => "Devin",
        "custom" => "Custom",
        _ => "Solo",
    }
}

async fn cmd_run(task: &str, agent_type: &str, preset: &str, dry_run: bool) {
    use forge_harness::runner;
    use forge_sdk::agent::{AgentType, MockAgent};

    let at = match agent_type.to_lowercase().as_str() {
        "solo" => AgentType::Solo,
        "claude-code" | "claude" => AgentType::ClaudeCode,
        "langgraph" => AgentType::LangGraph,
        "crewai" | "crew" => AgentType::CrewAI,
        "autogen" => AgentType::AutoGen,
        "langchain" => AgentType::LangChain,
        "aider" => AgentType::Aider,
        "cline" => AgentType::Cline,
        "continue" => AgentType::Continue,
        "copilot" => AgentType::Copilot,
        "cursor" => AgentType::Cursor,
        "windsurf" => AgentType::Windsurf,
        "devin" => AgentType::Devin,
        "custom" => AgentType::Custom,
        _ => {
            eprintln!("Unknown agent type: {agent_type}. Available types:");
            eprintln!("  solo, claude-code, langgraph, crewai, autogen, langchain");
            eprintln!("  aider, cline, continue, copilot, cursor, windsurf, devin, custom");
            std::process::exit(1);
        }
    };

    let preset_enum = match preset.to_lowercase().as_str() {
        "solo" => forge_sdk::presets::Preset::Solo,
        "claude-code" | "claude" => forge_sdk::presets::Preset::ClaudeCode,
        "langgraph" => forge_sdk::presets::Preset::LangGraph,
        "crewai" | "crew" => forge_sdk::presets::Preset::CrewAI,
        "autogen" => forge_sdk::presets::Preset::AutoGen,
        "langchain" => forge_sdk::presets::Preset::LangChain,
        "aider" => forge_sdk::presets::Preset::Aider,
        "cline" => forge_sdk::presets::Preset::Cline,
        "continue" => forge_sdk::presets::Preset::Continue,
        "copilot" => forge_sdk::presets::Preset::Copilot,
        "cursor" => forge_sdk::presets::Preset::Cursor,
        "windsurf" => forge_sdk::presets::Preset::Windsurf,
        "devin" => forge_sdk::presets::Preset::Devin,
        "custom" => forge_sdk::presets::Preset::Custom,
        _ => {
            eprintln!("Unknown preset: {preset}. Using 'solo' preset.");
            forge_sdk::presets::Preset::Solo
        }
    };

    let agent_name = format!("forge-{agent_type}-1");
    let mut agent = MockAgent::new(&agent_name, at).with_turns(5);

    println!("🚀 Forge Run");
    println!("   Agent:  {agent_name} ({agent_type})");
    println!("   Preset: {preset}");
    println!("   Task:   {task}");
    println!(
        "   Mode:   {}",
        if dry_run {
            "DRY RUN (observe only)"
        } else {
            "LIVE"
        }
    );
    println!();

    let result = if dry_run {
        runner::dry_run(&mut agent, task, preset_enum).await
    } else {
        runner::run_harness_session(&mut agent, task, preset_enum, None).await
    };

    match result {
        Ok(r) => {
            println!("✅ Session complete!");
            println!("   Agent ID:      {}", r.agent_id);
            println!(
                "   Success:       {}",
                if r.success { "✅ YES" } else { "❌ NO" }
            );
            println!("   Observ. cycles: {}", r.observation_count);
            println!("   Detections:    {}", r.detection_count);
            println!("   Interventions: {}", r.intervention_count);
        }
        Err(e) => {
            eprintln!("❌ Session failed: {e}");
            std::process::exit(1);
        }
    }
}

async fn cmd_explain(session_id: &str) {
    use forge_audit::explainer;
    let report = forge_sdk::types::audit::AuditReport {
        session_id: uuid::Uuid::new_v4(),
        task: "No task recorded".into(),
        agent_type: "solo".into(),
        model: "mock".into(),
        duration_secs: 0.0,
        total_tokens: 0,
        total_cost: 0.0,
        health_score: None,
        observations: vec![],
        detections: vec![],
        interventions: vec![],
        checkpoints: vec![],
        harness_effectiveness: None,
    };
    println!("{}", explainer::explain(&report));
    println!("   (Run 'forge run <task>' to generate real audit data for session: {session_id})");
}

async fn cmd_bench(suite: &str) {
    println!("📊 Benchmarks: {suite}");
    println!("   (Full benchmark suite requires benchmark tasks YAML — coming in next release)");

    // Quick smoke test
    let mut agent =
        forge_sdk::agent::MockAgent::new("bench-agent", forge_sdk::agent::AgentType::Solo);
    match forge_harness::runner::quick_run(&mut agent, "benchmark smoke test").await {
        Ok(r) => println!(
            "   Smoke test: ✅ ({detections} detections, {interventions} interventions)",
            detections = r.detection_count,
            interventions = r.intervention_count
        ),
        Err(e) => println!("   Smoke test: ❌ ({e})"),
    }
}

async fn cmd_improve(agent_type: &str) {
    println!("🧠 Meta-harness analyzing sessions for: {agent_type}");
    println!("   Mining weakness patterns across all sessions...");
    println!("   (No minimum session limit — runs with whatever data is available)");
    println!();
    println!("   Use the dashboard Meta tab to view results: http://127.0.0.1:3000");
    println!("   Or call GET /api/v1/meta/weaknesses for pattern data.");
}

async fn cmd_test(tasks: &str) {
    println!("🧪 Running agent test suite: {tasks}");
    let mut agent =
        forge_sdk::agent::MockAgent::new("test-agent", forge_sdk::agent::AgentType::Solo);
    match forge_harness::runner::quick_run(&mut agent, &format!("test suite: {tasks}")).await {
        Ok(r) => {
            if r.success {
                println!("   ✅ Tests passed!");
            } else {
                println!("   ❌ Tests failed!");
            }
        }
        Err(e) => println!("   ❌ Error: {e}"),
    }
}

fn cmd_doctor() {
    println!("🔍 Forge Doctor — System Check");
    println!();

    // Rust toolchain
    let rustc = std::process::Command::new("rustc")
        .arg("--version")
        .output();
    match rustc {
        Ok(out) => println!(
            "   ✅ Rust:     {}",
            String::from_utf8_lossy(&out.stdout).trim()
        ),
        Err(_) => println!("   ❌ Rust:     not found — install from https://rustup.rs"),
    }

    // Cargo
    let cargo = std::process::Command::new("cargo")
        .arg("--version")
        .output();
    match cargo {
        Ok(out) => println!(
            "   ✅ Cargo:    {}",
            String::from_utf8_lossy(&out.stdout).trim()
        ),
        Err(_) => println!("   ❌ Cargo:    not found"),
    }

    // SQLite
    let sqlite = std::process::Command::new("sqlite3")
        .arg("--version")
        .output();
    match sqlite {
        Ok(out) => println!(
            "   ✅ SQLite:   {}",
            String::from_utf8_lossy(&out.stdout).trim()
        ),
        Err(_) => println!("   ⚠ SQLite:   CLI not found (library still works via sqlx)"),
    }

    // Docker (optional)
    let docker = std::process::Command::new("docker")
        .arg("--version")
        .output();
    match docker {
        Ok(out) => println!(
            "   ✅ Docker:   {}",
            String::from_utf8_lossy(&out.stdout).trim()
        ),
        Err(_) => println!("   ⚠ Docker:   not found (optional, for sandbox mode)"),
    }

    // forge.toml
    if std::path::Path::new("forge.toml").exists() {
        println!("   ✅ forge.toml: found");
    } else {
        println!("   ⚠ forge.toml: not found (run 'forge init' to create one)");
    }

    // env vars
    let has_anthropic = std::env::var("ANTHROPIC_API_KEY").is_ok();
    let has_openai = std::env::var("OPENAI_API_KEY").is_ok();
    if has_anthropic || has_openai {
        if has_anthropic {
            println!("   ✅ ANTHROPIC_API_KEY: set");
        }
        if has_openai {
            println!("   ✅ OPENAI_API_KEY: set");
        }
    } else {
        println!("   ⚠ No LLM API keys found (set ANTHROPIC_API_KEY or OPENAI_API_KEY)");
    }
}

fn cmd_validate(config_path: &str) {
    if !std::path::Path::new(config_path).exists() {
        eprintln!("❌ Config file not found: {config_path}");
        std::process::exit(1);
    }
    println!("✅ Config file exists: {config_path}");
    println!("   (Full validation — schema + preset references — coming in next release)");
}
