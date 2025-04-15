//! Zoneinfo types

use core::fmt::Write;

use alloc::{borrow::ToOwned, string::String, vec::Vec};

use crate::{
    parser::{next_split, ContextParse, LineParseContext, TryFromStr, ZoneInfoParseError},
    rule::epoch_days_for_rule_date,
    utils,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    pub at_time: i64,
    pub offset: i64,
    pub dst: bool,
    pub savings: Time,
    pub letter: Option<String>,
    pub time_type: QualifiedTimeKind,
    pub format: String,
}

impl Ord for Transition {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.partial_cmp(other).expect("always some")
    }
}

impl PartialOrd for Transition {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.at_time.cmp(&other.at_time))
    }
}

// ==== Zone Table specific types ====

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneEntry {
    // Standard offset in seconds
    pub std_offset: Time,
    // Rule  in use
    pub rule: RuleIdentifier,
    // String format
    pub format: AbbreviationFormat,
    // Date until
    pub date: Option<UntilDateTime>,
}

impl ZoneEntry {
    pub(crate) fn is_named_rule(&self) -> bool {
        matches!(self.rule, RuleIdentifier::Rule(_))
    }
}

impl TryFromStr<LineParseContext> for ZoneEntry {
    type Error = ZoneInfoParseError;
    fn try_from_str(s: &str, ctx: &mut LineParseContext) -> Result<Self, Self::Error> {
        ctx.enter("ZoneEntry");
        let mut splits = s.split_whitespace();
        let std_offset = splits
            .next()
            .ok_or(ZoneInfoParseError::UnexpectedEndOfLine(
                ctx.line_number,
                ctx.span(),
            ))?
            .context_parse::<Time>(ctx)?;
        let rule = next_split(&mut splits, ctx)?.context_parse::<RuleIdentifier>(ctx)?;
        let format = splits
            .next()
            .ok_or(ZoneInfoParseError::UnexpectedEndOfLine(
                ctx.line_number,
                ctx.span(),
            ))?
            .context_parse::<AbbreviationFormat>(ctx)?;
        let datetime = splits.collect::<Vec<&str>>();
        let date = if datetime.is_empty() {
            None
        } else {
            let dt_str = datetime.join(" \t");
            Some(dt_str.context_parse::<UntilDateTime>(ctx)?)
        };

        ctx.exit();
        Ok(ZoneEntry {
            std_offset,
            rule,
            format,
            date,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuleIdentifier {
    None,
    Numeric(Time),
    Rule(String),
}

impl TryFromStr<LineParseContext> for RuleIdentifier {
    type Error = ZoneInfoParseError;
    fn try_from_str(s: &str, ctx: &mut LineParseContext) -> Result<Self, Self::Error> {
        ctx.enter("RuleIdentifier");
        if s == "-" {
            ctx.exit();
            return Ok(Self::None);
        }
        if s.contains(":") {
            ctx.exit();
            return Time::try_from_str(s, ctx).map(Self::Numeric);
        }
        ctx.exit();
        Ok(Self::Rule(s.to_owned()))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AbbreviationFormat {
    String(String),
    Numeric,
    Pair(String, String),
    Formattable(FormattableAbbr),
}

impl AbbreviationFormat {
    pub(crate) fn format_with_transition(&self, transition: &Transition) -> String {
        self.format(
            transition.offset,
            transition.letter.as_deref(),
            transition.dst,
        )
    }

    pub fn format(&self, offset: i64, letter: Option<&str>, is_dst: bool) -> String {
        match self {
            Self::String(s) => s.clone(),
            Self::Formattable(s) => s.to_formatted_string(letter.unwrap_or("")),
            Self::Pair(std, dst) => {
                if is_dst {
                    dst.clone()
                } else {
                    std.clone()
                }
            }
            Self::Numeric => offset_to_str(offset),
        }
    }
}

fn offset_to_str(n: i64) -> String {
    let mut output = String::new();
    if n.is_positive() {
        write!(&mut output, "+").expect("failed to write");
    } else {
        write!(&mut output, "-").expect("failed to write");
    }
    let hour = n.abs().div_euclid(3600);
    write!(&mut output, "{hour:02}").expect("failed to write");
    let minute = n.abs().rem_euclid(3600).div_euclid(60);
    if minute > 0 {
        write!(&mut output, "{minute:02}").expect("failed to write");
    }
    output
}

impl TryFromStr<LineParseContext> for AbbreviationFormat {
    type Error = ZoneInfoParseError;
    fn try_from_str(s: &str, ctx: &mut LineParseContext) -> Result<Self, Self::Error> {
        ctx.enter("Abbr. Format");
        let value = if s.contains("%s") {
            Ok(Self::Formattable(FormattableAbbr(s.to_owned())))
        } else if s.contains("%z") {
            Ok(Self::Numeric)
        } else if s.contains("/") {
            let (std, dst) = s.split_once('/').ok_or(ZoneInfoParseError::UnknownValue(
                ctx.line_number,
                s.to_owned(),
            ))?;
            Ok(Self::Pair(std.to_owned(), dst.to_owned()))
        } else {
            Ok(AbbreviationFormat::String(s.to_owned()))
        };
        ctx.exit();
        value
    }
}

impl Default for AbbreviationFormat {
    fn default() -> Self {
        Self::String("LMT".to_owned())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormattableAbbr(String);

impl FormattableAbbr {
    pub fn to_formatted_string(&self, letter: &str) -> String {
        self.0.replace("%s", letter)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UntilDateTime {
    pub date: Date,
    pub time: QualifiedTime,
}

impl UntilDateTime {
    pub fn as_date_secs(self) -> i64 {
        self.date.as_secs()
    }

    pub fn as_precise_ut_time(self, std_offset: &Time, save: &Time) -> i64 {
        self.as_date_secs() + self.time.to_universal_seconds(std_offset, save)
    }
}

impl TryFromStr<LineParseContext> for UntilDateTime {
    type Error = ZoneInfoParseError;
    fn try_from_str(s: &str, ctx: &mut LineParseContext) -> Result<Self, Self::Error> {
        ctx.enter("UntilDateTime");
        let mut splits = s.split_whitespace();
        let year = next_split(&mut splits, ctx)?.context_parse::<i32>(ctx)?;
        let date_or_end = splits.next();
        let date = if let Some(month) = date_or_end {
            let month = month.context_parse::<Month>(ctx)?;
            let day = next_split(&mut splits, ctx)
                .ok()
                .map(|s| s.context_parse::<DayOfMonth>(ctx))
                .transpose()?
                .unwrap_or(DayOfMonth::Day(1));
            Date { year, month, day }
        } else {
            ctx.exit();
            return Ok(UntilDateTime {
                date: Date {
                    year,
                    month: Month::Jan,
                    day: DayOfMonth::Day(1),
                },
                time: QualifiedTime::Local(Time::default()),
            });
        };

        let time = next_split(&mut splits, ctx)
            .ok()
            .map(|t| t.context_parse::<QualifiedTime>(ctx))
            .transpose()?
            .unwrap_or(QualifiedTime::Local(Time::default()));

        ctx.exit();
        Ok(Self { date, time })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Date {
    pub year: i32,
    pub month: Month,
    pub day: DayOfMonth,
}

impl Date {
    pub fn as_secs(&self) -> i64 {
        let epoch_days = epoch_days_for_rule_date(self.year, self.month, self.day);
        utils::epoch_seconds_for_epoch_days(epoch_days)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Time {
    pub sign: Sign,
    pub hour: i8,
    pub minute: i8,
    pub second: i8,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(i8)]
pub enum Sign {
    #[default]
    Positive = 1,
    Negative = -1,
}

impl Time {
    pub const fn as_secs(&self) -> i64 {
        (self.hour as i64 * 3600 + self.minute as i64 * 60 + self.second as i64) * self.sign as i64
    }
}

impl TryFromStr<LineParseContext> for Time {
    type Error = ZoneInfoParseError;
    fn try_from_str(s: &str, ctx: &mut LineParseContext) -> Result<Self, Self::Error> {
        ctx.enter("Time");
        let (s, sign) = if let Some(stripped) = s.strip_prefix('-') {
            (stripped, Sign::Negative)
        } else {
            (s, Sign::Positive)
        };
        if !s.contains(':') {
            let hour = s.context_parse::<i8>(ctx)?;
            ctx.exit();
            return Ok(Time {
                sign,
                hour,
                minute: 0,
                second: 0,
            });
        }
        let (hour, sub_hour) = s.split_once(':').ok_or(ZoneInfoParseError::UnknownValue(
            ctx.line_number,
            s.to_owned(),
        ))?;
        let hour = hour.context_parse::<i8>(ctx)?;
        if !sub_hour.contains(':') {
            let minute = sub_hour.context_parse::<i8>(ctx)?;
            ctx.exit();
            return Ok(Self {
                sign,
                hour,
                minute,
                second: 0,
            });
        }
        let (minute, second) = sub_hour
            .split_once(':')
            .ok_or(ZoneInfoParseError::UnknownValue(
                ctx.line_number,
                s.to_owned(),
            ))?;
        let minute = minute.context_parse::<i8>(ctx)?;
        let second = second.context_parse::<i8>(ctx)?;
        ctx.exit();
        Ok(Self {
            sign,
            hour,
            minute,
            second,
        })
    }
}

// ==== Rule types ====

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToYear {
    Max,
    Year(u16),
}

impl ToYear {
    pub(crate) fn parse_optional_to_year(
        s: &str,
        ctx: &mut LineParseContext,
    ) -> Result<Option<ToYear>, ZoneInfoParseError> {
        if s == "only" {
            Ok(None)
        } else {
            s.context_parse::<ToYear>(ctx).map(Some)
        }
    }

    pub(crate) fn to_i32(self) -> i32 {
        match self {
            Self::Max => i32::MAX,
            Self::Year(y) => y as i32,
        }
    }
}

impl TryFromStr<LineParseContext> for ToYear {
    type Error = ZoneInfoParseError;

    fn try_from_str(s: &str, ctx: &mut LineParseContext) -> Result<Self, Self::Error> {
        if s == "max" {
            return Ok(ToYear::Max);
        }
        s.context_parse::<u16>(ctx).map(ToYear::Year)
    }
}

// The default implementation
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Month {
    Jan = 1,
    Feb,
    Mar,
    Apr,
    May,
    Jun,
    Jul,
    Aug,
    Sep,
    Oct,
    Nov,
    Dec,
}

impl Month {
    pub(crate) fn month_start_to_day_of_year(self, year: i32) -> i32 {
        utils::month_to_day(self as u8, utils::in_leap_year(year))
    }

    pub(crate) fn month_end_to_day_of_year(self, year: i32) -> i32 {
        utils::month_to_day(self as u8 + 1, utils::in_leap_year(year)) - 1
    }
}

impl TryFromStr<LineParseContext> for Month {
    type Error = ZoneInfoParseError;
    fn try_from_str(s: &str, ctx: &mut LineParseContext) -> Result<Self, Self::Error> {
        ctx.enter("Month");
        let result = match s {
            "Jan" => Ok(Self::Jan),
            "Feb" => Ok(Self::Feb),
            "Mar" => Ok(Self::Mar),
            "Apr" => Ok(Self::Apr),
            "May" => Ok(Self::May),
            "Jun" => Ok(Self::Jun),
            "Jul" => Ok(Self::Jul),
            "Aug" => Ok(Self::Aug),
            "Sep" => Ok(Self::Sep),
            "Oct" => Ok(Self::Oct),
            "Nov" => Ok(Self::Nov),
            "Dec" => Ok(Self::Dec),
            _ => Err(ZoneInfoParseError::UnknownValue(
                ctx.line_number,
                s.to_owned(),
            )),
        };
        ctx.exit();
        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DayOfMonth {
    // Again, hacky default. Not a fan
    Last(WeekDay),
    WeekDayGEThanMonthDay(WeekDay, u8),
    // Potentially, depracated
    WeekDayLEThanMonthDay(WeekDay, u8),
    Day(u8),
}

impl TryFromStr<LineParseContext> for DayOfMonth {
    type Error = ZoneInfoParseError;
    fn try_from_str(s: &str, ctx: &mut LineParseContext) -> Result<Self, Self::Error> {
        ctx.enter("DayOfMonth");
        let result = if let Some(weekday) = s.strip_prefix("last") {
            Ok(DayOfMonth::Last(weekday.context_parse(ctx)?))
        } else if s.contains(">=") {
            let (week_day, day) = parse_date_split(s, ">=", ctx)?;
            Ok(DayOfMonth::WeekDayGEThanMonthDay(week_day, day))
        } else if s.contains("<=") {
            let (week_day, day) = parse_date_split(s, "<=", ctx)?;
            Ok(DayOfMonth::WeekDayLEThanMonthDay(week_day, day))
        } else {
            s.context_parse(ctx).map(DayOfMonth::Day)
        };
        ctx.exit();
        result
    }
}

fn parse_date_split(
    s: &str,
    pat: &str,
    ctx: &mut LineParseContext,
) -> Result<(WeekDay, u8), ZoneInfoParseError> {
    let (week_day, num) = s.split_once(pat).ok_or(ZoneInfoParseError::UnknownValue(
        ctx.line_number,
        s.to_owned(),
    ))?;
    let w = week_day.context_parse::<WeekDay>(ctx)?;
    let d = num.context_parse(ctx)?;
    Ok((w, d))
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum WeekDay {
    Mon = 1,
    Tues,
    Wed,
    Thurs,
    Fri,
    Sat,
    Sun,
}

impl TryFromStr<LineParseContext> for WeekDay {
    type Error = ZoneInfoParseError;
    fn try_from_str(s: &str, ctx: &mut LineParseContext) -> Result<Self, Self::Error> {
        match s {
            "Mon" => Ok(Self::Mon),
            "Tues" => Ok(Self::Tues),
            "Wed" => Ok(Self::Wed),
            "Thu" => Ok(Self::Thurs),
            "Fri" => Ok(Self::Fri),
            "Sat" => Ok(Self::Sat),
            "Sun" => Ok(Self::Sun),
            _ => Err(ZoneInfoParseError::UnknownValue(
                ctx.line_number,
                s.to_owned(),
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualifiedTimeKind {
    Local,
    Standard,
    Universal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QualifiedTime {
    // Local time including dst shifts
    Local(Time),
    // Local standard time including dst shifts
    Standard(Time),
    Universal(Time),
}

impl QualifiedTime {
    /// Returns universal seconds
    pub fn to_universal_seconds(&self, std_offset: &Time, save: &Time) -> i64 {
        match self {
            Self::Local(t) => t.as_secs() - std_offset.as_secs() - save.as_secs(),
            Self::Standard(t) => t.as_secs() - std_offset.as_secs(),
            Self::Universal(t) => t.as_secs(),
        }
    }

    pub fn time_kind(&self) -> QualifiedTimeKind {
        match self {
            Self::Local(_) => QualifiedTimeKind::Local,
            Self::Standard(_) => QualifiedTimeKind::Standard,
            Self::Universal(_) => QualifiedTimeKind::Universal,
        }
    }
}

impl TryFromStr<LineParseContext> for QualifiedTime {
    type Error = ZoneInfoParseError;
    fn try_from_str(s: &str, ctx: &mut LineParseContext) -> Result<Self, Self::Error> {
        // Standard suffix
        if let Some(time) = s.strip_suffix("s") {
            return at_time_variant_from_str(time, ctx, Self::Standard);
        // Universal suffix
        } else if let Some(time) = s.strip_suffix("u") {
            return at_time_variant_from_str(time, ctx, Self::Universal);
        } else if let Some(time) = s.strip_suffix("g") {
            return at_time_variant_from_str(time, ctx, Self::Universal);
        } else if let Some(time) = s.strip_suffix("z") {
            return at_time_variant_from_str(time, ctx, Self::Universal);
        } else if let Some(time) = s.strip_suffix("w") {
            return at_time_variant_from_str(time, ctx, Self::Local);
        }
        at_time_variant_from_str(s, ctx, Self::Local)
    }
}

fn at_time_variant_from_str<F>(
    s: &str,
    ctx: &mut LineParseContext,
    variant: F,
) -> Result<QualifiedTime, ZoneInfoParseError>
where
    F: FnOnce(Time) -> QualifiedTime,
{
    let time = s.context_parse::<Time>(ctx)?;
    Ok(variant(time))
}

#[cfg(test)]
mod tests {
    use alloc::borrow::ToOwned;

    use crate::types::FormattableAbbr;

    use super::AbbreviationFormat;

    #[test]
    fn abbr_formatting() {
        let abbr = AbbreviationFormat::Numeric.format(3600, Some("D"), true);
        assert_eq!(abbr, "+01");

        let abbr = AbbreviationFormat::Formattable(FormattableAbbr("C%sT".to_owned())).format(
            3600,
            Some("D"),
            false,
        );
        assert_eq!(abbr, "CDT");

        let abbr = AbbreviationFormat::Pair("CST".to_owned(), "CDT".to_owned()).format(
            3600,
            Some("D"),
            true,
        );
        assert_eq!(abbr, "CDT");

        let abbr = AbbreviationFormat::Formattable(FormattableAbbr("C%sT".to_owned())).format(
            3600,
            Some("S"),
            false,
        );
        assert_eq!(abbr, "CST");

        let abbr = AbbreviationFormat::Pair("CST".to_owned(), "CDT".to_owned()).format(
            3600,
            Some("S"),
            false,
        );
        assert_eq!(abbr, "CST");
    }
}
