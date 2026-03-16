//! Types used to represent a Rule line in a zoneinfo file.

use alloc::borrow::ToOwned;

use crate::parser::{ContextParse, LineParseContext, TryFromStr, ZoneInfoParseError};

/// The value present in the `TO` column of a rule line.
///
/// This value can either be "max" or an unsigned integer representing the year.
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
            Self::Max => 275_760,
            Self::Year(y) => y as i32,
        }
    }

    pub(crate) fn to_optional_u16(self) -> Option<u16> {
        match self {
            Self::Max => None,
            Self::Year(y) => Some(y),
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

/// The day of the month as listed by the `ON` column of a rule line.
///
/// The values can be a day, a GE or LE identifier (Sun>=8), or "lastSun", which
/// represents the last sunday of the month.
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
    let (week_day, num) = s
        .split_once(pat)
        .ok_or(ZoneInfoParseError::unknown(s, ctx))?;
    let w = week_day.context_parse::<WeekDay>(ctx)?;
    let d = num.context_parse(ctx)?;
    Ok((w, d))
}

/// A week day value, this is used in the `ON` column values.
///
/// NOTE: week days are zero based beginning with Sunday.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum WeekDay {
    Sun = 0,
    Mon,
    Tues,
    Wed,
    Thurs,
    Fri,
    Sat,
}

impl WeekDay {
    pub(crate) fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Sun,
            1 => Self::Mon,
            2 => Self::Tues,
            3 => Self::Wed,
            4 => Self::Thurs,
            5 => Self::Fri,
            6 => Self::Sat,
            _ => unreachable!("invalid week day value"),
        }
    }
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
