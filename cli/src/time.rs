//! The one clock the binary reads. `DEVFORGEAI_NOW` replaces it under the
//! `test-home` feature and is read by no code path otherwise.

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

/// The current instant as an RFC 3339 UTC string with second precision, the
/// form every timestamp the CLI writes takes.
pub fn now_rfc3339() -> String {
    format(now())
}

/// The current instant.
pub fn now() -> OffsetDateTime {
    #[cfg(feature = "test-home")]
    {
        if let Some(v) = std::env::var_os("DEVFORGEAI_NOW") {
            let s = v.to_string_lossy().trim().to_string();
            if !s.is_empty() {
                if let Ok(t) = OffsetDateTime::parse(&s, &Rfc3339) {
                    return t.to_offset(time::UtcOffset::UTC);
                }
            }
        }
    }
    OffsetDateTime::now_utc()
}

/// Render an instant in the RFC 3339 UTC form the CLI writes.
pub fn format(t: OffsetDateTime) -> String {
    let t = t
        .to_offset(time::UtcOffset::UTC)
        .replace_nanosecond(0)
        .unwrap_or(t);
    t.format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
        .replace("+00:00", "Z")
}

/// The RFC 3339 basic form the settings backup file name takes, as in
/// `20260910T140211Z`.
pub fn now_basic() -> String {
    let t = now().to_offset(time::UtcOffset::UTC);
    format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
        t.year(),
        u8::from(t.month()),
        t.day(),
        t.hour(),
        t.minute(),
        t.second()
    )
}

/// Parse an RFC 3339 timestamp, returning `None` on any other shape.
pub fn parse_rfc3339(s: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(s.trim(), &Rfc3339).ok()
}

/// Whole calendar days from `from` to `to`, counted on the UTC date alone.
pub fn calendar_days_between(from: OffsetDateTime, to: OffsetDateTime) -> i64 {
    let a = from.to_offset(time::UtcOffset::UTC).date();
    let b = to.to_offset(time::UtcOffset::UTC).date();
    (b.to_julian_day() - a.to_julian_day()) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_renders_a_z_suffix_with_second_precision() {
        let t = parse_rfc3339("2026-09-10T14:02:11.512Z").expect("parse");
        assert_eq!(format(t), "2026-09-10T14:02:11Z");
    }

    #[test]
    fn calendar_days_counts_dates_not_durations() {
        let a = parse_rfc3339("2026-09-06T23:59:00Z").expect("parse");
        let b = parse_rfc3339("2026-09-07T00:01:00Z").expect("parse");
        assert_eq!(calendar_days_between(a, b), 1);
    }

    #[cfg(feature = "test-home")]
    #[test]
    fn now_honours_devforgeai_now_under_the_test_feature() {
        // Serialised against the other environment-reading test by the
        // variable name alone: no other test sets DEVFORGEAI_NOW.
        std::env::set_var("DEVFORGEAI_NOW", "2026-01-02T03:04:05Z");
        assert_eq!(now_rfc3339(), "2026-01-02T03:04:05Z");
        std::env::remove_var("DEVFORGEAI_NOW");
    }
}
