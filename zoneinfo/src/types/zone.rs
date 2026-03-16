//! Types used to define a zone table line.

use core::fmt::Write;

use alloc::{borrow::ToOwned, string::String};

use crate::{
    parser::{next_split, ContextParse, LineParseContext, TryFromStr, ZoneInfoParseError},
    rule::epoch_days_for_rule_date,
    types::{rule::DayOfMonth, Month, QualifiedTime, Time},
    utils,
};

/// The value in the `NAME` column of a zone table that identifies the
/// active rule for that line.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum RuleIdentifier {
    None,
    Numeric(Time),
    Named(String),
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
        Ok(Self::Named(s.to_owned()))
    }
}

/// [`AbbreviationFormat`] is the value present in the `FORMAT` column of
/// a zone line
#[derive(Debug, Clone, PartialEq)]
pub enum AbbreviationFormat {
    String(String),
    Numeric,
    Pair(String, String),
    Formattable(FormattableAbbr),
}

impl AbbreviationFormat {
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
            let (std, dst) = s
                .split_once('/')
                .ok_or(ZoneInfoParseError::unknown(s, ctx))?;
            Ok(Self::Pair(std.to_owned(), dst.to_owned()))
        } else {
            Ok(AbbreviationFormat::String(s.to_owned()))
        };
        ctx.exit();
        value
    }
}

/// A formattable abbreviation (e.g. `C%sT`)
///
/// This type will need to be further formatted with a `LETTER` value from
/// the active rule.
#[derive(Debug, Clone, PartialEq)]
pub struct FormattableAbbr(pub(crate) String);

impl FormattableAbbr {
    pub fn to_formatted_string(&self, letter: &str) -> String {
        self.0.replace("%s", letter)
    }
}

/// Represents the value in the `[UNTIL]` column, which designates the final instant
/// that the current zone line is active.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UntilDateTime {
    pub date: Date,
    pub time: QualifiedTime,
}

impl UntilDateTime {
    pub fn as_date_secs(self) -> i64 {
        self.date.as_secs()
    }

    pub fn as_precise_ut_time(self, std_offset: i64, save: i64) -> i64 {
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

/// The date portion of an UNTIL date.
///
/// This is typically represented as YEAR, MONTH, DAY-OF-MONTH
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
