use crate::{
    rule::{LastRules, Rule},
    types::{DayOfMonth, Month, QualifiedTime, Sign, Time, WeekDay},
    utils::month_to_day,
    zone::ZoneEntry,
};
use alloc::string::String;
use core::fmt::Write;

/// The POSIX time zone designated by the [GNU documentation][gnu-docs]
///
/// [gnu-docs]: https://www.gnu.org/software/libc/manual/html_node/TZ-Variable.html
#[derive(Debug, PartialEq)]
pub struct PosixTimeZone {
    pub abbr: PosixAbbreviation,
    pub offset: Time,
    pub transition_info: Option<PosixTransition>,
}

impl PosixTimeZone {
    pub(crate) fn from_zone_and_savings(entry: &ZoneEntry, savings: Time) -> Self {
        let offset = entry.std_offset.add(savings);
        let formatted = entry
            .format
            .format(offset.as_secs(), None, savings != Time::default());
        let is_numeric = is_numeric(&formatted);
        let abbr = PosixAbbreviation {
            is_numeric,
            formatted,
        };
        Self {
            abbr,
            offset,
            transition_info: None,
        }
    }

    pub(crate) fn from_zone_and_rules(entry: &ZoneEntry, rules: &LastRules) -> Self {
        let offset = entry.std_offset.add(rules.standard.save);
        let formatted = entry.format.format(
            entry.std_offset.as_secs(),
            rules.standard.letter.as_deref(),
            rules.standard.is_dst(),
        );
        let is_numeric = is_numeric(&formatted);
        let abbr = PosixAbbreviation {
            is_numeric,
            formatted,
        };

        let transition_info = rules.saving.as_ref().map(|rule| {
            let formatted = entry.format.format(
                entry.std_offset.as_secs() + rule.save.as_secs(),
                rule.letter.as_deref(),
                rule.is_dst(),
            );
            let abbr = PosixAbbreviation {
                is_numeric,
                formatted,
            };
            let savings = rule.save;
            let start = PosixDateTime::from_rule_and_transition_info(
                rule,
                entry.std_offset,
                rules.standard.save,
            );
            let end = PosixDateTime::from_rule_and_transition_info(
                &rules.standard,
                entry.std_offset,
                rule.save,
            );
            PosixTransition {
                abbr,
                savings,
                start,
                end,
            }
        });

        PosixTimeZone {
            abbr,
            offset,
            transition_info,
        }
    }
}

impl PosixTimeZone {
    pub fn to_string(&self) -> Result<String, core::fmt::Error> {
        let mut posix_string = String::new();
        write_abbr(&self.abbr, &mut posix_string)?;
        write_inverted_time(&self.offset, &mut posix_string)?;

        if let Some(transition_info) = &self.transition_info {
            write_abbr(&transition_info.abbr, &mut posix_string)?;
            if transition_info.savings != Time::one_hour() {
                write_inverted_time(&self.offset.add(transition_info.savings), &mut posix_string)?;
            }
            write_date_time(&transition_info.start, &mut posix_string)?;
            write_date_time(&transition_info.end, &mut posix_string)?;
        }
        Ok(posix_string)
    }
}

/// The representation of a POSIX time zone transition
#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub struct PosixTransition {
    /// The transitions designated abbreviation
    pub abbr: PosixAbbreviation,
    /// The savings value to be added to the offset
    pub savings: Time,
    /// The start time for the transition
    pub start: PosixDateTime,
    /// The end time for the transition
    pub end: PosixDateTime,
}

