//! JSON-lines tracing layer with enforced redaction.
//!
//! One event per line: `{"ts","level","target","msg",…fields}` — directly
//! scrapeable by a local Grafana Alloy / Promtail → Loki pipeline. Rolling
//! file `%data_dir%/logs/portage.YYYY-MM-DD.jsonl`, 7 files retained.

use std::io;
use std::path::Path;

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

use super::redact;

/// Number of daily log files kept.
pub const MAX_LOG_FILES: usize = 7;

/// Install the global subscriber: JSONL to `data_dir/logs/` plus a
/// human-readable stderr layer honoring `RUST_LOG` (default `warn`, or
/// `debug` when `verbose`).
///
/// Returns a guard that must stay alive for the process lifetime so the
/// non-blocking writer flushes on exit.
pub fn init_tracing(
    data_dir: &Path,
    verbose: bool,
) -> io::Result<tracing_appender::non_blocking::WorkerGuard> {
    let logs = data_dir.join("logs");
    std::fs::create_dir_all(&logs)?;
    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("portage")
        .filename_suffix("jsonl")
        .max_log_files(MAX_LOG_FILES)
        .build(&logs)
        .map_err(io::Error::other)?;
    let (writer, guard) = tracing_appender::non_blocking(appender);

    let default = if verbose { "debug" } else { "warn" };
    let stderr_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default));

    tracing_subscriber::registry()
        .with(JsonlLayer::new(writer))
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(io::stderr)
                .with_filter(stderr_filter),
        )
        .try_init()
        .map_err(io::Error::other)?;
    Ok(guard)
}

/// A `tracing_subscriber::Layer` that serializes every event as one JSON
/// line, passing each field through [`redact::redact_value`] first.
pub struct JsonlLayer<W> {
    make_writer: W,
}

impl<W> JsonlLayer<W> {
    /// Wrap a writer factory (e.g. a rolling file appender).
    pub fn new(make_writer: W) -> Self {
        Self { make_writer }
    }
}

impl<S, W> Layer<S> for JsonlLayer<W>
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    W: for<'a> MakeWriter<'a> + 'static,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut map = serde_json::Map::new();
        let ts = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_default();
        map.insert("ts".into(), ts.into());
        map.insert("level".into(), event.metadata().level().to_string().into());
        map.insert("target".into(), event.metadata().target().into());

        let mut visitor = RedactingVisitor { map: &mut map };
        event.record(&mut visitor);

        let mut writer = self.make_writer.make_writer();
        let _ = serde_json::to_writer(&mut writer, &serde_json::Value::Object(map));
        let _ = io::Write::write_all(&mut writer, b"\n");
    }
}

/// Field visitor that applies redaction and renames `message` to `msg`.
struct RedactingVisitor<'a> {
    map: &'a mut serde_json::Map<String, serde_json::Value>,
}

impl RedactingVisitor<'_> {
    fn key(field: &Field) -> String {
        if field.name() == "message" {
            "msg".into()
        } else {
            field.name().into()
        }
    }
}

impl Visit for RedactingVisitor<'_> {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.map.insert(
            Self::key(field),
            redact::redact_value(field.name(), value).into(),
        );
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        if redact::is_sensitive_name(field.name()) {
            self.map.insert(Self::key(field), redact::MASK.into());
        } else {
            self.map.insert(Self::key(field), value.into());
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        if redact::is_sensitive_name(field.name()) {
            self.map.insert(Self::key(field), redact::MASK.into());
        } else {
            self.map.insert(Self::key(field), value.into());
        }
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.map.insert(Self::key(field), value.into());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.map.insert(Self::key(field), value.into());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.map.insert(
            Self::key(field),
            redact::redact_value(field.name(), &format!("{value:?}")).into(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tracing::subscriber::with_default;

    /// In-memory writer to capture JSONL output in tests.
    #[derive(Clone, Default)]
    struct Capture(Arc<Mutex<Vec<u8>>>);

    impl io::Write for Capture {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for Capture {
        type Writer = Capture;
        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    fn capture_lines(f: impl FnOnce()) -> Vec<serde_json::Value> {
        let cap = Capture::default();
        let subscriber = tracing_subscriber::registry().with(JsonlLayer::new(cap.clone()));
        with_default(subscriber, f);
        let bytes = cap.0.lock().unwrap().clone();
        String::from_utf8(bytes)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).expect("line is valid JSON"))
            .collect()
    }

    #[test]
    fn events_serialize_with_stable_fields() {
        let lines = capture_lines(|| {
            tracing::info!(plan_id = "file-plan-7f3c", size = 42_u64, "op committed");
        });
        assert_eq!(lines.len(), 1);
        let line = &lines[0];
        assert_eq!(line["level"], "INFO");
        assert_eq!(line["msg"], "op committed");
        assert_eq!(line["plan_id"], "file-plan-7f3c");
        assert_eq!(line["size"], 42);
        assert!(line["ts"].as_str().unwrap().contains('T'), "ts is RFC 3339");
        assert!(line["target"].as_str().is_some());
    }

    #[test]
    fn hostile_events_never_leak_secrets() {
        let secret = "eyJhbGciOiJIUzI1NiJ9.SECRET";
        let lines = capture_lines(|| {
            tracing::info!(
                token = secret,
                access_token = secret,
                authorization = "Bearer eyJhbGciOiJIUzI1NiJ9.SECRET",
                session_uri = "https://graph.example/dl?sig=SECRET",
                "connected: Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.SECRET done"
            );
        });
        let raw = serde_json::to_string(&lines).unwrap();
        assert!(!raw.contains("SECRET"), "secret leaked: {raw}");
        assert_eq!(lines[0]["token"], redact::MASK);
        assert_eq!(lines[0]["access_token"], redact::MASK);
        assert_eq!(lines[0]["authorization"], redact::MASK);
    }

    #[test]
    fn numeric_sensitive_fields_are_masked_too() {
        let lines = capture_lines(|| {
            tracing::info!(token = 12345_u64, "weird but must not leak");
        });
        assert_eq!(lines[0]["token"], redact::MASK);
    }
}
