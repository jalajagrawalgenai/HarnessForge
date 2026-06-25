use forge_sdk::types::audit::Checkpoint;
use uuid::Uuid;

pub struct CheckpointStore { checkpoints: Vec<Checkpoint> }

impl CheckpointStore {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self { Self { checkpoints: Vec::new() } }
    pub fn save(&mut self, cp: Checkpoint) { self.checkpoints.push(cp); }
    pub fn load(&self, id: &Uuid) -> Option<&Checkpoint> { self.checkpoints.iter().find(|c| &c.id == id) }
    pub fn list(&self) -> &[Checkpoint] { &self.checkpoints }
}
