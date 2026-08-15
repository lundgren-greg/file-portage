//! Redaction of secrets before anything reaches a log sink.
//!
//! Enforced in the subscriber layer, not by call-site convention: a hostile
//! or careless `tracing::info!(token = …)` must still produce a masked line.

/// Replacement for a redacted value.
pub const MASK: &str = "[REDACTED]";

/// Field names whose values are always masked, matched case-insensitively as
/// substrings (`access_token`, `Authorization`, `client_secret`, …).
const SENSITIVE_NAMES: &[&str] = &["token", "authorization", "secret", "password", "api_key"];

/// URL-ish field names whose query string is stripped (Graph `session_uri`
/// download URLs are preauthenticated — the query *is* the credential).
const URL_NAMES: &[&str] = &["session_uri", "url", "uri", "href"];

/// True if a value recorded under `name` must be fully masked.
pub fn is_sensitive_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    SENSITIVE_NAMES.iter().any(|s| lower.contains(s))
}

/// Redact a string value recorded under `name`.
///
/// - Sensitive names are fully masked.
/// - URL-ish names lose their query string.
/// - Any value is scrubbed of inline `Bearer <credential>` sequences.
pub fn redact_value(name: &str, value: &str) -> String {
    if is_sensitive_name(name) {
        return MASK.to_string();
    }
    let value = if URL_NAMES.iter().any(|s| name.eq_ignore_ascii_case(s)) {
        strip_query(value)
    } else {
        value.to_string()
    };
    scrub_bearer(&value)
}

/// Drop everything from the first `?` on.
fn strip_query(value: &str) -> String {
    match value.split_once('?') {
        Some((base, _)) => format!("{base}?{MASK}"),
        None => value.to_string(),
    }
}

/// Replace the credential after any `Bearer ` with the mask.
fn scrub_bearer(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    loop {
        match rest.find("Bearer ") {
            Some(idx) => {
                let after = idx + "Bearer ".len();
                out.push_str(&rest[..after]);
                out.push_str(MASK);
                let tail = &rest[after..];
                let end = tail
                    .find(|c: char| c.is_whitespace() || c == '"' || c == '\'')
                    .unwrap_or(tail.len());
                rest = &tail[end..];
            }
            None => {
                out.push_str(rest);
                return out;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensitive_names_are_masked() {
        for name in [
            "token",
            "access_token",
            "refresh_token",
            "Authorization",
            "client_secret",
            "password",
            "XAI_API_KEY",
        ] {
            assert_eq!(redact_value(name, "hunter2"), MASK, "name: {name}");
        }
    }

    #[test]
    fn url_query_strings_are_stripped() {
        assert_eq!(
            redact_value("session_uri", "https://graph.example/dl?sig=SECRET&x=1"),
            format!("https://graph.example/dl?{MASK}")
        );
        assert_eq!(
            redact_value("url", "https://example.com/plain"),
            "https://example.com/plain"
        );
    }

    #[test]
    fn inline_bearer_credentials_are_scrubbed() {
        assert_eq!(
            redact_value("msg", "sending Authorization: Bearer eyJhbGciOi to api"),
            format!("sending Authorization: Bearer {MASK} to api")
        );
        assert_eq!(
            redact_value("msg", "Bearer abc Bearer def"),
            format!("Bearer {MASK} Bearer {MASK}")
        );
    }

    #[test]
    fn ordinary_values_pass_through() {
        assert_eq!(redact_value("plan_id", "file-plan-7f3c"), "file-plan-7f3c");
        assert_eq!(redact_value("size", "1073741824"), "1073741824");
    }
}
