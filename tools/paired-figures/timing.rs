//! Process CPU and wall-clock phase metrics, separate from proof result bytes.
use std::{sync::OnceLock, time::Instant};

fn cpu_seconds() -> Option<f64> {
    static TICKS: OnceLock<Option<f64>> = OnceLock::new();
    let ticks = (*TICKS.get_or_init(|| {
        let output = std::process::Command::new("getconf").arg("CLK_TCK").output().ok()?;
        if !output.status.success() {
            return None;
        }
        let value = std::str::from_utf8(&output.stdout).ok()?.trim().parse::<f64>().ok()?;
        (value.is_finite() && value > 0.0).then_some(value)
    }))?;
    let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
    let (_, fields) = stat.rsplit_once(") ")?;
    let fields: Vec<_> = fields.split_whitespace().collect();
    let user = fields.get(11)?.parse::<u64>().ok()?;
    let system = fields.get(12)?.parse::<u64>().ok()?;
    let children_user = fields.get(13)?.parse::<u64>().ok()?;
    let children_system = fields.get(14)?.parse::<u64>().ok()?;
    Some(
        user.checked_add(system)?.checked_add(children_user)?.checked_add(children_system)? as f64
            / ticks,
    )
}

pub(crate) struct PhaseTimer {
    phase: String,
    start: Instant,
    cpu: Option<f64>,
}
impl PhaseTimer {
    pub(crate) fn new(phase: impl Into<String>) -> Self {
        Self { phase: phase.into(), start: Instant::now(), cpu: cpu_seconds() }
    }
    pub(crate) fn finish(self) {
        let wall = self.start.elapsed().as_secs_f64();
        let cpu = self.cpu.zip(cpu_seconds()).map(|(start, end)| (end - start).max(0.0));
        let percent = cpu.and_then(|seconds| (wall > 0.0).then_some(seconds / wall * 100.0));
        eprintln!(
            "{}",
            serde_json::json!({
                "phase": self.phase, "wall_seconds": wall, "cpu_seconds": cpu,
                "cpu_percent": percent, "cpu_scope": "process_and_waited_children",
            })
        );
    }
}
