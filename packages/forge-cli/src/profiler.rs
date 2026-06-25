use std::time::Instant;

pub struct Profiler {
    #[allow(dead_code)]
    start: Instant,
    measurements: Vec<(String, f64)>,
}

impl Profiler {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self { Self { start: Instant::now(), measurements: Vec::new() } }
    pub fn measure(&mut self, label: &str, f: impl FnOnce()) {
        let t0 = Instant::now(); f(); let elapsed = t0.elapsed().as_micros() as f64;
        self.measurements.push((label.into(), elapsed));
    }
    pub fn report(&self) -> String {
        let mut out = String::from("Forge Performance Profile\n═══════════════════════\n");
        let total: f64 = self.measurements.iter().map(|(_, t)| t).sum();
        for (label, time) in &self.measurements {
            out.push_str(&format!("  {}: {:>8.0}μs ({:.1}%)\n", label, time, time / total * 100.0));
        }
        out.push_str(&format!("  Total: {:>8.0}μs\n", total));
        out.push_str(&format!("  Overhead per turn: ~{}μs\n", (total / self.measurements.len().max(1) as f64) as u64));
        out
    }
}
