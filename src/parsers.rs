//! This module implements Temporal Date/Time parsing functionality.
use crate::{
    iso::{year_month_within_limits, IsoDate, IsoDateTime, IsoTime, MAX_ISO_YEAR, MIN_ISO_YEAR},
    options::{DisplayCalendar, DisplayOffset, DisplayTimeZone},
    Sign, TemporalError, TemporalResult,
};
use alloc::{
    borrow::Cow,
    format,
    string::{String, ToString},
};
use ixdtf::{
    encoding::Utf8,
    parsers::IxdtfParser,
    records::{Annotation, DateRecord, IxdtfParseRecord, TimeRecord, TimeZoneRecord, UtcOffsetRecordOrZ},
};
use ixdtf::ParseError;
use writeable::{impl_display_with_writeable, LengthHint, Writeable};

mod timezone;

pub(crate) use timezone::{parse_allowed_timezone_formats, parse_identifier};

/// Validation errors specific to temporal parsing
#[derive(Debug, Clone)]
pub enum TemporalValidationError {
    /// Year is outside the valid temporal range
    InvalidYear(i32),
    /// Combined date/time is outside representable range
    DateTimeOutOfRange,
    /// Parsing error from ixdtf
    ParseError(String),
}

impl TemporalValidationError {
    /// Convert to a TemporalError with appropriate message
    pub fn into_temporal_error(self) -> TemporalError {
        match self {
            Self::InvalidYear(year) => TemporalError::range().with_message(format!(
                "Year {year} is outside valid range ({MIN_ISO_YEAR} to {MAX_ISO_YEAR})"
            )),
            Self::DateTimeOutOfRange => {
                TemporalError::range().with_message("Date/time is outside representable range")
            }
            Self::ParseError(msg) => TemporalError::syntax().with_message(msg),
        }
    }
}

/// Maps ixdtf ParseError to TemporalValidationError
fn map_parse_error(err: ParseError) -> TemporalValidationError {
    use ParseError::*;
    let message = match err {
        InvalidMonthRange => "Month is outside valid range (1-12)".to_string(),
        InvalidDayRange => "Day is outside valid range for the given month/year".to_string(),
        DateYear => "Invalid year format".to_string(),
        DateMonth => "Invalid month format".to_string(),
        DateDay => "Invalid day format".to_string(),
        TimeHour => "Invalid hour format".to_string(),
        TimeMinuteSecond => "Invalid minute or second format".to_string(),
        TimeSecond => "Invalid second format".to_string(),
        FractionPart => "Invalid fractional seconds format".to_string(),
        ParseFloat => "Invalid fractional seconds value".to_string(),
        AbruptEnd { location } => format!("Unexpected end while parsing {location}"),
        InvalidEnd => "Unexpected character at end of input".to_string(),
        _ => format!("Parse error: {err:?}"),
    };
    TemporalValidationError::ParseError(message)
}

// ECMAScript Temporal specific validation
/// Validates a date record for ECMAScript Temporal year limits
fn validate_date_record_impl(record: DateRecord) -> Result<IsoDate, TemporalValidationError> {
    // Only validate ECMAScript Temporal year limits
    if !year_month_within_limits(record.year, record.month) {
        return Err(TemporalValidationError::InvalidYear(record.year));
    }

    Ok(IsoDate::new_unchecked(
        record.year,
        record.month,
        record.day,
    ))
}

/// Creates an IsoTime from a time record
fn validate_time_record_impl(record: TimeRecord) -> Result<IsoTime, TemporalValidationError> {
    // ixdtf validates time components
    IsoTime::from_time_record(record)
        .map_err(|_| TemporalValidationError::ParseError("Invalid time components".to_string()))
}

/// Public parser that wraps `IxdtfParser` and enforces Temporal parsing requirements.
#[derive(Debug, Default)]
pub struct TemporalParser;

impl TemporalParser {
    /// Creates a new `TemporalParser`.
    #[inline]
    pub const fn new() -> Self {
        Self
    }

