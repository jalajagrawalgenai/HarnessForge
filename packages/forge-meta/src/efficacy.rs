use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorEfficacy {
    pub name: String,
    pub fire_count: u64,
    pub true_positives: u64,
    pub false_positives: u64,
    pub precision: f64,
    pub recall: f64,
    pub avg_confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyEfficacy {
    pub name: String,
    pub application_count: u64,
    pub success_rate: f64,
    pub avg_improvement: f64,
    pub make_worse_rate: f64,
}

pub struct EfficacyReport;

impl EfficacyReport {
    pub fn detector_report(
        name: &str,
        fires: u64,
        tp: u64,
        fp: u64,
        avg_conf: f64,
    ) -> DetectorEfficacy {
        let total = tp + fp;
        DetectorEfficacy {
            name: name.into(),
            fire_count: fires,
            true_positives: tp,
            false_positives: fp,
            precision: if total > 0 {
                tp as f64 / total as f64
            } else {
                0.0
            },
            recall: if fires > 0 {
                tp as f64 / fires as f64
            } else {
                0.0
            },
            avg_confidence: avg_conf,
        }
    }
    pub fn strategy_report(
        name: &str,
        apps: u64,
        success: f64,
        improvement: f64,
        worse: f64,
    ) -> StrategyEfficacy {
        StrategyEfficacy {
            name: name.into(),
            application_count: apps,
            success_rate: success,
            avg_improvement: improvement,
            make_worse_rate: worse,
        }
    }
}
