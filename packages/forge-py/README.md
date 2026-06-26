# Forge SDK for Python

Wrap any AI agent with 12-dimension observation, 16 detectors, and 14 autonomous intervention strategies.

## Installation

```bash
pip install forge-agent-sdk
```

Requirements: Python 3.10+ (including 3.14). Works on Windows, macOS, and Linux.

## Quick Start

### 1. One-Line Run (Simplest)

```python
from forge_sdk import quick_run

result = quick_run("Write a function to validate email addresses")
print(f"Success:       {result.success}")
print(f"Observations:  {result.observation_count}")
print(f"Detections:    {result.detection_count}")
print(f"Interventions: {result.intervention_count}")
```

### 2. Explore What's Available

```python
from forge_sdk import list_presets, list_detectors, list_strategies, list_observers

print("31 Presets:  ", list_presets())
print("16 Detectors:", list_detectors())
print("14 Strategies:", list_strategies())
print("12 Observers:", list_observers())
```

### 3. Full Harness API

```python
from forge_sdk import create_harness

# Create a harness with the "solo" preset
harness = create_harness(preset="solo")

# Run a task
result = harness.run("Build a REST API for a todo app")
print(result.to_dict())

# Dry run — observe only, no intervention
result = harness.dry_run("Test run — observe only")
print(f"Would have intervened: {result.intervention_count} times")

# Custom preset with more turns
result = harness.run_with("Add JWT authentication", preset="claude-code", turns=8)
```

### 4. Expected Output

```
Success:       True
Observations:  9
Detections:    0
Interventions: 0
```

When the harness detects an issue:

```
⚠  HARNESS [T6]: StaleContext detected. Context pressure 87%.
   → Strategy: Compact. Context reduced 87% → 58%. Saved 4.2K tokens.
```

## API Reference

### Functions

| Function | Description |
|---|---|
| `quick_run(task, preset="solo", turns=4)` | One-shot: create harness, run task, return result |
| `create_harness(preset="solo")` | Create a `PyHarness` instance for repeated use |
| `list_presets()` | List all 31 available presets |
| `list_detectors()` | List all 16 detectors |
| `list_strategies()` | List all 14 intervention strategies |
| `list_observers()` | List all 12 observation dimensions |
| `get_version()` | Get the forge-agent-sdk version string |

### PyHarness Methods

| Method | Description |
|---|---|
| `harness.run(task)` | Run task through full observe → detect → intervene pipeline |
| `harness.dry_run(task)` | Observe and detect only (no intervention) |
| `harness.run_with(task, preset, turns)` | Run with custom preset and turn count |

### HarnessRunResult Fields

| Field | Type | Description |
|---|---|---|
| `agent_id` | `str` | Agent identifier |
| `success` | `bool` | Whether the task completed successfully |
| `observation_count` | `int` | Pipeline observation cycles executed |
| `detection_count` | `int` | Issues detected during the run |
| `intervention_count` | `int` | Interventions applied during the run |
| `to_dict()` | `dict` | Convert result to a Python dictionary |

### Presets

All 31 presets: `solo`, `langgraph`, `crewai`, `autogen`, `langchain`, `openai-swarm`, `semantic-kernel`, `haystack`, `dspy`, `llamaindex`, `taskweaver`, `agno`, `atomic-agents`, `bee-agent`, `pydantic-ai`, `claude-code`, `aider`, `cline`, `continue`, `vercel-ai`, `copilot`, `cursor`, `windsurf`, `devin`, `amazon-q`, `replit-agent`, `pearai`, `bolt-new`, `lovable`, `v0`, `custom`.

```python
# Use any preset the same way
harness = create_harness(preset="claude-code")
harness = create_harness(preset="langgraph")
result = quick_run("task", preset="crewai")
```

## Build from Source

```bash
git clone https://github.com/jalajagrawalgenai/HarnessForge.git
cd HarnessForge/packages/forge-py
pip install maturin
maturin develop

# Verify
python -c "from forge_sdk import get_version; print(get_version())"
```

## Publishing

```bash
cd packages/forge-py
pip install maturin twine
maturin build --release
twine upload target/wheels/forge_sdk-*.whl
```

Or push a `v*` tag — CI builds wheels for Python 3.10–3.14 on Ubuntu, Windows, and macOS automatically.
