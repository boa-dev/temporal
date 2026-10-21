//! This module implements Temporal Date/Time parsing functionality.

use crate::{
    error::ErrorMessage,
    iso::{IsoDate, IsoTime},
    options::{DisplayCalendar, DisplayOffset, DisplayTimeZone},
    parsers::intermediates::{ParsedDate, ParsedDateTime, ParsedZonedDateTime},
    provider::TimeZoneProvider,
    Calendar, Sign, TemporalError, TemporalResult, TemporalUnwrap, TimeZone,
};
use alloc::{borrow::Cow, vec::Vec};
use icu_calendar::AnyCalendarKind;
pub use ixdtf::encoding::{EncodingType, Utf16, Utf8};
use ixdtf::{
    parsers::{IsoDurationParser, IxdtfParser, TimeZoneParser},
    records::{
        Annotation, DateRecord, DurationParseRecord, FullPrecisionOffset, TimeRecord,
        TimeZoneAnnotation, TimeZoneRecord, UtcOffsetRecord, UtcOffsetRecordOrZ,
    },
};
use writeable::{impl_display_with_writeable, LengthHint, Writeable};

pub mod intermediates;
mod time_zone;

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
    for (i, out) in output.iter_mut().enumerate().rev() {
        let v = (value % 10) as u8;
        value /= 10;
        if precision == 0 && v != 0 {
            // i is 0-indexed, but we want a 1-indexed precision
            precision = i + 1;
        }
        *out = v;
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
                        || ns != 0
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

mod sealed {
    /// Prevents outside implementations of [`super::TemporalEncoding`].
    pub trait Sealed {}

    impl Sealed for super::Utf8 {}
    impl Sealed for super::Utf16 {}
}

/// Extends `ixdtf`'s [`EncodingType`] with ASCII normalization for calendar/time-zone
/// identifiers. Sealed: implemented only for [`Utf8`] and [`Utf16`].
///
/// Normalization happens at the edge that needs UTF-8 bytes, not during parsing —
/// parse records keep identifiers in the source encoding.
pub trait TemporalEncoding: EncodingType + sealed::Sealed {
    /// Normalizes an ASCII identifier slice (calendar id, IANA name) to UTF-8.
    /// Zero-copy for [`Utf8`].
    #[doc(hidden)]
    fn ascii_identifier(source: &[Self::CodeUnit]) -> TemporalResult<Cow<'_, [u8]>>;

    /// Case-insensitive ASCII comparison without allocating; used for calendar-id
    /// literal checks (e.g. `"iso8601"`) during parsing.
    #[doc(hidden)]
    fn eq_ascii_ignore_case(source: &[Self::CodeUnit], ascii: &[u8]) -> bool {
        source.len() == ascii.len()
            && (0..source.len()).all(|i| {
                let Some(expected) = ascii.get(i) else {
                    return false;
                };
                Self::get_ascii(source, i)
                    .ok()
                    .flatten()
                    .is_some_and(|byte| byte.eq_ignore_ascii_case(expected))
            })
    }
}

impl TemporalEncoding for Utf8 {
    fn ascii_identifier(source: &[u8]) -> TemporalResult<Cow<'_, [u8]>> {
        Ok(Cow::Borrowed(source))
    }
}

impl TemporalEncoding for Utf16 {
    fn ascii_identifier(source: &[u16]) -> TemporalResult<Cow<'_, [u8]>> {
        let mut buf = Vec::with_capacity(source.len());
        for i in 0..source.len() {
            let byte = Utf16::get_ascii(source, i)?.ok_or_else(TemporalError::assert)?;
            buf.push(byte);
        }
        Ok(Cow::Owned(buf))
    }
}

