use ixdtf::{
    encoding::Utf8,
    parsers::{IxdtfParser, TimeZoneParser},
    records::{TimeZoneRecord, UtcOffsetRecord, UtcOffsetRecordOrZ},
};

use crate::provider::TimeZoneProvider;
use crate::{builtins::time_zone::UtcOffset, TemporalError, TemporalResult, TimeZone};

use super::{parse_ixdtf, ParseVariant};
use crate::parsers::parse_zoned_date_time;

#[inline]
pub(crate) fn parse_allowed_timezone_formats(
    s: &str,
    provider: &(impl TimeZoneProvider + ?Sized),
) -> Option<TimeZone> {
    let (offset, annotation) =
        if let Ok(r) = parse_zoned_date_time(s.as_bytes()) {
            (r.offset, r.tz)
        } else if let Some(r) = parse_ixdtf(s.as_bytes(), ParseVariant::DateTime)
            .ok()
            .filter(|r| r.date.is_some() && r.time.is_some() && r.offset.is_some())
        {
            (r.offset, r.tz)
        } else if let Ok(r) = parse_ixdtf(s.as_bytes(), ParseVariant::DateTime) {
            (r.offset, r.tz)
        } else if let Ok(r) = IxdtfParser::from_str(s).parse_time() {
            (r.offset, r.tz)
        } else if let Ok(r) = parse_ixdtf(s.as_bytes(), ParseVariant::MonthDay) {
            (r.offset, r.tz)
        } else if let Ok(r) = parse_ixdtf(s.as_bytes(), ParseVariant::YearMonth) {
            (r.offset, r.tz)
        } else {
            return None;
        };

    if let Some(annotation) = annotation {
        return TimeZone::from_time_zone_record(annotation.tz, provider).ok();
    };

    if let Some(offset) = offset {
        match offset {
            UtcOffsetRecordOrZ::Z => return Some(TimeZone::utc_with_provider(provider)),
            UtcOffsetRecordOrZ::Offset(offset) => {
                let offset = match offset {
                    UtcOffsetRecord::MinutePrecision(offset) => offset,
                    _ => return None,
                };
                return Some(TimeZone::UtcOffset(UtcOffset::from_ixdtf_minute_record(
                    offset,
                )));
            }
        }
    }

    None
}

#[inline]
pub(crate) fn parse_identifier(source: &str) -> TemporalResult<TimeZoneRecord<'_, Utf8>> {
    let mut parser = TimeZoneParser::from_str(source);
    parser.parse_identifier().or(Err(
        TemporalError::range().with_message("Invalid TimeZone Identifier")
    ))
}
