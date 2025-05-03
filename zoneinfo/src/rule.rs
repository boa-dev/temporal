use core::ops::RangeInclusive;

use alloc::{borrow::ToOwned, collections::BTreeSet, string::String, vec, vec::Vec};

use crate::{
    parser::{next_split, ContextParse, LineParseContext, ZoneInfoParseError},
    types::{DayOfMonth, Month, QualifiedTime, QualifiedTimeKind, Time, ToYear, Transition},
    utils::{self, epoch_seconds_for_epoch_days},
    zone::ZoneBuildContext,
};

/// An internal struct for returning the applicable rules found
/// for a year.
#[derive(Debug)]
pub(crate) struct ApplicableRules {
    // Preloaded saving of the applicable rules' dst
    pub(crate) saving: Time,
    pub(crate) transitions: BTreeSet<Transition>,
}

/// The `Rule` is a collection of zone info rules under the same
/// rule name.
///
/// These rule collections can be seen throughout zoneinfo files.
///
/// # Example
///
/// The `Chicago` rules can be seen below.
///
/// ```txt
/// # Rule    NAME    FROM    TO    -    IN    ON    AT    SAVE    LETTER
/// Rule    Chicago    1920    only    -    Jun    13    2:00    1:00    D
/// Rule    Chicago    1920    1921    -    Oct    lastSun    2:00    0    S
/// Rule    Chicago    1921    only    -    Mar    lastSun    2:00    1:00    D
/// Rule    Chicago    1922    1966    -    Apr    lastSun    2:00    1:00    D
/// Rule    Chicago    1922    1954    -    Sep    lastSun    2:00    0    S
/// Rule    Chicago    1955    1966    -    Oct    lastSun    2:00    0    S
/// ```
///
/// Interestingly, Rules appear to be sorted in chronological
/// order from their start date (FROM). However, their end dates may differ
/// meaning at any one time there can be rule pairs of: [std, dst],
/// [dst, std], [std, empty], or [dst, empty]
///
#[derive(Debug, Clone)]
pub struct Rules {
    rules: Vec<Rule>,
}

impl Rules {
    pub fn initialize(rule: Rule) -> Self {
        Self { rules: vec![rule] }
    }

    pub fn extend(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    // NOTE: To be precise, we will need the savings value active across year boundaries.
    pub(crate) fn get_rules_for_year(
        &self,
        year: i32,
        std_offset: &Time,
        use_until: i64,
        ctx: &mut ZoneBuildContext,
    ) -> ApplicableRules {
        let mut ordered = BTreeSet::default();
        for rule in &self.rules {
            if rule.range().contains(&year) {
                let transition_time =
                    rule.transition_time_for_year(year, std_offset, &Time::default());
                let transition = Transition {
                    at_time: transition_time,
                    offset: std_offset.as_secs() + rule.save.as_secs(),
                    dst: rule.is_dst(),
                    savings: rule.save,
                    letter: rule.letter.clone(),
                    time_type: rule.at.time_kind(),
                    format: String::new(),
                };
                let _ = ordered.insert(transition);
            }
        }

        let mut saving = ctx.saving;

        // We must push to a vec then create a BTreeSet, because rules
        // are unordered, and we NEED the savings value to compute the
        // the std transition time.
        let mut transitions = BTreeSet::default();
        for mut transition in ordered {
            let new_time = match transition.time_type {
                QualifiedTimeKind::Local => transition.at_time - saving.as_secs(),
                _ => transition.at_time,
            };
            // Check and see if this transition is valid for use until
            if new_time < use_until {
                saving = transition.savings;
                transition.at_time = new_time;
                let _ = transitions.insert(transition);
            }
        }

        ApplicableRules {
            saving,
            transitions,
        }
    }

    /// A method to search for the last applicable savings for a transition point.
    ///
    /// The last savings needs to be searched for from the beginning because the
    /// rules are sorted by start date, not the end date. So, in theory, a rule
    /// could be the second rule of ten, but still be active longer then the
    /// following eight rules.
    pub(crate) fn search_last_savings(&self, transition_point: i64) -> Time {
        // Reasonable assumption: when searching for a last savings value,
        // we are dealing with an orphan. This means we do not need to check years
        // with an upper bound or inside them
        let mut rule_savings = (i64::MIN, Time::default());
        for rule in &self.rules {
            let year = rule.to.map(ToYear::to_i32).unwrap_or(i32::from(rule.from));
            let epoch_days = epoch_days_for_rule_date(year, rule.in_month, rule.on_date);
            let rule_date_in_seconds = epoch_seconds_for_epoch_days(epoch_days);
            // But we do want to keep track of the savings.
            if rule_date_in_seconds < transition_point && rule_savings.0 < rule_date_in_seconds {
                rule_savings = (rule_date_in_seconds, rule.save)
            } else if transition_point < rule_date_in_seconds {
                break;
            }
        }

        rule_savings.1
    }
}

/// A zone info rule.
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub from: u16,
    pub to: Option<ToYear>,
    pub in_month: Month,
    pub on_date: DayOfMonth,
    pub at: QualifiedTime,
    pub save: Time,
    pub letter: Option<String>,
}