    /// Parses a string into a `PlainDateTime` compatible record.
    pub fn parse_date_time<'a>(
        &self,
        source: impl Into<Cow<'a, str>>,
    ) -> TemporalResult<ParsedDateTime> {
        let source = source.into();
        let record = parse_date_time(source.as_bytes())?;
        self.validate_and_build_date_time(record)
    }

    /// Parses a string into a `ZonedDateTime` compatible record.
    pub fn parse_zoned_date_time<'a>(
        &self,
        source: impl Into<Cow<'a, str>>,
    ) -> TemporalResult<ParsedZonedDateTime> {
        let source = source.into();
        let record = parse_zoned_date_time(&source)?;
        self.validate_and_build_zoned_date_time(record)
    }

    /// Parses a string into an `Instant` compatible record.
    pub fn parse_instant<'a>(
        &self,
        source: impl Into<Cow<'a, str>>,
    ) -> TemporalResult<ParsedInstant> {
        let source = source.into();
        let record = parse_instant(source.as_bytes())?;
        self.validate_and_build_instant(record)
    }

    /// Parses a string into a `PlainTime` compatible record.
    pub fn parse_time<'a>(&self, source: impl Into<Cow<'a, str>>) -> TemporalResult<ParsedTime> {
        let source = source.into();
        let record = parse_time(source.as_bytes())?;
        self.validate_and_build_time(record)
    }

    /// Parses a string into a `PlainYearMonth` compatible record.
    pub fn parse_year_month<'a>(
        &self,
        source: impl Into<Cow<'a, str>>,
    ) -> TemporalResult<ParsedYearMonth> {
        let source = source.into();
        let record = parse_year_month(source.as_bytes())?;
        self.validate_and_build_year_month(record)
    }

    /// Parses a string into a `PlainMonthDay` compatible record.
    pub fn parse_month_day<'a>(
        &self,
        source: impl Into<Cow<'a, str>>,
    ) -> TemporalResult<ParsedMonthDay> {
        let source = source.into();
        let record = parse_month_day(source.as_bytes())?;
        self.validate_and_build_month_day(record)
    }

    // Private validation methods that enforce invariants

    fn validate_and_build_date_time(
        &self,
        record: IxdtfParseRecord,
    ) -> TemporalResult<ParsedDateTime> {
        let date_record = record.date.ok_or_else(|| {
            TemporalError::range().with_message("Date component is required for DateTime parsing")
        })?;

        let time_record = record.time.ok_or_else(|| {
            TemporalError::range().with_message("Time component is required for DateTime parsing")
        })?;

        let iso_date = self.validate_date_record(date_record)?;
        let iso_time = self.validate_time_record(time_record)?;

        // Validate DateTime is within valid limits
        let iso_datetime = IsoDateTime::new(iso_date, iso_time)?;

        Ok(ParsedDateTime {
            iso: iso_datetime,
            calendar: record
                .calendar
                .map(|c| String::from_utf8_lossy(c).into_owned()),
            offset: record.offset,
        })
    }

    fn validate_and_build_zoned_date_time(
        &self,
        record: IxdtfParseRecord,
    ) -> TemporalResult<ParsedZonedDateTime> {
        let date_record = record.date.ok_or_else(|| {
            TemporalError::range()
                .with_message("Date component is required for ZonedDateTime parsing")
        })?;

        let time_record = record.time.ok_or_else(|| {
            TemporalError::range()
                .with_message("Time component is required for ZonedDateTime parsing")
        })?;

        let timezone = record.tz.ok_or_else(|| {
            TemporalError::range()
                .with_message("TimeZone annotation is required for ZonedDateTime parsing")
        })?;

        let iso_date = self.validate_date_record(date_record)?;
        let iso_time = self.validate_time_record(time_record)?;

        // Validate DateTime is within valid limits
        let iso_datetime = IsoDateTime::new(iso_date, iso_time)?;

        // Extract timezone identifier from the timezone record
        let timezone_bytes = match timezone.tz {
            TimeZoneRecord::Name(name_bytes) => name_bytes.to_vec(),
            TimeZoneRecord::Offset(_) => {
                return Err(
                    TemporalError::range().with_message("Expected timezone name but found offset")
                );
            }
            _ => {
                return Err(TemporalError::range().with_message("Unsupported timezone record type"));
            }
        };

        Ok(ParsedZonedDateTime {
            iso: iso_datetime,
            calendar: record
                .calendar
                .map(|c| String::from_utf8_lossy(c).into_owned()),
            offset: record.offset,
            timezone: String::from_utf8_lossy(&timezone_bytes).into_owned(),
        })
    }

    fn validate_and_build_instant(
        &self,
        record: IxdtfParseInstantRecord,
    ) -> TemporalResult<ParsedInstant> {
        let iso_date = self.validate_date_record(record.date)?;
        let iso_time = self.validate_time_record(record.time)?;

        // Validate DateTime is within valid limits
        let iso_datetime = IsoDateTime::new(iso_date, iso_time)?;

        Ok(ParsedInstant {
            iso: iso_datetime,
            offset: record.offset,
        })
    }

    fn validate_and_build_time(&self, record: TimeRecord) -> TemporalResult<ParsedTime> {
        let iso_time = self.validate_time_record(record)?;

        Ok(ParsedTime { iso: iso_time })
    }

    fn validate_and_build_year_month(
        &self,
        record: IxdtfParseRecord,
    ) -> TemporalResult<ParsedYearMonth> {
        let date_record = record.date.ok_or_else(|| {
            TemporalError::range().with_message("Date component is required for YearMonth parsing")
        })?;

        let iso_date = self.validate_date_record(date_record)?;

        Ok(ParsedYearMonth {
            iso: iso_date,
            calendar: record
                .calendar
                .map(|c| String::from_utf8_lossy(c).into_owned()),
        })
    }

    fn validate_and_build_month_day(
        &self,
        record: IxdtfParseRecord,
    ) -> TemporalResult<ParsedMonthDay> {
        let date_record = record.date.ok_or_else(|| {
            TemporalError::range().with_message("Date component is required for MonthDay parsing")
        })?;

        let iso_date = self.validate_date_record(date_record)?;

        Ok(ParsedMonthDay {
            iso: iso_date,
            calendar: record
                .calendar
                .map(|c| String::from_utf8_lossy(c).into_owned()),
        })
    }

    /// Validates a date record using the shared validation logic
    fn validate_date_record(&self, record: DateRecord) -> TemporalResult<IsoDate> {
        validate_date_record_impl(record).map_err(|e| e.into_temporal_error())
    }

    /// Validates a time record using the shared validation logic
    fn validate_time_record(&self, record: TimeRecord) -> TemporalResult<IsoTime> {
        validate_time_record_impl(record).map_err(|e| e.into_temporal_error())
    }
}

/// Parsed result for PlainDateTime operations
#[derive(Debug, Clone)]
pub struct ParsedDateTime {
    /// The validated ISO DateTime components
    pub iso: IsoDateTime,
    /// Optional calendar identifier as a string
    pub calendar: Option<String>,
    /// Optional UTC offset information
    pub offset: Option<UtcOffsetRecordOrZ>,
}

impl ParsedDateTime {
    /// Get the calendar identifier as a string slice, defaulting to "iso8601"
    pub fn calendar(&self) -> &str {
        self.calendar.as_deref().unwrap_or("iso8601")
    }
}

