//! Zoneinfo types
//!
//! This module contains general types that are present in a zone info
//! file.
//!
//! For more information, see [How to Read tz Database Source Files][tz-how-to].
//!
//! [tz-how-to]: https://data.iana.org/time-zones/tz-how-to.html

use alloc::borrow::ToOwned;

use crate::{
    parser::{ContextParse, LineParseContext, TryFromStr, ZoneInfoParseError},
    utils,
};

pub mod rule;
pub mod zone;

// General shared types between the two lines

/// An enum representing a three letter abbreviated month (e.g. `Jan`, `Sep`).
///
/// The month value is present in the `IN` column of a rule line or the date
/// month portion in the [UNTIL] column of a zone line.
///
/// Note: month is 1 based (1-12).
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
    /// Calculates the day of year for the start of the month
    pub(crate) fn month_start_to_day_of_year(self, year: i32) -> i32 {
        utils::month_to_day(self as u8, utils::num_leap_days(year))
    }

    /// Calculates the day of year for the end of the month
    pub(crate) fn month_end_to_day_of_year(self, year: i32) -> i32 {
        utils::month_to_day(self as u8 + 1, utils::num_leap_days(year)) - 1
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
            _ => Err(ZoneInfoParseError::unknown(s, ctx)),
        };
        ctx.exit();
        result
    }
}

/// `Time` represents any [-]hh:mm:ss time value
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Time {
    pub sign: Sign,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

/// A non zero sign type that represents whether a value is positive or negative.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(i8)]
pub enum Sign {
    #[default]
    Positive = 1,
    Negative = -1,
}

impl Time {
    pub(crate) const fn one_hour() -> Self {
        Time {
            sign: Sign::Positive,
            hour: 1,
            minute: 0,
            second: 0,
        }
    }

    pub(crate) const fn two_hour() -> Self {
        Time {
            sign: Sign::Positive,
            hour: 2,
            minute: 0,
            second: 0,
        }
    }

    pub const fn as_secs(&self) -> i64 {
        (self.hour as i64 * 3600 + self.minute as i64 * 60 + self.second as i64) * self.sign as i64
    }

    pub const fn from_seconds(seconds: i64) -> Self {
        let sign = if seconds < 0 {
            Sign::Negative
        } else {
            Sign::Positive
        };
        let (hour, rem) = (
            seconds.abs().div_euclid(3600),
            seconds.abs().rem_euclid(3600),
        );
        let (minute, second) = (rem.abs().div_euclid(60), rem.abs().rem_euclid(60));
        debug_assert!(hour < u8::MAX as i64);
        Self {
            sign,
            hour: hour as u8,
            minute: minute as u8,
            second: second as u8,
        }
    }

    pub fn add(&self, other: Self) -> Self {
        // NOTE: this is a nightmare. Redo
        let result = self.as_secs() + other.as_secs();
        Self::from_seconds(result)
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
            let hour = s.context_parse::<u8>(ctx)?;
            ctx.exit();
            return Ok(Time {
                sign,
                hour,
                minute: 0,
                second: 0,
            });
        }
        let (hour, sub_hour) = s
            .split_once(':')
            .ok_or(ZoneInfoParseError::unknown(s, ctx))?;
        let hour = hour.context_parse::<u8>(ctx)?;
        if !sub_hour.contains(':') {
            let minute = sub_hour.context_parse::<u8>(ctx)?;
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
        let minute = minute.context_parse::<u8>(ctx)?;
        let second = second.context_parse::<u8>(ctx)?;
        ctx.exit();
        Ok(Self {
            sign,
            hour,
            minute,
            second,
        })
    }
}

/// This enum represents whether a time is local, standard, or universal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualifiedTimeKind {
    Local,
    Standard,
    Universal,
}

/// `QualifiedTime` represents any [-]hh:mm:ss[u|s|g|z|w] time value,
/// where the time value is qualified with a kind.
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
    pub fn to_universal_seconds(&self, std_offset: i64, save: i64) -> i64 {
        match self {
            Self::Local(t) => t.as_secs() - std_offset - save,
            Self::Standard(t) => t.as_secs() - std_offset,
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

    use super::{
        zone::{AbbreviationFormat, FormattableAbbr},
        Sign, Time,
    };

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

    #[test]
    fn time_add() {
        let one = Time {
            sign: Sign::Positive,
            hour: 1,
            ..Default::default()
        };
        let result = one.add(Time::default());
        assert_eq!(result, one);

        let two = Time {
            sign: Sign::Positive,
            hour: 2,
            ..Default::default()
        };
        let three = one.add(two);
        assert_eq!(
            three,
            Time {
                sign: Sign::Positive,
                hour: 3,
                ..Default::default()
            }
        );

        let neg_three = Time {
            sign: Sign::Negative,
            hour: 3,
            ..Default::default()
        };
        let neg_one = neg_three.add(two);
        assert_eq!(
            neg_one,
            Time {
                sign: Sign::Negative,
                hour: 1,
                ..Default::default()
            }
        );

        let neg_four = neg_one.add(neg_three);
        assert_eq!(
            neg_four,
            Time {
                sign: Sign::Negative,
                hour: 4,
                ..Default::default()
            }
        );

        let one_half = Time {
            sign: Sign::Positive,
            hour: 1,
            minute: 30,
            ..Default::default()
        };
        let neg_one_half = neg_three.add(one_half);
        assert_eq!(
            neg_one_half,
            Time {
                sign: Sign::Negative,
                hour: 1,
                minute: 30,
                ..Default::default()
            }
        );

        let neg_half = one.add(neg_one_half);
        assert_eq!(
            neg_half,
            Time {
                sign: Sign::Negative,
                hour: 0,
                minute: 30,
                ..Default::default()
            }
        )
    }
}