/// A parsed-but-not-yet-normalized IXDTF record, generic over source encoding.
///
/// Calendar/time-zone identifiers stay in the source encoding; normalize via
/// [`TemporalEncoding::ascii_identifier`] only where actually needed.
///
/// `date`/`tz` are `Option` since this is the shared shape for every
/// [`ParseVariant`] (bare `Time` has no date). Use [`DateParseRecord`] or
/// [`ZonedDateParseRecord`] where the guarantee is known instead of re-deriving it.
#[derive(Debug, PartialEq)]
pub(crate) struct TemporalParseRecord<'a, T: EncodingType> {
    pub(crate) date: Option<DateRecord>,
    pub(crate) time: Option<TimeRecord>,
    pub(crate) offset: Option<UtcOffsetRecordOrZ>,
    pub(crate) tz: Option<TimeZoneAnnotation<'a, T>>,
    pub(crate) calendar: Option<&'a [T::CodeUnit]>,
}

/// A parsed IXDTF record for a grammar production that always carries a date
/// (`TemporalDateTimeString`, `TemporalYearMonthString`, `TemporalMonthDayString`).
///
/// Unlike [`TemporalParseRecord`], `date` is guaranteed present.
#[derive(Debug, PartialEq)]
pub(crate) struct DateParseRecord<'a, T: EncodingType> {
    pub(crate) date: DateRecord,
    pub(crate) time: Option<TimeRecord>,
    pub(crate) offset: Option<UtcOffsetRecordOrZ>,
    pub(crate) tz: Option<TimeZoneAnnotation<'a, T>>,
    pub(crate) calendar: Option<&'a [T::CodeUnit]>,
}

/// A parsed IXDTF record for `TemporalZonedDateTimeString`, which additionally
/// guarantees a time zone annotation is present (`parse_zoned_date_time_record` rejects
/// strings without one before ever constructing this type).
#[derive(Debug, PartialEq)]
pub(crate) struct ZonedDateParseRecord<'a, T: EncodingType> {
    pub(crate) date: DateRecord,
    pub(crate) time: Option<TimeRecord>,
    pub(crate) offset: Option<UtcOffsetRecordOrZ>,
    pub(crate) tz: TimeZoneAnnotation<'a, T>,
    pub(crate) calendar: Option<&'a [T::CodeUnit]>,
}

#[derive(PartialEq)]
enum ParseVariant {
    YearMonth,
    MonthDay,
    DateTime,
    Time,
}

