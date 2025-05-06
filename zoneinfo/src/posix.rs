use crate::{
    rule::{LastRules, Rule},
    types::{DayOfMonth, Month, QualifiedTime, Sign, Time, WeekDay},
    utils::month_to_day,
    zone::ZoneEntry,
};
use alloc::string::String;
use core::fmt::Write;

#[derive(Debug)]
pub struct MonthWeekDay(pub Month, pub u8, pub WeekDay);

#[derive(Debug)]
pub enum PosixDate {
    JulianNoLeap(u16),
    JulianLeap(u16),
    MonthWeekDay(MonthWeekDay),
}

impl PosixDate {
    pub(crate) fn from_rule(rule: &Rule) -> Self {
        match rule.on_date {
            DayOfMonth::Day(day) if rule.in_month == Month::Jan || rule.in_month == Month::Feb => {
                PosixDate::JulianNoLeap(month_to_day(rule.in_month as u8, 1) as u16 + day as u16)
            }
            DayOfMonth::Day(day) => {
                PosixDate::JulianNoLeap(month_to_day(rule.in_month as u8, 1) as u16 + day as u16)
            }
            DayOfMonth::Last(wd) => PosixDate::MonthWeekDay(MonthWeekDay(rule.in_month, 5, wd)),
            DayOfMonth::WeekDayGEThanMonthDay(week_day, day_of_month) => {
                let week = 1 + (day_of_month - 1) / 7;
                PosixDate::MonthWeekDay(MonthWeekDay(rule.in_month, week, week_day))
            }
            DayOfMonth::WeekDayLEThanMonthDay(week_day, day_of_month) => {
                let week = day_of_month / 7;
                PosixDate::MonthWeekDay(MonthWeekDay(rule.in_month, week, week_day))
            }
        }
    }
}

#[derive(Debug)]
pub struct PosixDateTime {
    pub date: PosixDate,
    pub time: Time,
}

impl PosixDateTime {
    pub(crate) fn from_rule_and_transition_info(rule: &Rule, offset: Time, savings: Time) -> Self {
        let date = PosixDate::from_rule(rule);
        let time = match rule.at {
            QualifiedTime::Local(time) => time,
            QualifiedTime::Standard(standard_time) => standard_time.add(rule.save),
            QualifiedTime::Universal(universal_time) => universal_time.add(offset).add(savings),
        };
        Self { date, time }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct PosixTransition {
    pub abbr: PosixAbbreviation,
    pub savings: Time,
    pub start: PosixDateTime,
    pub end: PosixDateTime,
}

#[non_exhaustive]
#[derive(Debug)]
pub struct PosixTimeZone {
    pub abbr: PosixAbbreviation,
    pub offset: Time,
    pub transition_info: Option<PosixTransition>,
}

#[non_exhaustive]
#[derive(Debug)]
pub struct PosixAbbreviation {
    is_numeric: bool,
    formatted: String,
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
            if transition_info.savings != Time::one() {
                write_inverted_time(&self.offset.add(transition_info.savings), &mut posix_string)?;
            }
            write_date_time(&transition_info.start, &mut posix_string)?;
            write_date_time(&transition_info.end, &mut posix_string)?;
        }
        Ok(posix_string)
    }
}

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
    if datetime.time != Time::two() {
        write!(output, "/")?;
        write_time(&datetime.time, output)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "std")]
    #[test]
    fn posix_string_test() {
        use std::path::Path;

        use crate::{ZoneInfoCompiler, ZoneInfoData};

        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let zoneinfo = ZoneInfoData::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();

        let mut zic = ZoneInfoCompiler::new(zoneinfo);

        let chicago_posix = zic.get_posix_time_zone("America/Chicago").unwrap();
        assert_eq!(
            chicago_posix.to_string(),
            Ok("CST6CDT,M3.2.0,M11.1.0".into())
        );

        let lord_howe_posix = zic.get_posix_time_zone("Australia/Lord_Howe").unwrap();
        assert_eq!(
            lord_howe_posix.to_string(),
            Ok("<+1030>-10:30<+11>-11,M10.1.0,M4.1.0".into())
        );

        let troll_posix = zic.get_posix_time_zone("Antarctica/Troll").unwrap();
        assert_eq!(
            troll_posix.to_string(),
            Ok("<+00>0<+02>-2,M3.5.0/1,M10.5.0/3".into())
        );

        let dublin_posix = zic.get_posix_time_zone("Europe/Dublin").unwrap();
        assert_eq!(
            dublin_posix.to_string(),
            Ok("IST-1GMT0,M10.5.0,M3.5.0/1".into())
        );

        let minsk_posix = zic.get_posix_time_zone("Europe/Minsk").unwrap();
        assert_eq!(minsk_posix.to_string(), Ok("<+03>-3".into()));

        let moscow_posix = zic.get_posix_time_zone("Europe/Moscow").unwrap();
        assert_eq!(moscow_posix.to_string(), Ok("MSK-3".into()));
    }
}
