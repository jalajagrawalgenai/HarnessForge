use crate::harness_proposer::HarnessEdit;
use chrono::Utc;
use forge_sdk::error::ForgeError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HarnessVersion {
    pub version: String,
    pub created_at: chrono::DateTime<Utc>,
    pub change_summary: String,
    pub improvement_pct: Option<f64>,
}

#[derive(Default)]
pub struct EditRegistry {
    versions: Vec<HarnessVersion>,
    applied_edits: Vec<HarnessEdit>,
    major: u32,
    minor: u32,
}

impl EditRegistry {
    pub fn new() -> Self {
        Self {
            versions: vec![HarnessVersion {
                version: "v1.0.0".into(),
                created_at: Utc::now(),
                change_summary: "Initial harness".into(),
                improvement_pct: None,
            }],
            applied_edits: Vec::new(),
            major: 1,
            minor: 0,
        }
    }

    pub fn current_version(&self) -> String {
        self.versions
            .last()
            .map(|v| v.version.clone())
            .unwrap_or_else(|| "v1.0.0".into())
    }

    pub fn apply(&mut self, edit: HarnessEdit) -> Result<(), ForgeError> {
        self.minor += 1;
        let version = format!("v{}.{}.0", self.major, self.minor);
        self.versions.push(HarnessVersion {
            version: version.clone(),
            created_at: Utc::now(),
            change_summary: edit.rationale.clone(),
            improvement_pct: None,
        });
        self.applied_edits.push(edit);
        Ok(())
    }

    pub fn bump_major(&mut self, edit: HarnessEdit) -> Result<(), ForgeError> {
        self.major += 1;
        self.minor = 0;
        let version = format!("v{}.{}.0", self.major, self.minor);
        self.versions.push(HarnessVersion {
            version: version.clone(),
            created_at: Utc::now(),
            change_summary: format!("MAJOR: {}", edit.rationale),
            improvement_pct: None,
        });
        self.applied_edits.push(edit);
        Ok(())
    }

    pub fn rollback(&mut self) -> Result<String, ForgeError> {
        if self.versions.len() <= 1 {
            return Err(ForgeError::Internal(
                "No previous version to rollback to".into(),
            ));
        }
        // Remove latest
        self.versions.pop();
        if !self.applied_edits.is_empty() {
            self.applied_edits.pop();
        }
        if self.minor > 0 {
            self.minor -= 1;
        }
        Ok(self.current_version())
    }

    pub fn versions(&self) -> &[HarnessVersion] {
        &self.versions
    }
    pub fn edits(&self) -> &[HarnessEdit] {
        &self.applied_edits
    }
    pub fn total_edits(&self) -> usize {
        self.applied_edits.len()
    }

    /// Show diff between two versions
    pub fn diff(&self, v1: &str, v2: &str) -> Option<Vec<&HarnessEdit>> {
        let i1 = self.versions.iter().position(|v| v.version == v1)?;
        let i2 = self.versions.iter().position(|v| v.version == v2)?;
        let (start, end) = if i1 < i2 { (i1, i2) } else { (i2, i1) };
        Some(
            self.applied_edits[start..end.min(self.applied_edits.len())]
                .iter()
                .collect(),
        )
    }
}