/// Parsed result for ZonedDateTime operations
#[derive(Debug, Clone)]
pub struct ParsedZonedDateTime {
    /// The validated ISO DateTime components
    pub iso: IsoDateTime,
    /// Optional calendar identifier as a string
    pub calendar: Option<String>,
    /// Optional UTC offset information
    pub offset: Option<UtcOffsetRecordOrZ>,
    /// Time zone identifier as a string
    pub timezone: String,
}

impl ParsedZonedDateTime {
    /// Get the calendar identifier as a string slice, defaulting to "iso8601"
    pub fn calendar(&self) -> &str {
        self.calendar.as_deref().unwrap_or("iso8601")
    }

    /// Get the timezone identifier as a string slice
    pub fn timezone(&self) -> &str {
        &self.timezone
    }
}

/// Parsed result for Instant operations
#[derive(Debug, Clone)]
pub struct ParsedInstant {
    /// The validated ISO DateTime components
    pub iso: IsoDateTime,
    /// UTC offset information (required for instants)
    pub offset: UtcOffsetRecordOrZ,
}

/// Parsed result for PlainTime operations
#[derive(Debug, Clone)]
pub struct ParsedTime {
    /// The validated ISO Time components
    pub iso: IsoTime,
}

/// Parsed result for PlainYearMonth operations
#[derive(Debug, Clone)]
pub struct ParsedYearMonth {
    /// The validated ISO Date components
    pub iso: IsoDate,
    /// Optional calendar identifier as a string
    pub calendar: Option<String>,
}

impl ParsedYearMonth {
    /// Get the calendar identifier as a string slice, defaulting to "iso8601"
    pub fn calendar(&self) -> &str {
        self.calendar.as_deref().unwrap_or("iso8601")
    }
}

/// Parsed result for PlainMonthDay operations
#[derive(Debug, Clone)]
pub struct ParsedMonthDay {
    /// The validated ISO Date components
    pub iso: IsoDate,
    /// Optional calendar identifier as a string
    pub calendar: Option<String>,
}

impl ParsedMonthDay {
    /// Get the calendar identifier as a string slice, defaulting to "iso8601"
    pub fn calendar(&self) -> &str {
        self.calendar.as_deref().unwrap_or("iso8601")
    }
}

// TODO: Move `Writeable` functionality to `ixdtf` crate

#[derive(Debug, Default)]
pub struct IxdtfStringBuilder<'a> {
    inner: FormattableIxdtf<'a>,
}

impl<'a> IxdtfStringBuilder<'a> {
    pub fn with_date(mut self, iso: IsoDate) -> Self {
        self.inner.date = Some(FormattableDate(iso.year, iso.month, iso.day));
        self
    }

    pub fn with_time(mut self, time: IsoTime, precision: Precision) -> Self {
        let nanosecond = (time.millisecond as u32 * 1_000_000)
            + (time.microsecond as u32 * 1000)
            + time.nanosecond as u32;

        self.inner.time = Some(FormattableTime {
            hour: time.hour,
            minute: time.minute,
            second: time.second,
            nanosecond,
            precision,
            include_sep: true,
        });
        self
    }

    pub fn with_minute_offset(
        mut self,
        sign: Sign,
        hour: u8,
        minute: u8,
        show: DisplayOffset,
    ) -> Self {
        let time = FormattableTime {
            hour,
            minute,
            second: 9,
            nanosecond: 0,
            precision: Precision::Minute,
            include_sep: true,
        };

        self.inner.utc_offset = Some(FormattableUtcOffset {
            show,
            offset: UtcOffset::Offset(FormattableOffset { sign, time }),
        });
        self
    }

    pub fn with_z(mut self, show: DisplayOffset) -> Self {
        self.inner.utc_offset = Some(FormattableUtcOffset {
            show,
            offset: UtcOffset::Z,
        });
        self
    }

    pub fn with_timezone(mut self, timezone: &'a str, show: DisplayTimeZone) -> Self {
        self.inner.timezone = Some(FormattableTimeZone { show, timezone });
        self
    }

    pub fn with_calendar(mut self, calendar: &'static str, show: DisplayCalendar) -> Self {
        self.inner.calendar = Some(FormattableCalendar { show, calendar });
        self
    }

    pub fn build(self) -> alloc::string::String {
        self.inner.to_string()
    }
}

impl Writeable for IxdtfStringBuilder<'_> {
    fn write_to<W: core::fmt::Write + ?Sized>(&self, sink: &mut W) -> core::fmt::Result {
        self.inner.write_to(sink)
    }

    fn writeable_length_hint(&self) -> LengthHint {
        self.inner.writeable_length_hint()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precision {
    #[default]
    Auto,
    Minute,
    Digit(u8),
}

#[derive(Debug)]
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
        if (self.nanosecond == 0 && self.precision == Precision::Auto)
            || self.precision == Precision::Digit(0)
        {
            return Ok(());
        }
        sink.write_char('.')?;
        write_nanosecond(self.nanosecond, self.precision, sink)
    }

    fn writeable_length_hint(&self) -> LengthHint {
        let sep = self.include_sep as usize;
        if self.precision == Precision::Minute {
            return LengthHint::exact(4 + sep);
        }
        let time_base = 6 + (sep * 2);
        if self.nanosecond == 0 || self.precision == Precision::Digit(0) {
            return LengthHint::exact(time_base);
        }
        if let Precision::Digit(d) = self.precision {
            return LengthHint::exact(time_base + 1 + d as usize);
        }
        LengthHint::between(time_base + 2, time_base + 10)
    }
}

#[derive(Debug)]
pub struct FormattableUtcOffset {
    pub show: DisplayOffset,
    pub offset: UtcOffset,
}

#[derive(Debug)]
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

    fn writeable_length_hint(&self) -> LengthHint {
        match &self.offset {
            UtcOffset::Z => LengthHint::exact(1),
            UtcOffset::Offset(o) => o.writeable_length_hint(),
        }
    }
}

