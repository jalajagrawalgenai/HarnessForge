# Harness Architecture

Forge uses a two-level harness architecture:

## Level 1: Session Harness

Observes every agent event in real-time → detects issues → selects strategies → applies interventions → logs to audit trail.

## Level 2: Meta-Harness

Mines weakness patterns across sessions → proposes minimal edits → regression tests → applies improvements.

This is based on the Self-Harness paper (Shanghai AI Lab, June 2026).