impl Rule {
    fn range(&self) -> RangeInclusive<i32> {
        i32::from(self.from)..=self.to.map(ToYear::to_i32).unwrap_or(self.from as i32)
    }

    fn is_dst(&self) -> bool {
        match &self.letter {
            Some(letter) if letter == "D" => true,
            // NOTE: Potentially remove? Need to test
            // "S" cannot be reliably used as an indicator for "standard", because it's also "Summer"
            Some(letter) if letter == "S" && self.save == Time::default() => false,
            // Yes, there are other letters than S and D, like US's W and P, and Europe's M
            _ => self.save != Time::default(),
        }
    }

    /// Returns the transition time for that year
    fn transition_time_for_year(&self, year: i32, std_offset: &Time, saving: &Time) -> i64 {
        let epoch_days = epoch_days_for_rule_date(year, self.in_month, self.on_date);
        let epoch_seconds = epoch_seconds_for_epoch_days(epoch_days);
        epoch_seconds
            + self
                .at
                .to_universal_seconds(std_offset.as_secs(), saving.as_secs())
    }
}

/// epoch_days_for_rule_date calculates the epoch days given values provided for a specific `Rule`
pub(crate) fn epoch_days_for_rule_date(year: i32, month: Month, day_of_month: DayOfMonth) -> i32 {
    let day_of_year_for_month = month.month_start_to_day_of_year(year);
    let epoch_days_for_year = utils::epoch_days_for_year(year);
    let epoch_days = epoch_days_for_year + day_of_year_for_month;
    let day_of_month = match day_of_month {
        DayOfMonth::Last(weekday) => {
            let mut day_of_month = month.month_end_to_day_of_year(year) - day_of_year_for_month;
            loop {
                let target_days = epoch_days + day_of_month;
                let target_week_day = utils::epoch_days_to_week_day(target_days);
                if target_week_day == weekday as u8 {
                    break;
                }
                day_of_month -= 1;
            }
            day_of_month
        }
        DayOfMonth::WeekDayGEThanMonthDay(week_day, d) => {
            let mut day_of_month = d as i32 - 1;
            loop {
                let target_days = epoch_days + day_of_month;
                let target_week_day = utils::epoch_days_to_week_day(target_days);
                if week_day as u8 == target_week_day {
                    break day_of_month;
                }
                day_of_month += 1;
            }
        }
        DayOfMonth::WeekDayLEThanMonthDay(week_day, d) => {
            let mut day_of_month = d as i32 - 1;
            loop {
                let target_days = epoch_days + day_of_month;
                let target_week_day = utils::epoch_days_to_week_day(target_days);
                if week_day as u8 == target_week_day {
                    break day_of_month;
                }
                day_of_month -= 1;
            }
        }
        DayOfMonth::Day(day) => day as i32 - 1,
    };
    epoch_days + day_of_month
}

