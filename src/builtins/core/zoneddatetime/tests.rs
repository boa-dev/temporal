use super::ZonedDateTime;
use crate::{
    builtins::{calendar::CalendarFields, zoneddatetime::ZonedDateTimeFields},
    options::{
        DifferenceSettings, Disambiguation, OffsetDisambiguation, Overflow, RoundingIncrement,
        RoundingMode, RoundingOptions, Unit,
    },
    partial::{PartialTime, PartialZonedDateTime},
    tzdb::FsTzdbProvider,
    unix_time::EpochNanoseconds,
    Calendar, MonthCode, TimeZone, UtcOffset,
};
use core::str::FromStr;
use tinystr::tinystr;

#[test]
fn basic_zdt_test() {
    let provider = &FsTzdbProvider::default();
    let nov_30_2023_utc = 1_701_308_952_000_000_000i128;

    let zdt = ZonedDateTime::try_new_with_provider(
        nov_30_2023_utc,
        Calendar::from_str("iso8601").unwrap(),
        TimeZone::try_from_str_with_provider("UTC", provider).unwrap(),
        provider,
    )
    .unwrap();

    assert_eq!(zdt.year().unwrap(), 2023);
    assert_eq!(zdt.month().unwrap(), 11);
    assert_eq!(zdt.day().unwrap(), 30);
    assert_eq!(zdt.hour().unwrap(), 1);
    assert_eq!(zdt.minute().unwrap(), 49);
    assert_eq!(zdt.second().unwrap(), 12);

    let zdt_minus_five = ZonedDateTime::try_new_with_provider(
        nov_30_2023_utc,
        Calendar::from_str("iso8601").unwrap(),
        TimeZone::try_from_str_with_provider("America/New_York", provider).unwrap(),
        provider,
    )
    .unwrap();

    assert_eq!(zdt_minus_five.year().unwrap(), 2023);
    assert_eq!(zdt_minus_five.month().unwrap(), 11);
    assert_eq!(zdt_minus_five.day().unwrap(), 29);
    assert_eq!(zdt_minus_five.hour().unwrap(), 20);
    assert_eq!(zdt_minus_five.minute().unwrap(), 49);
    assert_eq!(zdt_minus_five.second().unwrap(), 12);

    let zdt_plus_eleven = ZonedDateTime::try_new_with_provider(
        nov_30_2023_utc,
        Calendar::from_str("iso8601").unwrap(),
        TimeZone::try_from_str_with_provider("Australia/Sydney", provider).unwrap(),
        provider,
    )
    .unwrap();

    assert_eq!(zdt_plus_eleven.year().unwrap(), 2023);
    assert_eq!(zdt_plus_eleven.month().unwrap(), 11);
    assert_eq!(zdt_plus_eleven.day().unwrap(), 30);
    assert_eq!(zdt_plus_eleven.hour().unwrap(), 12);
    assert_eq!(zdt_plus_eleven.minute().unwrap(), 49);
    assert_eq!(zdt_plus_eleven.second().unwrap(), 12);
}

#[test]
// https://tc39.es/proposal-temporal/docs/zoneddatetime.html#round
fn round_with_provider_test() {
    let provider = &FsTzdbProvider::default();
    let dt = b"1995-12-07T03:24:30.000003500-08:00[America/Los_Angeles]";
    let zdt = ZonedDateTime::from_utf8_with_provider(
        dt,
        Disambiguation::default(),
        OffsetDisambiguation::Use,
        provider,
    )
    .unwrap();

    let result = zdt
        .round_with_provider(
            RoundingOptions {
                smallest_unit: Some(Unit::Hour),
                ..Default::default()
            },
            provider,
        )
        .unwrap();
    assert_eq!(
        result.to_string_with_provider(provider).unwrap(),
        "1995-12-07T03:00:00-08:00[America/Los_Angeles]"
    );

    let result = zdt
        .round_with_provider(
            RoundingOptions {
                smallest_unit: Some(Unit::Minute),
                increment: Some((RoundingIncrement::try_new(30)).unwrap()),
                ..Default::default()
            },
            provider,
        )
        .unwrap();
    assert_eq!(
        result.to_string_with_provider(provider).unwrap(),
        "1995-12-07T03:30:00-08:00[America/Los_Angeles]"
    );

    let result = zdt
        .round_with_provider(
            RoundingOptions {
                smallest_unit: Some(Unit::Minute),
                increment: Some((RoundingIncrement::try_new(30)).unwrap()),
                rounding_mode: Some(RoundingMode::Floor),
                ..Default::default()
            },
            provider,
        )
        .unwrap();
    assert_eq!(
        result.to_string_with_provider(provider).unwrap(),
        "1995-12-07T03:00:00-08:00[America/Los_Angeles]"
    );
}

