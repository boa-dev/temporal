use alloc::borrow::ToOwned;
use alloc::string::String;
use core::{iter::Peekable, str::Chars};
use ixdtf::parsers::{
    records::{TimeZoneRecord, UtcOffsetRecord, UtcOffsetRecordOrZ},
    IxdtfParser,
};

use crate::{TemporalError, TemporalResult, TimeZone};

use super::{parse_ixdtf, ParseVariant};

#[inline]
pub(crate) fn parse_allowed_timezone_formats(s: &str) -> Option<TimeZone> {
    let (offset, annotation) = if let Ok((offset, annotation)) =
        parse_ixdtf(s, ParseVariant::DateTime).map(|r| (r.offset, r.tz))
    {
        (offset, annotation)
    } else if let Ok((offset, annotation)) = IxdtfParser::from_str(s)
        .parse_time()
        .map(|r| (r.offset, r.tz))
    {
        (offset, annotation)
    } else if let Ok((offset, annotation)) =
        parse_ixdtf(s, ParseVariant::YearMonth).map(|r| (r.offset, r.tz))
    {
        (offset, annotation)
    } else if let Ok((offset, annotation)) =
        parse_ixdtf(s, ParseVariant::MonthDay).map(|r| (r.offset, r.tz))
    {
        (offset, annotation)
    } else {
        return None;
    };

    if let Some(annotation) = annotation {
        match annotation.tz {
            TimeZoneRecord::Name(s) => {
                let identifier = String::from_utf8_lossy(s).into_owned();
                return Some(TimeZone::IanaIdentifier(identifier));
            }
            TimeZoneRecord::Offset(offset) => return Some(timezone_from_offset_record(offset)),
            _ => {}
        }
    };

    if let Some(offset) = offset {
        match offset {
            UtcOffsetRecordOrZ::Z => return Some(TimeZone::default()),
            UtcOffsetRecordOrZ::Offset(offset) => return Some(timezone_from_offset_record(offset)),
        }
    }

    None
}

fn timezone_from_offset_record(record: UtcOffsetRecord) -> TimeZone {
    let minutes = (record.hour as i16 * 60) + record.minute as i16 + (record.second as i16 / 60);
    TimeZone::OffsetMinutes(minutes * record.sign as i16)
}

#[inline]
pub(crate) fn parse_identifier(source: &str) -> TemporalResult<TimeZone> {
    let mut cursor = source.chars().peekable();
    if cursor.peek().is_some_and(is_ascii_sign) {
        let offset_minutes = parse_offset(&mut cursor)?;
        return Ok(TimeZone::OffsetMinutes(offset_minutes));
    } else if parse_iana_component(&mut cursor) {
        return Ok(TimeZone::IanaIdentifier(source.to_owned()));
    }
    Err(TemporalError::range().with_message("Invalid TimeZone Identifier"))
}

#[inline]
pub(crate) fn parse_offset(chars: &mut Peekable<Chars<'_>>) -> TemporalResult<i16> {
    let sign = chars.next().map_or(1, |c| if c == '+' { 1 } else { -1 });
    // First offset portion
    let hours = parse_digit_pair(chars)?;

    let sep = chars.peek().is_some_and(|ch| *ch == ':');
    if sep {
        let _ = chars.next();
    }

    let digit_peek = chars.peek().map(|ch| ch.is_ascii_digit());

    let minutes = match digit_peek {
        Some(true) => parse_digit_pair(chars)?,
        Some(false) => return Err(non_ascii_digit()),
        None => 0,
    };

    Ok((hours * 60 + minutes) * sign)
}

fn parse_digit_pair(chars: &mut Peekable<Chars<'_>>) -> TemporalResult<i16> {
    let valid = chars
        .peek()
        .map_or(Err(abrupt_end()), |ch| Ok(ch.is_ascii_digit()))?;
    let first = if valid {
        chars.next().expect("validated.")
    } else {
        return Err(non_ascii_digit());
    };
    let valid = chars
        .peek()
        .map_or(Err(abrupt_end()), |ch| Ok(ch.is_ascii_digit()))?;
    let second = if valid {
        chars.next().expect("validated.")
    } else {
        return Err(non_ascii_digit());
    };

    let tens = (first.to_digit(10).expect("validated") * 10) as i16;
    let ones = second.to_digit(10).expect("validated") as i16;

    Ok(tens + ones)
}

fn parse_iana_component(chars: &mut Peekable<Chars<'_>>) -> bool {
    // Confirm leading Tz char
    if !chars.peek().is_some_and(is_tz_leading_char) {
        return false;
    }
    chars.next();

    // Move and check that chars are an expected tz char
    while chars.peek().is_some_and(is_tz_char) {
        chars.next();
    }

    // Check for sub component and parse
    if chars.peek().is_some_and(is_slash) {
        chars.next();
        return parse_iana_component(chars);
    }

    // Confirm full source text has been parsed.
    chars.peek().is_none()
}

// NOTE: Spec calls for throwing a RangeError when parse node is a list of errors for timezone.

fn abrupt_end() -> TemporalError {
    TemporalError::range().with_message("Abrupt end while parsing offset string")
}

fn non_ascii_digit() -> TemporalError {
    TemporalError::range().with_message("Non ascii digit found while parsing offset string")
}

fn is_ascii_sign(ch: &char) -> bool {
    *ch == '+' || *ch == '-'
}

fn is_slash(ch: &char) -> bool {
    *ch == '/'
}

fn is_tz_leading_char(ch: &char) -> bool {
    ch.is_alphabetic() || *ch == '.' || *ch == '_'
}

fn is_tz_char(ch: &char) -> bool {
    is_tz_leading_char(ch) || ch.is_ascii_digit() || *ch == '+' || *ch == '-'
}
