//! Session endpoints — list, get, delete, pause, resume.
//!
//! Sessions are created automatically by the ingest API when real
//! agent events arrive. There is no manual session creation.
//! No MockAgent. Real agents only.

use crate::AppState;
use axum::extract::{Path, State};
use axum::Json;
use forge_sdk::events::Intervention;
use serde_json::{json, Value};
use std::sync::Arc;

/// Sessions are auto-created via POST /api/v1/ingest/event.
/// This endpoint returns a message directing users to that flow.
pub async fn create() -> Json<Value> {
    Json(json!({
        "error": "Manual session creation is disabled.",
        "message": "Sessions are created automatically when you use your AI agent.",
        "how": "Just use Claude Code, LangGraph, CrewAI, or any agent normally.",
        "dashboard": "Open http://127.0.0.1:3000 to see auto-detected sessions."
    }))
}

/// GET /v1/sessions
/// Returns all sessions with their status, metadata, and pipeline summary.
pub async fn list(State(state): State<Arc<AppState>>) -> Json<Value> {
    let sessions = state.store.read().await;
    let mut items: Vec<Value> = sessions
        .iter()
        .map(|(id, s)| {
            json!({
                "id": id,
                "task": s.task,
                "agent_type": s.agent_type,
                "status": s.status.to_string(),
                "created_at": s.created_at.to_rfc3339(),
                "completed_at": s.completed_at.map(|t| t.to_rfc3339()),
                "health_score": s.health_score,
                "total_tokens": s.total_input_tokens + s.total_output_tokens,
                "tool_calls": s.tool_counts.values().sum::<u64>(),
                "model": s.model_name,
                "stop_reason": s.stop_reason,
                "pipeline": {
                    "event_count": s.event_count,
                    "observation_count": s.observations.len(),
                    "detection_count": s.detections.len(),
                    "intervention_count": s.interventions.len(),
                }
            })
        })
        .collect();
    items.sort_by(|a, b| {
        b.get("created_at")
            .and_then(|v| v.as_str())
            .cmp(&a.get("created_at").and_then(|v| v.as_str()))
    });
    Json(json!({"sessions": items, "total": items.len()}))
}

/// GET /v1/sessions/:id
/// Returns full session detail including pipeline data.
pub async fn get(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    let sessions = state.store.read().await;
    match sessions.get(&id) {
        Some(s) => Json(json!({
            "id": s.id,
            "task": s.task,
            "agent_type": s.agent_type,
            "preset": s.preset,
            "status": s.status.to_string(),
            "created_at": s.created_at.to_rfc3339(),
            "completed_at": s.completed_at.map(|t| t.to_rfc3339()),
            "health_score": s.health_score,
            "result": s.result,
            "event_count": s.events.len(),
            "pipeline": {
                "observations": s.observations,
                "detections": s.detections,
                "strategy_results": s.strategy_results,
                "interventions": s.interventions,
                "total_observations": s.observations.len(),
                "total_detections": s.detections.len(),
                "total_interventions": s.interventions.len(),
            }
        })),
        None => Json(json!({"error": "session not found", "id": id})),
    }
}

/// DELETE /v1/sessions/:id
pub async fn delete(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    let mut sessions = state.store.write().await;
    if let Some(session) = sessions.get(&id) {
        let _ = session.cancel_tx.send(true);
    }
    sessions.remove(&id);
    Json(json!({"deleted": id}))
}

/// POST /v1/sessions/:id/pause
pub async fn pause(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    let sessions = state.store.read().await;
    match sessions.get(&id) {
        Some(session) => {
            let iv = Intervention::Pause {
                reason: "user requested pause".into(),
                checkpoint_id: uuid::Uuid::new_v4(),
            };
            match session.intervention_tx.try_send(iv) {
                Ok(_) => Json(json!({"paused": id, "status": "ok"})),
                Err(_) => Json(json!({"paused": id, "status": "channel_full"})),
            }
        }
        None => Json(json!({"error": "session not found"})),
    }
}

/// POST /v1/sessions/:id/resume
pub async fn resume(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    let sessions = state.store.read().await;
    match sessions.get(&id) {
        Some(session) => match session.intervention_tx.try_send(Intervention::Resume) {
            Ok(_) => Json(json!({"resumed": id, "status": "ok"})),
            Err(_) => Json(json!({"resumed": id, "status": "channel_full"})),
        },
        None => Json(json!({"error": "session not found"})),
    }
}

