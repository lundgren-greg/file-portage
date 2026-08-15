//! Observability: structured local logging and in-process metrics.
//!
//! Design (`docs/design.md`, Observability): local files only — a user-run
//! Grafana Alloy / Promtail / textfile collector can scrape them. Portage
//! itself never opens a port, never pushes, never sends telemetry.

pub mod logging;
pub mod metrics;
pub mod redact;

pub use logging::init_tracing;
pub use metrics::MetricsRegistry;