#[inline]
fn parse_ixdtf<T: TemporalEncoding>(
    source: &[T::CodeUnit],
    variant: ParseVariant,
) -> TemporalResult<TemporalParseRecord<'_, T>> {
    fn cast_handler<'a, T: EncodingType>(
        _: &mut IxdtfParser<'a, T>,
        handler: impl FnMut(Annotation<'a, T>) -> Option<Annotation<'a, T>>,
    ) -> impl FnMut(Annotation<'a, T>) -> Option<Annotation<'a, T>> {
        handler
    }

    let mut first_calendar: Option<Annotation<T>> = None;
    let mut critical_duplicate_calendar = false;
    let mut parser = IxdtfParser::<T>::new(source);

    let handler = cast_handler(&mut parser, |annotation: Annotation<T>| {
        if T::check_calendar_key(annotation.key) {
            match &first_calendar {
                Some(cal) => {
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

    let record = match variant {
        ParseVariant::YearMonth => parser.parse_year_month_with_annotation_handler(handler),
        ParseVariant::MonthDay => parser.parse_month_day_with_annotation_handler(handler),
        ParseVariant::DateTime => parser.parse_with_annotation_handler(handler),
        ParseVariant::Time => parser.parse_time_with_annotation_handler(handler),
    }?;

    // The ixdtf crate allows arbitrary precision for offsets, but Temporal only allows 9
    if let Some(UtcOffsetRecordOrZ::Offset(UtcOffsetRecord::FullPrecisionOffset(
        FullPrecisionOffset {
            fraction: Some(fraction),
            ..
        },
    ))) = record.offset
    {
        if fraction.to_nanoseconds().is_none() {
            return Err(
                TemporalError::range().with_enum(ErrorMessage::FractionalTimeMoreThanNineDigits)
            );
        }
    }

    // Keep calendar bytes in source encoding; normalize only where consumed.
    let calendar = first_calendar.map(|v| v.value);

    // Note: this method only handles the specific AnnotatedFoo nonterminals;
    // so if we are parsing MonthDay/YearMonth we will never have a DateDay/DateYear parse node.
    //
    // 3. If goal is TemporalYearMonthString, and parseResult does not contain a DateDay Parse Node, then
    //  a. If calendar is not empty, and the ASCII-lowercase of calendar is not "iso8601", throw a RangeError exception.
    // 4. If goal is TemporalMonthDayString and parseResult does not contain a DateYear Parse Node, then
    //  a. If calendar is not empty, and the ASCII-lowercase of calendar is not "iso8601", throw a RangeError exception.
    //  b. Set yearAbsent to true.
    if variant == ParseVariant::MonthDay || variant == ParseVariant::YearMonth {
        if let Some(cal) = calendar {
            if !T::eq_ascii_ignore_case(cal, b"iso8601") {
                return Err(TemporalError::range()
                    .with_message("YearMonth/MonthDay formats only allowed for ISO calendar."));
            }
        }
    }

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

    // Temporal requires parsed dates to be checked for validity at parse time
    // https://tc39.es/proposal-temporal/#sec-temporal-iso8601grammar-static-semantics-early-errors
    // https://tc39.es/proposal-temporal/#sec-temporal-iso8601grammar-static-semantics-isvaliddate
    // https://tc39.es/proposal-temporal/#sec-temporal-iso8601grammar-static-semantics-isvalidmonthday
    if let Some(date) = record.date {
        // ixdtf currently returns always-valid reference years/days
        // in these cases (year = 0, day = 1), but we should set our own
        // for robustness.
        let year = if variant == ParseVariant::MonthDay {
            1972
        } else {
            date.year
        };
        let day = if variant == ParseVariant::YearMonth {
            1
        } else {
            date.day
        };
        if !crate::iso::is_valid_date(year, date.month, day) {
            return Err(TemporalError::range()
                .with_message("DateTime strings must contain a valid ISO date."));
        }
    }

    Ok(TemporalParseRecord {
        date: record.date,
        time: record.time,
        offset: record.offset,
        tz: record.tz,
        calendar,
    })
}

/// A utility function for parsing a `DateTime` string
#[inline]
pub(crate) fn parse_date_time_record<T: TemporalEncoding>(
    source: &[T::CodeUnit],
) -> TemporalResult<DateParseRecord<'_, T>> {
    let record = parse_ixdtf::<T>(source, ParseVariant::DateTime)?;

    if record.offset == Some(UtcOffsetRecordOrZ::Z) {
        return Err(TemporalError::range()
            .with_message("UTC designator is not valid for DateTime parsing."));
    }

    Ok(DateParseRecord {
        date: record.date.temporal_unwrap()?,
        time: record.time,
        offset: record.offset,
        tz: record.tz,
        calendar: record.calendar,
    })
}

#[inline]
pub(crate) fn parse_zoned_date_time_record<T: TemporalEncoding>(
    source: &[T::CodeUnit],
) -> TemporalResult<ZonedDateParseRecord<'_, T>> {
    let record = parse_ixdtf::<T>(source, ParseVariant::DateTime)?;

    // TODO: Support rejecting subminute precision in time zone annootations
    let Some(tz) = record.tz else {
        return Err(TemporalError::range()
            .with_message("Time zone annotation is required for parsing a zoned date time."));
    };

    Ok(ZonedDateParseRecord {
        date: record.date.temporal_unwrap()?,
        time: record.time,
        offset: record.offset,
        tz,
        calendar: record.calendar,
    })
}

pub(crate) struct IxdtfParseInstantRecord {
    pub(crate) date: DateRecord,
    pub(crate) time: TimeRecord,
    pub(crate) offset: UtcOffsetRecordOrZ,
}

/// A utility function for parsing an `Instant` string
#[inline]
pub(crate) fn parse_instant_record<T: TemporalEncoding>(
    source: &[T::CodeUnit],
) -> TemporalResult<IxdtfParseInstantRecord> {
    let record = parse_ixdtf::<T>(source, ParseVariant::DateTime)?;

    let TemporalParseRecord {
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

// Ensure that the record does not have a Z offset element.
//
// This handles the [~Zoned] in TemporalFooString productions
fn check_offset_no_z<T: TemporalEncoding>(
    record: TemporalParseRecord<'_, T>,
) -> TemporalResult<TemporalParseRecord<'_, T>> {
    if record.offset == Some(UtcOffsetRecordOrZ::Z) {
        return Err(TemporalError::range()
            .with_message("UTC designator is not valid for plain date/time parsing."));
    }

    Ok(record)
}

/// A utility function for parsing a `YearMonth` string
#[inline]
pub(crate) fn parse_year_month_record<T: TemporalEncoding>(
    source: &[T::CodeUnit],
) -> TemporalResult<DateParseRecord<'_, T>> {
    let ym_record = parse_ixdtf::<T>(source, ParseVariant::YearMonth).and_then(check_offset_no_z);

    let record = match ym_record {
        Ok(record) => record,
        // Format and return the error from parsing YearMonth.
        Err(e) => return parse_date_time_record::<T>(source).map_err(|_| e),
    };

    Ok(DateParseRecord {
        date: record.date.temporal_unwrap()?,
        time: record.time,
        offset: record.offset,
        tz: record.tz,
        calendar: record.calendar,
    })
}

/// A utilty function for parsing a `MonthDay` String.
pub(crate) fn parse_month_day_record<T: TemporalEncoding>(
    source: &[T::CodeUnit],
) -> TemporalResult<DateParseRecord<'_, T>> {
    let md_record = parse_ixdtf::<T>(source, ParseVariant::MonthDay).and_then(check_offset_no_z);

    let record = match md_record {
        Ok(record) => record,
        // Format and return the error from parsing MonthDay.
        Err(e) => return parse_date_time_record::<T>(source).map_err(|_| e),
    };

    Ok(DateParseRecord {
        date: record.date.temporal_unwrap()?,
        time: record.time,
        offset: record.offset,
        tz: record.tz,
        calendar: record.calendar,
    })
}

// Ensures that a TemporalParseRecord was parsed with [~Zoned][+TimeRequired]
fn check_time_record<T: TemporalEncoding>(
    record: TemporalParseRecord<'_, T>,
) -> TemporalResult<TimeRecord> {
    // Handle [~Zoned]
    let record = check_offset_no_z(record)?;
    // Handle [+TimeRequired]
    let Some(time) = record.time else {
        return Err(TemporalError::range()
            .with_message("PlainTime can only be parsed from strings with a time component."));
    };
    Ok(time)
}

#[inline]
pub(crate) fn parse_time_record<T: TemporalEncoding>(
    source: &[T::CodeUnit],
) -> TemporalResult<TimeRecord> {
    let time_record = parse_ixdtf::<T>(source, ParseVariant::Time);

    let Err(e) = time_record else {
        return time_record.and_then(check_time_record);
    };

    match parse_date_time_record::<T>(source) {
        // `parse_date_time_record` already rejects a `Z` offset, so only the
        // [+TimeRequired] check remains here.
        Ok(dt) => dt.time.ok_or_else(|| {
            TemporalError::range()
                .with_message("PlainTime can only be parsed from strings with a time component.")
        }),
        // Format and return the error from parsing Time.
        _ => Err(e),
    }
}

/// The outcome of looking for a calendar annotation across the Temporal grammar
/// productions that may carry one.
#[derive(Debug, PartialEq)]
pub enum CalendarAnnotation<'a, T: TemporalEncoding> {
    /// No calendar-bearing production matched the source.
    NotFound,
    /// A production matched, but carried no `u-ca` annotation (implicitly ISO).
    Implicit,
    /// A production matched with an explicit calendar identifier.
    Explicit(&'a [T::CodeUnit]),
}

/// Unstable: used internally by `temporal_capi`.
#[inline]
pub fn parse_allowed_calendar_formats<T: TemporalEncoding>(
    s: &[T::CodeUnit],
) -> CalendarAnnotation<'_, T> {
    for variant in [
        ParseVariant::DateTime,
        ParseVariant::Time,
        ParseVariant::YearMonth,
        ParseVariant::MonthDay,
    ] {
        if let Ok(r) = parse_ixdtf::<T>(s, variant) {
            return match r.calendar {
                Some(id) => CalendarAnnotation::Explicit(id),
                None => CalendarAnnotation::Implicit,
            };
        }
    }
    CalendarAnnotation::NotFound
}

/// Converts a parsed time component to an [`IsoTime`], defaulting to midnight when
/// absent. Errors if the component is present but invalid.
fn iso_time_or_midnight(time: Option<TimeRecord>) -> TemporalResult<IsoTime> {
    Ok(time
        .map(IsoTime::from_time_record)
        .transpose()?
        .unwrap_or_default())
}

fn calendar_kind<T: TemporalEncoding>(
    calendar: Option<&[T::CodeUnit]>,
) -> TemporalResult<AnyCalendarKind> {
    let Some(calendar) = calendar else {
        return Ok(AnyCalendarKind::Iso);
    };
    let ascii = T::ascii_identifier(calendar)?;
    // HijriSimulatedMecca parses fine here; Calendar::new turns it into ISO.
    Calendar::try_kind_from_utf8(&ascii)
}

/// Parser for Temporal strings, generic over source encoding.
///
/// Crate-internal entry point wrapping `ixdtf`'s parsers with Temporal's validation
/// rules. The date/time methods return the spec-mandated intermediates from
/// [`intermediates`]; the remaining methods return `ixdtf` records directly.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TemporalParser<'a, T: TemporalEncoding> {
    source: &'a [T::CodeUnit],
}