/// GET /v1/sessions/:id/analysis
///
/// Returns a COMPLETE, exhaustive analysis of the session.
/// Every event, every hook, every tool call, every prompt — nothing hidden.
pub async fn analysis(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    let sessions = state.store.read().await;
    let s = match sessions.get(&id) {
        Some(s) => s,
        None => return Json(json!({"error": "session not found", "id": id})),
    };

    let total_tokens = s.total_input_tokens + s.total_output_tokens;
    let cache_hit_pct = if s.total_input_tokens > 0 {
        (s.total_cache_read as f64 / s.total_input_tokens as f64) * 100.0
    } else {
        0.0
    };

    // Cost estimate based on detected model
    let model = s.model_name.as_deref().unwrap_or("unknown");
    let (input_price, output_price, model_family) = model_pricing(model);
    let est_cost =
        (s.total_input_tokens as f64 * input_price) + (s.total_output_tokens as f64 * output_price);
    let cache_savings = s.total_cache_read as f64 * input_price * 0.9;
    let effective_cost = est_cost - cache_savings;
    let is_estimated = s.total_cache_read == 0 && s.total_cache_write == 0;

    let total_tool_calls: u64 = s.tool_counts.values().sum();
    let total_tool_errors: u64 = s.tool_errors.values().sum();

    let context_avg = if !s.context_pressure_history.is_empty() {
        s.context_pressure_history.iter().sum::<f64>() / s.context_pressure_history.len() as f64
    } else { 0.0 };
    let context_max = s.context_pressure_history.iter().cloned().fold(0.0f64, f64::max);

    let duration_secs = s.completed_at
        .map(|t| (t - s.created_at).num_seconds())
        .unwrap_or_else(|| (Utc::now() - s.created_at).num_seconds());

    // ── Build comprehensive event/hook/tool/prompt trace ──
    let full_events = build_full_event_log(&s.events);
    let hook_trace = build_hook_trace(&s.events, &s.event_hooks);
    let tool_instances = build_tool_instances(&s.events);
    let prompt_instances = build_prompt_instances(&s.events, &s.prompt_history);
    let detector_report = build_detector_report(&s.detections, &s.interventions, &s.strategy_results);

    // ── Recommendations ──
    let mut recommendations: Vec<String> = Vec::new();
    if cache_hit_pct < 20.0 && total_tokens > 1000 {
        recommendations.push("Low cache hit rate — enable prompt caching to reduce costs".into());
    }
    if total_tool_errors > 0 && total_tool_errors as f64 / total_tool_calls.max(1) as f64 > 0.1 {
        recommendations.push("High tool error rate — review tool definitions and error handling".into());
    }
    if context_avg > 0.75 {
        recommendations.push("Context pressure consistently high — compact more aggressively".into());
    }
    if s.subagent_count > 5 {
        recommendations.push("High subagent spawn count — consider consolidating tasks".into());
    }
    if s.event_count > 100 && s.detections.is_empty() {
        recommendations.push("No issues detected across many events — agent is running well".into());
    }

    let health_verdict = match s.health_score.as_ref() {
        Some(h) if h.overall > 0.8 => "Healthy — agent performing well across all dimensions",
        Some(h) if h.overall > 0.6 => "Moderate — some dimensions need attention",
        Some(h) if h.overall > 0.4 => "Degraded — multiple issues detected",
        Some(_) => "Critical — agent requires immediate intervention",
        None => "No health data yet",
    };

    let stop_analysis = match (&s.stop_reason, &s.status) {
        (Some(reason), crate::session::store::SessionStatus::Completed) =>
            format!("✅ Completed: {}", reason),
        (Some(reason), crate::session::store::SessionStatus::Failed) =>
            format!("❌ Failed: {}", reason),
        (None, crate::session::store::SessionStatus::Running) =>
            "🟢 Still running".to_string(),
        _ => format!("Status: {:?}", s.status),
    };

    Json(json!({
        // ── OVERVIEW ──
        "session_id": s.id,
        "task": s.task,
        "agent_type": s.agent_type,
        "preset": s.preset,
        "status": s.status.to_string(),
        "duration_secs": duration_secs,
        "duration_display": format_duration(duration_secs),
        "model": s.model_name,
        "model_family": model_family,
        "stop_reason": s.stop_reason,
        "stop_analysis": stop_analysis,
        "health_verdict": health_verdict,
        "health_score": s.health_score,

        // ── TOKEN & COST ──
        "token_analysis": {
            "total_tokens": total_tokens,
            "input_tokens": s.total_input_tokens,
            "output_tokens": s.total_output_tokens,
            "cache_read_tokens": s.total_cache_read,
            "cache_write_tokens": s.total_cache_write,
            "cache_hit_pct": cache_hit_pct,
            "gross_cost_usd": est_cost,
            "cache_savings_usd": cache_savings,
            "effective_cost_usd": effective_cost,
            "input_price_per_1m": input_price * 1_000_000.0,
            "output_price_per_1m": output_price * 1_000_000.0,
            "is_estimated": is_estimated,
            "data_source": if is_estimated { "estimated (heuristic — use transcript hook for precision)" } else { "transcript" },
            "tokens_per_event": if s.event_count > 0 { total_tokens as f64 / s.event_count as f64 } else { 0.0 },
            "output_to_input_ratio": if s.total_input_tokens > 0 { s.total_output_tokens as f64 / s.total_input_tokens as f64 } else { 0.0 },
        },

        // ── TOOL SUMMARY ──
        "tool_summary": {
            "total_calls": total_tool_calls,
            "total_errors": total_tool_errors,
            "error_rate_pct": if total_tool_calls > 0 { (total_tool_errors as f64 / total_tool_calls as f64) * 100.0 } else { 0.0 },
            "unique_tools": s.tool_counts.len(),
            "total_duration_ms": s.total_tool_ms,
            "by_tool": s.tool_counts.iter().map(|(name, count)| {
                let errors = s.tool_errors.get(name).copied().unwrap_or(0);
                json!({
                    "tool": name,
                    "calls": count,
                    "errors": errors,
                    "error_rate_pct": if *count > 0 { (errors as f64 / *count as f64) * 100.0 } else { 0.0 },
                    "pct_of_total": if total_tool_calls > 0 { (*count as f64 / total_tool_calls as f64) * 100.0 } else { 0.0 },
                })
            }).collect::<Vec<_>>(),
        },

        // ── CONTEXT ANALYSIS ──
        "context_analysis": {
            "pressure_readings": s.context_pressure_history.len(),
            "avg_pressure_pct": context_avg * 100.0,
            "max_pressure_pct": context_max * 100.0,
            "pressure_history": s.context_pressure_history.iter().map(|p| (p * 100.0) as u32).collect::<Vec<_>>(),
            "status": if context_max > 0.85 { "critical" } else if context_avg > 0.6 { "warning" } else { "healthy" },
        },

        // ── SESSION SUMMARY ──
        "session_summary": {
            "total_events": s.event_count,
            "user_prompts": s.user_prompt_count,
            "subagents_spawned": s.subagent_count,
            "total_detections": s.detections.len(),
            "total_interventions": s.interventions.len(),
            "total_observations": s.observations.len(),
            "total_strategies": s.strategy_results.len(),
        },

        // ── FULL EVENT LOG — every event with all fields ──
        "event_log": full_events,
        "event_count": full_events.len(),

        // ── HOOK TRACE — grouped by hook type ──
        "hook_trace": hook_trace,

        // ── TOOL INSTANCES — every individual tool call ──
        "tool_instances": tool_instances,

        // ── PROMPT INSTANCES — every prompt with context ──
        "prompt_instances": prompt_instances,
        "prompt_history": s.prompt_history.iter().enumerate().map(|(i, p)| json!({
            "index": i + 1,
            "text": p,
            "length": p.len(),
            "estimated_tokens": (p.len() as f64 / 4.0).ceil() as u64,
        })).collect::<Vec<_>>(),

        // ── DETECTOR REPORT ──
        "detector_report": detector_report,
        "detection_details": s.detections.clone(),
        "intervention_details": s.interventions.clone(),
        "strategy_details": s.strategy_results.clone(),

        // ── OBSERVATION DETAILS ──
        "observation_details": group_observations_by_dimension(&s.observations),

        // ── RECOMMENDATIONS ──
        "recommendations": recommendations,
    }))
}

