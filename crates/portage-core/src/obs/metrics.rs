//! In-process metrics with Prometheus text export.
//!
//! No listener, no push: `to_prometheus()` renders the text exposition
//! format, and [`MetricsRegistry::write_prom`] atomically rewrites
//! `%data_dir%/metrics/portage.prom` for a local textfile collector.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io;
use std::path::Path;
use std::sync::Mutex;

/// Counter and gauge registry. Cheap to clone values out; thread-safe.
#[derive(Debug, Default)]
pub struct MetricsRegistry {
    counters: Mutex<BTreeMap<String, u64>>,
    gauges: Mutex<BTreeMap<String, f64>>,
}

impl MetricsRegistry {
    /// New empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add `by` to a monotonically increasing counter.
    pub fn inc(&self, name: &str, by: u64) {
        let mut counters = self.counters.lock().expect("metrics lock");
        *counters.entry(name.to_string()).or_insert(0) += by;
    }

    /// Set a gauge to an absolute value.
    pub fn set_gauge(&self, name: &str, value: f64) {
        self.gauges
            .lock()
            .expect("metrics lock")
            .insert(name.to_string(), value);
    }

    /// Current counter value (0 if never incremented).
    pub fn counter(&self, name: &str) -> u64 {
        *self
            .counters
            .lock()
            .expect("metrics lock")
            .get(name)
            .unwrap_or(&0)
    }

    /// Render the Prometheus text exposition format. Metric names use the
    /// design's dotted names with dots mapped to underscores and a
    /// `portage_` prefix (`index.files_seen` → `portage_index_files_seen`).
    pub fn to_prometheus(&self) -> String {
        let mut out = String::new();
        for (name, value) in self.counters.lock().expect("metrics lock").iter() {
            let prom = prom_name(name);
            let _ = writeln!(out, "# TYPE {prom} counter");
            let _ = writeln!(out, "{prom} {value}");
        }
        for (name, value) in self.gauges.lock().expect("metrics lock").iter() {
            let prom = prom_name(name);
            let _ = writeln!(out, "# TYPE {prom} gauge");
            let _ = writeln!(out, "{prom} {value}");
        }
        out
    }

    /// Atomically rewrite `dir/metrics/portage.prom` (write temp + rename)
    /// so a scraper never observes a torn file.
    pub fn write_prom(&self, data_dir: &Path) -> io::Result<()> {
        let metrics_dir = data_dir.join("metrics");
        std::fs::create_dir_all(&metrics_dir)?;
        let tmp = metrics_dir.join("portage.prom.tmp");
        let dest = metrics_dir.join("portage.prom");
        std::fs::write(&tmp, self.to_prometheus())?;
        std::fs::rename(&tmp, &dest)?;
        Ok(())
    }
}

/// Map a dotted metric name to a valid Prometheus name.
fn prom_name(name: &str) -> String {
    let mut prom = String::with_capacity(name.len() + 8);
    prom.push_str("portage_");
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            prom.push(c);
        } else {
            prom.push('_');
        }
    }
    prom
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_accumulate_and_gauges_overwrite() {
        let m = MetricsRegistry::new();
        m.inc("index.files_seen", 2);
        m.inc("index.files_seen", 3);
        m.set_gauge("space.free", 4.0);
        m.set_gauge("space.free", 1.5);
        assert_eq!(m.counter("index.files_seen"), 5);
        let text = m.to_prometheus();
        assert!(text.contains("portage_index_files_seen 5"));
        assert!(text.contains("portage_space_free 1.5"));
    }

    #[test]
    fn prometheus_output_is_well_formed() {
        let m = MetricsRegistry::new();
        m.inc("apply.bytes_copied", 1024);
        m.set_gauge("space.free", 8589934592.0);
        for line in m.to_prometheus().lines() {
            if line.starts_with('#') {
                assert!(line.starts_with("# TYPE portage_"), "bad help line: {line}");
            } else {
                let mut parts = line.split_whitespace();
                let name = parts.next().unwrap();
                let value = parts.next().unwrap();
                assert!(parts.next().is_none());
                assert!(name.starts_with("portage_"));
                assert!(
                    name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'),
                    "invalid prom name: {name}"
                );
                assert!(value.parse::<f64>().is_ok(), "invalid value: {value}");
            }
        }
    }

    #[test]
    fn write_prom_is_atomic_rewrite() {
        let tmp = tempfile::tempdir().unwrap();
        let m = MetricsRegistry::new();
        m.inc("plan.ops", 7);
        m.write_prom(tmp.path()).unwrap();
        let dest = tmp.path().join("metrics").join("portage.prom");
        let first = std::fs::read_to_string(&dest).unwrap();
        assert!(first.contains("portage_plan_ops 7"));

        m.inc("plan.ops", 1);
        m.write_prom(tmp.path()).unwrap();
        let second = std::fs::read_to_string(&dest).unwrap();
        assert!(second.contains("portage_plan_ops 8"));
        assert!(
            !tmp.path().join("metrics").join("portage.prom.tmp").exists(),
            "temp file left behind"
        );
    }
}
