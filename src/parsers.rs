//! This module implements Temporal Date/Time parsing functionality.

use crate::{
    options::{DisplayCalendar, DisplayOffset, DisplayTimeZone},
    Sign, TemporalError, TemporalResult, TemporalUnwrap,
};
use alloc::format;
use ixdtf::parsers::{
    records::{Annotation, DateRecord, IxdtfParseRecord, TimeRecord, UtcOffsetRecordOrZ},
    IxdtfParser,
};
use writeable::{impl_display_with_writeable, Writeable};

// TODO: Move `Writeable` functionality to `ixdtf` crate

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precision {
    Auto,
    Minute,
    Digit(u8),
}

pub struct FormattableTime {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub nanosecond: u32,
    pub precision: Precision,
    pub include_sep: bool,
}

impl Writeable for FormattableTime {
    fn write_to<W: core::fmt::Write + ?Sized>(&self, sink: &mut W) -> core::fmt::Result {
        write_padded_u8(self.hour, sink)?;
        if self.include_sep {
            sink.write_char(':')?;
        }
        write_padded_u8(self.minute, sink)?;
        if self.precision == Precision::Minute {
            return Ok(());
        }
        if self.include_sep {
            sink.write_char(':')?;
        }
        write_padded_u8(self.second, sink)?;
        if self.nanosecond == 0 {
            return Ok(());
        }
        sink.write_char('.')?;
        write_nanosecond(self.nanosecond, self.precision, sink)?;

        Ok(())
    }
}

pub struct FormattableUtcOffset {
    pub show: DisplayOffset,
    pub offset: UtcOffset,
}

pub enum UtcOffset {
    Z,
    Offset(FormattableOffset),
}

impl Writeable for FormattableUtcOffset {
    fn write_to<W: core::fmt::Write + ?Sized>(&self, sink: &mut W) -> core::fmt::Result {
        if self.show == DisplayOffset::Never {
            return Ok(());
        }
        match &self.offset {
            UtcOffset::Z => sink.write_char('Z'),
            UtcOffset::Offset(offset) => offset.write_to(sink),
        }
    }
}

fn write_padded_u8<W: core::fmt::Write + ?Sized>(num: u8, sink: &mut W) -> core::fmt::Result {
    // NOTE:
    if num < 10 {
        sink.write_char('0')?;
    }
    num.write_to(sink)
}

fn write_nanosecond<W: core::fmt::Write + ?Sized>(
    nanoseconds: u32,
    precision: Precision,
    sink: &mut W,
) -> core::fmt::Result {
    let (digits, index) = write_u32_to_ascii_digits(nanoseconds);
    let slice = match precision {
        Precision::Digit(digit) if digit < 9 => &digits[..digit as usize],
        _ => &digits[..index],
    };
    // SAFETY: Index must be within a valid range of 0..=9 and is
    // valid aschii digit chars.
    sink.write_str(unsafe { core::str::from_utf8_unchecked(slice) })
}

pub fn write_u32_to_ascii_digits(mut value: u32) -> ([u8; 9], usize) {
    let mut output = [0; 9];
    let mut precision = 0;
    // let mut precision_check = 0;
    let mut i = 9;
    while i != 0 {
        let v = (value % 10) as u8;
        value /= 10;
        /*
        if precision_check == 0 && v !=0 {
            precision = i;
        }
        */
        if precision == 0 && v != 0 {
            precision = i;
        }
        // precision_check += v;
        output[i - 1] = v + 48;
        i -= 1;
    }

    (output, precision)
}

pub struct FormattableOffset {
    pub sign: Sign,
    pub time: FormattableTime,
}

impl Writeable for FormattableOffset {
    fn write_to<W: core::fmt::Write + ?Sized>(&self, sink: &mut W) -> core::fmt::Result {
        match self.sign {
            Sign::Negative => sink.write_char('-')?,
            _ => sink.write_char('+')?,
        }
        self.time.write_to(sink)
    }
}