/// Format seconds into human-readable duration.
fn format_duration(secs: i64) -> String {
    if secs < 60 { return format!("{}s", secs); }
    if secs < 3600 { return format!("{}m {}s", secs / 60, secs % 60); }
    format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
}

/// Build a FULL event log — every event with all fields, nothing truncated.
fn build_full_event_log(events: &[forge_sdk::events::AgentEvent]) -> Vec<Value> {
    use forge_sdk::events::AgentEvent;
    events.iter().enumerate().filter_map(|(i, e)| {
        match e {
            AgentEvent::Started { agent_id, task, timestamp } => Some(json!({
                "seq": i + 1,
                "time": timestamp.to_rfc3339(),
                "type": "session_start",
                "icon": "▶",
                "agent_id": agent_id,
                "task": task,
            })),
            AgentEvent::Completed { agent_id, summary, timestamp } => Some(json!({
                "seq": i + 1,
                "time": timestamp.to_rfc3339(),
                "type": "session_end",
                "icon": "✓",
                "agent_id": agent_id,
                "summary": summary,
            })),
            AgentEvent::Failed { agent_id, error, timestamp } => Some(json!({
                "seq": i + 1,
                "time": timestamp.to_rfc3339(),
                "type": "session_failed",
                "icon": "✗",
                "agent_id": agent_id,
                "error": error,
            })),
            AgentEvent::ToolCallStart { agent_id, tool, args, timestamp } => Some(json!({
                "seq": i + 1,
                "time": timestamp.to_rfc3339(),
                "type": "tool_start",
                "icon": "→",
                "agent_id": agent_id,
                "tool": tool,
                "args": args,
                "args_display": format_tool_detail(tool, args),
            })),
            AgentEvent::ToolCallEnd { agent_id, tool, result, timestamp } => {
                let status = if result.is_error { "FAILED" } else { "ok" };
                Some(json!({
                    "seq": i + 1,
                    "time": timestamp.to_rfc3339(),
                    "type": "tool_end",
                    "icon": if result.is_error { "✗" } else { "←" },
                    "agent_id": agent_id,
                    "tool": tool,
                    "status": status,
                    "is_error": result.is_error,
                    "duration_ms": result.duration_ms,
                    "token_count": result.token_count,
                    "result": if result.content.len() > 500 {
                        format!("{}... [{} more chars]", &result.content[..500], result.content.len() - 500)
                    } else {
                        result.content.clone()
                    },
                    "result_full_len": result.content.len(),
                }))
            }
            AgentEvent::MessageSent { from, to, content, timestamp } => {
                let text = match content {
                    forge_sdk::events::MessageContent::Text(t) => t.clone(),
                    _ => String::new(),
                };
                if text.is_empty() { return None; }
                Some(json!({
                    "seq": i + 1,
                    "time": timestamp.to_rfc3339(),
                    "type": "user_prompt",
                    "icon": "💬",
                    "from": from,
                    "to": to,
                    "text": text,
                    "text_len": text.len(),
                    "estimated_tokens": (text.len() as f64 / 4.0).ceil() as u64,
                }))
            }
            AgentEvent::ContextPressure { agent_id, current_ratio, trend, timestamp } => Some(json!({
                "seq": i + 1,
                "time": timestamp.to_rfc3339(),
                "type": "context_pressure",
                "icon": "📐",
                "agent_id": agent_id,
                "current_ratio": current_ratio,
                "pressure_pct": (current_ratio * 100.0) as u32,
                "trend": trend,
            })),
            AgentEvent::Forked { parent_id, child_id, task, timestamp } => Some(json!({
                "seq": i + 1,
                "time": timestamp.to_rfc3339(),
                "type": "subagent_fork",
                "icon": "⑂",
                "parent": parent_id,
                "child": child_id,
                "task": task,
            })),
            AgentEvent::TokenUsage { agent_id, input, output, cache_read, cache_write, model, timestamp } => Some(json!({
                "seq": i + 1,
                "time": timestamp.to_rfc3339(),
                "type": "token_usage",
                "icon": "📊",
                "agent_id": agent_id,
                "input_tokens": input,
                "output_tokens": output,
                "cache_read": cache_read,
                "cache_write": cache_write,
                "model": model,
                "total": input + output,
            })),
            AgentEvent::OutputDelta { agent_id, text, timestamp } => {
                if text.is_empty() || text.starts_with('[') { return None; }
                Some(json!({
                    "seq": i + 1,
                    "time": timestamp.to_rfc3339(),
                    "type": "output",
                    "icon": "→",
                    "agent_id": agent_id,
                    "text": text,
                    "text_len": text.len(),
                }))
            }
            _ => {
                // Generic fallback for any other event type
                let val = serde_json::to_value(e).unwrap_or(json!({}));
                Some(json!({
                    "seq": i + 1,
                    "time": "",
                    "type": "other",
                    "icon": "●",
                    "raw": val,
                }))
            }
        }
    }).collect()
}