impl<'a> TemporalParser<'a, Utf8> {
    #[must_use]
    pub(crate) fn from_str(source: &'a str) -> Self {
        Self::from_encoded(source.as_bytes())
    }

    #[must_use]
    pub(crate) fn from_utf8(source: &'a [u8]) -> Self {
        Self::from_encoded(source)
    }
}

impl<'a> TemporalParser<'a, Utf16> {
    #[must_use]
    pub(crate) fn from_utf16(source: &'a [u16]) -> Self {
        Self::from_encoded(source)
    }
}

impl<'a, T: TemporalEncoding> TemporalParser<'a, T> {
    #[must_use]
    pub(crate) fn from_encoded(source: &'a [T::CodeUnit]) -> Self {
        Self { source }
    }

    /// Parses a `TemporalDateTimeString` into a [`ParsedDate`].
    #[must_use = "parsing returns the parsed intermediate"]
    pub(crate) fn parse_date(&self) -> TemporalResult<ParsedDate> {
        let record = parse_date_time_record::<T>(self.source)?;
        let calendar = calendar_kind::<T>(record.calendar)?;
        // Time is discarded, but must still be validated here (observable).
        iso_time_or_midnight(record.time)?;

        Ok(ParsedDate {
            record: record.date,
            calendar,
        })
    }