fn write_padded_u8<W: core::fmt::Write + ?Sized>(num: u8, sink: &mut W) -> core::fmt::Result {
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
    let (digits, index) = u32_to_digits(nanoseconds);
    let precision = match precision {
        Precision::Digit(digit) if digit <= 9 => digit as usize,
        _ => index,
    };
    write_digit_slice_to_precision(digits, 0, precision, sink)
}

pub fn u32_to_digits(mut value: u32) -> ([u8; 9], usize) {
    let mut output = [0; 9];
    let mut precision = 0;
    let mut i = 9;
    while i != 0 {
        let v = (value % 10) as u8;
        value /= 10;
        if precision == 0 && v != 0 {
            precision = i;
        }
        output[i - 1] = v;
        i -= 1;
    }

    (output, precision)
}

pub fn write_digit_slice_to_precision<W: core::fmt::Write + ?Sized>(
    digits: [u8; 9],
    base: usize,
    precision: usize,
    sink: &mut W,
) -> core::fmt::Result {
    for digit in digits.iter().take(precision).skip(base) {
        digit.write_to(sink)?;
    }
    Ok(())
}

#[derive(Debug)]
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

    fn writeable_length_hint(&self) -> LengthHint {
        self.time.writeable_length_hint() + 1
    }
}

impl_display_with_writeable!(FormattableIxdtf<'_>);
impl_display_with_writeable!(FormattableMonthDay<'_>);
impl_display_with_writeable!(FormattableYearMonth<'_>);
impl_display_with_writeable!(FormattableDuration);
impl_display_with_writeable!(FormattableDate);
impl_display_with_writeable!(FormattableTime);
impl_display_with_writeable!(FormattableUtcOffset);
impl_display_with_writeable!(FormattableOffset);
impl_display_with_writeable!(FormattableTimeZone<'_>);
impl_display_with_writeable!(FormattableCalendar<'_>);

#[derive(Debug)]
pub struct FormattableDate(pub i32, pub u8, pub u8);

impl Writeable for FormattableDate {
    fn write_to<W: core::fmt::Write + ?Sized>(&self, sink: &mut W) -> core::fmt::Result {
        write_year(self.0, sink)?;
        sink.write_char('-')?;
        write_padded_u8(self.1, sink)?;
        sink.write_char('-')?;
        write_padded_u8(self.2, sink)
    }

    fn writeable_length_hint(&self) -> LengthHint {
        let year_length = if (0..=9999).contains(&self.0) { 4 } else { 7 };

        LengthHint::exact(6 + year_length)
    }
}

fn write_year<W: core::fmt::Write + ?Sized>(year: i32, sink: &mut W) -> core::fmt::Result {
    if (0..=9999).contains(&year) {
        write_four_digit_year(year, sink)
    } else {
        write_extended_year(year, sink)
    }
}

fn write_four_digit_year<W: core::fmt::Write + ?Sized>(
    mut y: i32,
    sink: &mut W,
) -> core::fmt::Result {
    (y / 1_000).write_to(sink)?;
    y %= 1_000;
    (y / 100).write_to(sink)?;
    y %= 100;
    (y / 10).write_to(sink)?;
    y %= 10;
    y.write_to(sink)
}

fn write_extended_year<W: core::fmt::Write + ?Sized>(y: i32, sink: &mut W) -> core::fmt::Result {
    let sign = if y < 0 { '-' } else { '+' };
    sink.write_char(sign)?;
    let (digits, _) = u32_to_digits(y.unsigned_abs());
    // SAFETY: digits slice is made up up valid ASCII digits.
    write_digit_slice_to_precision(digits, 3, 9, sink)
}

#[derive(Debug)]
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

    fn writeable_length_hint(&self) -> writeable::LengthHint {
        if self.show == DisplayTimeZone::Never {
            return LengthHint::exact(0);
        }
        let critical = (self.show == DisplayTimeZone::Critical) as usize;
        LengthHint::exact(2 + critical + self.timezone.len())
    }
}

#[derive(Debug)]
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

    fn writeable_length_hint(&self) -> LengthHint {
        if self.show == DisplayCalendar::Never
            || self.show == DisplayCalendar::Auto && self.calendar == "iso8601"
        {
            return LengthHint::exact(0);
        }
        let critical = (self.show == DisplayCalendar::Critical) as usize;
        LengthHint::exact(7 + critical + self.calendar.len())
    }
}

#[derive(Debug)]
pub struct FormattableMonthDay<'a> {
    pub date: FormattableDate,
    pub calendar: FormattableCalendar<'a>,
}

impl Writeable for FormattableMonthDay<'_> {
    fn write_to<W: core::fmt::Write + ?Sized>(&self, sink: &mut W) -> core::fmt::Result {
        if self.calendar.show == DisplayCalendar::Always
            || self.calendar.show == DisplayCalendar::Critical
            || self.calendar.calendar != "iso8601"
        {
            write_year(self.date.0, sink)?;
            sink.write_char('-')?;
        }
        write_padded_u8(self.date.1, sink)?;
        sink.write_char('-')?;
        write_padded_u8(self.date.2, sink)?;
        self.calendar.write_to(sink)
    }

    fn writeable_length_hint(&self) -> LengthHint {
        let base_length = self.calendar.writeable_length_hint() + LengthHint::exact(5);
        if self.calendar.show == DisplayCalendar::Always
            || self.calendar.show == DisplayCalendar::Critical
            || self.calendar.calendar != "iso8601"
        {
            let year_length = if (0..=9999).contains(&self.date.0) {
                4
            } else {
                7
            };
            return base_length + LengthHint::exact(year_length);
        }
        base_length
    }
}

#[derive(Debug)]
pub struct FormattableYearMonth<'a> {
    pub date: FormattableDate,
    pub calendar: FormattableCalendar<'a>,
}