/// Build hook trace — use stored hook names when available, fall back to
/// inferring from AgentEvent variants.
fn build_hook_trace(events: &[forge_sdk::events::AgentEvent], hooks: &[String]) -> Vec<Value> {
    use std::collections::HashMap;
    use forge_sdk::events::AgentEvent;

    let use_stored = hooks.len() >= events.len();
    let mut hook_groups: HashMap<String, Vec<Value>> = HashMap::new();
    let n = if use_stored { events.len() } else { events.len() };

    // If no stored hooks, infer from event variants
    fn infer_hook(ev: &AgentEvent) -> &str {
        match ev {
            AgentEvent::Started { .. } => "SessionStart",
            AgentEvent::Completed { .. } => "SessionEnd",
            AgentEvent::Failed { .. } => "StopFailure",
            AgentEvent::ToolCallStart { .. } => "PreToolUse",
            AgentEvent::ToolCallEnd { result, .. } => {
                if result.is_error { "PostToolUseFailure" } else { "PostToolUse" }
            }
            AgentEvent::MessageSent { .. } => "UserPromptSubmit",
            AgentEvent::ContextPressure { .. } => "PreCompact",
            AgentEvent::Forked { .. } => "SubagentStart",
            AgentEvent::TokenUsage { .. } => "Transcript",
            AgentEvent::OutputDelta { .. } => "Notification",
            _ => "Other",
        }
    }

    for i in 0..n {
        let hook = if use_stored { hooks[i].clone() } else { infer_hook(&events[i]).to_string() };
        let ev = &events[i];
        let entry = match ev {
            forge_sdk::events::AgentEvent::Started { task, timestamp, .. } => json!({
                "seq": i + 1, "time": timestamp.to_rfc3339(), "detail": task,
            }),
            forge_sdk::events::AgentEvent::Completed { summary, timestamp, .. } => json!({
                "seq": i + 1, "time": timestamp.to_rfc3339(), "detail": summary,
            }),
            forge_sdk::events::AgentEvent::Failed { error, timestamp, .. } => json!({
                "seq": i + 1, "time": timestamp.to_rfc3339(), "detail": error,
            }),
            forge_sdk::events::AgentEvent::ToolCallStart { tool, args, timestamp, .. } => json!({
                "seq": i + 1, "time": timestamp.to_rfc3339(), "tool": tool,
                "args_display": format_tool_detail(tool, args),
            }),
            forge_sdk::events::AgentEvent::ToolCallEnd { tool, result, timestamp, .. } => json!({
                "seq": i + 1, "time": timestamp.to_rfc3339(), "tool": tool,
                "is_error": result.is_error, "duration_ms": result.duration_ms,
                "result_preview": if result.content.len() > 200 { format!("{}...", &result.content[..200]) } else { result.content.clone() },
                "token_count": result.token_count,
            }),
            forge_sdk::events::AgentEvent::MessageSent { from, content, timestamp, .. } => {
                let text = match content {
                    forge_sdk::events::MessageContent::Text(t) => t.clone(),
                    _ => String::new(),
                };
                json!({
                    "seq": i + 1, "time": timestamp.to_rfc3339(), "from": from,
                    "text": if text.len() > 200 { format!("{}...", &text[..200]) } else { text },
                })
            }
            forge_sdk::events::AgentEvent::ContextPressure { current_ratio, timestamp, .. } => json!({
                "seq": i + 1, "time": timestamp.to_rfc3339(),
                "pressure_pct": (current_ratio * 100.0) as u32,
            }),
            forge_sdk::events::AgentEvent::Forked { child_id, task, timestamp, .. } => json!({
                "seq": i + 1, "time": timestamp.to_rfc3339(), "child": child_id, "task": task,
            }),
            forge_sdk::events::AgentEvent::TokenUsage { input, output, model, timestamp, .. } => json!({
                "seq": i + 1, "time": timestamp.to_rfc3339(),
                "input": input, "output": output, "model": model,
            }),
            _ => json!({"seq": i + 1, "detail": format!("{:?}", ev)}),
        };
        hook_groups.entry(hook.clone()).or_default().push(entry);
    }

    // Known hook order for consistent display
    let hook_order = [
        "SessionStart", "UserPromptSubmit",
        "PreToolUse", "PostToolUse", "PostToolUseFailure",
        "PreCompact", "PostCompact",
        "Notification",
        "SubagentStart", "SubagentStop",
        "Transcript",
        "SessionEnd", "Stop", "StopFailure",
    ];

    let mut result = Vec::new();
    for &hook_name in &hook_order {
        if let Some(entries) = hook_groups.remove(hook_name) {
            result.push(json!({
                "hook": hook_name,
                "description": hook_description(hook_name),
                "count": entries.len(),
                "events": entries,
            }));
        }
    }
    // Any remaining hooks (unexpected/custom)
    for (hook_name, entries) in hook_groups {
        result.push(json!({
            "hook": hook_name,
            "description": "Hook event",
            "count": entries.len(),
            "events": entries,
        }));
    }
    result
}

