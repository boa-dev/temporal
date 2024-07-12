//! This module implements Temporal Date/Time parsing functionality.

use crate::{TemporalError, TemporalResult, TemporalUnwrap};

use ixdtf::parsers::{
    records::{Annotation, IxdtfParseRecord, TimeRecord},
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
    let mut parser = IxdtfParser::new(source);

    let handler = cast_handler(&mut parser, |annotation: Annotation<'_>| {
        if annotation.key == "u-ca" {
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
pub(crate) fn parse_date_time(source: &str) -> TemporalResult<IxdtfParseRecord> {
    parse_ixdtf(source, ParseVariant::DateTime)
}

/// A utility function for parsing an `Instant` string
#[allow(unused)]
pub(crate) fn parse_instant(source: &str) -> TemporalResult<IxdtfParseRecord> {
    let record = parse_ixdtf(source, ParseVariant::DateTime)?;

    // Validate required fields on an Instant value
    if record.time.is_none() || record.date.is_none() || record.offset.is_none() {
        return Err(
            TemporalError::range().with_message("Required fields missing from Instant string.")
        );
    }

    Ok(record)
}

/// A utility function for parsing a `YearMonth` string
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
pub(crate) fn parse_month_day(source: &str) -> TemporalResult<IxdtfParseRecord> {
    let md_record = parse_ixdtf(source, ParseVariant::MonthDay);

    if let Ok(md) = md_record {
        return Ok(md);
    }

    let dt_parse = parse_ixdtf(source, ParseVariant::DateTime);

    match dt_parse {
        Ok(dt) => Ok(dt),
        // Format and return the error from parsing YearMonth.
        _ => md_record.map_err(|e| TemporalError::range().with_message(format!("{e}"))),
    }
}

pub(crate) fn parse_time(source: &str) -> TemporalResult<TimeRecord> {
    let time_record = IxdtfParser::new(source).parse_time();

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

// TODO: ParseTemporalTimeString, ParseTimeZoneString, ParseZonedDateTimeString