    /// Parses a `TemporalYearMonthString` into a [`ParsedDate`].
    #[must_use = "parsing returns the parsed intermediate"]
    pub(crate) fn parse_year_month(&self) -> TemporalResult<ParsedDate> {
        let record = parse_year_month_record::<T>(self.source)?;
        let calendar = calendar_kind::<T>(record.calendar)?;
        iso_time_or_midnight(record.time)?;

        Ok(ParsedDate {
            record: record.date,
            calendar,
        })
    }

    /// Parses a `TemporalMonthDayString` into a [`ParsedDate`].
    #[must_use = "parsing returns the parsed intermediate"]
    pub(crate) fn parse_month_day(&self) -> TemporalResult<ParsedDate> {
        let record = parse_month_day_record::<T>(self.source)?;
        let calendar = calendar_kind::<T>(record.calendar)?;
        iso_time_or_midnight(record.time)?;

        Ok(ParsedDate {
            record: record.date,
            calendar,
        })
    }

    /// Parses a `TemporalDateTimeString` into a [`ParsedDateTime`].
    #[must_use = "parsing returns the parsed intermediate"]
    pub(crate) fn parse_date_time(&self) -> TemporalResult<ParsedDateTime> {
        let record = parse_date_time_record::<T>(self.source)?;
        let calendar = calendar_kind::<T>(record.calendar)?;
        let time = iso_time_or_midnight(record.time)?;

        Ok(ParsedDateTime {
            date: ParsedDate {
                record: record.date,
                calendar,
            },
            time,
        })
    }