impl_display_with_writeable!(FormattableIxdtf<'_>);
impl_display_with_writeable!(FormattableDate);
impl_display_with_writeable!(FormattableTime);
impl_display_with_writeable!(FormattableUtcOffset);
impl_display_with_writeable!(FormattableOffset);
impl_display_with_writeable!(FormattableTimeZone<'_>);
impl_display_with_writeable!(FormattableCalendar<'_>);

pub struct FormattableDate(pub i32, pub u8, pub u8);

impl Writeable for FormattableDate {
    fn write_to<W: core::fmt::Write + ?Sized>(&self, sink: &mut W) -> core::fmt::Result {
        if (0..=9999).contains(&self.0) {
            write_four_digit_year(self.0, sink)?;
        } else {
            write_extended_year(self.0, sink)?;
        }
        sink.write_char('-')?;
        write_padded_u8(self.1, sink)?;
        sink.write_char('-')?;
        write_padded_u8(self.2, sink)?;
        Ok(())
    }
}

fn write_four_digit_year<W: core::fmt::Write + ?Sized>(
    mut y: i32,
    sink: &mut W,
) -> core::fmt::Result {
    let mut divisor = 1_000;
    while divisor >= 1 {
        (y / divisor).write_to(sink)?;
        y %= divisor;
        divisor /= 10;
    }
    Ok(())
}

fn write_extended_year<W: core::fmt::Write + ?Sized>(y: i32, sink: &mut W) -> core::fmt::Result {
    let sign = if y < 0 { '-' } else { '+' };
    sink.write_char(sign)?;
    let (digits, _) = write_u32_to_ascii_digits(y.unsigned_abs());
    // SAFETY: digits slice is made up up valid ASCII digits.
    let value = unsafe { core::str::from_utf8_unchecked(&digits[3..]) };
    sink.write_str(value)
}

pub struct FormattableTimeZone<'a> {
    pub show: DisplayTimeZone,
    pub timezone: &'a str,
}

impl Writeable for FormattableTimeZone<'_> {
    fn write_to<W: core::fmt::Write + ?Sized>(&self, sink: &mut W) -> core::fmt::Result {
        if self.show == DisplayTimeZone::Never {
            return Ok(());
        }
        sink.write_char('[')?;
        if self.show == DisplayTimeZone::Critical {
            sink.write_char('!')?;
        }
        sink.write_str(self.timezone)?;
        sink.write_char(']')
    }
}

pub struct FormattableCalendar<'a> {
    pub show: DisplayCalendar,
    pub calendar: &'a str,
}

impl Writeable for FormattableCalendar<'_> {
    fn write_to<W: core::fmt::Write + ?Sized>(&self, sink: &mut W) -> core::fmt::Result {
        if self.show == DisplayCalendar::Never
            || self.show == DisplayCalendar::Auto && self.calendar == "iso8601"
        {
            return Ok(());
        }
        sink.write_char('[')?;
        if self.show == DisplayCalendar::Critical {
            sink.write_char('!')?;
        }
        sink.write_str("u-ca=")?;
        sink.write_str(self.calendar)?;
        sink.write_char(']')
    }
}

pub struct FormattableIxdtf<'a> {
    pub date: Option<FormattableDate>,
    pub time: Option<FormattableTime>,
    pub utc_offset: Option<FormattableUtcOffset>,
    pub timezone: Option<FormattableTimeZone<'a>>,
    pub calendar: Option<FormattableCalendar<'a>>,
}