#[test]
fn zdt_from_partial() {
    let provider = &FsTzdbProvider::default();
    let fields = ZonedDateTimeFields {
        calendar_fields: CalendarFields::new()
            .with_year(1970)
            .with_month_code(MonthCode(tinystr!(4, "M01")))
            .with_day(1),
        time: Default::default(),
        offset: None,
    };
    let partial = PartialZonedDateTime {
        fields,
        timezone: Some(TimeZone::default()),
        calendar: Calendar::ISO,
    };

    let result = ZonedDateTime::from_partial_with_provider(partial, None, None, None, provider);
    assert!(result.is_ok());

    // This ensures that the start-of-day branch isn't hit by default time
    let provider = &FsTzdbProvider::default();

    let fields = ZonedDateTimeFields {
        calendar_fields: CalendarFields::new()
            .with_year(1970)
            .with_month_code(MonthCode(tinystr!(4, "M01")))
            .with_day(1),
        time: PartialTime::default(),
        offset: Some(UtcOffset::from_minutes(30)),
    };
    let partial = PartialZonedDateTime {
        fields,
        timezone: Some(TimeZone::default()),
        calendar: Calendar::ISO,
    };

    let result = ZonedDateTime::from_partial_with_provider(
        partial,
        None,
        None,
        Some(OffsetDisambiguation::Use),
        provider,
    );
    assert!(result.is_ok());
}

#[test]
fn zdt_from_str() {
    let provider = &FsTzdbProvider::default();

    let zdt_str = b"1970-01-01T00:00[UTC][u-ca=iso8601]";
    let result = ZonedDateTime::from_utf8_with_provider(
        zdt_str,
        Disambiguation::Compatible,
        OffsetDisambiguation::Reject,
        provider,
    );
    assert!(result.is_ok());
}

#[test]
fn zdt_hours_in_day() {
    let provider = &FsTzdbProvider::default();
    let zdt_str = b"2025-07-04T12:00[UTC][u-ca=iso8601]";
    let result = ZonedDateTime::from_utf8_with_provider(
        zdt_str,
        Disambiguation::Compatible,
        OffsetDisambiguation::Reject,
        provider,
    )
    .unwrap();

    assert_eq!(result.hours_in_day_with_provider(provider).unwrap(), 24.)
}