    /// Parses a `TemporalZonedDateTimeString` into a [`ParsedZonedDateTime`].
    #[must_use = "parsing returns the parsed intermediate"]
    pub(crate) fn parse_zoned_date_time(
        &self,
        provider: &(impl TimeZoneProvider + ?Sized),
    ) -> TemporalResult<ParsedZonedDateTime> {
        // Steps from the parse bits of ToZonedDateTime

        // 3. Let matchBehaviour be match-minutes.
        let mut match_minutes = true;

        // b. Let result be ? ParseISODateTime(item, « TemporalDateTimeString[+Zoned] »).
        let record = parse_zoned_date_time_record::<T>(self.source)?;

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
                (Some(crate::UtcOffset::from_ixdtf_record(offset)?), false)
            }
            None => (None, false),
        };

        // h-j. Resolve the calendar, defaulting to ISO.
        let calendar = calendar_kind::<T>(record.calendar)?;

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

    /// Parses a `TemporalInstantString`.
    #[must_use = "parsing returns the parsed record"]
    pub(crate) fn parse_instant(&self) -> TemporalResult<IxdtfParseInstantRecord> {
        parse_instant_record::<T>(self.source)
    }

    /// Parses a `TemporalTimeString`.
    #[must_use = "parsing returns the parsed record"]
    pub(crate) fn parse_time(&self) -> TemporalResult<TimeRecord> {
        parse_time_record::<T>(self.source)
    }

    /// Parses a `TemporalDurationString`.
    #[must_use = "parsing returns the parsed record"]
    pub(crate) fn parse_duration(&self) -> TemporalResult<DurationParseRecord> {
        Ok(IsoDurationParser::<T>::new(self.source).parse()?)
    }

    /// Parses a `UTCOffset`, per `ParseDateTimeUTCOffset`.
    #[must_use = "parsing returns the parsed record"]
    pub(crate) fn parse_utc_offset(&self) -> TemporalResult<UtcOffsetRecord> {
        Ok(TimeZoneParser::<T>::new(self.source).parse_offset()?)
    }

    /// Parses a `TimeZoneIdentifier` (IANA name or offset).
    #[must_use = "parsing returns the parsed record"]
    pub(crate) fn parse_time_zone_identifier(&self) -> TemporalResult<TimeZoneRecord<'a, T>> {
        TimeZoneParser::<T>::new(self.source)
            .parse_identifier()
            .or(Err(
                TemporalError::range().with_message("Invalid TimeZone Identifier")
            ))
    }

    /// Resolves a `TimeZone` from any Temporal production that may carry one.
    ///
    /// This is the fallback half of `ParseTemporalTimeZoneString`, used after a
    /// bare identifier parse fails.
    pub(crate) fn parse_time_zone(
        &self,
        provider: &(impl TimeZoneProvider + ?Sized),
    ) -> Option<TimeZone> {
        time_zone::parse_allowed_timezone_formats::<T>(self.source, provider)
    }

    /// Finds a calendar annotation across the productions that may carry one.
    pub(crate) fn parse_calendar(&self) -> CalendarAnnotation<'a, T> {
        parse_allowed_calendar_formats::<T>(self.source)
    }
}

#[cfg(test)]
mod tests {
    use super::{FormattableDate, FormattableDuration, FormattableOffset};
    use crate::parsers::{FormattableTime, FormattableTimeDuration, Precision};
    use alloc::format;
    use writeable::assert_writeable_eq;