impl Writeable for FormattableIxdtf<'_> {
    fn write_to<W: core::fmt::Write + ?Sized>(&self, sink: &mut W) -> core::fmt::Result {
        if let Some(date) = &self.date {
            date.write_to(sink)?;
        }
        if let Some(time) = &self.time {
            if self.date.is_some() {
                sink.write_char('T')?;
            }
            time.write_to(sink)?;
        }
        if self.date.is_none() && self.time.is_none() && self.utc_offset.is_some() {
            return Err(core::fmt::Error);
        }
        if let Some(offset) = &self.utc_offset {
            offset.write_to(sink)?;
        }
        if let Some(timezone) = &self.timezone {
            timezone.write_to(sink)?;
        }
        if let Some(calendar) = &self.calendar {
            calendar.write_to(sink)?;
        }

        Ok(())
    }
}

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
    .map_err(|e| TemporalError::range().with_message(format!("{e}")))?;

    if critical_duplicate_calendar {
        // TODO: Add tests for the below.
        // Parser handles non-matching calendar, so the value thrown here should only be duplicates.
        return Err(TemporalError::range()
            .with_message("Duplicate calendar value with critical flag found."));
    }

    // Validate that the DateRecord exists.
    if record.date.is_none() {
        return Err(
            TemporalError::range().with_message("DateTime strings must contain a Date value.")
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

#[cfg(test)]
mod tests {
    use crate::parsers::{FormattableTime, Precision};

    use super::{write_u32_to_ascii_digits, FormattableDate, FormattableOffset};

    #[test]
    fn offset_string() {
        let offset = FormattableOffset {
            sign: crate::Sign::Positive,
            time: FormattableTime {
                hour: 4,
                minute: 0,
                second: 0,
                nanosecond: 0,
                precision: Precision::Minute,
                include_sep: true,
            },
        };
        assert_eq!(offset.to_string(), "+04:00");

        let offset = FormattableOffset {
            sign: crate::Sign::Negative,
            time: FormattableTime {
                hour: 5,
                minute: 0,
                second: 30,
                nanosecond: 0,
                precision: Precision::Minute,
                include_sep: true,
            },
        };
        assert_eq!(offset.to_string(), "-05:00");

        let offset = FormattableOffset {
            sign: crate::Sign::Negative,
            time: FormattableTime {
                hour: 5,
                minute: 0,
                second: 30,
                nanosecond: 0,
                precision: Precision::Auto,
                include_sep: true,
            },
        };
        assert_eq!(offset.to_string(), "-05:00:30");

        let offset = FormattableOffset {
            sign: crate::Sign::Negative,
            time: FormattableTime {
                hour: 5,
                minute: 0,
                second: 00,
                nanosecond: 123050000,
                precision: Precision::Auto,
                include_sep: true,
            },
        };
        assert_eq!(offset.to_string(), "-05:00:00.12305");
    }

    #[test]
    fn date_string() {
        let date = FormattableDate(987654, 12, 8).to_string();
        assert_eq!(&date, "+987654-12-08");

        let date = FormattableDate(-987654, 12, 8).to_string();
        assert_eq!(&date, "-987654-12-08");

        let date = FormattableDate(0, 12, 8).to_string();
        assert_eq!(&date, "0000-12-08");

        let date = FormattableDate(10_000, 12, 8).to_string();
        assert_eq!(&date, "+010000-12-08");

        let date = FormattableDate(-10_000, 12, 8).to_string();
        assert_eq!(&date, "-010000-12-08");
    }

    #[test]
    fn write_u32_tests() {
        let v = 123_000_000;
        let (output, precision) = write_u32_to_ascii_digits(v);
        assert_eq!(output, [49, 50, 51, 48, 48, 48, 48, 48, 48]);
        assert_eq!(precision, 3);
        assert_eq!(
            unsafe { core::str::from_utf8_unchecked(&output[..precision]) },
            "123"
        );

        let v = 0;
        let (output, precision) = write_u32_to_ascii_digits(v);
        assert_eq!(output, [48, 48, 48, 48, 48, 48, 48, 48, 48]);
        assert_eq!(precision, 0);
        assert_eq!(
            unsafe { core::str::from_utf8_unchecked(&output[..precision]) },
            ""
        );

        let v = 123_020_000;
        let (output, precision) = write_u32_to_ascii_digits(v);
        assert_eq!(output, [49, 50, 51, 48, 50, 48, 48, 48, 48]);
        assert_eq!(precision, 5);
        assert_eq!(
            unsafe { core::str::from_utf8_unchecked(&output[..precision]) },
            "12302"
        );
    }
}
