// forge-audit/src/explainer.rs — Human-readable audit report generator

use forge_sdk::types::audit::AuditReport;

/// Generates human-readable text from an audit report
pub fn explain(report: &AuditReport) -> String {
    let mut out = String::new();

    out.push_str("══════════════════════════════════════════════════\n");
    out.push_str("FORGE AUDIT REPORT\n");
    out.push_str(&format!("Session: {}\n", report.session_id));
    out.push_str(&format!("Task: \"{}\"\n", report.task));
    out.push_str(&format!("Agent: {} ({})\n", report.agent_type, report.model));
    out.push_str(&format!("Duration: {:.1} minutes\n", report.duration_secs / 60.0));
    if let Some(eff) = report.harness_effectiveness {
        out.push_str(&format!("Harness Effectiveness: {:.2}\n", eff));
    }
    out.push_str("══════════════════════════════════════════════════\n\n");

    // Observations
    out.push_str(&format!(
        "OBSERVATIONS ({} events):\n",
        report.observations.len()
    ));
    for obs in &report.observations {
        out.push_str(&format!(
            "  {}: {} events tracked\n",
            obs.dimension, obs.event_count
        ));
    }
    out.push('\n');

    // Detections
    out.push_str(&format!(
        "DETECTIONS ({} issues found):\n",
        report.detections.len()
    ));
    if report.detections.is_empty() {
        out.push_str("  ✅ No issues detected.\n");
    }
    for det in &report.detections {
        out.push_str(&format!(
            "  [Turn {}] {} (confidence: {:.0}%)\n",
            det.turn,
            det.detector,
            det.confidence * 100.0
        ));
        out.push_str(&format!("    → Category: {}\n", det.category));
        out.push_str(&format!("    → Severity: {}\n", det.severity));
    }
    out.push('\n');

    // Interventions
    out.push_str(&format!(
        "INTERVENTIONS ({} applied):\n",
        report.interventions.len()
    ));
    for int in &report.interventions {
        out.push_str(&format!(
            "  [Turn {}] {} applied\n",
            int.turn, int.strategy
        ));
        out.push_str(&format!("    → Outcome: {}\n", int.outcome));
        if let Some(impact) = &int.impact {
            out.push_str(&format!("    → Impact: {}\n", impact));
        }
    }
    out.push('\n');

    // Checkpoints
    out.push_str(&format!(
        "CHECKPOINTS ({} saved):\n",
        report.checkpoints.len()
    ));
    for cp in &report.checkpoints {
        out.push_str(&format!("  [Turn {}] {}\n", cp.turn, cp.reason));
    }
    out.push_str("\n══════════════════════════════════════════════════\n");

    // Summary
    if report.detections.is_empty() {
        out.push_str("Session completed with no issues detected.\n");
    } else if let Some(eff) = report.harness_effectiveness {
        if eff > 0.8 {
            out.push_str("Harness effectively caught and resolved issues.\n");
        } else {
            out.push_str("Harness detected issues but improvements possible.\n");
        }
    }

    out
}
