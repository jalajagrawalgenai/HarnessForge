# Self-Improvement (Meta-Harness)

Forge's meta-harness implements the Self-Harness loop: it watches the watcher and improves the harness over time.

## How It Works

1. **Weakness Mining** — Analyze completed sessions for weakness patterns:
   - `high_tool_error_rate` — >30% tool failures (critical if >50%)
   - `undetected_issues` — >10 tool calls with 0 detections (detectors may be too lenient)
   - `high_context_pressure` — avg pressure >70% (warning if >85%)
   - `intervention_gap` — more detections than interventions (strategies undershooting)

2. **Rule Proposal** — Generate minimal, targeted edits:
   - `tool_error_threshold` — Lower threshold from 0.2 to 0.15 for error-prone agent types
   - `detection_budget` — Increase intervention aggressiveness when many detections occur

3. **A/B Testing** — Compare harness configs across sessions to validate improvements

4. **Proposal Validation** — Accept or reject proposed edits

## API Endpoints

| Endpoint | Method | Description |
|---|---|---|
| `/v1/meta/improve` | POST | Run improvement cycle on completed sessions |
| `/v1/meta/weaknesses` | GET | Current cached weakness patterns |
| `/v1/meta/edits` | GET | Pending harness rule edits |
| `/v1/meta/edits/:id/accept` | POST | Accept a proposed rule change |
| `/v1/meta/edits/:id/reject` | POST | Reject a proposed rule change |
| `/v1/meta/ab-tests` | GET | Sessions available for A/B testing |

## Example Response (POST /v1/meta/improve)

```json
{
  "status": "completed",
  "sessions_analyzed": 5,
  "weaknesses_found": 3,
  "weaknesses": [
    {
      "pattern": "high_tool_error_rate",
      "count": 2,
      "severity": "warning",
      "instances": [
        {
          "session_id": "abc123",
          "agent_type": "claude-code",
          "detail": "40% tool error rate (4 errors / 10 calls)"
        }
      ]
    }
  ],
  "message": "Analyzed 5 sessions, found 3 weakness patterns"
}
```

When no completed sessions exist, returns:
```json
{
  "status": "not_enough_data",
  "message": "No completed sessions to analyze. Run some agent sessions first."
}
```

## Research Basis

Validated by the Self-Harness paper (Shanghai AI Lab, arXiv:2606.09498, June 2026): 33-60% pass rate improvement through systematic harness self-improvement.