#[test]
// https://github.com/tc39/test262/blob/d9b10790bc4bb5b3e1aa895f11cbd2d31a5ec743/test/intl402/Temporal/ZonedDateTime/from/dst-skipped-cross-midnight.js
fn dst_skipped_cross_midnight() {
    let provider = &FsTzdbProvider::default();
    let start_of_day = ZonedDateTime::from_utf8_with_provider(
        b"1919-03-31[America/Toronto]",
        Disambiguation::Compatible,
        OffsetDisambiguation::Reject,
        provider,
    )
    .unwrap();
    let midnight_disambiguated = ZonedDateTime::from_utf8_with_provider(
        b"1919-03-31T00[America/Toronto]",
        Disambiguation::Compatible,
        OffsetDisambiguation::Reject,
        provider,
    )
    .unwrap();

    assert_eq!(
        start_of_day.epoch_nanoseconds(),
        &EpochNanoseconds(-1601753400000000000)
    );
    assert_eq!(
        midnight_disambiguated.epoch_nanoseconds(),
        &EpochNanoseconds(-1601751600000000000)
    );
    let diff = start_of_day
        .instant
        .until(
            &midnight_disambiguated.instant,
            DifferenceSettings {
                largest_unit: Some(Unit::Hour),
                smallest_unit: Some(Unit::Nanosecond),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(diff.years(), 0);
    assert_eq!(diff.months(), 0);
    assert_eq!(diff.weeks(), 0);
    assert_eq!(diff.days(), 0);
    assert_eq!(diff.hours(), 0);
    assert_eq!(diff.minutes(), 30);
    assert_eq!(diff.seconds(), 0);
    assert_eq!(diff.milliseconds(), 0);
    assert_eq!(diff.microseconds(), 0);
    assert_eq!(diff.nanoseconds(), 0);
}

#[cfg(feature = "compiled_data")]
#[test]
fn zdt_offset_match_minutes() {
    // Cases taken from intl402/Temporal/ZonedDateTime/compare/sub-minute-offset

    let provider = &*crate::builtins::TZ_PROVIDER;

    // Rounded mm accepted
    let _ = ZonedDateTime::from_utf8_with_provider(
        b"1970-01-01T00:00-00:45[Africa/Monrovia]",
        Default::default(),
        OffsetDisambiguation::Reject,
        provider,
    )
    .unwrap();
    // unrounded mm::ss accepted
    let _ = ZonedDateTime::from_utf8_with_provider(
        b"1970-01-01T00:00:00-00:44:30[Africa/Monrovia]",
        Default::default(),
        OffsetDisambiguation::Reject,
        provider,
    )
    .unwrap();
    assert!(
        ZonedDateTime::from_utf8_with_provider(
            b"1970-01-01T00:00:00-00:44:40[Africa/Monrovia]",
            Default::default(),
            OffsetDisambiguation::Reject,
            provider
        )
        .is_err(),
        "Incorrect unrounded mm::ss rejected"
    );
    assert!(
        ZonedDateTime::from_utf8_with_provider(
            b"1970-01-01T00:00:00-00:45:00[Africa/Monrovia]",
            Default::default(),
            OffsetDisambiguation::Reject,
            provider
        )
        .is_err(),
        "Rounded mm::ss rejected"
    );
    assert!(
        ZonedDateTime::from_utf8_with_provider(
            b"1970-01-01T00:00+00:44:30.123456789[+00:45]",
            Default::default(),
            OffsetDisambiguation::Reject,
            provider
        )
        .is_err(),
        "Rounding not accepted between ISO offset and timezone"
    );

    assert!(
        ZonedDateTime::from_partial_with_provider(
            PartialZonedDateTime {
                fields: ZonedDateTimeFields {
                    calendar_fields: CalendarFields::new()
                        .with_year(1970)
                        .with_month_code(MonthCode(tinystr!(4, "M01")))
                        .with_day(1),
                    time: PartialTime::default(),
                    offset: Some(UtcOffset::from_minutes(30)),
                },
                timezone: Some(TimeZone::try_from_identifier_str("Africa/Monrovia").unwrap()),
                ..PartialZonedDateTime::default()
            },
            None,
            None,
            None,
            provider
        )
        .is_err(),
        "Rounding not accepted between ISO offset and timezone"
    );
}

// overflow-reject-throws.js
#[test]
fn overflow_reject_throws() {
    let provider = &FsTzdbProvider::default();

    let zdt = ZonedDateTime::try_new_with_provider(
        217178610123456789,
        Calendar::default(),
        TimeZone::default(),
        provider,
    )
    .unwrap();

    let overflow = Overflow::Reject;

    let result_1 = zdt.with_with_provider(
        ZonedDateTimeFields {
            calendar_fields: CalendarFields::new().with_month(29),
            time: Default::default(),
            offset: None,
        },
        None,
        None,
        Some(overflow),
        provider,
    );

    let result_2 = zdt.with_with_provider(
        ZonedDateTimeFields {
            calendar_fields: CalendarFields::new().with_day(31),
            time: Default::default(),
            offset: None,
        },
        None,
        None,
        Some(overflow),
        provider,
    );

    let result_3 = zdt.with_with_provider(
        ZonedDateTimeFields {
            calendar_fields: CalendarFields::new(),
            time: PartialTime {
                hour: Some(29),
                ..Default::default()
            },
            offset: None,
        },
        None,
        None,
        Some(overflow),
        provider,
    );

    let result_4 = zdt.with_with_provider(
        ZonedDateTimeFields {
            calendar_fields: CalendarFields::default(),
            time: PartialTime {
                nanosecond: Some(9000),
                ..Default::default()
            },
            offset: None,
        },
        None,
        None,
        Some(overflow),
        provider,
    );

    assert!(result_1.is_err());
    assert!(result_2.is_err());
    assert!(result_3.is_err());
    assert!(result_4.is_err());
}
