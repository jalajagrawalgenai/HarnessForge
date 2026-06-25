# Forge SDK for Python

Wrap any AI agent with 12-dimension observation, 16 detectors, and 14 autonomous intervention strategies.

## Installation

```bash
pip install forge-sdk
```

Or from source:
```bash
cd packages/forge-py
pip install maturin
maturin develop
```

## Quick Start

```python
from forge_sdk import create_harness, quick_run, list_presets

# See available presets
for p in list_presets():
    print(p)

# Quick run — simplest way to try Forge
result = quick_run("Write a function to validate email addresses")
print(f"Success: {result.success}")
print(f"Observations: {result.observation_count}")
print(f"Detections: {result.detection_count}")
print(f"Interventions: {result.intervention_count}")

# Or use a harness directly
harness = create_harness(preset="solo")
result = harness.run("Refactor the auth module to use JWT")
print(result.to_dict())
```

## API Reference

### `create_harness(preset="solo")`
Create a Forge harness with the given preset.
Presets: solo, claude-code, langgraph, crewai, autogen, langchain, dspy, llamaindex, aider, cline, continue, copilot, cursor, windsurf, devin, custom

### `harness.run(task)`
Run a task through the harness. Returns HarnessRunResult.

### `harness.run_with(task, preset, turns)`
Run with custom preset and turn count.

### `harness.dry_run(task)`
Observe and detect, but don't intervene.

### `quick_run(task, preset="solo", turns=4)`
One-shot convenience function.

### `HarnessRunResult`
- `agent_id` — Agent identifier
- `success` — Whether the task succeeded
- `observation_count` — Number of pipeline observation cycles
- `detection_count` — Issues detected
- `intervention_count` — Interventions applied
- `to_dict()` — Convert to Python dict
