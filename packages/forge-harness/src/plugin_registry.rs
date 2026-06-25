// forge-harness/src/plugin_registry.rs — Observer/Detector/Strategy registry

use forge_sdk::traits::detector::Detector;
use forge_sdk::traits::observer::Observer;
use forge_sdk::traits::strategy::Strategy;
use std::collections::HashMap;
use std::sync::Arc;

/// Manages registration and lookup of all harness plugins
#[derive(Default)]
pub struct PluginRegistry {
    observers: HashMap<String, Arc<dyn Observer>>,
    detectors: HashMap<String, Arc<dyn Detector>>,
    strategies: Vec<Arc<dyn Strategy>>, // Sorted by priority
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    // ─── Observers ───

    pub fn register_observer(&mut self, observer: Arc<dyn Observer>) {
        self.observers.insert(observer.name().to_string(), observer);
    }

    pub fn observers(&self) -> Vec<&Arc<dyn Observer>> {
        self.observers.values().collect()
    }

    pub fn observer(&self, name: &str) -> Option<&Arc<dyn Observer>> {
        self.observers.get(name)
    }

    // ─── Detectors ───

    pub fn register_detector(&mut self, detector: Arc<dyn Detector>) {
        self.detectors.insert(detector.name().to_string(), detector);
    }

    pub fn detectors(&self) -> Vec<&Arc<dyn Detector>> {
        self.detectors.values().collect()
    }

    pub fn detector(&self, name: &str) -> Option<&Arc<dyn Detector>> {
        self.detectors.get(name)
    }

    // ─── Strategies ───

    pub fn register_strategy(&mut self, strategy: Arc<dyn Strategy>) {
        self.strategies.push(strategy);
        // Keep sorted by priority (highest first)
        self.strategies
            .sort_by_key(|s| std::cmp::Reverse(s.priority()));
    }

    pub fn strategies(&self) -> &[Arc<dyn Strategy>] {
        &self.strategies
    }
}

/// Macro-style preset registration
pub struct PresetRegistrar;

impl PresetRegistrar {
    /// Register all plugins for Solo mode
    pub fn register_solo_preset(_registry: &mut PluginRegistry) {
        // Observers will be added in forge-observers
        // Detectors will be added in forge-detectors
        // Strategies will be added in forge-strategies
    }
}