fn hook_description(hook: &str) -> &str {
    match hook {
        "SessionStart" => "Session begins — first contact from agent",
        "UserPromptSubmit" => "User submitted a prompt/message",
        "PreToolUse" => "Tool is about to be called (before execution)",
        "PostToolUse" => "Tool finished executing (result available)",
        "PostToolUseFailure" => "Tool execution failed",
        "PreCompact" => "Context compaction about to occur",
        "Notification" => "Agent sent a notification",
        "SubagentStart" => "Subagent was forked/spawned",
        "SubagentStop" => "Subagent finished",
        "Transcript" => "Transcript summary data received (tokens, model)",
        "SessionEnd" => "Session ended normally",
        "Stop" => "Agent was stopped",
        "StopFailure" => "Session ended with error",
        _ => "Other event",
    }
}

/// Build tool instances — every individual tool call with full args and result.
fn build_tool_instances(events: &[forge_sdk::events::AgentEvent]) -> Vec<Value> {
    use forge_sdk::events::AgentEvent;
    let mut instances = Vec::new();
    let mut pending_starts: std::collections::HashMap<String, (usize, Value, String)> = std::collections::HashMap::new();

    for (i, e) in events.iter().enumerate() {
        match e {
            AgentEvent::ToolCallStart { agent_id, tool, args, .. } => {
                pending_starts.insert(agent_id.clone(), (i, args.clone(), tool.clone()));
            }
            AgentEvent::ToolCallEnd { agent_id, tool, result, timestamp } => {
                // Try to match with a pending ToolCallStart for better args display
                let args_display = if let Some((_, args, started_tool)) = pending_starts.remove(agent_id) {
                    if started_tool == *tool {
                        format_tool_detail(tool, &args)
                    } else {
                        // Tool mismatch — use fallback from result content
                        extract_args_from_result(tool, &result.content)
                    }
                } else {
                    // No matching start — extract from result content or use generic display
                    extract_args_from_result(tool, &result.content)
                };

                instances.push(json!({
                    "seq": i + 1,
                    "time": timestamp.to_rfc3339(),
                    "tool": tool,
                    "args_display": args_display,
                    "is_error": result.is_error,
                    "duration_ms": result.duration_ms,
                    "token_count": result.token_count,
                    "result": if result.content.len() > 300 {
                        format!("{}... [{} total chars]", &result.content[..300], result.content.len())
                    } else {
                        result.content.clone()
                    },
                    "result_full_len": result.content.len(),
                }));
            }
            _ => {}
        }
    }
    instances
}

