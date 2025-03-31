//! Zone structures

use core::{iter::Peekable, ops::Range, str::Lines};

use alloc::{borrow::ToOwned, collections::BTreeSet, string::String, vec::Vec};
use hashbrown::HashMap;

use crate::{
    epoch_seconds_for_year,
    parser::{remove_comments, LineParseContext, TryFromStr, ZoneInfoParseError},
    rule::RuleTable,
    types::{QualifiedTimeKind, RuleIdentifier, Time, Transition, UntilDateTime, ZoneEntry},
};

#[derive(Debug, Clone)]
pub(crate) struct ZoneBuildContext {
    pub(crate) saving: Time,
    pub(crate) epoch_year: i64,
    pub(crate) year_seconds: i64,
    pub(crate) year_range: Range<i64>,
    pub(crate) use_start: i64,
    pub(crate) start_kind: QualifiedTimeKind,
    pub(crate) previous_offset: Time,
    pub(crate) previous_rule: RuleIdentifier,
}

impl Default for ZoneBuildContext {
    fn default() -> Self {
        Self {
            saving: Time::default(),
            epoch_year: 0,
            year_seconds: 0,
            year_range: 0..0,
            use_start: i64::MIN,
            start_kind: QualifiedTimeKind::Universal,
            previous_offset: Time::default(),
            previous_rule: RuleIdentifier::None,
        }
    }
}

impl ZoneBuildContext {
    pub(crate) fn update(&mut self, year: i32) {
        // NOTE: May need to adjust for offset + savings.
        let year_seconds = epoch_seconds_for_year(year);
        let year_plus_one = epoch_seconds_for_year(year + 1);
        self.year_seconds = year_seconds;
        self.year_range = year_seconds..year_plus_one;
        self.epoch_year = year_seconds;
        self.use_start = i64::MIN;
    }

    pub(crate) fn update_for_zone_entry(
        &mut self,
        zone: &ZoneEntry,
        last_transition: Option<&Transition>,
        savings: Time,
    ) {
        if let Some(last_transition) = last_transition {
            if last_transition.dst {
                self.saving = savings;
            } else {
                self.saving = Time::default();
            }
        }
        self.previous_offset = zone.std_offset;
        self.previous_rule = zone.rule.clone();

        if let Some(use_until) = zone.date {
            self.start_kind = use_until.time.time_kind();
            self.use_start = use_until.as_precise_ut_time(&zone.std_offset, &self.saving);
            self.year_seconds = match self.start_kind {
                QualifiedTimeKind::Universal => self.epoch_year,
                QualifiedTimeKind::Standard => self.epoch_year - zone.std_offset.as_secs(),
                // Uh, how to handle dst. Does it matter? This will prob blow up on southern hemisphere
                QualifiedTimeKind::Local => {
                    self.epoch_year - zone.std_offset.as_secs() - self.saving.as_secs()
                }
            };
        }
    }

    pub(crate) fn is_zone_beyond_year(&self) -> bool {
        self.year_seconds < self.use_start && !self.is_start_in_year_range()
    }

    pub(crate) fn in_zone_skippable(&self, until_time: i64) -> bool {
        !(self.use_start..=until_time).contains(&self.year_seconds)
            && !self.is_start_in_year_range()
    }

    pub(crate) fn is_start_in_year_range(&self) -> bool {
        self.year_range.contains(&self.use_start)
    }

    pub(crate) fn zone_was_rule(&self) -> bool {
        matches!(self.previous_rule, RuleIdentifier::Rule(_))
    }
}
#[derive(Debug, Clone, Default)]
pub struct ZoneTable {
    pub table: Vec<ZoneEntry>,
    pub associates: HashMap<String, RuleTable>,
}

impl IntoIterator for ZoneTable {
    type Item = ZoneEntry;
    type IntoIter = alloc::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.table.into_iter()
    }
}

impl ZoneTable {
    pub fn associate_rules(&mut self, rules: &HashMap<String, RuleTable>) {
        for entry in &mut self.table {
            if let RuleIdentifier::Rule(associate_rule) = &entry.rule {
                if self.associates.contains_key(associate_rule) {
                    continue;
                }
                if let Some(rules) = rules.get(associate_rule).cloned() {
                    let _ = self.associates.insert(associate_rule.clone(), rules);
                }
            }
        }
    }