impl Writeable for FormattableYearMonth<'_> {
    fn write_to<W: core::fmt::Write + ?Sized>(&self, sink: &mut W) -> core::fmt::Result {
        write_year(self.date.0, sink)?;
        sink.write_char('-')?;
        write_padded_u8(self.date.1, sink)?;
        if self.calendar.show == DisplayCalendar::Always
            || self.calendar.show == DisplayCalendar::Critical
            || self.calendar.calendar != "iso8601"
        {
            sink.write_char('-')?;
            write_padded_u8(self.date.2, sink)?;
        }

        self.calendar.write_to(sink)
    }

    fn writeable_length_hint(&self) -> LengthHint {
        let year_length = if (0..=9999).contains(&self.date.0) {
            4
        } else {
            7
        };
        let base_length =
            self.calendar.writeable_length_hint() + LengthHint::exact(year_length + 3);
        if self.calendar.show == DisplayCalendar::Always
            || self.calendar.show == DisplayCalendar::Critical
            || self.calendar.calendar != "iso8601"
        {
            return base_length + LengthHint::exact(3);
        }
        base_length
    }
}

#[derive(Debug, Default)]
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

    fn writeable_length_hint(&self) -> LengthHint {
        let date_length = self
            .date
            .as_ref()
            .map(|d| d.writeable_length_hint())
            .unwrap_or(LengthHint::exact(0));
        let time_length = self
            .time
            .as_ref()
            .map(|t| {
                let t_present = self.date.is_some() as usize;
                t.writeable_length_hint() + t_present
            })
            .unwrap_or(LengthHint::exact(0));
        let utc_length = self
            .utc_offset
            .as_ref()
            .map(|utc| utc.writeable_length_hint())
            .unwrap_or(LengthHint::exact(0));
        let timezone_length = self
            .timezone
            .as_ref()
            .map(|tz| tz.writeable_length_hint())
            .unwrap_or(LengthHint::exact(0));
        let cal_length = self
            .calendar
            .as_ref()
            .map(|cal| cal.writeable_length_hint())
            .unwrap_or(LengthHint::exact(0));

        date_length + time_length + utc_length + timezone_length + cal_length
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FormattableDateDuration {
    pub years: u32,
    pub months: u32,
    pub weeks: u32,
    pub days: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum FormattableTimeDuration {
    Hours(u64, Option<u32>),
    Minutes(u64, u64, Option<u32>),
    Seconds(u64, u64, u64, Option<u32>),
}

pub struct FormattableDuration {
    pub precision: Precision,
    pub sign: Sign,
    pub date: Option<FormattableDateDuration>,
    pub time: Option<FormattableTimeDuration>,
}

impl Writeable for FormattableDuration {
    fn write_to<W: core::fmt::Write + ?Sized>(&self, sink: &mut W) -> core::fmt::Result {
        if self.sign == Sign::Negative {
            sink.write_char('-')?;
        }
        sink.write_char('P')?;
        if let Some(date) = self.date {
            checked_write_u32_with_suffix(date.years, 'Y', sink)?;
            checked_write_u32_with_suffix(date.months, 'M', sink)?;
            checked_write_u32_with_suffix(date.weeks, 'W', sink)?;
            checked_write_u64_with_suffix(date.days, 'D', sink)?;
        }
        if let Some(time) = self.time {
            match time {
                FormattableTimeDuration::Hours(hours, fraction) => {
                    let ns = fraction.unwrap_or(0);
                    if hours + u64::from(ns) != 0 {
                        sink.write_char('T')?;
                    }
                    if hours == 0 {
                        return Ok(());
                    }
                    hours.write_to(sink)?;
                    if ns != 0 {
                        sink.write_char('.')?;
                        ns.write_to(sink)?;
                    }
                    sink.write_char('H')?;
                }
                FormattableTimeDuration::Minutes(hours, minutes, fraction) => {
                    let ns = fraction.unwrap_or(0);
                    if hours + minutes + u64::from(ns) != 0 {
                        sink.write_char('T')?;
                    }
                    checked_write_u64_with_suffix(hours, 'H', sink)?;
                    if minutes == 0 {
                        return Ok(());
                    }
                    minutes.write_to(sink)?;
                    if ns != 0 {
                        sink.write_char('.')?;
                        ns.write_to(sink)?;
                    }
                    sink.write_char('M')?;
                }
                FormattableTimeDuration::Seconds(hours, minutes, seconds, fraction) => {
                    let ns = fraction.unwrap_or(0);
                    let unit_below_minute = self.date.is_none() && hours == 0 && minutes == 0;

                    let write_second = seconds != 0
                        || unit_below_minute
                        || matches!(self.precision, Precision::Digit(_));

                    if hours != 0 || minutes != 0 || write_second {
                        sink.write_char('T')?;
                    }

                    checked_write_u64_with_suffix(hours, 'H', sink)?;
                    checked_write_u64_with_suffix(minutes, 'M', sink)?;
                    if write_second {
                        seconds.write_to(sink)?;
                        if self.precision == Precision::Digit(0)
                            || (self.precision == Precision::Auto && ns == 0)
                        {
                            sink.write_char('S')?;
                            return Ok(());
                        }
                        sink.write_char('.')?;
                        write_nanosecond(ns, self.precision, sink)?;
                        sink.write_char('S')?;
                    }
                }
            }
        }
        Ok(())
    }
}

fn checked_write_u32_with_suffix<W: core::fmt::Write + ?Sized>(
    val: u32,
    suffix: char,
    sink: &mut W,
) -> core::fmt::Result {
    if val == 0 {
        return Ok(());
    }
    val.write_to(sink)?;
    sink.write_char(suffix)
}

fn checked_write_u64_with_suffix<W: core::fmt::Write + ?Sized>(
    val: u64,
    suffix: char,
    sink: &mut W,
) -> core::fmt::Result {
    if val == 0 {
        return Ok(());
    }
    val.write_to(sink)?;
    sink.write_char(suffix)
}

// TODO: Determine if these should be separate structs, i.e. TemporalDateTimeParser/TemporalInstantParser, or
// maybe on global `TemporalParser` around `IxdtfParser` that handles the Temporal idiosyncracies.
#[derive(PartialEq)]
enum ParseVariant {
    YearMonth,
    MonthDay,
    DateTime,
    Time,
}

#[inline]
fn parse_ixdtf(source: &[u8], variant: ParseVariant) -> TemporalResult<IxdtfParseRecord<Utf8>> {
    fn cast_handler<'a>(
        _: &mut IxdtfParser<'a, Utf8>,
        handler: impl FnMut(Annotation<'a, Utf8>) -> Option<Annotation<'a, Utf8>>,
    ) -> impl FnMut(Annotation<'a, Utf8>) -> Option<Annotation<'a, Utf8>> {
        handler
    }

    let mut first_calendar: Option<Annotation<Utf8>> = None;
    let mut critical_duplicate_calendar = false;
    let mut parser = IxdtfParser::from_utf8(source);

    let handler = cast_handler(&mut parser, |annotation: Annotation<Utf8>| {
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
        ParseVariant::Time => parser.parse_time_with_annotation_handler(handler),
    }
    .map_err(|e| map_parse_error(e).into_temporal_error())?;

    if critical_duplicate_calendar {
        // TODO: Add tests for the below.
        // Parser handles non-matching calendar, so the value thrown here should only be duplicates.
        return Err(TemporalError::range()
            .with_message("Duplicate calendar value with critical flag found."));
    }

    // Validate that the DateRecord exists.
    if variant != ParseVariant::Time && record.date.is_none() {
        return Err(
            TemporalError::range().with_message("DateTime strings must contain a Date value.")
        );
    }

    record.calendar = first_calendar.map(|v| v.value);

    Ok(record)
}

/// A utility function for parsing a `DateTime` string
#[inline]
pub(crate) fn parse_date_time(source: &[u8]) -> TemporalResult<IxdtfParseRecord<Utf8>> {
    let record = parse_ixdtf(source, ParseVariant::DateTime)?;

    if record.offset == Some(UtcOffsetRecordOrZ::Z) {
        return Err(TemporalError::range()
            .with_message("UTC designator is not valid for DateTime parsing."));
    }

    // Only validate ECMAScript Temporal specific requirements
    if let Some(date_record) = record.date {
        validate_date_record_impl(date_record).map_err(|e| e.into_temporal_error())?;
    }
    if let Some(time_record) = record.time {
        validate_time_record_impl(time_record).map_err(|e| e.into_temporal_error())?;
    }

    Ok(record)
}

#[inline]
pub(crate) fn parse_zoned_date_time(source: &str) -> TemporalResult<IxdtfParseRecord<Utf8>> {
    let record = parse_ixdtf(source.as_bytes(), ParseVariant::DateTime)?;

    // TODO: Support rejecting subminute precision in time zone annotations
    if record.tz.is_none() {
        return Err(TemporalError::range()
            .with_message("Time zone annotation is required for parsing a zoned date time."));
    }

    // Only validate ECMAScript Temporal specific requirements
    if let Some(date_record) = record.date {
        validate_date_record_impl(date_record).map_err(|e| e.into_temporal_error())?;
    }
    if let Some(time_record) = record.time {
        validate_time_record_impl(time_record).map_err(|e| e.into_temporal_error())?;
    }

    Ok(record)
}

pub(crate) struct IxdtfParseInstantRecord {
    pub(crate) date: DateRecord,
    pub(crate) time: TimeRecord,
    pub(crate) offset: UtcOffsetRecordOrZ,
}

/// A utility function for parsing an `Instant` string
#[inline]
pub(crate) fn parse_instant(source: &[u8]) -> TemporalResult<IxdtfParseInstantRecord> {
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

    // Only validate ECMAScript Temporal specific requirements
    validate_date_record_impl(date).map_err(|e| e.into_temporal_error())?;
    validate_time_record_impl(time).map_err(|e| e.into_temporal_error())?;

    Ok(IxdtfParseInstantRecord { date, time, offset })
}

// Ensure that the record does not have an offset element.
//
// This handles the [~Zoned] in TemporalFooString productions
fn check_offset(record: IxdtfParseRecord<Utf8>) -> TemporalResult<IxdtfParseRecord<Utf8>> {
    if record.offset == Some(UtcOffsetRecordOrZ::Z) {
        return Err(TemporalError::range()
            .with_message("UTC designator is not valid for plain date/time parsing."));
    }
    Ok(record)
}

/// A utility function for parsing a `YearMonth` string
#[inline]
pub(crate) fn parse_year_month(source: &[u8]) -> TemporalResult<IxdtfParseRecord<Utf8>> {
    let ym_record = parse_ixdtf(source, ParseVariant::YearMonth);

    let Err(ref e) = ym_record else {
        return ym_record.and_then(check_offset);
    };

    let dt_parse = parse_date_time(source);

    match dt_parse {
        Ok(dt) => check_offset(dt),
        // Format and return the error from parsing YearMonth.
        _ => Err(TemporalError::range().with_message(format!("{e}"))),
    }
}

/// A utilty function for parsing a `MonthDay` String.
pub(crate) fn parse_month_day(source: &[u8]) -> TemporalResult<IxdtfParseRecord<Utf8>> {
    let md_record = parse_ixdtf(source, ParseVariant::MonthDay);
    let Err(ref e) = md_record else {
        return md_record.and_then(check_offset);
    };

    let dt_parse = parse_date_time(source);

    match dt_parse {
        Ok(dt) => check_offset(dt),
        // Format and return the error from parsing MonthDay.
        _ => Err(TemporalError::range().with_message(format!("{e}"))),
    }
}

// Ensures that an IxdtfParseRecord was parsed with [~Zoned][+TimeRequired]
fn check_time_record(record: IxdtfParseRecord<Utf8>) -> TemporalResult<TimeRecord> {
    // Handle [~Zoned]
    let record = check_offset(record)?;
    // Handle [+TimeRequired]
    let Some(time) = record.time else {
        return Err(TemporalError::range()
            .with_message("PlainTime can only be parsed from strings with a time component."));
    };
    Ok(time)
}

#[inline]
pub(crate) fn parse_time(source: &[u8]) -> TemporalResult<TimeRecord> {
    let time_record = parse_ixdtf(source, ParseVariant::Time);

    let Err(ref e) = time_record else {
        return time_record.and_then(check_time_record);
    };

    let dt_parse = parse_date_time(source);

    match dt_parse {
        Ok(dt) => check_time_record(dt),
        // Format and return the error from parsing MonthDay.
        _ => Err(TemporalError::range().with_message(format!("{e}"))),
    }
}

/// Consider this API to be unstable: it is used internally by temporal_capi but
/// will likely be replaced with a proper TemporalParser API at some point.
#[inline]
pub fn parse_allowed_calendar_formats(s: &[u8]) -> Option<&[u8]> {
    if let Ok(r) = parse_ixdtf(s, ParseVariant::DateTime).map(|r| r.calendar) {
        return Some(r.unwrap_or(&[]));
    } else if let Ok(r) = IxdtfParser::from_utf8(s).parse_time().map(|r| r.calendar) {
        return Some(r.unwrap_or(&[]));
    } else if let Ok(r) = parse_ixdtf(s, ParseVariant::YearMonth).map(|r| r.calendar) {
        return Some(r.unwrap_or(&[]));
    } else if let Ok(r) = parse_ixdtf(s, ParseVariant::MonthDay).map(|r| r.calendar) {
        return Some(r.unwrap_or(&[]));
    }
    None
}

// TODO: ParseTimeZoneString, ParseZonedDateTimeString

#[cfg(test)]
mod tests {
    use super::{FormattableDate, FormattableOffset, TemporalParser};
    use crate::parsers::{FormattableTime, Precision};
    use alloc::{format, string::String};
    use writeable::assert_writeable_eq;

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
        assert_writeable_eq!(offset, "+04:00");

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
        assert_writeable_eq!(offset, "-05:00");

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
        assert_writeable_eq!(offset, "-05:00:30");

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
        assert_writeable_eq!(offset, "-05:00:00.12305");
    }

    #[test]
    fn time_to_precision() {
        let time = FormattableTime {
            hour: 5,
            minute: 0,
            second: 00,
            nanosecond: 123050000,
            precision: Precision::Digit(8),
            include_sep: true,
        };
        assert_writeable_eq!(time, "05:00:00.12305000");

        let time = FormattableTime {
            hour: 5,
            minute: 0,
            second: 00,
            nanosecond: 123050002,
            precision: Precision::Digit(9),
            include_sep: true,
        };
        assert_writeable_eq!(time, "05:00:00.123050002");

        let time = FormattableTime {
            hour: 5,
            minute: 0,
            second: 00,
            nanosecond: 123050000,
            precision: Precision::Digit(1),
            include_sep: true,
        };
        assert_writeable_eq!(time, "05:00:00.1");

        let time = FormattableTime {
            hour: 5,
            minute: 0,
            second: 00,
            nanosecond: 123050000,
            precision: Precision::Digit(0),
            include_sep: true,
        };
        assert_writeable_eq!(time, "05:00:00");
    }

    #[test]
    fn date_string() {
        let date = FormattableDate(2024, 12, 8);
        assert_writeable_eq!(date, "2024-12-08");

        let date = FormattableDate(987654, 12, 8);
        assert_writeable_eq!(date, "+987654-12-08");

        let date = FormattableDate(-987654, 12, 8);
        assert_writeable_eq!(date, "-987654-12-08");

        let date = FormattableDate(0, 12, 8);
        assert_writeable_eq!(date, "0000-12-08");

        let date = FormattableDate(10_000, 12, 8);
        assert_writeable_eq!(date, "+010000-12-08");

        let date = FormattableDate(-10_000, 12, 8);
        assert_writeable_eq!(date, "-010000-12-08");
    }

    #[test]
    fn temporal_parser_date_time() {
        let parser = TemporalParser::new();

        // Test basic datetime parsing
        let result = parser.parse_date_time("2025-01-15T14:30:00");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.iso.date.year, 2025);
        assert_eq!(parsed.iso.date.month, 1);
        assert_eq!(parsed.iso.date.day, 15);
        assert_eq!(parsed.iso.time.hour, 14);
        assert_eq!(parsed.iso.time.minute, 30);
        assert_eq!(parsed.iso.time.second, 0);

        // Test with calendar annotation
        let result = parser.parse_date_time("2025-01-15T14:30:00[u-ca=gregory]");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(parsed.calendar.is_some());
        assert_eq!(parsed.calendar.unwrap(), "gregory");

        // Test invalid year
        let result = parser.parse_date_time("999999-01-15T14:30:00");
        assert!(result.is_err());

        // Test invalid month
        let result = parser.parse_date_time("2025-13-15T14:30:00");
        assert!(result.is_err());

        // Test invalid day
        let result = parser.parse_date_time("2025-02-30T14:30:00");
        assert!(result.is_err());

        // Test invalid hour
        let result = parser.parse_date_time("2025-01-15T25:30:00");
        assert!(result.is_err());

        // Test invalid minute
        let result = parser.parse_date_time("2025-01-15T14:60:00");
        assert!(result.is_err());
    }

    #[test]
    fn temporal_parser_instant() {
        let parser = TemporalParser::new();

        // Test basic instant parsing with Z
        let result = parser.parse_instant("2025-01-15T14:30:00Z");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.iso.date.year, 2025);

        // Test instant with offset
        let result = parser.parse_instant("2025-01-15T14:30:00+05:30");
        assert!(result.is_ok());

        // Test instant without offset (should fail)
        let result = parser.parse_instant("2025-01-15T14:30:00");
        assert!(result.is_err());
    }

    #[test]
    fn temporal_parser_zoned_date_time() {
        let parser = TemporalParser::new();

        // Test basic zoned datetime parsing
        let result = parser.parse_zoned_date_time("2025-01-15T14:30:00Z[America/New_York]");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.iso.date.year, 2025);
        assert_eq!(parsed.timezone, "America/New_York");

        // Test without timezone annotation (should fail)
        let result = parser.parse_zoned_date_time("2025-01-15T14:30:00Z");
        assert!(result.is_err());
    }

    #[test]
    fn temporal_parser_time() {
        let parser = TemporalParser::new();

        // Test basic time parsing
        let result = parser.parse_time("14:30:00");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.iso.hour, 14);
        assert_eq!(parsed.iso.minute, 30);
        assert_eq!(parsed.iso.second, 0);

        // Test time with fractional seconds
        let result = parser.parse_time("14:30:00.123456789");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.iso.millisecond, 123);
        assert_eq!(parsed.iso.microsecond, 456);
        assert_eq!(parsed.iso.nanosecond, 789);
    }

    #[test]
    fn temporal_parser_year_month() {
        let parser = TemporalParser::new();

        // Test basic year-month parsing
        let result = parser.parse_year_month("2025-01");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.iso.year, 2025);
        assert_eq!(parsed.iso.month, 1);

        // Test with calendar
        let result = parser.parse_year_month("2025-01[u-ca=hebrew]");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(parsed.calendar.is_some());
        assert_eq!(parsed.calendar.unwrap(), "hebrew");
    }

    #[test]
    fn temporal_parser_month_day() {
        let parser = TemporalParser::new();

        // Test basic month-day parsing
        let result = parser.parse_month_day("01-15");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.iso.month, 1);
        assert_eq!(parsed.iso.day, 15);

        // Test leap day validation
        let result = parser.parse_month_day("02-29");
        assert!(result.is_ok()); // Should be OK as it could be valid in a leap year
    }

    #[test]
    fn temporal_parser_invariant_validation() {
        let parser = TemporalParser::new();

        // Test year limits - out of range years should fail
        let result = parser.parse_date_time("-271822-01-01T00:00:00");
        assert!(result.is_err());

        let result = parser.parse_date_time("275761-01-01T00:00:00");
        assert!(result.is_err());

        // Test valid dates with normal years
        let result = parser.parse_date_time("2025-01-01T12:00:00");
        assert!(result.is_ok());

        let result = parser.parse_date_time("1970-01-01T12:00:00");
        assert!(result.is_ok());

        // Test invalid days for specific months
        let result = parser.parse_date_time("2025-04-31T00:00:00"); // April has only 30 days
        assert!(result.is_err());

        let result = parser.parse_date_time("2025-02-29T00:00:00"); // 2025 is not a leap year
        assert!(result.is_err());

        let result = parser.parse_date_time("2024-02-29T00:00:00"); // 2024 is a leap year
        assert!(result.is_ok());
    }

    #[test]
    fn temporal_parser_cow_strings() {
        let parser = TemporalParser::new();

        // Test with &str
        let result = parser.parse_date_time("2025-01-15T14:30:00");
        assert!(result.is_ok());

        // Test with String
        let owned = String::from("2025-01-15T14:30:00");
        let result = parser.parse_date_time(&owned);
        assert!(result.is_ok());

        // Test with owned String directly
        let owned = String::from("2025-01-15T14:30:00");
        let result = parser.parse_date_time(owned);
        assert!(result.is_ok());

        // Test with Cow::Borrowed
        use alloc::borrow::Cow;
        let cow_borrowed: Cow<str> = Cow::Borrowed("2025-01-15T14:30:00");
        let result = parser.parse_date_time(cow_borrowed);
        assert!(result.is_ok());

        // Test with Cow::Owned
        let cow_owned: Cow<str> = Cow::Owned(String::from("2025-01-15T14:30:00"));
        let result = parser.parse_date_time(cow_owned);
        assert!(result.is_ok());
    }

    #[test]
    fn temporal_parser_better_error_messages() {
        let parser = TemporalParser::new();

        // Test that invalid inputs produce errors
        let result = parser.parse_date_time("999999-01-15T14:30:00");
        assert!(result.is_err());

        let result = parser.parse_date_time("2025-04-31T14:30:00"); // April only has 30 days
        assert!(result.is_err());

        // Test that our validation functions produce the expected error messages directly
        use super::validate_date_record_impl;
        use ixdtf::parsers::records::DateRecord;

        // Test that basic day validation is now handled by ixdtf, not our validator
        let invalid_day_record = DateRecord {
            year: 2025,
            month: 4,
            day: 31,
        };
        // This should pass because validate_date_record_impl only checks year limits now
        // Day validation is handled by ixdtf during parsing
        let result = validate_date_record_impl(invalid_day_record);
        assert!(result.is_ok());

        // Test year validation error message
        let invalid_year_record = DateRecord {
            year: 275761, // Beyond valid range
            month: 1,
            day: 1,
        };
        let result = validate_date_record_impl(invalid_year_record);
        assert!(result.is_err());
        let error = result.unwrap_err().into_temporal_error();
        let error_msg = format!("{error}");
        assert!(error_msg.contains("275761"));
        assert!(error_msg.contains("outside valid range"));
    }
}
