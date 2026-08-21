//! Common DTO types shared across entity DTOs

use serde::{Deserialize, Deserializer};
use utoipa::ToSchema;
use chrono::NaiveDateTime;

/// Parse an i32 from a serde_json::Value, accepting either a number or a string-encoded
/// integer. React frontends send `{"id": "1"}` (string) for multi-select form values,
/// while Angular sends `{"id": 1}` (number). The backend must accept both.
fn parse_i32_value<E: serde::de::Error>(value: &serde_json::Value) -> Result<i32, E> {
    match value {
        serde_json::Value::Number(n) => n
            .as_i64()
            .and_then(|n| i32::try_from(n).ok())
            .ok_or_else(|| E::custom("id is not a valid i32")),
        serde_json::Value::String(s) => s.parse::<i32>().map_err(E::custom),
        _ => Err(E::custom("id must be a number or string")),
    }
}

/// Helper struct to deserialize relationship references that can be either:
/// - A plain integer ID: 1 or "1"
/// - An object with an id field: {"id": 1, "name": "..."} or {"id": "1", ...}
#[derive(Debug, Clone, ToSchema)]
pub struct RelationshipId(pub i32);

impl<'de> Deserialize<'de> for RelationshipId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let value = serde_json::Value::deserialize(deserializer)?;
        let id = match &value {
            serde_json::Value::Object(map) => {
                let id_value = map
                    .get("id")
                    .ok_or_else(|| D::Error::custom("missing id field"))?;
                parse_i32_value::<D::Error>(id_value)?
            }
            _ => parse_i32_value::<D::Error>(&value)?,
        };
        Ok(RelationshipId(id))
    }
}

/// Custom deserializer for optional relationship fields
/// Handles: null, plain integer/string ID, or object with id field (number or string)
pub fn deserialize_optional_relationship<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let opt: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::Object(map)) => {
            let id_value = map
                .get("id")
                .ok_or_else(|| D::Error::custom("missing id field"))?;
            Ok(Some(parse_i32_value::<D::Error>(id_value)?))
        }
        Some(value) => Ok(Some(parse_i32_value::<D::Error>(&value)?)),
    }
}

/// Custom deserializer for NaiveDateTime that handles ISO 8601 format from frontend
pub fn deserialize_naive_datetime<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    // Try ISO 8601 format first (from Angular frontend)
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&s) {
        return Ok(dt.naive_utc());
    }
    // Try parsing as DateTime with Z suffix
    if let Ok(dt) = chrono::DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.fZ") {
        return Ok(dt.naive_utc());
    }
    // Try standard NaiveDateTime format
    if let Ok(dt) = NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f") {
        return Ok(dt);
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(dt);
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt);
    }
    Err(serde::de::Error::custom(format!("Unable to parse datetime: {}", s)))
}

/// Custom deserializer for Option<NaiveDateTime> that handles ISO 8601 format from frontend
pub fn deserialize_option_naive_datetime<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(s) if s.is_empty() => Ok(None),
        Some(s) => {
            // Try ISO 8601 format first (from Angular frontend)
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&s) {
                return Ok(Some(dt.naive_utc()));
            }
            // Try parsing as DateTime with Z suffix
            if let Ok(dt) = chrono::DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.fZ") {
                return Ok(Some(dt.naive_utc()));
            }
            // Try standard NaiveDateTime format
            if let Ok(dt) = NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f") {
                return Ok(Some(dt));
            }
            if let Ok(dt) = NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S") {
                return Ok(Some(dt));
            }
            if let Ok(dt) = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S") {
                return Ok(Some(dt));
            }
            Err(serde::de::Error::custom(format!("Unable to parse datetime: {}", s)))
        }
    }
}

