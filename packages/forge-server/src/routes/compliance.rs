use axum::extract::Query;
use axum::Json;
use forge_compliance::{
    eu_ai_act_article_14_checklist, eu_ai_act_article_15_checklist,
    generate_report as gen_compliance_report, soc2_checklist, ComplianceFramework,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Deserialize, Default)]
pub struct ReportQuery {
    pub framework: Option<String>,
    pub session_id: Option<String>,
}

pub async fn list_frameworks() -> Json<Value> {
    Json(json!({
        "frameworks": [
            {"id":"EuAiAct","name":"EU AI Act","articles":["Article 14 - Human Oversight","Article 15 - Record-Keeping"]},
            {"id":"Soc2Type2","name":"SOC 2 Type II","criteria":["Security","Availability","Confidentiality"]},
            {"id":"Iso27001","name":"ISO 27001","description":"Information Security Management"},
            {"id":"Gdpr","name":"GDPR","description":"General Data Protection Regulation"},
            {"id":"Hipaa","name":"HIPAA","description":"Health Insurance Portability and Accountability Act"},
            {"id":"PciDss","name":"PCI DSS","description":"Payment Card Industry Data Security Standard"}
        ]
    }))
}

pub async fn generate_report(Query(q): Query<ReportQuery>) -> Json<Value> {
    let framework = match q.framework.as_deref().unwrap_or("EuAiAct") {
        "EuAiAct" => ComplianceFramework::EuAiAct,
        "Soc2Type2" | "soc2" => ComplianceFramework::Soc2Type2,
        "Iso27001" => ComplianceFramework::Iso27001,
        "Gdpr" => ComplianceFramework::Gdpr,
        "Hipaa" => ComplianceFramework::Hipaa,
        "PciDss" => ComplianceFramework::PciDss,
        _ => ComplianceFramework::EuAiAct,
    };
    let session_id = q.session_id.as_deref().unwrap_or("unknown");
    let report = gen_compliance_report(framework, session_id);
    Json(serde_json::to_value(report).unwrap_or(json!({"error":"serialization failed"})))
}

pub async fn get_report(Query(q): Query<ReportQuery>) -> Json<Value> {
    let framework = match q.framework.as_deref().unwrap_or("EuAiAct") {
        "EuAiAct" => ComplianceFramework::EuAiAct,
        "Soc2Type2" | "soc2" => ComplianceFramework::Soc2Type2,
        _ => ComplianceFramework::EuAiAct,
    };
    let session_id = q.session_id.as_deref().unwrap_or("unknown");
    let report = gen_compliance_report(framework, session_id);
    Json(serde_json::to_value(report).unwrap_or(json!({"error":"serialization failed"})))
}

pub async fn get_checklist(Query(q): Query<HashMap<String, String>>) -> Json<Value> {
    let framework = q.get("framework").map(|s| s.as_str()).unwrap_or("EuAiAct");
    let checks = match framework {
        "EuAiAct-14" | "A14" => eu_ai_act_article_14_checklist(),
        "EuAiAct-15" | "A15" => eu_ai_act_article_15_checklist(),
        "Soc2Type2" | "soc2" => soc2_checklist(),
        _ => {
            let mut c = eu_ai_act_article_14_checklist();
            c.extend(eu_ai_act_article_15_checklist());
            c
        }
    };
    Json(serde_json::to_value(checks).unwrap_or(json!([])))
}