    /// Get teh first transition time for this zone table.
    ///
    /// No transition will be lower than this.
    fn get_first_transition_time(&self) -> i64 {
        let lmt_entry = &self.table[0];
        let lmt_date = lmt_entry.date.expect("must exist");
        lmt_date.as_precise_ut_time(&lmt_entry.std_offset, &Time::default())
    }

    pub fn get_first_transition(&self) -> Transition {
        let lmt_entry = &self.table[0];
        let lmt_date = lmt_entry.date.expect("must exist");
        Transition {
            at_time: self.get_first_transition_time(),
            offset: lmt_entry.std_offset.as_secs(),
            dst: false,
            letter: None,
            time_type: lmt_date.time.time_kind(),
            format: String::from("LMT"), // This value is constant for the first transition
        }
    }

    pub(crate) fn calculate_transitions_for_year(
        &self,
        year: i32,
        ctx: &mut ZoneBuildContext,
    ) -> BTreeSet<Transition> {
        // NOTES: We need to be careful here, zones until time may
        // be at the start of the year but could be mid year or
        // multiple times in a year (EX: America/Chicago)

        // Year seconds should be Jan 1 for year.
        let mut output = BTreeSet::default();
        // By default, the zone is the last zone set
        for entry in &self.table {
            // calculate the rough year time for the date.
            //
            // We don't need hour precision currently, because we are deally with the
            // year range.
            let until_time_or_max = entry
                .date
                .map(UntilDateTime::as_date_secs)
                .unwrap_or(i64::MAX);
            // Exit looping entries once year exceeds the until time.
            if ctx.is_zone_beyond_year() {
                break;
            }
            // if the year is not within the start_time to use_until range
            //   and start time is not in this years full range, skip rule.
            if ctx.in_zone_skippable(until_time_or_max) {
                // Update the zone entry context
                ctx.update_for_zone_entry(entry, output.last(), ctx.saving);
                continue;
            }
            // We've determined that are year is viable for this zone entry.
            // Let's move foward
            let mut rule_transitions = BTreeSet::default();
            let savings = match &entry.rule {
                RuleIdentifier::None => {
                    let actual_start = if ctx.use_start == i64::MIN {
                        self.get_first_transition_time()
                    } else {
                        ctx.use_start
                    };
                    // NOTE to self, probably need a global saving var
                    // TODO: Adjust at time for use start kind
                    rule_transitions.insert(Transition {
                        at_time: actual_start,
                        offset: entry.std_offset.as_secs(),
                        dst: false,
                        letter: None,
                        time_type: ctx.start_kind,
                        format: String::new(),
                    });
                    Time::default() // No savings on an empty rule, return 0 savings
                }
                RuleIdentifier::Numeric(t) => {
                    let actual_start = if ctx.use_start == i64::MIN {
                        self.get_first_transition_time()
                    } else {
                        ctx.use_start
                    };
                    rule_transitions.insert(Transition {
                        at_time: actual_start,
                        offset: entry.std_offset.as_secs() + t.as_secs(),
                        dst: true,
                        letter: None,
                        time_type: ctx.start_kind,
                        format: String::new(),
                    });
                    *t
                }
                // TODO: Return something different from rules to support format
                RuleIdentifier::Rule(s) => {
                    let rules = self.associates.get(s).expect("rules were not associated.");
                    let applicable_rules = rules.get_rules_for_year(year, &entry.std_offset, ctx);
                    // If this zone is before any of the would be transitions, skip
                    rule_transitions = applicable_rules.transitions;
                    applicable_rules.saving
                }
            };
            // At this point, we've determined the potential transitions
            // for the year as well as the savings during DST, according
            // to the rule.

            // We now need to determine if `use_start` is a transition
            // based of the context we have.
            if ctx.is_start_in_year_range() {
                let mut temp = None;
                // Whether the Rule transition is the same (EX: Lord Howe)
                let same_rule = ctx.previous_rule == entry.rule;
                let non_rule_zones_are_different = !ctx.zone_was_rule()
                    && (ctx.previous_offset != entry.std_offset || ctx.saving != savings);
                for transition in &rule_transitions {
                    if transition.at_time < ctx.use_start
                        && !same_rule
                        && non_rule_zones_are_different
                    {
                        let mut transition_clone = transition.clone();
                        transition_clone.at_time = ctx.use_start;
                        let _ = temp.insert(transition_clone);
                    }
                }
                let empty_rules = rule_transitions.is_empty();
                if empty_rules || (!same_rule && temp.is_none() && non_rule_zones_are_different) {
                    let _ = temp.insert(Transition {
                        at_time: ctx.use_start,
                        offset: entry.std_offset.as_secs(),
                        dst: false, // An assumption that needs to be proved
                        letter: None,
                        time_type: ctx.start_kind,
                        format: String::new(),
                    });
                }
                if let Some(temp) = temp {
                    let _ = rule_transitions.insert(temp);
                }
            }
            // Continue by determining the ending instant of the current rule, i64::MAX stands for x into infinite.
            let use_until_instant = entry
                .date
                .map(|dt| dt.as_precise_ut_time(&entry.std_offset, &savings))
                .unwrap_or(i64::MAX);

            for mut transition in rule_transitions {
                // If and only if the transition is less than the instant is it added to the output.
                let adjusted_transition_time = match transition.time_type {
                    QualifiedTimeKind::Universal => transition.at_time,
                    QualifiedTimeKind::Standard => transition.at_time + transition.offset,
                    QualifiedTimeKind::Local => {
                        transition.at_time + transition.offset + savings.as_secs()
                    }
                };
                if (ctx.use_start..=use_until_instant).contains(&transition.at_time)
                    && ctx.year_range.contains(&adjusted_transition_time)
                {
                    // Format handled here.
                    transition.format = entry.format.format(&transition);
                    output.insert(transition);
                }
            }

            // Update our local "global" values.
            ctx.update_for_zone_entry(entry, output.last(), savings);
        }
        output
    }
}