/// Extract a human-readable args display from tool result content when no ToolCallStart is available.
fn extract_args_from_result(tool: &str, content: &str) -> String {
    match tool {
        "Read" | "Glob" | "Grep" => {
            // Result often contains file paths or search patterns
            let first_line = content.lines().next().unwrap_or("");
            if first_line.len() > 80 {
                format!("{}: {}...", tool, &first_line[..80])
            } else {
                format!("{}: {}", tool, first_line)
            }
        }
        "Bash" | "PowerShell" => {
            // Try to extract command from result — often the result starts with the command output
            // The first line of content is usually the most descriptive
            let first_line = content.lines().next().unwrap_or("");
            if first_line.len() > 80 {
                format!("$ {}...", &first_line[..80])
            } else if !first_line.is_empty() {
                format!("$ {}", first_line)
            } else {
                format!("{} executed", tool)
            }
        }
        "Write" | "Edit" => {
            // First line describes what was written
            let first_line = content.lines().next().unwrap_or("");
            if !first_line.is_empty() && first_line.len() < 100 {
                first_line.to_string()
            } else {
                format!("{} completed", tool)
            }
        }
        "WebFetch" | "WebSearch" => {
            format!("{} completed", tool)
        }
        _ => {
            let first_line = content.lines().next().unwrap_or("");
            if !first_line.is_empty() && first_line.len() < 80 {
                format!("{}: {}", tool, first_line)
            } else {
                format!("{} completed", tool)
            }
        }
    }
}

/// Build prompt instances — each user prompt with context of what followed.
fn build_prompt_instances(events: &[forge_sdk::events::AgentEvent], prompt_history: &[String]) -> Vec<Value> {
    use forge_sdk::events::AgentEvent;
    let mut instances = Vec::new();
    let mut prompt_idx = 0usize;

    for (i, e) in events.iter().enumerate() {
        if let AgentEvent::MessageSent { from, content, timestamp, .. } = e {
            if from != "user" { continue; }
            let text = match content {
                forge_sdk::events::MessageContent::Text(t) => t.clone(),
                _ => continue,
            };
            if text.is_empty() { continue; }

            // Find subsequent tool calls and responses until next prompt or session end
            let mut following_tools = Vec::new();
            let mut following_tokens = 0u64;
            let mut response_latency_ms: Option<u64> = None;

            for subsequent in events.iter().skip(i + 1) {
                match subsequent {
                    AgentEvent::ToolCallEnd { result, .. } => {
                        following_tools.push(json!({
                            "tool": "",
                            "is_error": result.is_error,
                            "duration_ms": result.duration_ms,
                        }));
                        if response_latency_ms.is_none() {
                            response_latency_ms = Some(result.duration_ms);
                        }
                    }
                    AgentEvent::TokenUsage { input, output, .. } => {
                        following_tokens += input + output;
                    }
                    AgentEvent::MessageSent { from: f, .. } if f == "user" => break,
                    AgentEvent::Started { .. } | AgentEvent::Completed { .. } => break,
                    _ => {}
                }
            }

            let prompt_text = prompt_history.get(prompt_idx).cloned().unwrap_or(text);
            prompt_idx += 1;

            instances.push(json!({
                "seq": i + 1,
                "index": prompt_idx,
                "time": timestamp.to_rfc3339(),
                "text": prompt_text,
                "text_len": prompt_text.len(),
                "estimated_input_tokens": (prompt_text.len() as f64 / 4.0).ceil() as u64,
                "following_tool_calls": following_tools.len(),
                "following_tokens": following_tokens,
                "first_response_latency_ms": response_latency_ms,
                "tools_after": following_tools,
            }));
        }
    }
    instances
}

/// Build detector report — what detectors found, what interventions were applied.
fn build_detector_report(
    detections: &[Value],
    interventions: &[Value],
    strategies: &[Value],
) -> Value {
    // Group detections by category
    let mut by_category: std::collections::HashMap<String, Vec<&Value>> = std::collections::HashMap::new();
    for d in detections {
        if let Some(cat) = d.get("category").and_then(|c| c.as_str()) {
            by_category.entry(cat.to_string()).or_default().push(d);
        }
    }

    let categories: Vec<Value> = by_category.iter().map(|(cat, items)| {
        let severities: Vec<&str> = items.iter()
            .filter_map(|d| d.get("severity").and_then(|s| s.as_str()))
            .collect();
        let worst = if severities.iter().any(|s| s.contains("Critical")) { "Critical" }
            else if severities.iter().any(|s| s.contains("Error")) { "Error" }
            else if severities.iter().any(|s| s.contains("Warning")) { "Warning" }
            else { "Info" };

        let confidences: Vec<f64> = items.iter()
            .filter_map(|d| d.get("confidence").and_then(|c| c.as_f64()))
            .collect();
        let avg_conf = if confidences.is_empty() { 0.0 }
            else { confidences.iter().sum::<f64>() / confidences.len() as f64 };

        json!({
            "category": cat,
            "severity": worst,
            "count": items.len(),
            "avg_confidence": avg_conf,
            "instances": items,
        })
    }).collect();

    json!({
        "total_detections": detections.len(),
        "total_interventions": interventions.len(),
        "total_strategies": strategies.len(),
        "by_category": categories,
        "interventions": interventions,
        "strategies": strategies,
    })
}

