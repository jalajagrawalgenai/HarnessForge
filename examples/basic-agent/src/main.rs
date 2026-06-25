//! Forge Harness — Basic Agent Example
//!
//! This example demonstrates:
//! 1. Creating a MockAgent that simulates an AI agent
//! 2. Running it through the Forge harness with observation + detection + intervention
//! 3. Seeing the results: what was detected, what interventions were applied
//!
//! Run with: cargo run -p forge-example-basic-agent

use forge_sdk::agent::{AgentType, MockAgent};
use forge_sdk::presets::Preset;
use forge_harness::runner;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           Forge Harness — Basic Agent Example                ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // ── 1. Create a mock agent ──────────────────────────────────────
    // In production, you'd implement AgentAdapter for your real agent
    // (Claude API, LangGraph, CrewAI, etc.)
    let mut agent = MockAgent::new("example-agent", AgentType::Solo)
        .with_turns(4)     // Simulate 4 thinking/tool turns
        .with_success(true);

    let task = "Write a function to validate email addresses";

    // ── 2. Run through the harness ───────────────────────────────────
    println!("🚀 Running agent with Solo preset...");
    println!("   Task:  {}", task);
    println!("   Agent: {}", agent.id);
    println!();

    let result = runner::run_harness_session(
        &mut agent,
        task,
        Preset::Solo,  // Full observer/detector/strategy suite for solo agents
        None,          // No audit store (use Some(store) for persistence)
    ).await?;

    // ── 3. Display results ──────────────────────────────────────────
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    SESSION RESULTS                            ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ Agent ID:       {:<44} ║", result.agent_id);
    println!("║ Success:        {:<44} ║", if result.success { "✅ YES" } else { "❌ NO" });
    println!("║ Observation cycles: {:<40} ║", result.observation_count);
    println!("║ Detections:     {:<44} ║", result.detection_count);
    println!("║ Interventions:  {:<44} ║", result.intervention_count);
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // ── 4. What happened? ───────────────────────────────────────────
    if result.detection_count > 0 {
        println!("🔍 The harness detected issues and intervened!");
        println!("   This is what Forge is for — catching problems before they escalate.");
    } else {
        println!("✅ No issues detected — agent ran cleanly.");
    }

    println!();
    println!("── Try it yourself ──────────────────────────────────────────");
    println!("  forge init --name my-agent --agent-type solo");
    println!("  cd my-agent && cargo run");
    println!("  forge run \"Your task here\"");
    println!();
    println!("  Or use the SDK directly:");
    println!("  let harness = Harness::builder().preset(Preset::Solo).build();");
    println!("  let result = runner::quick_run(&mut agent, \"task\").await?;");

    Ok(())
}