impl ZoneTable {
    #[allow(clippy::while_let_on_iterator)]
    pub fn parse_full_table(
        lines: &mut Peekable<Lines<'_>>,
        ctx: &mut LineParseContext,
    ) -> Result<(String, Self), ZoneInfoParseError> {
        ctx.enter("zone table");
        let mut table = Vec::default();
        ctx.line_number += 1;
        let header = lines.next().ok_or(ZoneInfoParseError::UnexpectedEndOfLine(
            ctx.line_number,
            ctx.span(),
        ))?;
        let (identifier, entry) = Self::parse_header_line(header, ctx)?;
        table.push(entry);
        while let Some(line) = lines.next() {
            let cleaned_line = remove_comments(line);
            if cleaned_line.trim().is_empty() {
                ctx.line_number += 1;
                continue;
            }
            let entry = ZoneEntry::try_from_str(cleaned_line, ctx)?;
            let last_row = entry.date.is_none();
            table.push(entry);
            ctx.line_number += 1;
            if last_row {
                break;
            }
        }

        ctx.exit();
        Ok((
            identifier,
            Self {
                table,
                associates: HashMap::default(),
            },
        ))
    }

    pub fn parse_header_line(
        header_line: &str,
        ctx: &mut LineParseContext,
    ) -> Result<(String, ZoneEntry), ZoneInfoParseError> {
        ctx.enter("zone header");
        let cleaned = remove_comments(header_line);
        let mut splits = cleaned.split_ascii_whitespace();
        if splits.next() != Some("Zone") {
            return Err(ZoneInfoParseError::InvalidZoneHeader(ctx.line_number));
        }
        let identifier = splits
            .next()
            .ok_or(ZoneInfoParseError::MissingIdentifier(ctx.line_number))?;

        let zone_str = splits.collect::<Vec<&str>>().join(" \t");
        let entry = ZoneEntry::try_from_str(&zone_str, ctx)?;
        ctx.exit();
        Ok((identifier.to_owned(), entry))
    }
}

