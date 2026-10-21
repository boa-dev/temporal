//! Parsed-but-not-validated intermediate records.
//!
//! The Temporal specification performs observable operations between the parse
//! and validate steps, so parsing must yield these intermediates rather than a
//! finished builtin. `TemporalParser` produces them, applying the resolution
//! steps (calendar canonicalization, time zone resolution, early time validity)
//! in the order the specification requires.

use super::{
    parse_date_time, parse_month_day, parse_year_month, parse_zoned_date_time, TemporalEncoding,
    TemporalParser, Utf16, Utf8,
};
use crate::iso::IsoTime;
use crate::provider::TimeZoneProvider;
use crate::Calendar;
use crate::TemporalResult;
use crate::TimeZone;
use crate::UtcOffset;
use alloc::borrow::Cow;
use icu_calendar::AnyCalendarKind;
use ixdtf::records::DateRecord;
use ixdtf::records::UtcOffsetRecordOrZ;

fn extract_kind<T: TemporalEncoding>(
    calendar: Option<Cow<'_, [T::CodeUnit]>>,
) -> TemporalResult<AnyCalendarKind> {
    let Some(calendar) = calendar else {
        return Ok(AnyCalendarKind::Iso);
    };
    let ascii = T::ascii_identifier(&calendar)?;
    // HijriSimulatedMecca parses fine here; Calendar::new turns it into ISO.
    Calendar::try_kind_from_utf8(&ascii)
}

/// A parsed-but-not-validated date
#[derive(Copy, Clone, Debug)]
pub struct ParsedDate {
    pub(crate) record: DateRecord,
    pub(crate) calendar: AnyCalendarKind,
}

impl ParsedDate {
    /// Converts a UTF-8 encoded string into a `ParsedDate`.
    pub fn from_utf8(s: &[u8]) -> TemporalResult<Self> {
        TemporalParser::<Utf8>::from_utf8(s).parse_date()
    }

    /// Converts a UTF-16 encoded string into a `ParsedDate`.
    pub fn from_utf16(s: &[u16]) -> TemporalResult<Self> {
        TemporalParser::<Utf16>::from_utf16(s).parse_date()
    }

    /// Converts a UTF-8 encoded YearMonth string into a `ParsedDate`.
    pub fn year_month_from_utf8(s: &[u8]) -> TemporalResult<Self> {
        TemporalParser::<Utf8>::from_utf8(s).parse_year_month()
    }

    /// Converts a UTF-16 encoded YearMonth string into a `ParsedDate`.
    pub fn year_month_from_utf16(s: &[u16]) -> TemporalResult<Self> {
        TemporalParser::<Utf16>::from_utf16(s).parse_year_month()
    }

    /// Converts a UTF-8 encoded MonthDay string into a `ParsedDate`.
    pub fn month_day_from_utf8(s: &[u8]) -> TemporalResult<Self> {
        TemporalParser::<Utf8>::from_utf8(s).parse_month_day()
    }

    /// Converts a UTF-16 encoded MonthDay string into a `ParsedDate`.
    pub fn month_day_from_utf16(s: &[u16]) -> TemporalResult<Self> {
        TemporalParser::<Utf16>::from_utf16(s).parse_month_day()
    }
}

/// A parsed-but-not-validated datetime
#[derive(Copy, Clone, Debug)]
pub struct ParsedDateTime {
    pub(crate) date: ParsedDate,
    pub(crate) time: IsoTime,
}

impl ParsedDateTime {
    /// Converts a UTF-8 encoded string into a `ParsedDateTime`.
    pub fn from_utf8(s: &[u8]) -> TemporalResult<Self> {
        TemporalParser::<Utf8>::from_utf8(s).parse_date_time()
    }

    /// Converts a UTF-16 encoded string into a `ParsedDateTime`.
    pub fn from_utf16(s: &[u16]) -> TemporalResult<Self> {
        TemporalParser::<Utf16>::from_utf16(s).parse_date_time()
    }
}

