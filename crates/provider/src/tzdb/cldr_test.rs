use icu_time::zone::iana::{IanaParserExtended, TimeZoneAndCanonicalAndNormalized};

use super::CompiledNormalizer;
use crate::provider::TimeZoneNormalizer;

/// This tests against CLDR through ICU4X.
#[test]
fn test_cldr_timezones() {
    for TimeZoneAndCanonicalAndNormalized {
        canonical,
        normalized,
        ..
    } in IanaParserExtended::new().iter_all()
    {
        if canonical.starts_with("Etc") {
            // These are handled elsewhere
            continue;
        }
        if matches!(normalized, "Canada/East-Saskatchewan" | "US/Pacific-New") {
            // These are present in CLDR but not tzdb. Special case as known exceptions.
            continue;
        }
        if matches!(normalized, "CST6CDT" | "EST5EDT" | "MST7MDT" | "PST8PDT") {
            // tzdb 2026e turned these from links into independent zones using US
            // federal rules, so they are primary identifiers here. CLDR still
            // lists them as aliases. Skip until CLDR catches up.
            continue;
        }

        let primary_id = CompiledNormalizer
            .normalized(canonical.as_bytes())
            .expect(canonical);

        let normalized_id = CompiledNormalizer
            .normalized(normalized.as_bytes())
            .expect(normalized);

        let canonicalized_id = CompiledNormalizer
            .canonicalized(normalized_id)
            .expect(normalized);

        assert_eq!(
            canonicalized_id, primary_id,
            "{normalized} should canonicalize to the same thing as {canonical}"
        );
    }
}