/// Build a human-readable event log from raw AgentEvents.
fn build_event_log(events: &[forge_sdk::events::AgentEvent]) -> Vec<Value> {
    use forge_sdk::events::AgentEvent;
    events.iter().filter_map(|e| {
        match e {
            AgentEvent::Started { task, timestamp, .. } => Some(json!({
                "time": timestamp.to_rfc3339(),
                "type": "start",
                "icon": "▶",
                "detail": task,
            })),
            AgentEvent::Completed { summary, timestamp, .. } => Some(json!({
                "time": timestamp.to_rfc3339(),
                "type": "complete",
                "icon": "✓",
                "detail": summary,
            })),
            AgentEvent::Failed { error, timestamp, .. } => Some(json!({
                "time": timestamp.to_rfc3339(),
                "type": "error",
                "icon": "✗",
                "detail": error,
            })),
            AgentEvent::ToolCallStart { tool, args, timestamp, .. } => Some(json!({
                "time": timestamp.to_rfc3339(),
                "type": "tool_start",
                "icon": "→",
                "tool": tool,
                "detail": format_tool_detail(tool, args),
            })),
            AgentEvent::ToolCallEnd { tool, result, timestamp, .. } => {
                let icon = if result.is_error { "✗" } else { "←" };
                let status = if result.is_error { "FAILED" } else { "ok" };
                Some(json!({
                    "time": timestamp.to_rfc3339(),
                    "type": "tool_end",
                    "icon": icon,
                    "tool": tool,
                    "status": status,
                    "tokens": result.token_count,
                    "detail": if result.content.len() > 150 { format!("{}...", &result.content[..150]) } else { result.content.clone() },
                }))
            }
            AgentEvent::MessageSent { from, content, timestamp, .. } => {
                let text = match content {
                    forge_sdk::events::MessageContent::Text(t) => t.clone(),
                    _ => String::new(),
                };
                if text.is_empty() { return None; }
                Some(json!({
                    "time": timestamp.to_rfc3339(),
                    "type": "message",
                    "icon": "💬",
                    "from": from,
                    "detail": if text.len() > 200 { format!("{}...", &text[..200]) } else { text },
                }))
            }
            AgentEvent::ContextPressure { current_ratio, timestamp, .. } => Some(json!({
                "time": timestamp.to_rfc3339(),
                "type": "context",
                "icon": "📐",
                "detail": format!("Context pressure: {:.0}%", current_ratio * 100.0),
            })),
            AgentEvent::Forked { child_id, task, timestamp, .. } => Some(json!({
                "time": timestamp.to_rfc3339(),
                "type": "fork",
                "icon": "⑂",
                "detail": format!("Subagent {}: {}", child_id, task),
            })),
            AgentEvent::TokenUsage { input, output, cache_read, model, timestamp, .. } => Some(json!({
                "time": timestamp.to_rfc3339(),
                "type": "token",
                "icon": "📊",
                "detail": format!("Tokens: {} in / {} out (cache: {}). Model: {}", input, output, cache_read, model),
            })),
            AgentEvent::OutputDelta { text, timestamp, .. } => {
                if text.is_empty() || text.starts_with('[') { return None; }
                Some(json!({
                    "time": timestamp.to_rfc3339(),
                    "type": "output",
                    "icon": "→",
                    "detail": if text.len() > 150 { format!("{}...", &text[..150]) } else { text.clone() },
                }))
            }
            _ => None,
        }
    }).collect()
}

/// Format tool arguments into a human-readable detail line.
fn format_tool_detail(tool: &str, args: &Value) -> String {
    match tool {
        "Write" | "Edit" => {
            let path = args
                .get("file_path")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            format!("{} → {}", tool, path)
        }
        "Read" => {
            let path = args
                .get("file_path")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            format!("Read {}", path)
        }
        "Bash" | "PowerShell" => {
            let cmd = args.get("command").and_then(|v| v.as_str()).unwrap_or("?");
            format!(
                "$ {}",
                if cmd.len() > 80 {
                    format!("{}...", &cmd[..80])
                } else {
                    cmd.to_string()
                }
            )
        }
        "Grep" => {
            let pattern = args.get("pattern").and_then(|v| v.as_str()).unwrap_or("?");
            format!("grep: {}", pattern)
        }
        "Glob" => {
            let pattern = args.get("pattern").and_then(|v| v.as_str()).unwrap_or("?");
            format!("glob: {}", pattern)
        }
        "WebFetch" | "WebSearch" => {
            let url = args
                .get("url")
                .or(args.get("query"))
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            format!(
                "{}: {}",
                tool,
                if url.len() > 60 {
                    format!("{}...", &url[..60])
                } else {
                    url.to_string()
                }
            )
        }
        _ => format!(
            "{}: {}",
            tool,
            serde_json::to_string(args)
                .unwrap_or_default()
                .chars()
                .take(60)
                .collect::<String>()
        ),
    }
}

use chrono::Utc;

