use forge_sdk::types::audit::AuditEvent;
use sha2::{Digest, Sha256};

pub fn compute_hash_chain(events: &[AuditEvent], genesis: &str) -> Vec<String> {
    let mut hashes = Vec::with_capacity(events.len());
    let mut prev = genesis.to_string();
    for event in events {
        let mut hasher = Sha256::new();
        hasher.update(prev.as_bytes());
        hasher.update(
            serde_json::to_string(&event.event_data)
                .unwrap_or_default()
                .as_bytes(),
        );
        let hash = hex::encode(hasher.finalize());
        hashes.push(hash.clone());
        prev = hash;
    }
    hashes
}

pub fn verify_integrity(events: &[AuditEvent], genesis: &str) -> bool {
    if events.is_empty() {
        return true;
    }
    let mut hasher = Sha256::new();
    hasher.update(genesis.as_bytes());
    let mut expected = hex::encode(hasher.finalize());
    for event in events {
        let mut h = Sha256::new();
        h.update(expected.as_bytes());
        h.update(
            serde_json::to_string(&event.event_data)
                .unwrap_or_default()
                .as_bytes(),
        );
        let computed = hex::encode(h.finalize());
        if computed != event.hash_chain {
            return false;
        }
        expected = computed;
    }
    true
}
