"""
Forge -- AI Agent Harness SDK.

pip install forge-agent-sdk  ->  That's it.

Forge auto-detects your AI agent sessions (Claude Code, LangGraph, CrewAI,
AutoGen, etc.) and runs the harness pipeline: 12 observers -> 16 detectors ->
14 autonomous intervention strategies. No manual setup needed.
"""
import os, sys, json, shutil

os.environ.setdefault("PYO3_USE_ABI3_FORWARD_COMPATIBILITY", "1")


def _forge_state_dir():
    home = os.environ.get("USERPROFILE") or os.environ.get("HOME") or "."
    return os.path.join(home, ".forge")


def _claude_settings_path():
    home = os.environ.get("USERPROFILE") or os.environ.get("HOME") or "."
    return os.path.join(home, ".claude", "settings.json")


def _is_registered():
    sp = _claude_settings_path()
    if not os.path.exists(sp):
        return False
    try:
        with open(sp) as f:
            settings = json.load(f)
        for entries in settings.get("hooks", {}).values():
            for entry in entries:
                for h in entry.get("hooks", []):
                    if "forge" in h.get("command", ""):
                        return True
    except Exception:
        pass
    return False


def _setup_forge():
    sd = _forge_state_dir()
    os.makedirs(sd, exist_ok=True)
    hook_src = os.path.join(os.path.dirname(__file__), "hooks", "observe_hook.mjs")
    hook_dst = os.path.join(sd, "observe_hook.mjs")
    if os.path.exists(hook_src) and not os.path.exists(hook_dst):
        try:
            shutil.copy2(hook_src, hook_dst)
        except Exception:
            pass
    if not _is_registered():
        sp = _claude_settings_path()
        hook_cmd = "node " + sd.replace("\\", "/") + "/observe_hook.mjs"
        events = [
            "SessionStart", "UserPromptSubmit", "PreToolUse", "PostToolUse",
            "PostToolUseFailure", "SessionEnd", "Stop", "PreCompact",
            "PostCompact", "SubagentStart", "SubagentStop", "Notification",
        ]
        settings = {}
        if os.path.exists(sp):
            try:
                with open(sp) as f:
                    settings = json.load(f)
            except Exception:
                pass
        else:
            os.makedirs(os.path.dirname(sp), exist_ok=True)
        settings.setdefault("hooks", {})
        for ev in events:
            entries = settings["hooks"].setdefault(ev, [])
            ca = next((e for e in entries if e.get("matcher", "") == ""), None)
            if ca is None:
                ca = {"matcher": "", "hooks": []}
                entries.append(ca)
            hl = ca.setdefault("hooks", [])
            if not any("forge" in h.get("command", "") for h in hl):
                hl.append({"type": "command", "command": hook_cmd})
        try:
            with open(sp, "w") as f:
                json.dump(settings, f, indent=2)
        except Exception:
            pass


_marker = os.path.join(_forge_state_dir(), ".setup_done")
if not os.path.exists(_marker):
    try:
        _setup_forge()
        with open(_marker, "w") as f:
            f.write("1")
    except Exception:
        pass

from .forge_sdk import (
    HarnessRunResult, PyHarness, create_harness, quick_run,
    list_presets, list_detectors, list_strategies, list_observers,
    get_version, serve,
)

__all__ = [
    "HarnessRunResult", "PyHarness", "create_harness", "quick_run",
    "list_presets", "list_detectors", "list_strategies", "list_observers",
    "get_version", "serve",
]
