use std::sync::Arc;
use crate::harness::HarnessBuilder;

pub enum Preset { Solo, LangGraph, CrewAI, AutoGen, Custom }

impl Preset {
    pub fn apply(&self, builder: HarnessBuilder) -> HarnessBuilder {
        match self {
            Preset::Solo => builder,
            Preset::LangGraph => builder,
            Preset::CrewAI => builder,
            Preset::AutoGen => builder,
            Preset::Custom => builder,
        }
    }
}
