use forge_sdk::types::audit::AuditReport;

pub struct AuditDiff;

impl AuditDiff {
    pub fn compare(a: &AuditReport, b: &AuditReport) -> DiffResult {
        DiffResult {
            session_a: a.session_id.to_string(),
            session_b: b.session_id.to_string(),
            token_delta: b.total_tokens as i64 - a.total_tokens as i64,
            cost_delta: b.total_cost - a.total_cost,
            duration_delta: b.duration_secs - a.duration_secs,
            detection_delta: b.detections.len() as i64 - a.detections.len() as i64,
            intervention_delta: b.interventions.len() as i64 - a.interventions.len() as i64,
            health_delta: b.health_score.unwrap_or(0.0) - a.health_score.unwrap_or(0.0),
        }
    }
}

#[derive(Debug)]
pub struct DiffResult {
    pub session_a: String,
    pub session_b: String,
    pub token_delta: i64,
    pub cost_delta: f64,
    pub duration_delta: f64,
    pub detection_delta: i64,
    pub intervention_delta: i64,
    pub health_delta: f64,
}
