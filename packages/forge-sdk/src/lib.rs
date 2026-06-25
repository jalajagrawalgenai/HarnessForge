// forge-sdk/src/lib.rs — Public API for the Forge agent harness

pub mod agent;
pub mod error;
pub mod events;
pub mod harness;
pub mod prelude;
pub mod presets;
pub mod traits;
pub mod types;

// Re-exports for convenient access
pub use agent::AgentAdapter;
pub use agent::AgentType;
pub use agent::BridgeMethod;
pub use agent::MockAgent;
pub use harness::{Harness, HarnessBuilder, HarnessConfig, HarnessRunResult, HarnessRuntime};
pub use prelude::*;
pub use presets::Preset;