#[cfg(test)]
mod tests {
    use alloc::borrow::ToOwned;
    use alloc::collections::BTreeSet;
    use alloc::string::String;
    use hashbrown::HashMap;

    use crate::{
        parser::{LineParseContext, TryFromStr},
        rule::{Rule, RuleTable},
        types::{
            AbbreviationFormat, Date, DayOfMonth, Month, QualifiedTime, RuleIdentifier, Sign, Time,
            ToYear, UntilDateTime, WeekDay, ZoneEntry,
        },
        zone::ZoneBuildContext,
    };

    use super::ZoneTable;

    const CHICAGO: &str = r#"Zone America/Chicago	-5:50:36 -	LMT	1883 Nov 18 18:00u
                    -6:00	US	C%sT	1920
                    -6:00	Chicago	C%sT	1936 Mar  1  2:00
                    -5:00	-	EST	1936 Nov 15  2:00
                    -6:00	Chicago	C%sT	1942
                    -6:00	US	C%sT	1946
                    -6:00	Chicago	C%sT	1967
                    -6:00	US	C%sT"#;

    fn parse_chicago() -> (String, ZoneTable) {
        let mut lines = CHICAGO.lines().peekable();
        let mut ctx = LineParseContext::default();
        ZoneTable::parse_full_table(&mut lines, &mut ctx).unwrap()
    }

    #[test]
    fn chicago_table() {
        let (ident, table) = parse_chicago();
        assert_eq!(ident, "America/Chicago");
        let mut table_iter = table.into_iter();
        assert_eq!(
            table_iter.next(),
            Some(ZoneEntry {
                std_offset: Time {
                    sign: Sign::Negative,
                    hour: 5,
                    minute: 50,
                    second: 36,
                },
                rule: RuleIdentifier::None,
                format: AbbreviationFormat::String("LMT".to_owned()),
                date: Some(UntilDateTime {
                    date: Date {
                        year: 1883,
                        month: Month::Nov,
                        day: DayOfMonth::Day(18),
                    },
                    time: QualifiedTime::Universal(Time {
                        sign: Sign::Positive,
                        hour: 18,
                        minute: 0,
                        second: 0
                    })
                })
            })
        );
    }

    #[test]
    fn time_parse() {
        let time = "-5:50:36";
        let result = Time::try_from_str(time, &mut LineParseContext::default()).unwrap();
        assert_eq!(
            result,
            Time {
                sign: Sign::Negative,
                hour: 5,
                minute: 50,
                second: 36,
            }
        );
    }

    #[test]
    fn chicago_transition() {
        let mut rules = RuleTable::initialize(Rule {
            from: 1918,
            to: Some(ToYear::Year(1919)),
            in_month: Month::Mar,
            on_date: DayOfMonth::Last(WeekDay::Sun),
            at: QualifiedTime::Local(Time {
                sign: Sign::Positive,
                hour: 2,
                minute: 0,
                second: 0,
            }),
            save: Time {
                sign: Sign::Positive,
                hour: 1,
                minute: 0,
                second: 0,
            },
            letter: Some("D".to_owned()),
        });
        rules.extend(Rule {
            from: 1918,
            to: Some(ToYear::Year(1919)),
            in_month: Month::Oct,
            on_date: DayOfMonth::Last(WeekDay::Sun),
            at: QualifiedTime::Local(Time {
                sign: Sign::Positive,
                hour: 2,
                minute: 0,
                second: 0,
            }),
            save: Time {
                sign: Sign::Positive,
                hour: 0,
                minute: 0,
                second: 0,
            },
            letter: Some("S".to_owned()),
        });
        let mut rule_map = HashMap::new();
        rule_map.insert("US".to_owned(), rules);
        let (_, mut table) = parse_chicago();

        table.associate_rules(&rule_map);
        let mut build_context = ZoneBuildContext::default();
        build_context.update(1918);
        let transitions = table.calculate_transitions_for_year(1918, &mut build_context);
        let transition_times = transitions
            .iter()
            .map(|t| t.at_time)
            .collect::<BTreeSet<i64>>();
        assert_eq!(
            transition_times,
            BTreeSet::from_iter([-1633276800, -1615136400])
        );
    }
}