/// A parsed-but-not-validated zoned datetime
#[derive(Clone, Debug)]
pub struct ParsedZonedDateTime {
    pub(crate) date: ParsedDate,
    /// None time is START-OF-DAY
    pub(crate) time: Option<IsoTime>,
    /// Whether or not the string has a UTC designator (`Z`)
    ///
    /// Incompatible with having an offset (you can still have a offset-format timezone)
    pub(crate) has_utc_designator: bool,
    /// Whether or not to allow offsets rounded to the minute
    ///
    /// (Typically only needs to be set when parsing, can be false otherwise)
    pub(crate) match_minutes: bool,
    /// An optional offset string
    pub(crate) offset: Option<UtcOffset>,
    /// The time zone
    pub(crate) timezone: TimeZone,
}

impl ParsedZonedDateTime {
    /// Converts a UTF-8 encoded string into a `ParsedZonedDateTime`, using compiled data
    #[cfg(feature = "compiled_data")]
    pub fn from_utf8(source: &[u8]) -> TemporalResult<Self> {
        Self::from_utf8_with_provider(source, &*crate::builtins::TZ_PROVIDER)
    }

    /// Converts a UTF-16 encoded string into a `ParsedZonedDateTime`, using compiled data
    #[cfg(feature = "compiled_data")]
    pub fn from_utf16(source: &[u16]) -> TemporalResult<Self> {
        Self::from_utf16_with_provider(source, &*crate::builtins::TZ_PROVIDER)
    }

    /// Converts a UTF-8 encoded string into a `ParsedZonedDateTime`.
    pub fn from_utf8_with_provider(
        source: &[u8],
        provider: &(impl TimeZoneProvider + ?Sized),
    ) -> TemporalResult<Self> {
        TemporalParser::<Utf8>::from_utf8(source).parse_zoned_date_time(provider)
    }

    /// Converts a UTF-16 encoded string into a `ParsedZonedDateTime`.
    pub fn from_utf16_with_provider(
        source: &[u16],
        provider: &(impl TimeZoneProvider + ?Sized),
    ) -> TemporalResult<Self> {
        TemporalParser::<Utf16>::from_utf16(source).parse_zoned_date_time(provider)
    }
}

impl<T: TemporalEncoding> TemporalParser<'_, T> {
    /// Parses a `TemporalDateTimeString` into a [`ParsedDate`].
    pub(crate) fn parse_date(&self) -> TemporalResult<ParsedDate> {
        let record = parse_date_time::<T>(self.source)?;
        let calendar = extract_kind::<T>(record.calendar)?;
        // Time is discarded, but must still be validated here (observable).
        validate_time(record.time)?;

        Ok(ParsedDate {
            record: record.date,
            calendar,
        })
    }

    /// Parses a `TemporalYearMonthString` into a [`ParsedDate`].
    pub(crate) fn parse_year_month(&self) -> TemporalResult<ParsedDate> {
        let record = parse_year_month::<T>(self.source)?;
        let calendar = extract_kind::<T>(record.calendar)?;
        validate_time(record.time)?;

        Ok(ParsedDate {
            record: record.date,
            calendar,
        })
    }

    /// Parses a `TemporalMonthDayString` into a [`ParsedDate`].
    pub(crate) fn parse_month_day(&self) -> TemporalResult<ParsedDate> {
        let record = parse_month_day::<T>(self.source)?;
        let calendar = extract_kind::<T>(record.calendar)?;
        validate_time(record.time)?;

        Ok(ParsedDate {
            record: record.date,
            calendar,
        })
    }

    /// Parses a `TemporalDateTimeString` into a [`ParsedDateTime`].
    pub(crate) fn parse_date_time(&self) -> TemporalResult<ParsedDateTime> {
        let record = parse_date_time::<T>(self.source)?;
        let calendar = extract_kind::<T>(record.calendar)?;
        let time = validate_time(record.time)?;

        Ok(ParsedDateTime {
            date: ParsedDate {
                record: record.date,
                calendar,
            },
            time,
        })
    }

    /// Parses a `TemporalZonedDateTimeString` into a [`ParsedZonedDateTime`].
    pub(crate) fn parse_zoned_date_time(
        &self,
        provider: &(impl TimeZoneProvider + ?Sized),
    ) -> TemporalResult<ParsedZonedDateTime> {
        // Steps from the parse bits of ToZonedDateTime

        // 3. Let matchBehaviour be match-minutes.
        let mut match_minutes = true;

        // b. Let result be ? ParseISODateTime(item, « TemporalDateTimeString[+Zoned] »).
        let record = parse_zoned_date_time::<T>(self.source)?;

        // c-e. Resolve the (guaranteed present) time zone annotation.
        let time_zone = TimeZone::from_time_zone_record::<T>(record.tz.tz, provider)?;

        // f-g. Resolve the offset, tracking whether `Z` was used.
        let (offset, has_utc_designator) = match record.offset {
            Some(UtcOffsetRecordOrZ::Z) => (None, true),
            Some(UtcOffsetRecordOrZ::Offset(offset)) => {
                // iii. More than one MinuteSecond node means match-exactly.
                if offset.second().is_some() {
                    match_minutes = false;
                }
                (Some(UtcOffset::from_ixdtf_record(offset)?), false)
            }
            None => (None, false),
        };

        // h-j. Resolve the calendar, defaulting to ISO.
        let calendar = extract_kind::<T>(record.calendar)?;

        let time = record.time.map(IsoTime::from_time_record).transpose()?;

        Ok(ParsedZonedDateTime {
            date: ParsedDate {
                record: record.date,
                calendar,
            },
            time,
            has_utc_designator,
            match_minutes,
            offset,
            timezone: time_zone,
        })
    }
}