// Track 1 Phase 1a (2026-05-11): tests target the SQL variant deserializers. These
// are pure-Rust unit tests — no DB, no async, no fixtures. Covers the React/Angular
// asymmetry the rationale comments document: React form values send `{"id": "1"}`
// (string), Angular sends `{"id": 1}` (number). Both must round-trip; null and
// malformed inputs must reject with informative errors.
#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct OptionalRelWrapper {
        #[serde(deserialize_with = "deserialize_optional_relationship", default)]
        rel: Option<i32>,
    }

    #[derive(Debug, Deserialize)]
    struct DateTimeWrapper {
        #[serde(deserialize_with = "deserialize_naive_datetime")]
        ts: NaiveDateTime,
    }

    #[derive(Debug, Deserialize)]
    struct OptionalDateTimeWrapper {
        #[serde(deserialize_with = "deserialize_option_naive_datetime", default)]
        ts: Option<NaiveDateTime>,
    }

    #[test]
    fn test_relationship_id_accepts_integer_form() {
        // Angular form shape: {"id": 1}
        let rid: RelationshipId = serde_json::from_str("1").unwrap();
        assert_eq!(rid.0, 1);
    }

    #[test]
    fn test_relationship_id_accepts_string_form() {
        // React form shape: "1" — the asymmetry the comment documents.
        let rid: RelationshipId = serde_json::from_str("\"1\"").unwrap();
        assert_eq!(rid.0, 1);
    }

    #[test]
    fn test_relationship_id_accepts_object_with_integer_id() {
        // Angular full-object shape: {"id": 1, "name": "..."}
        let rid: RelationshipId = serde_json::from_str(r#"{"id": 42, "name": "foo"}"#).unwrap();
        assert_eq!(rid.0, 42);
    }

    #[test]
    fn test_relationship_id_accepts_object_with_string_id() {
        // React full-object shape: {"id": "1", ...}
        let rid: RelationshipId = serde_json::from_str(r#"{"id": "7"}"#).unwrap();
        assert_eq!(rid.0, 7);
    }

    #[test]
    fn test_relationship_id_rejects_object_missing_id_field() {
        let err = serde_json::from_str::<RelationshipId>(r#"{"name": "foo"}"#).unwrap_err();
        assert!(err.to_string().contains("missing id field"), "got: {}", err);
    }

    #[test]
    fn test_relationship_id_rejects_unparseable_string() {
        let err = serde_json::from_str::<RelationshipId>("\"not-a-number\"").unwrap_err();
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_relationship_id_rejects_non_numeric_value() {
        // Booleans, arrays, null are not valid relationship IDs.
        let err = serde_json::from_str::<RelationshipId>("true").unwrap_err();
        assert!(err.to_string().contains("id must be a number or string"), "got: {}", err);
    }

    #[test]
    fn test_relationship_id_rejects_overflow() {
        // i64 too large to fit in i32 returns the i32 conversion error.
        let err = serde_json::from_str::<RelationshipId>("9999999999").unwrap_err();
        assert!(err.to_string().contains("id is not a valid i32"), "got: {}", err);
    }

    #[test]
    fn test_deserialize_optional_relationship_returns_none_for_null() {
        let w: OptionalRelWrapper = serde_json::from_str(r#"{"rel": null}"#).unwrap();
        assert_eq!(w.rel, None);
    }

    #[test]
    fn test_deserialize_optional_relationship_returns_none_for_missing() {
        let w: OptionalRelWrapper = serde_json::from_str("{}").unwrap();
        assert_eq!(w.rel, None);
    }

    #[test]
    fn test_deserialize_optional_relationship_handles_integer_form() {
        let w: OptionalRelWrapper = serde_json::from_str(r#"{"rel": 5}"#).unwrap();
        assert_eq!(w.rel, Some(5));
    }

    #[test]
    fn test_deserialize_optional_relationship_handles_string_form() {
        let w: OptionalRelWrapper = serde_json::from_str(r#"{"rel": "9"}"#).unwrap();
        assert_eq!(w.rel, Some(9));
    }

    #[test]
    fn test_deserialize_optional_relationship_handles_object_form() {
        let w: OptionalRelWrapper = serde_json::from_str(r#"{"rel": {"id": 3, "name": "x"}}"#).unwrap();
        assert_eq!(w.rel, Some(3));
    }

    #[test]
    fn test_deserialize_optional_relationship_rejects_object_missing_id() {
        let err = serde_json::from_str::<OptionalRelWrapper>(r#"{"rel": {"name": "x"}}"#).unwrap_err();
        assert!(err.to_string().contains("missing id field"), "got: {}", err);
    }

    #[test]
    fn test_deserialize_naive_datetime_parses_rfc3339() {
        let w: DateTimeWrapper = serde_json::from_str(r#"{"ts": "2026-05-11T12:34:56+00:00"}"#).unwrap();
        assert_eq!(w.ts.to_string(), "2026-05-11 12:34:56");
    }

    #[test]
    fn test_deserialize_naive_datetime_parses_z_suffix_with_fraction() {
        let w: DateTimeWrapper = serde_json::from_str(r#"{"ts": "2026-05-11T12:34:56.789Z"}"#).unwrap();
        assert_eq!(w.ts.format("%Y-%m-%d %H:%M:%S").to_string(), "2026-05-11 12:34:56");
    }

    #[test]
    fn test_deserialize_naive_datetime_parses_naive_with_fraction() {
        let w: DateTimeWrapper = serde_json::from_str(r#"{"ts": "2026-05-11T12:34:56.123"}"#).unwrap();
        assert_eq!(w.ts.format("%Y-%m-%d %H:%M:%S").to_string(), "2026-05-11 12:34:56");
    }

    #[test]
    fn test_deserialize_naive_datetime_parses_naive_no_fraction() {
        let w: DateTimeWrapper = serde_json::from_str(r#"{"ts": "2026-05-11T12:34:56"}"#).unwrap();
        assert_eq!(w.ts.to_string(), "2026-05-11 12:34:56");
    }

    #[test]
    fn test_deserialize_naive_datetime_parses_space_separated() {
        let w: DateTimeWrapper = serde_json::from_str(r#"{"ts": "2026-05-11 12:34:56"}"#).unwrap();
        assert_eq!(w.ts.to_string(), "2026-05-11 12:34:56");
    }

    #[test]
    fn test_deserialize_naive_datetime_rejects_unparseable() {
        let err = serde_json::from_str::<DateTimeWrapper>(r#"{"ts": "not-a-date"}"#).unwrap_err();
        assert!(err.to_string().contains("Unable to parse datetime"), "got: {}", err);
    }

    #[test]
    fn test_deserialize_option_naive_datetime_returns_none_for_null() {
        let w: OptionalDateTimeWrapper = serde_json::from_str(r#"{"ts": null}"#).unwrap();
        assert_eq!(w.ts, None);
    }

    #[test]
    fn test_deserialize_option_naive_datetime_returns_none_for_missing() {
        let w: OptionalDateTimeWrapper = serde_json::from_str("{}").unwrap();
        assert_eq!(w.ts, None);
    }

    #[test]
    fn test_deserialize_option_naive_datetime_returns_none_for_empty_string() {
        let w: OptionalDateTimeWrapper = serde_json::from_str(r#"{"ts": ""}"#).unwrap();
        assert_eq!(w.ts, None);
    }

    #[test]
    fn test_deserialize_option_naive_datetime_parses_rfc3339() {
        let w: OptionalDateTimeWrapper = serde_json::from_str(r#"{"ts": "2026-05-11T12:34:56+00:00"}"#).unwrap();
        assert!(w.ts.is_some());
        assert_eq!(w.ts.unwrap().to_string(), "2026-05-11 12:34:56");
    }

    #[test]
    fn test_deserialize_option_naive_datetime_parses_z_suffix() {
        let w: OptionalDateTimeWrapper = serde_json::from_str(r#"{"ts": "2026-05-11T12:34:56.0Z"}"#).unwrap();
        assert!(w.ts.is_some());
    }

    #[test]
    fn test_deserialize_option_naive_datetime_parses_naive_with_fraction() {
        let w: OptionalDateTimeWrapper = serde_json::from_str(r#"{"ts": "2026-05-11T12:34:56.123"}"#).unwrap();
        assert!(w.ts.is_some());
    }

    #[test]
    fn test_deserialize_option_naive_datetime_parses_naive_no_fraction() {
        let w: OptionalDateTimeWrapper = serde_json::from_str(r#"{"ts": "2026-05-11T12:34:56"}"#).unwrap();
        assert!(w.ts.is_some());
    }

    #[test]
    fn test_deserialize_option_naive_datetime_parses_space_separated() {
        let w: OptionalDateTimeWrapper = serde_json::from_str(r#"{"ts": "2026-05-11 12:34:56"}"#).unwrap();
        assert!(w.ts.is_some());
    }

    #[test]
    fn test_deserialize_option_naive_datetime_rejects_unparseable() {
        let err = serde_json::from_str::<OptionalDateTimeWrapper>(r#"{"ts": "garbage"}"#).unwrap_err();
        assert!(err.to_string().contains("Unable to parse datetime"), "got: {}", err);
    }
}

