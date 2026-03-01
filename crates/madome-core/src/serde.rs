// Module name shadows the `serde` crate — use `::serde` for the external crate.
use ::serde::Serializer;
use chrono::{DateTime, SecondsFormat, Utc};

/// Serialize `DateTime<Utc>` as RFC 3339 with 3-digit fractional seconds.
/// Matches legacy extra-rs implementation exactly.
pub fn to_rfc3339_ms<S>(dt: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&dt.to_rfc3339_opts(SecondsFormat::Millis, true))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn should_format_datetime_as_rfc3339_with_millis() {
        let dt = Utc.with_ymd_and_hms(2023, 2, 11, 11, 9, 0).unwrap();
        let result = dt.to_rfc3339_opts(SecondsFormat::Millis, true);
        assert_eq!(result, "2023-02-11T11:09:00.000Z");
    }
}
