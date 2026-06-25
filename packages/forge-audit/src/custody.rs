use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyEntry {
    pub id: Uuid,
    pub session_id: Uuid,
    pub user: String,
    pub action: CustodyAction,
    pub timestamp: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustodyAction {
    Viewed,
    Exported(String),
    Modified,
    Accessed,
}

pub struct ChainOfCustody {
    entries: Vec<CustodyEntry>,
}

impl ChainOfCustody {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
    pub fn log(&mut self, session_id: Uuid, user: &str, action: CustodyAction) {
        self.entries.push(CustodyEntry {
            id: Uuid::new_v4(),
            session_id,
            user: user.into(),
            action,
            timestamp: Utc::now(),
        });
    }
    pub fn for_session(&self, session_id: &Uuid) -> Vec<&CustodyEntry> {
        self.entries
            .iter()
            .filter(|e| &e.session_id == session_id)
            .collect()
    }
    pub fn all(&self) -> &[CustodyEntry] {
        &self.entries
    }
}