    #[test]
    fn duration() {
        let duration = FormattableDuration {
            sign: crate::Sign::Positive,
            date: Some(super::FormattableDateDuration {
                years: 1,
                months: 0,
                weeks: 0,
                days: 0,
            }),
            time: Some(FormattableTimeDuration::Seconds(0, 0, 0, Some(1))),
            precision: Precision::Auto,
        };
        assert_writeable_eq!(duration, "P1YT0.000000001S");
    }

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
}

#[cfg(test)]
mod utf16_tests {
    //! Proves the generic parse functions are genuinely encoding-generic: UTF-8
    //! and UTF-16 encodings of the same source string parse to identical records.
    use super::{parse_date_time_record, TemporalEncoding, Utf16, Utf8};
    use alloc::vec::Vec;

    fn utf16(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    #[test]
    fn date_time_parity() {
        let s = "2024-07-19T15:43:12.5[America/New_York][u-ca=iso8601]";
        let utf8 = parse_date_time_record::<Utf8>(s.as_bytes()).unwrap();
        let wide = utf16(s);
        let utf16_record = parse_date_time_record::<Utf16>(&wide).unwrap();

        assert_eq!(utf8.date, utf16_record.date);
        assert_eq!(utf8.time, utf16_record.time);
        assert_eq!(utf8.offset, utf16_record.offset);
        assert_eq!(utf8.calendar, Some(b"iso8601".as_slice()));
        let utf16_calendar = Utf16::ascii_identifier(utf16_record.calendar.unwrap()).unwrap();
        assert_eq!(&*utf16_calendar, b"iso8601".as_slice());
        assert!(utf8.tz.is_some());
        assert!(utf16_record.tz.is_some());
    }

    #[test]
    fn multi_segment_calendar_id_parity() {
        // `islamic-umalqura` is two dash-joined BCP-47 subtags, over 8 chars total.
        let s = "2024-07-19[u-ca=islamic-umalqura]";
        let utf8 = parse_date_time_record::<Utf8>(s.as_bytes()).unwrap();
        let wide = utf16(s);
        let utf16_record = parse_date_time_record::<Utf16>(&wide).unwrap();

        assert_eq!(utf8.calendar, Some(b"islamic-umalqura".as_slice()));
        let cal = utf16_record.calendar.unwrap();
        let utf16_calendar = Utf16::ascii_identifier(cal).unwrap();
        assert_eq!(&*utf16_calendar, b"islamic-umalqura".as_slice());
    }

    #[test]
    fn year_month_month_day_time_calendar_parity() {
        use super::CalendarAnnotation;

        for (s, expected) in [
            ("2024-07", None),
            ("--07-19", None),
            ("T15:43:12", None),
            ("2024-07-19[u-ca=hebrew]", Some(b"hebrew".as_slice())),
        ] {
            let utf8 = match super::parse_allowed_calendar_formats::<Utf8>(s.as_bytes()) {
                CalendarAnnotation::Implicit => None,
                CalendarAnnotation::Explicit(id) => Some(id.to_vec()),
                CalendarAnnotation::NotFound => panic!("{s} should match a dated production"),
            };
            let wide = utf16(s);
            let utf16_annotation = match super::parse_allowed_calendar_formats::<Utf16>(&wide) {
                CalendarAnnotation::Implicit => None,
                CalendarAnnotation::Explicit(id) => {
                    Some(Utf16::ascii_identifier(id).unwrap().into_owned())
                }
                CalendarAnnotation::NotFound => panic!("{s} should match a dated production"),
            };

            let expected = expected.map(<[u8]>::to_vec);
            assert_eq!(utf8, expected);
            assert_eq!(utf16_annotation, expected);
        }
    }

    #[test]
    fn invalid_input_errors_identically() {
        let s = "2025-02-29"; // not a leap year
        let wide = utf16(s);
        assert!(parse_date_time_record::<Utf8>(s.as_bytes()).is_err());
        assert!(parse_date_time_record::<Utf16>(&wide).is_err());
    }
}
