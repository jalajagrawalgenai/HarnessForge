use forge_sdk::error::ForgeError;
use forge_sdk::types::audit::{AuditEvent, AuditReport};

pub enum ExportFormat {
    Json,
    Csv,
    Pdf,
}

pub fn export_events(events: &[AuditEvent], format: ExportFormat) -> Result<String, ForgeError> {
    match format {
        ExportFormat::Json => {
            serde_json::to_string_pretty(events).map_err(|e| ForgeError::Audit(e.to_string()))
        }
        ExportFormat::Csv => {
            let mut csv = String::from("id,session_id,phase,event_type,sequence,created_at\n");
            for e in events {
                csv.push_str(&format!(
                    "{},{},{:?},{},{},{}\n",
                    e.id, e.session_id, e.phase, e.event_type, e.sequence, e.created_at
                ));
            }
            Ok(csv)
        }
        ExportFormat::Pdf => Ok(format!(
            "Audit Report — {} events (PDF export placeholder)",
            events.len()
        )),
    }
}

pub fn export_report(report: &AuditReport, format: ExportFormat) -> Result<String, ForgeError> {
    match format {
        ExportFormat::Json => {
            serde_json::to_string_pretty(report).map_err(|e| ForgeError::Audit(e.to_string()))
        }
        ExportFormat::Csv => Ok(format!(
            "session_id,task,duration,tokens,cost,health\n{},{},{:.1},{},{},{:?}",
            report.session_id,
            report.task,
            report.duration_secs,
            report.total_tokens,
            report.total_cost,
            report.health_score
        )),
        ExportFormat::Pdf => Ok(format!(
            "Forge Audit Report — {} (PDF placeholder)",
            report.task
        )),
    }
}