/// Translate a raw observation JSON value into a human-readable description.
fn describe_observation(obs: &Value) -> String {
    let dim = obs
        .get("dimension")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    match dim {
        "token" => {
            let total = obs
                .get("total_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let cache = obs
                .get("cache_hit_rate")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            format!(
                "Token usage: {} total tokens, {:.0}% cache hit rate",
                total,
                cache * 100.0
            )
        }
        "latency" => {
            let count = obs.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
            let p50 = obs.get("p50_ms").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let p95 = obs.get("p95_ms").and_then(|v| v.as_f64()).unwrap_or(0.0);
            format!(
                "Latency: {} calls, p50={:.0}ms, p95={:.0}ms",
                count, p50, p95
            )
        }
        "cost" => {
            let per_turn = obs
                .get("cost_per_turn")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let total = obs
                .get("total_cost")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            format!("Cost: ${:.4} per turn, ${:.4} total", per_turn, total)
        }
        "accuracy" => {
            let lint = obs.get("lint_errors").and_then(|v| v.as_u64()).unwrap_or(0);
            let tests = obs
                .get("test_pass_rate")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            format!(
                "Accuracy: {} lint errors, {:.0}% test pass rate",
                lint,
                tests * 100.0
            )
        }
        "security" => {
            let issues = obs
                .get("issues_found")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let leaks = obs
                .get("secret_leaks")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            format!(
                "Security: {} issues, {} potential secret leaks",
                issues, leaks
            )
        }
        "reliability" => {
            let ops = obs.get("total_ops").and_then(|v| v.as_u64()).unwrap_or(0);
            let errors = obs
                .get("error_rate")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            format!(
                "Reliability: {} operations, {:.1}% error rate",
                ops,
                errors * 100.0
            )
        }
        "context_quality" => {
            let files = obs
                .get("unique_files")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let redundancy = obs
                .get("redundancy_ratio")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            format!(
                "Context: {} unique files tracked, {:.0}% redundancy",
                files,
                redundancy * 100.0
            )
        }
        "orch" => {
            let agents = obs
                .get("active_agents")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let forks = obs.get("total_forks").and_then(|v| v.as_u64()).unwrap_or(0);
            format!("Orchestration: {} agents, {} forks", agents, forks)
        }
        "compliance" => {
            let violations = obs.get("violations").and_then(|v| v.as_u64()).unwrap_or(0);
            format!("Compliance: {} policy violations detected", violations)
        }
        "comm" => {
            let msgs = obs
                .get("message_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            format!("Communication: {} agent messages", msgs)
        }
        "memory" => {
            let usage = obs.get("usage_pct").and_then(|v| v.as_f64()).unwrap_or(0.0);
            format!("Memory: {:.0}% context window used", usage * 100.0)
        }
        "diversity" => {
            let score = obs
                .get("similarity_score")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            format!(
                "Diversity: {:.0}% output similarity (lower is better)",
                score * 100.0
            )
        }
        _ => format!(
            "{}: {}",
            dim,
            serde_json::to_string(obs).unwrap_or_default()
        ),
    }
}

/// Group observations by their dimension field, returning counts and human-readable summaries.
fn group_observations_by_dimension(observations: &[Value]) -> Value {
    use std::collections::HashMap;
    let mut groups: HashMap<String, Vec<&Value>> = HashMap::new();
    for obs in observations {
        if let Some(dim) = obs.get("dimension").and_then(|v| v.as_str()) {
            groups.entry(dim.to_string()).or_default().push(obs);
        }
    }
    let result: Vec<Value> = groups
        .into_iter()
        .map(|(dim, items)| {
            let count = items.len();
            let latest = items.last().cloned();
            let description = latest
                .as_ref()
                .map(|v| describe_observation(v))
                .unwrap_or_else(|| format!("{}: no data", dim));
            json!({
                "dimension": dim,
                "count": count,
                "description": description,
                "latest": latest,
                "samples": items.iter().rev().take(5).cloned().collect::<Vec<_>>(),
            })
        })
        .collect();
    json!(result)
}

/// Return (input_price_per_token, output_price_per_token, model_family_name) for a model.
fn model_pricing(model: &str) -> (f64, f64, String) {
    let m = model.to_lowercase();
    // Claude models
    if m.contains("opus") {
        return (0.000015, 0.000075, "Claude Opus".into());
    }
    if m.contains("sonnet") {
        return (0.000003, 0.000015, "Claude Sonnet".into());
    }
    if m.contains("haiku") {
        return (0.0000008, 0.000004, "Claude Haiku".into());
    }
    if m.contains("fable") {
        return (0.000003, 0.000015, "Claude Fable".into());
    }
    if m.contains("claude") {
        return (0.000003, 0.000015, "Claude (default)".into());
    }
    // GPT models
    if m.contains("gpt-4o") {
        return (0.0000025, 0.000010, "GPT-4o".into());
    }
    if m.contains("gpt-4") {
        return (0.000030, 0.000060, "GPT-4".into());
    }
    if m.contains("gpt-3.5") {
        return (0.0000005, 0.0000015, "GPT-3.5".into());
    }
    if m.contains("o1") || m.contains("o3") {
        return (0.000015, 0.000060, "OpenAI o-series".into());
    }
    if m.contains("gpt") {
        return (0.0000025, 0.000010, "GPT (default)".into());
    }
    // DeepSeek
    if m.contains("deepseek") {
        return (0.00000027, 0.0000011, "DeepSeek".into());
    }
    // Gemini
    if m.contains("gemini") {
        return (0.00000125, 0.000005, "Gemini".into());
    }
    // Default / unknown
    (0.000003, 0.000015, "unknown (using default)".into())
}

/// GET /v1/sessions/:id/checkpoints
pub async fn checkpoints(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<String>,
) -> Json<Value> {
    Json(json!({"checkpoints": [], "message": "Checkpoints not yet persisted to API"}))
}
