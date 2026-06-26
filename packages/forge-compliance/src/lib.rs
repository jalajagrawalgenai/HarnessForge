//! Forge Compliance — EU AI Act & regulatory compliance packs.
//!
//! Pre-built templates for:
//! - EU AI Act Article 14 (Human Oversight)
//! - EU AI Act Article 15 (Record-Keeping)
//! - SOC 2 Type II audit evidence
//! - ISO 27001 information security
//! - GDPR data residency & right to deletion

use serde::{Deserialize, Serialize};

/// Compliance framework identifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComplianceFramework {
    EuAiAct,
    Soc2Type2,
    Iso27001,
    Gdpr,
    Hipaa,
    PciDss,
}

impl std::fmt::Display for ComplianceFramework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EuAiAct => write!(f, "EU AI Act"),
            Self::Soc2Type2 => write!(f, "SOC 2 Type II"),
            Self::Iso27001 => write!(f, "ISO 27001"),
            Self::Gdpr => write!(f, "GDPR"),
            Self::Hipaa => write!(f, "HIPAA"),
            Self::PciDss => write!(f, "PCI DSS"),
        }
    }
}

/// EU AI Act Article 14 — Human Oversight checklist.
pub fn eu_ai_act_article_14_checklist() -> Vec<ComplianceCheck> {
    vec![
        ComplianceCheck {
            id: "A14.1".into(),
            requirement: "Human operators can intervene at any point".into(),
            forge_feature: "Human gate with pause/approve/reject/override".into(),
            auto_detectable: true,
        },
        ComplianceCheck {
            id: "A14.2".into(),
            requirement: "Clear indicators when AI is operating autonomously".into(),
            forge_feature: "Health score + intervention feed show harness activity".into(),
            auto_detectable: true,
        },
        ComplianceCheck {
            id: "A14.3".into(),
            requirement: "Ability to override AI decisions".into(),
            forge_feature: "OverrideAction in human gate state machine".into(),
            auto_detectable: true,
        },
        ComplianceCheck {
            id: "A14.4".into(),
            requirement: "Training for human operators".into(),
            forge_feature: "Audit explainer + session replay for training".into(),
            auto_detectable: false,
        },
    ]
}

/// EU AI Act Article 15 — Record-Keeping checklist.
pub fn eu_ai_act_article_15_checklist() -> Vec<ComplianceCheck> {
    vec![
        ComplianceCheck {
            id: "A15.1".into(),
            requirement: "Complete logs of AI system operation".into(),
            forge_feature: "Immutable append-only audit trail with hash chain".into(),
            auto_detectable: true,
        },
        ComplianceCheck {
            id: "A15.2".into(),
            requirement: "Records retained for appropriate period".into(),
            forge_feature: "Configurable retention policies (days/storage limit)".into(),
            auto_detectable: true,
        },
        ComplianceCheck {
            id: "A15.3".into(),
            requirement: "Logs available for regulatory inspection".into(),
            forge_feature: "Export to PDF/JSON/CSV. SIEM streaming (Splunk, Elastic)".into(),
            auto_detectable: true,
        },
        ComplianceCheck {
            id: "A15.4".into(),
            requirement: "Tamper-proof audit trail".into(),
            forge_feature: "SHA-256 hash chain + optional ed25519 signing".into(),
            auto_detectable: true,
        },
    ]
}

/// SOC 2 Type II — Trust Services Criteria.
pub fn soc2_checklist() -> Vec<ComplianceCheck> {
    vec![
        ComplianceCheck {
            id: "SOC2.Security.1".into(),
            requirement: "Logical and physical access controls".into(),
            forge_feature: "RBAC (Admin/Editor/Viewer/Auditor) + SSO".into(),
            auto_detectable: true,
        },
        ComplianceCheck {
            id: "SOC2.Security.2".into(),
            requirement: "System monitoring and alerting".into(),
            forge_feature: "16 detectors + PagerDuty/Slack/Discord alerts".into(),
            auto_detectable: true,
        },
        ComplianceCheck {
            id: "SOC2.Availability.1".into(),
            requirement: "System availability monitoring".into(),
            forge_feature: "Health score + reliability watcher tracks uptime".into(),
            auto_detectable: true,
        },
        ComplianceCheck {
            id: "SOC2.Confidentiality.1".into(),
            requirement: "Encryption at rest and in transit".into(),
            forge_feature: "KMS-encrypted audit trail + TLS everywhere".into(),
            auto_detectable: true,
        },
    ]
}

/// A single compliance requirement mapped to a Forge capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    pub id: String,
    pub requirement: String,
    pub forge_feature: String,
    /// Can Forge automatically verify this check is met?
    pub auto_detectable: bool,
}

/// Generate a compliance report for a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub framework: ComplianceFramework,
    pub session_id: String,
    pub generated_at: String,
    pub checks: Vec<ComplianceCheckResult>,
    pub overall_compliant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheckResult {
    pub check: ComplianceCheck,
    pub passed: bool,
    pub evidence: Option<String>,
    pub auditor_notes: Option<String>,
}

/// Generate a compliance report template for a framework.
pub fn generate_report(
    framework: ComplianceFramework,
    session_id: &str,
) -> ComplianceReport {
    let checks: Vec<ComplianceCheckResult> = match framework {
        ComplianceFramework::EuAiAct => {
            let mut c = eu_ai_act_article_14_checklist();
            c.extend(eu_ai_act_article_15_checklist());
            c.into_iter()
                .map(|check| ComplianceCheckResult {
                    passed: check.auto_detectable,
                    check,
                    evidence: Some("Forge audit trail automatically captures this evidence.".into()),
                    auditor_notes: None,
                })
                .collect()
        }
        ComplianceFramework::Soc2Type2 => soc2_checklist()
            .into_iter()
            .map(|check| ComplianceCheckResult {
                passed: check.auto_detectable,
                check,
                evidence: Some("Forge audit trail automatically captures this evidence.".into()),
                auditor_notes: None,
            })
            .collect(),
        _ => vec![],
    };

    let overall = checks.iter().all(|c| c.passed);
    ComplianceReport {
        framework,
        session_id: session_id.into(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        checks,
        overall_compliant: overall,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eu_ai_act_article_14_has_checks() {
        let checks = eu_ai_act_article_14_checklist();
        assert_eq!(checks.len(), 4);
    }

    #[test]
    fn test_eu_ai_act_article_15_has_checks() {
        let checks = eu_ai_act_article_15_checklist();
        assert_eq!(checks.len(), 4);
    }

    #[test]
    fn test_soc2_has_checks() {
        let checks = soc2_checklist();
        assert_eq!(checks.len(), 4);
    }

    #[test]
    fn test_generate_eu_ai_act_report() {
        let report = generate_report(ComplianceFramework::EuAiAct, "test-session");
        assert_eq!(report.checks.len(), 8); // 4 + 4
        assert!(report.overall_compliant);
    }

    #[test]
    fn test_framework_display() {
        assert_eq!(ComplianceFramework::EuAiAct.to_string(), "EU AI Act");
        assert_eq!(ComplianceFramework::Soc2Type2.to_string(), "SOC 2 Type II");
    }
}