impl Rule {
    /// Parse a `Rule` from a line
    ///
    /// A rule line is made up of the following columns:
    ///
    /// # Rule    NAME    FROM    TO    -    IN    ON    AT    SAVE    LETTER
    ///
    /// The "-" is a reserved field that represents the deprecated TYPE
    /// field. It is preserved for backward compatibility reasons.
    pub fn parse_from_line(
        line: &str,
        context: &mut LineParseContext,
    ) -> Result<(String, Self), ZoneInfoParseError> {
        context.enter("Rule");
        let mut splits = line.split_whitespace();
        let first = splits.next(); // Consume "Rule"
        debug_assert!(first == Some("Rule"));
        // AKA the NAME field
        let identifier = next_split(&mut splits, context)?.to_owned();
        let from = next_split(&mut splits, context)?.context_parse::<u16>(context)?;
        let to = ToYear::parse_optional_to_year(next_split(&mut splits, context)?, context)?;
        next_split(&mut splits, context)?; // Skip the deprecated TYPE field
        let in_month = next_split(&mut splits, context)?.context_parse::<Month>(context)?;
        let on_date = next_split(&mut splits, context)?.context_parse::<DayOfMonth>(context)?;
        let at = next_split(&mut splits, context)?.context_parse::<QualifiedTime>(context)?;
        let save = next_split(&mut splits, context)?.context_parse::<Time>(context)?;
        let potential_letter = next_split(&mut splits, context)?;
        let letter = if potential_letter == "-" {
            None
        } else {
            Some(potential_letter.to_owned())
        };

        context.exit();
        let data = Rule {
            from,
            to,
            in_month,
            on_date,
            at,
            save,
            letter,
        };

        Ok((identifier, data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Sign, WeekDay};

    const TEST_DATA: [&str; 22] = [
        "Rule	Algeria	1916	only	-	Jun	14	23:00s	1:00	S",
        "Rule	Algeria	1916	1919	-	Oct	Sun>=1	23:00s	0	-",
        "Rule	Algeria	1917	only	-	Mar	24	23:00s	1:00	S",
        "Rule	Algeria	1918	only	-	Mar	 9	23:00s	1:00	S",
        "Rule	Algeria	1919	only	-	Mar	 1	23:00s	1:00	S",
        "Rule	Algeria	1920	only	-	Feb	14	23:00s	1:00	S",
        "Rule	Algeria	1920	only	-	Oct	23	23:00s	0	-",
        "Rule	Algeria	1921	only	-	Mar	14	23:00s	1:00	S",
        "Rule	Algeria	1921	only	-	Jun	21	23:00s	0	-",
        "Rule	Algeria	1939	only	-	Sep	11	23:00s	1:00	S",
        "Rule	Algeria	1939	only	-	Nov	19	 1:00	0	-",
        "Rule	Algeria	1944	1945	-	Apr	Mon>=1	 2:00	1:00	S",
        "Rule	Algeria	1944	only	-	Oct	 8	 2:00	0	-",
        "Rule	Algeria	1945	only	-	Sep	16	 1:00	0	-",
        "Rule	Algeria	1971	only	-	Apr	25	23:00s	1:00	S",
        "Rule	Algeria	1971	only	-	Sep	26	23:00s	0	-",
        "Rule	Algeria	1977	only	-	May	 6	 0:00	1:00	S",
        "Rule	Algeria	1977	only	-	Oct	21	 0:00	0	-",
        "Rule	Algeria	1978	only	-	Mar	24	 1:00	1:00	S",
        "Rule	Algeria	1978	only	-	Sep	22	 3:00	0	-",
        "Rule	Algeria	1980	only	-	Apr	25	 0:00	1:00	S",
        "Rule	Algeria	1980	only	-	Oct	31	 2:00	0	-",
    ];

    #[test]
    fn rule_test() {
        let (identifier, data) =
            Rule::parse_from_line(TEST_DATA[0], &mut LineParseContext::default()).unwrap();
        assert_eq!(identifier, "Algeria");
        assert_eq!(
            data,
            Rule {
                from: 1916,
                to: None,
                in_month: Month::Jun,
                on_date: DayOfMonth::Day(14),
                at: QualifiedTime::Standard(Time {
                    sign: Sign::Positive,
                    hour: 23,
                    minute: 0,
                    second: 0
                }),
                save: Time {
                    sign: Sign::Positive,
                    hour: 1,
                    minute: 0,
                    second: 0
                },
                letter: Some("S".to_owned()),
            }
        );
    }

    #[test]
    fn cycle_test() {
        for line in TEST_DATA {
            let _success = Rule::parse_from_line(line, &mut LineParseContext::default()).unwrap();
        }
    }

    #[test]
    fn date_calcs() {
        // Test epoch
        let epoch_days = epoch_days_for_rule_date(1970, Month::Jan, DayOfMonth::Day(1));
        assert_eq!(epoch_days, 0);

        // Test modern day
        let epoch_days = epoch_days_for_rule_date(2025, Month::Mar, DayOfMonth::Day(29));
        assert_eq!(epoch_days, 20176);
        let epoch_days = epoch_days_for_rule_date(
            2025,
            Month::Mar,
            DayOfMonth::WeekDayGEThanMonthDay(WeekDay::Sat, 29),
        );
        assert_eq!(epoch_days, 20176);
        let epoch_days = epoch_days_for_rule_date(
            2025,
            Month::Mar,
            DayOfMonth::WeekDayGEThanMonthDay(WeekDay::Sat, 25),
        );
        assert_eq!(epoch_days, 20176);
        let epoch_days = epoch_days_for_rule_date(
            2025,
            Month::Mar,
            DayOfMonth::WeekDayLEThanMonthDay(WeekDay::Sat, 29),
        );
        assert_eq!(epoch_days, 20176);
        let epoch_days = epoch_days_for_rule_date(
            2025,
            Month::Mar,
            DayOfMonth::WeekDayLEThanMonthDay(WeekDay::Sat, 30),
        );
        assert_eq!(epoch_days, 20176);
        let epoch_days = epoch_days_for_rule_date(2025, Month::Mar, DayOfMonth::Last(WeekDay::Sun));
        assert_eq!(epoch_days, 20177);

        // Test pre epoch
        let epoch_days = epoch_days_for_rule_date(1969, Month::Dec, DayOfMonth::Day(31));
        assert_eq!(epoch_days, -1);
        let epoch_days = epoch_days_for_rule_date(1969, Month::Dec, DayOfMonth::Last(WeekDay::Sun));
        assert_eq!(epoch_days, -4);
        let epoch_days = epoch_days_for_rule_date(
            1969,
            Month::Dec,
            DayOfMonth::WeekDayGEThanMonthDay(WeekDay::Sun, 25),
        );
        assert_eq!(epoch_days, -4);
        let epoch_days = epoch_days_for_rule_date(
            1969,
            Month::Dec,
            DayOfMonth::WeekDayLEThanMonthDay(WeekDay::Sun, 30),
        );
        assert_eq!(epoch_days, -4);
    }
}
