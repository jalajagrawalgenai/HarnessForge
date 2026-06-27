//! Claude Code hooks auto-registration.
//! Registers the Forge observe hook in ~/.claude/settings.json on `forge serve`.

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const OBSERVE_EVENTS: &[&str] = &[
    "SessionStart",
    "UserPromptSubmit",
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "SessionEnd",
    "Stop",
    "PreCompact",
    "PostCompact",
    "SubagentStart",
    "SubagentStop",
    "Notification",
];

fn home_dir() -> PathBuf {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn setup_hooks(port: u16) {
    let state_dir = home_dir().join(".forge");
    let _ = fs::create_dir_all(&state_dir);

    let port_file = state_dir.join("port");
    let _ = fs::write(&port_file, port.to_string());

    write_hook_script(&state_dir);
    register_in_claude_settings(port);
    tracing::info!("Forge hooks registered for port {}", port);
}

fn write_hook_script(state_dir: &Path) {
    let hook_dest = state_dir.join("observe_hook.mjs");
    if hook_dest.exists() {
        return;
    }

    let script = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../forge-py/forge_sdk/hooks/observe_hook.mjs"
    ));

    if let Err(e) = fs::write(&hook_dest, script) {
        tracing::warn!("Could not write hook script: {}", e);
    }
}

fn register_in_claude_settings(port: u16) {
    let settings_path = home_dir().join(".claude").join("settings.json");
    let hook_cmd = format!(
        "node {}/observe_hook.mjs --port {}",
        home_dir().join(".forge").display(),
        port
    );

    let mut settings: Value = if settings_path.exists() {
        fs::read_to_string(&settings_path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or(json!({}))
    } else {
        if let Some(p) = settings_path.parent() {
            let _ = fs::create_dir_all(p);
        }
        json!({})
    };

    if settings.get("hooks").is_none() {
        settings["hooks"] = json!({});
    }

    let hooks = settings["hooks"].as_object_mut().expect("hooks object");

    for event_name in OBSERVE_EVENTS {
        let entry = hooks.entry(*event_name).or_insert_with(|| json!([]));
        let entries = entry.as_array_mut().expect("event array");

        let catch_all = entries.iter_mut().find(|e| {
            e.get("matcher")
                .and_then(|m| m.as_str())
                .map(|m| m.is_empty())
                .unwrap_or(false)
        });

        if let Some(ca) = catch_all {
            let hl = ca["hooks"].as_array_mut().expect("hooks array");
            let already = hl.iter().any(|h| {
                h.get("command")
                    .and_then(|c| c.as_str())
                    .map(|c| c.contains("forge"))
                    .unwrap_or(false)
            });
            if !already {
                hl.push(json!({"type": "command", "command": hook_cmd}));
            }
        } else {
            entries.push(json!({
                "matcher": "",
                "hooks": [{"type": "command", "command": hook_cmd}]
            }));
        }
    }

    if let Ok(s) = serde_json::to_string_pretty(&settings) {
        let _ = fs::write(&settings_path, &s);
    }
}
