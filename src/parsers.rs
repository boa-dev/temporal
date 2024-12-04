//! This module implements Temporal Date/Time parsing functionality.

use alloc::format;

use crate::{TemporalError, TemporalResult, TemporalUnwrap};

use ixdtf::parsers::{
    records::{Annotation, DateRecord, IxdtfParseRecord, TimeRecord, UtcOffsetRecordOrZ},
    IxdtfParser,
};

// TODO: Determine if these should be separate structs, i.e. TemporalDateTimeParser/TemporalInstantParser, or
// maybe on global `TemporalParser` around `IxdtfParser` that handles the Temporal idiosyncracies.
enum ParseVariant {
    YearMonth,
    MonthDay,
    DateTime,
}

#[inline]
fn parse_ixdtf(source: &str, variant: ParseVariant) -> TemporalResult<IxdtfParseRecord> {
    fn cast_handler<'a>(
        _: &mut IxdtfParser<'a>,
        handler: impl FnMut(Annotation<'a>) -> Option<Annotation<'a>>,
    ) -> impl FnMut(Annotation<'a>) -> Option<Annotation<'a>> {
        handler
    }

    let mut first_calendar: Option<Annotation> = None;
    let mut critical_duplicate_calendar = false;
    let mut parser = IxdtfParser::from_str(source);

    let handler = cast_handler(&mut parser, |annotation: Annotation<'_>| {
        if annotation.key == "u-ca".as_bytes() {
            match first_calendar {
                Some(ref cal) => {
                    if cal.critical || annotation.critical {
                        critical_duplicate_calendar = true
                    }
                }
                None => first_calendar = Some(annotation),
            }
            return None;
        }

        // Make the parser handle any unknown annotation.
        Some(annotation)
    });

    let mut record = match variant {
        ParseVariant::YearMonth => parser.parse_year_month_with_annotation_handler(handler),
        ParseVariant::MonthDay => parser.parse_month_day_with_annotation_handler(handler),
        ParseVariant::DateTime => parser.parse_with_annotation_handler(handler),
    }
    .map_err(|e| TemporalError::general(format!("{e}")))?;

    if critical_duplicate_calendar {
        // TODO: Add tests for the below.
        // Parser handles non-matching calendar, so the value thrown here should only be duplicates.
        return Err(TemporalError::syntax()
            .with_message("Duplicate calendar value with critical flag found."));
    }

    // Validate that the DateRecord exists.
    if record.date.is_none() {
        return Err(
            TemporalError::syntax().with_message("DateTime strings must contain a Date value.")
        );
    }

    record.calendar = first_calendar.map(|v| v.value);

    Ok(record)
}

/// A utility function for parsing a `DateTime` string
#[inline]
pub(crate) fn parse_date_time(source: &str) -> TemporalResult<IxdtfParseRecord> {
    parse_ixdtf(source, ParseVariant::DateTime)
}

pub(crate) struct IxdtfParseInstantRecord {
    pub(crate) date: DateRecord,
    pub(crate) time: TimeRecord,
    pub(crate) offset: UtcOffsetRecordOrZ,
}

/// A utility function for parsing an `Instant` string
#[inline]
pub(crate) fn parse_instant(source: &str) -> TemporalResult<IxdtfParseInstantRecord> {
    let record = parse_ixdtf(source, ParseVariant::DateTime)?;

    let IxdtfParseRecord {
        date: Some(date),
        time: Some(time),
        offset: Some(offset),
        ..
    } = record
    else {
        return Err(
            TemporalError::range().with_message("Required fields missing from Instant string.")
        );
    };

    Ok(IxdtfParseInstantRecord { date, time, offset })
}

/// A utility function for parsing a `YearMonth` string
#[inline]
pub(crate) fn parse_year_month(source: &str) -> TemporalResult<IxdtfParseRecord> {
    let ym_record = parse_ixdtf(source, ParseVariant::YearMonth);

    if let Ok(ym) = ym_record {
        return Ok(ym);
    }

    let dt_parse = parse_ixdtf(source, ParseVariant::DateTime);

    match dt_parse {
        Ok(dt) => Ok(dt),
        // Format and return the error from parsing YearMonth.
        _ => ym_record.map_err(|e| TemporalError::range().with_message(format!("{e}"))),
    }
}

/// A utilty function for parsing a `MonthDay` String.
#[inline]
pub(crate) fn parse_month_day(source: &str) -> TemporalResult<IxdtfParseRecord> {
    let md_record = parse_ixdtf(source, ParseVariant::MonthDay);
    // Error needs to be a RangeError
    md_record.map_err(|e| TemporalError::range().with_message(format!("{e}")))
}

#[inline]
pub(crate) fn parse_time(source: &str) -> TemporalResult<TimeRecord> {
    let time_record = IxdtfParser::from_str(source).parse_time();

    let time_err = match time_record {
        Ok(time) => return time.time.temporal_unwrap(),
        Err(e) => TemporalError::range().with_message(format!("{e}")),
    };

    let dt_parse = parse_ixdtf(source, ParseVariant::DateTime);

    match dt_parse {
        Ok(dt) if dt.time.is_some() => Ok(dt.time.temporal_unwrap()?),
        // Format and return the error from parsing Time.
        _ => Err(time_err),
    }
}

// TODO: ParseTimeZoneString, ParseZonedDateTimeString