#[non_exhaustive]
#[derive(Debug, PartialEq, Clone)]
pub struct PosixAbbreviation {
    /// Flag whether formatted abbreviation is numeric
    pub is_numeric: bool,
    /// The formatted abbreviation
    pub formatted: String,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct MonthWeekDay(pub Month, pub u8, pub WeekDay);

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PosixDate {
    JulianNoLeap(u16),
    JulianLeap(u16),
    MonthWeekDay(MonthWeekDay),
}

impl PosixDate {
    /// Creates a [`PosixDate`] from a provided rule. This method returns both a posix date and an
    /// integer, representing the days off the target weekday in seconds.
    pub(crate) fn from_rule(rule: &Rule) -> (Self, i64) {
        match rule.on_date {
            DayOfMonth::Day(day) if rule.in_month == Month::Jan || rule.in_month == Month::Feb => (
                PosixDate::JulianNoLeap(month_to_day(rule.in_month as u8, 1) as u16 + day as u16),
                0,
            ),
            DayOfMonth::Day(day) => (
                PosixDate::JulianLeap(month_to_day(rule.in_month as u8, 1) as u16 + day as u16),
                0,
            ),
            DayOfMonth::Last(wd) => (
                PosixDate::MonthWeekDay(MonthWeekDay(rule.in_month, 5, wd)),
                0,
            ),
            DayOfMonth::WeekDayGEThanMonthDay(week_day, day_of_month) => {
                // Handle week day offset correctly (See America/Santiago; i.e. Sun>=2)
                //
                // To do this for the GE case, we work with a zero based day of month,
                // This ensures that day_of_month being 1 aligns with Sun = 0, for
                // Sun>=1 purposes.
                //
                // The primary purpose for this approach as noted in zic.c is to support
                // America/Santiago timestamps beyond 2038.
                //
                // See the below link for more info.
                //
                // https://github.com/eggert/tz/commit/07351e0248b5a42151e49e4506bca0363c846f8c

                // Calculate the difference between the day of month and the week day.
                let zero_based_day_of_month = day_of_month - 1;
                let week_day_from_dom = zero_based_day_of_month % 7;
                // N.B., this could be a negative. If we look at Sun>=2, then this becomes
                // 0 - 1.
                let mut adjusted_week_day = week_day as i8 - week_day_from_dom as i8;

                // Calculate what week we are in.
                //
                // Since we are operating with a zero based day of month, we add
                let week = 1 + zero_based_day_of_month / 7;

                // If we have shifted beyond the month, add 7 to shift back into the first
                // week.
                if adjusted_week_day < 0 {
                    adjusted_week_day += 7;
                }
                let week_day = WeekDay::from_u8(adjusted_week_day as u8);
                // N.B. The left of time the target weekday becomes a time overflow added
                // to the minutes.
                (
                    PosixDate::MonthWeekDay(MonthWeekDay(rule.in_month, week, week_day)),
                    week_day_from_dom as i64 * 86_400,
                )
            }
            DayOfMonth::WeekDayLEThanMonthDay(week_day, day_of_month) => {
                // Handle week day offset correctly
                //
                // We don't worry about the last day of the month in this scenario, which
                // is the upper bound as that is handled by DayOfMonth::Last
                let week_day_from_dom = day_of_month as i8 % 7;
                let mut adjusted_week_day = week_day as i8 - week_day_from_dom;
                let week = day_of_month / 7;
                if adjusted_week_day < 0 {
                    adjusted_week_day += 7;
                }
                (
                    PosixDate::MonthWeekDay(MonthWeekDay(
                        rule.in_month,
                        week,
                        WeekDay::from_u8(adjusted_week_day as u8),
                    )),
                    week_day_from_dom as i64 * 86_400,
                )
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PosixDateTime {
    /// The designated [`PosixDate`]
    pub date: PosixDate,
    /// The local time for a [`PosixDateTime`] at which a transition occurs.
    ///
    /// N.B., this can be in the range of -167..=167
    pub time: Time,
}

impl PosixDateTime {
    pub(crate) fn from_rule_and_transition_info(rule: &Rule, offset: Time, savings: Time) -> Self {
        let (date, time_overflow) = PosixDate::from_rule(rule);
        let time = match rule.at {
            QualifiedTime::Local(time) => time,
            QualifiedTime::Standard(standard_time) => standard_time
                .add(rule.save),
            QualifiedTime::Universal(universal_time) => universal_time
                .add(offset)
                .add(savings)
        };
        let time = time.add(Time::from_seconds(time_overflow));
        Self { date, time }
    }
}

// ==== Helper functions ====

fn is_numeric(str: &str) -> bool {
    str.parse::<i16>().is_ok()
}

fn write_abbr(posix_abbr: &PosixAbbreviation, output: &mut String) -> core::fmt::Result {
    if posix_abbr.is_numeric {
        write!(output, "<")?;
        write!(output, "{}", posix_abbr.formatted)?;
        write!(output, ">")?;
        return Ok(());
    }
    write!(output, "{}", posix_abbr.formatted)
}

fn write_inverted_time(time: &Time, output: &mut String) -> core::fmt::Result {
    // Yep, it's inverted
    if time.sign == Sign::Positive && time.hour != 0 {
        write!(output, "-")?;
    }
    write_time(time, output)
}

fn write_time(time: &Time, output: &mut String) -> core::fmt::Result {
    write!(output, "{}", time.hour)?;
    if time.minute == 0 && time.second == 0 {
        return Ok(());
    }
    write!(output, ":{}", time.minute)?;
    if time.second > 0 {
        write!(output, ":{}", time.second)?;
    }
    Ok(())
}

fn write_date_time(datetime: &PosixDateTime, output: &mut String) -> core::fmt::Result {
    write!(output, ",")?;
    match datetime.date {
        PosixDate::JulianLeap(d) => write!(output, "{d}")?,
        PosixDate::JulianNoLeap(d) => write!(output, "J{d}")?,
        PosixDate::MonthWeekDay(MonthWeekDay(month, week, day)) => {
            write!(output, "M{}.{week}.{}", month as u8, day as u8)?
        }
    }
    if datetime.time != Time::two_hour() {
        write!(output, "/")?;
        write_time(&datetime.time, output)?;
    }
    Ok(())
}