/// Validates a parsed time component, defaulting to midnight when absent.
fn validate_time(time: Option<ixdtf::records::TimeRecord>) -> TemporalResult<IsoTime> {
    Ok(time
        .map(IsoTime::from_time_record)
        .transpose()?
        .unwrap_or_default())
}

#[test]
fn test_parsed_intermediate_invalid_dates() {
    // https://tc39.es/proposal-temporal/#sec-temporal-iso8601grammar-static-semantics-early-errors
    //
    // We need to error on invalid date/times *before* producing a ParsedFoo, since this is observable
    assert!(ParsedDate::from_utf8(b"2025-02-29").is_err());
    assert!(ParsedDate::from_utf8(b"2025-02-28").is_ok());
    assert!(ParsedDate::year_month_from_utf8(b"202513").is_err());
    assert!(ParsedDate::year_month_from_utf8(b"202512").is_ok());
    assert!(ParsedDate::month_day_from_utf8(b"1000").is_err());
    assert!(ParsedDate::month_day_from_utf8(b"1001").is_ok());
    assert!(ParsedDateTime::from_utf8(b"2025-02-28T25:69:69").is_err());
    assert!(ParsedDateTime::from_utf8(b"2025-02-28T23:59:59").is_ok());
}

#[test]
#[cfg(feature = "compiled_data")]
fn test_parsed_intermediate_invalid_zoned_dates() {
    assert!(ParsedZonedDateTime::from_utf8(b"2025-02-29T00:00:00Z[utc]").is_err());
    assert!(ParsedZonedDateTime::from_utf8(b"2025-02-28T00:00:00Z[utc]").is_ok());
}

// Pins the resolution order inside the parse step, which is observable:
//
// V8 runs the option getters (`GetTemporalDisambiguationOption` etc.) *between*
// `ParsedFoo::from_*` and `Foo::from_parsed*`, so reordering these resolutions
// changes which error a JS program sees.
#[test]
fn test_resolution_order_within_parse() {
    use crate::TemporalError;

    // Calendar is resolved before the time component is validated.
    assert_eq!(
        ParsedDateTime::from_utf8(b"2025-01-01T00:00:00.1234567891[u-ca=bogus]").unwrap_err(),
        TemporalError::range().with_message("unknown calendar")
    );
}

#[test]
#[cfg(feature = "compiled_data")]
fn test_zoned_resolution_order_within_parse() {
    use crate::error::ErrorMessage;
    use crate::TemporalError;

    // Time zone is resolved before the calendar.
    assert_eq!(
        ParsedZonedDateTime::from_utf8(b"2025-01-01T00:00:00[Bogus/Zone][u-ca=bogus]").unwrap_err(),
        TemporalError::range().with_message("Unknown time zone identifier")
    );

    // Per ParseDateTimeUTCOffset, a >9-digit offset fraction is a grammar error,
    // so it is rejected while parsing (step b), ahead of time zone resolution
    // (steps c-e).
    assert_eq!(
        ParsedZonedDateTime::from_utf8(b"2025-01-01T00:00:00+05:00:00.1234567891[Bogus/Zone]")
            .unwrap_err(),
        TemporalError::range().with_enum(ErrorMessage::FractionalTimeMoreThanNineDigits)
    );
}
