//! This module implements Temporal Date/Time parsing functionality.

use crate::{TemporalError, TemporalResult};

use ixdtf::parsers::{
    records::{Annotation, IxdtfParseRecord},
    IxdtfParser,
};

// TODO: Determine if these should be separate structs, i.e. TemporalDateTimeParser/TemporalInstantParser, or
// maybe on global `TemporalParser` around `IxdtfParser` that handles the Temporal idiosyncracies.

#[inline]
fn parse_temporal_ixdtf_variation(source: &str) -> TemporalResult<IxdtfParseRecord> {
    let mut first_calendar: Option<Annotation> = None;
    let mut critical_duplicate_calendar = false;
    let result = IxdtfParser::new(source)
        .parse_with_annotation_handler(|annotation| {
            if annotation.key == "u-ca" {
                match first_calendar {
                    Some(ref cal) => {
                        if cal.critical && annotation.critical {
                            critical_duplicate_calendar = true
                        }
                    }
                    None => {
                        first_calendar.get_or_insert(annotation.clone());
                    }
                }
            }

            Some(annotation)
        })
        .map_err(|e| TemporalError::general(format!("{e}")))?;

    if critical_duplicate_calendar {
        // TODO: Add tests for the below.
        // Parser handles non-matching calendar, so the value thrown here should only be duplicates.
        return Err(TemporalError::syntax()
            .with_message("Duplicate calendar value with critical flag found."));
    }

    // Validate that the DateRecord exists.
    if result.date.is_none() {
        return Err(
            TemporalError::syntax().with_message("DateTime strings must contain a Date value.")
        );
    }

    Ok(result)
}

/// A utility function for parsing a `DateTime` string
pub(crate) fn parse_date_time(source: &str) -> TemporalResult<IxdtfParseRecord> {
    parse_temporal_ixdtf_variation(source)
}

/// A utility function for parsing an `Instant` string
#[allow(unused)]
pub(crate) fn parse_instant(source: &str) -> TemporalResult<IxdtfParseRecord> {
    let record = parse_temporal_ixdtf_variation(source)?;

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
    let mut first_calendar: Option<Annotation> = None;
    let mut critical_duplicate_calendar = false;
    let ym_record =
        IxdtfParser::new(source).parse_year_month_with_annotation_handler(|annotation| {
            if annotation.key == "u-ca" {
                match first_calendar {
                    Some(ref cal) => {
                        if cal.critical && annotation.critical {
                            critical_duplicate_calendar = true
                        }
                    }
                    None => {
                        first_calendar.get_or_insert(annotation.clone());
                    }
                }
            }

            Some(annotation)
        });

    if let Ok(ym) = ym_record {
        return Ok(ym);
    }

    let dt_parse = parse_temporal_ixdtf_variation(source);

    match dt_parse {
        Ok(dt) => Ok(dt),
        // Format and return the error from parsing YearMonth.
        _ => ym_record.map_err(|e| TemporalError::range().with_message(format!("{e}"))),
    }
}

/// A utilty function for parsing a `MonthDay` String.
pub(crate) fn parse_month_day(source: &str) -> TemporalResult<IxdtfParseRecord> {
    let mut first_calendar: Option<Annotation> = None;
    let mut critical_duplicate_calendar = false;
    let md_record =
        IxdtfParser::new(source).parse_month_day_with_annotation_handler(|annotation| {
            if annotation.key == "u-ca" {
                match first_calendar {
                    Some(ref cal) => {
                        if cal.critical && annotation.critical {
                            critical_duplicate_calendar = true
                        }
                    }
                    None => {
                        first_calendar.get_or_insert(annotation.clone());
                    }
                }
            }

            Some(annotation)
        });

    if let Ok(md) = md_record {
        return Ok(md);
    }

    let dt_parse = parse_temporal_ixdtf_variation(source);

    match dt_parse {
        Ok(dt) => Ok(dt),
        // Format and return the error from parsing YearMonth.
        _ => md_record.map_err(|e| TemporalError::range().with_message(format!("{e}"))),
    }
}

// TODO: ParseTemporalTimeString, ParseTimeZoneString, ParseZonedDateTimeString
