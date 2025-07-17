//! Implementation of zone info's [`ZoneRecord`]

use core::{iter::Peekable, ops::Range, str::Lines};

use alloc::{borrow::ToOwned, collections::BTreeSet, string::String, vec::Vec};
use hashbrown::HashMap;

use crate::{
    compiler::{LocalTimeRecord, Transition},
    epoch_seconds_for_year,
    parser::{
        next_split, remove_comments, ContextParse, LineParseContext, TryFromStr, ZoneInfoParseError,
    },
    posix::PosixTimeZone,
    rule::Rules,
    types::{AbbreviationFormat, QualifiedTimeKind, RuleIdentifier, Time, UntilDateTime},
};

/// The zone build context.
///
/// This struct is primarily used as an intermediary type that tracks
/// the state of a zone build across year boundaries.
#[derive(Debug, Clone)]
pub(crate) struct ZoneBuildContext {
    pub(crate) saving: Time,
    pub(crate) epoch_year: i64,
    /// Universal time
    pub(crate) year_seconds: i64,
    pub(crate) year_range: Range<i64>,
    /// Universal time
    pub(crate) use_start: i64,
    pub(crate) start_kind: QualifiedTimeKind,
    pub(crate) previous_offset: i64,
    pub(crate) previous_rule: RuleIdentifier,
    pub(crate) previous_format: String,
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
            previous_offset: 0,
            previous_rule: RuleIdentifier::None,
            previous_format: String::default(),
        }
    }
}

impl ZoneBuildContext {
    /// Create a new zone build context with the initial local time record
    /// from prior to the first transition.
    pub(crate) fn new(lmt: &LocalTimeRecord) -> Self {
        Self {
            saving: lmt.saving,
            previous_offset: lmt.offset,
            previous_rule: RuleIdentifier::None,
            previous_format: lmt.designation.clone(),
            ..Default::default()
        }
    }

    /// Update the current build context data with the current year and until DateTime.
    pub(crate) fn update(&mut self, year: i32, until: &UntilDateTime) {
        let use_start = until.as_precise_ut_time(self.previous_offset, self.saving.as_secs());
        // NOTE: May need to adjust for offset + savings.
        let year_seconds = epoch_seconds_for_year(year);
        let year_plus_one = epoch_seconds_for_year(year + 1);
        self.year_seconds = year_seconds;
        self.year_range = year_seconds..year_plus_one;
        self.epoch_year = year_seconds;
        self.use_start = use_start;
        self.start_kind = until.time.time_kind();
    }

    /// Update's the build context with the zone entry info and the last transition data.
    pub(crate) fn update_for_zone_entry(&mut self, zone: &ZoneEntry, last: Option<&Transition>) {
        let (savings, format) = last
            .map(|transition| {
                (
                    transition.savings,
                    zone.format.format(
                        zone.std_offset.as_secs(),
                        transition.letter.as_deref(),
                        transition.savings != Time::default(),
                    ),
                )
            })
            .unwrap_or((
                Time::default(),
                zone.format.format(zone.std_offset.as_secs(), None, false),
            ));
        self.saving = savings;
        self.previous_offset = zone.std_offset.as_secs();
        self.previous_rule = zone.rule.clone();
        self.previous_format = format;

        if let Some(use_until) = zone.date {
            self.start_kind = use_until.time.time_kind();
            self.use_start =
                use_until.as_precise_ut_time(zone.std_offset.as_secs(), savings.as_secs());
            self.year_seconds = match self.start_kind {
                QualifiedTimeKind::Universal => self.epoch_year,
                QualifiedTimeKind::Standard => self.epoch_year + zone.std_offset.as_secs(),
                // Uh, how to handle dst. Does it matter? This will prob blow up on southern hemisphere
                QualifiedTimeKind::Local => {
                    self.epoch_year + zone.std_offset.as_secs() + self.saving.as_secs()
                }
            };
        }
    }

    /// Check if the zone is beyond the year
    pub(crate) fn is_zone_beyond_year(&self, offset: i64) -> bool {
        self.year_seconds < self.use_start && !self.is_start_in_year_range(offset)
    }

    /// Checks if a zone entry is skippable.
    pub(crate) fn in_skippable_zone(&self, until_time: i64, offset: i64) -> bool {
        !(self.use_start..=until_time).contains(&self.year_seconds)
            && !self.is_start_in_year_range(offset)
    }

    /// Checks if the use start time is within the current year range.
    pub(crate) fn is_start_in_year_range(&self, offset: i64) -> bool {
        self.year_range
            .contains(&(self.use_start.saturating_add(offset)))
    }

    /// Checks if the zone entry was a named rule.
    pub(crate) fn zone_was_named_rule(&self) -> bool {
        matches!(self.previous_rule, RuleIdentifier::Named(_))
    }
}

/// `ZoneEntry` represents a single row in a `ZoneTable`
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
        matches!(self.rule, RuleIdentifier::Named(_))
    }
}

impl TryFromStr<LineParseContext> for ZoneEntry {
    type Error = ZoneInfoParseError;
    fn try_from_str(s: &str, ctx: &mut LineParseContext) -> Result<Self, Self::Error> {
        ctx.enter("ZoneEntry");
        let mut splits = s.split_whitespace();
        let std_offset = splits
            .next()
            .ok_or(ZoneInfoParseError::unexpected_eol(ctx))?
            .context_parse::<Time>(ctx)?;
        let rule = next_split(&mut splits, ctx)?.context_parse::<RuleIdentifier>(ctx)?;
        let format = splits
            .next()
            .ok_or(ZoneInfoParseError::unexpected_eol(ctx))?
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

// TODO: Potentially remove the first record from the
// table. The first record is compiled separately
// anyways, so that would clean that up.
/// The `ZoneRecord` represents the zoneinfo files' Zone record.
///
/// A ZoneRecord is made up of a single record, with zero or
/// more continuation lines.
///
/// # Example
///
/// The `America/Chicago` zone record
///
/// ```txt
/// # Zone    NAME        STDOFF    RULES    FORMAT    [UNTIL]
/// Zone America/Chicago    -5:50:36 -    LMT    1883 Nov 18 18:00u
///             -6:00    US    C%sT    1920
///             -6:00    Chicago    C%sT    1936 Mar  1  2:00
///             -5:00    -    EST    1936 Nov 15  2:00
///             -6:00    Chicago    C%sT    1942
///             -6:00    US    C%sT    1946
///             -6:00    Chicago    C%sT    1967
///             -6:00    US    C%sT
/// ```
///
#[derive(Debug, Clone, Default)]
pub struct ZoneRecord {
    /// The zone entries of the `ZoneRecord`
    pub entries: Vec<ZoneEntry>,
    /// Any associated rules for the zone table.
    pub associates: HashMap<String, Rules>,
}

impl IntoIterator for ZoneRecord {
    type Item = ZoneEntry;
    type IntoIter = alloc::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl ZoneRecord {
    pub fn get_posix_time_zone(&self) -> PosixTimeZone {
        let entry = self
            .entries
            .last()
            .expect("At least one entry should exist.");
        match &entry.rule {
            RuleIdentifier::None => PosixTimeZone::from_zone_and_savings(entry, Time::default()),
            RuleIdentifier::Numeric(t) => PosixTimeZone::from_zone_and_savings(entry, *t),
            RuleIdentifier::Named(id) => {
                let rules_table = self.associates.get(id).expect("rules must be associated");
                let last_rules = rules_table.get_last_rules();
                PosixTimeZone::from_zone_and_rules(entry, &last_rules)
            }
        }
    }

    /// Associate the current `ZoneTable` with rules
    pub fn associate_rules(&mut self, rules: &HashMap<String, Rules>) {
        if self.associates.is_empty() {
            for entry in &mut self.entries {
                if let RuleIdentifier::Named(associate_rule) = &entry.rule {
                    if self.associates.contains_key(associate_rule) {
                        continue;
                    }
                    if let Some(rules) = rules.get(associate_rule).cloned() {
                        let _ = self.associates.insert(associate_rule.clone(), rules);
                    }
                }
            }
        }
    }

    /// Get the first transition time for this zone table.
    ///
    /// No transition will be lower than this.
    pub(crate) fn get_first_local_record(&self) -> LocalTimeRecord {
        let lmt_entry = &self.entries[0];
        LocalTimeRecord {
            offset: lmt_entry.std_offset.as_secs(),
            // An assumption
            saving: Time::default(),
            letter: None,
            designation: lmt_entry
                .format
                .format(lmt_entry.std_offset.as_secs(), None, false),
        }
    }

    pub(crate) fn get_first_until_date(&self) -> Option<&UntilDateTime> {
        self.entries[0].date.as_ref()
    }

    // TODO: the clarity of this could probably be further improved by using
    // some sort of local time record in `Transition`
    /// Calculates the transitions for the provided year with the given context.
    ///
    /// For more information, see source code comments.
    pub(crate) fn calculate_transitions_for_year(
        &self,
        year: i32,
        ctx: &mut ZoneBuildContext,
        output: &mut BTreeSet<Transition>,
    ) {
        // NOTES: We need to be careful here, zones until time may
        // be at the start of the year but could be mid year or
        // multiple times in a year (EX: America/Chicago)

        // Year seconds should be Jan 1 for year.
        // By default, the zone is the last zone set
        for entry in &self.entries {
            if entry == &self.entries[0] {
                continue;
            }

            // Calculate the UntilTime with the previous zones inputs.
            let until_time_or_max = entry
                .date
                .map(|d| d.as_precise_ut_time(ctx.previous_offset, ctx.saving.as_secs()))
                .unwrap_or(i64::MAX);
            // Exit looping entries once year exceeds the until time.
            if ctx.is_zone_beyond_year(entry.std_offset.as_secs()) {
                break;
            }
            // if the year is not within the start_time to use_until range
            //   and start time is not in this years full range, skip rule.
            if ctx.in_skippable_zone(until_time_or_max, entry.std_offset.as_secs()) {
                // Update the zone entry context
                ctx.update_for_zone_entry(entry, output.last());
                continue;
            }
            // We've determined that are year is viable for this zone entry.
            // Let's move foward

            let mut rule_transitions = BTreeSet::default();
            let savings = match &entry.rule {
                RuleIdentifier::None => {
                    // Transitions only occur if the offsets are different or we are at the first zone
                    let same_offset = ctx.previous_offset + ctx.saving.as_secs()
                        == entry.std_offset.as_secs()
                        && ctx.previous_format
                            == entry.format.format(entry.std_offset.as_secs(), None, false);
                    if same_offset && ctx.saving.as_secs() == 0 {
                        ctx.update_for_zone_entry(entry, output.last());
                        continue;
                    }
                    let at_time = ctx.use_start - ctx.saving.as_secs();
                    let time_type = ctx.start_kind;
                    rule_transitions.insert(Transition {
                        at_time,
                        offset: entry.std_offset.as_secs(),
                        dst: false,
                        savings: Time::default(),
                        letter: None,
                        time_type,
                        format: String::new(),
                    });
                    Time::default() // No savings on an empty rule, return 0 savings
                }
                RuleIdentifier::Numeric(t) => {
                    // Transitions only occur if the offsets are different
                    let same_offset = ctx.previous_offset + ctx.saving.as_secs()
                        == entry.std_offset.as_secs() + t.as_secs()
                        && ctx.previous_format
                            == entry.format.format(entry.std_offset.as_secs(), None, true);
                    if same_offset {
                        ctx.update_for_zone_entry(entry, output.last());
                        continue;
                    }
                    let at_time = ctx.use_start - ctx.saving.as_secs();
                    let time_type = ctx.start_kind;
                    rule_transitions.insert(Transition {
                        at_time,
                        offset: entry.std_offset.as_secs() + t.as_secs(),
                        dst: true,
                        savings: *t,
                        letter: None,
                        time_type,
                        format: String::new(),
                    });
                    *t
                }
                RuleIdentifier::Named(s) => {
                    let rules = self.associates.get(s).expect("rules were not associated.");
                    let applicable_rules =
                        rules.get_rules_for_year(year, &entry.std_offset, until_time_or_max, ctx);
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
            if ctx.is_start_in_year_range(entry.std_offset.as_secs()) {
                // Have to keep in mind the various states that we can be
                // in at this moment.
                //
                // zone considerations:
                // Due to using `use_start`, the previous zone rule comes
                // into play, primarily with non named rules (Numeric or
                // None rules).
                //
                // rule_transitions:
                //   - 0 (there were no rules that could be found).
                //   - 1 (there is a one off zone or implied non DST rule)
                //   - 2 (multiple viable transitions available)
                //
                let mut temp = None;

                // Figuring out savings tends to be a bit more complex, then
                // may be preferred.
                let different_offsets = ctx.previous_offset != entry.std_offset.as_secs();

                // Determine the type of zone pair that we are dealing
                // with. We care about both being named rules, primarily
                // for the cases where one is not a named zone.
                let both_named_rules = ctx.zone_was_named_rule() && entry.is_named_rule();

                // Further checks on pairs with at least one non named zone
                // Have the offsets or savings changed between the two? If
                // not, then there's no transition to worry about.
                let non_named_rule_zones_are_different = !both_named_rules && different_offsets;

                // NOTE: Potentially need to go to a reverse and
                // Cycle through our rule transitions, and find out if there are any
                // transitions that `use_start` may supercede. In order to do this,
                // we start with previous savings value and update the value with the
                // transition's savings while iterating.
                for transition in &rule_transitions {
                    // Lord Howe has a silent transition from Rule
                    // `LH` to `LH` where the savings changes from
                    // `1:00` to `0:30`. Why is it there? Idk, but
                    // we ignore such cases in favor of rule outcomes
                    //
                    // Meanwhile, Paris has a non-silent transition from
                    // France with offset 00:00 to France with offset 1:00
                    //
                    // NOTE: It may be worthwhile to add format as a column
                    // here to confirm.
                    let same_rule = ctx.previous_rule == entry.rule
                        && ctx.previous_offset == transition.offset
                        && ctx.previous_format
                            == entry.format.format(
                                transition.offset,
                                transition.letter.as_deref(),
                                transition.dst,
                            );

                    if transition.at_time < ctx.use_start
                        && (!same_rule || non_named_rule_zones_are_different)
                    {
                        let mut transition_clone = transition.clone();
                        transition_clone.at_time = ctx.use_start;
                        let _ = temp.insert(transition_clone);
                    } else if temp.is_some() && transition.at_time < ctx.use_start {
                        // Invalidate the previous cloned transition
                        let _ = temp.take();
                    }
                }
                let different_offset_vals = ctx.previous_offset + ctx.saving.as_secs()
                    != entry.std_offset.as_secs() + savings.as_secs();

                // If transitions is <= 1 at this point (and did
                // not meet the different_rules check), that means
                // `use_start` is less than the existing transition
                // and at least one of the transitions is a Numeric
                // or None zone. Due to `use_start`, being less than
                // the transition, we should be dealing with (None, Name)
                // or (Numeric, Name) zone pairs. So check if the zones
                // are different and need a transition.
                let transition_is_valid = match rule_transitions.last() {
                    Some(_) if rule_transitions.len() == 1 => {
                        !both_named_rules && ctx.previous_offset != entry.std_offset.as_secs()
                    }
                    Some(t) => {
                        // The major case here is the shift for Antarctica/Troll
                        // from using a format of -00 => +00. We are arguably greedy
                        // here by assuming the EOY rule is the same that would be
                        // the start of the same year. This should hold true except
                        // for triple rule years.
                        ctx.use_start < rule_transitions.first().expect("must exist").at_time
                            && (ctx.previous_offset != entry.std_offset.as_secs()
                                || ctx.previous_format
                                    != entry.format.format(
                                        entry.std_offset.as_secs(),
                                        t.letter.as_deref(),
                                        t.dst,
                                    ))
                    }
                    // First we check if there is no valid rule transitions
                    // and the rules are not the same, which would mean
                    // `use_start` is the transition.
                    None => different_offset_vals || !entry.is_named_rule(),
                };

                if transition_is_valid {
                    let (offset, savings) = if let RuleIdentifier::Named(rule) = &entry.rule {
                        // NOTE: See Riga 1941 for an example
                        let rule = self.associates.get(rule).expect("rule must be associated.");
                        let savings = rule.search_last_active_savings(ctx.use_start);
                        (entry.std_offset.as_secs() + savings.as_secs(), savings)
                    } else {
                        (entry.std_offset.as_secs(), savings)
                    };
                    // Set DST based off savings
                    let dst = savings != Time::default();
                    let _ = temp.insert(Transition {
                        at_time: ctx.use_start,
                        offset,
                        dst,
                        savings,
                        letter: None,
                        time_type: ctx.start_kind,
                        format: String::new(),
                    });
                }
                if let Some(temp) = temp {
                    let _ = rule_transitions.insert(temp);
                }
            }

            // TODO (potentially): use i32::MAX over i64::MAX?
            // Continue by determining the ending instant of the current rule, i64::MAX stands for x into infinite.
            let mut active_savings = ctx.saving;
            for mut transition in rule_transitions {
                let use_until_instant = entry
                    .date
                    .map(|dt| {
                        dt.as_precise_ut_time(entry.std_offset.as_secs(), active_savings.as_secs())
                    })
                    .unwrap_or(i64::MAX);

                // If and only if the transition is less than the instant is it added to the output.
                // let adjusted_transition_time = adjust_time_to_local(transition.time_type, transition.at_time, transition.offset, savings.as_secs());
                let adjusted_transition_time = match transition.time_type {
                    QualifiedTimeKind::Universal => transition.at_time,
                    QualifiedTimeKind::Standard => transition.at_time + transition.offset,
                    QualifiedTimeKind::Local => {
                        transition.at_time + transition.offset + active_savings.as_secs()
                    }
                };
                if (ctx.use_start..use_until_instant).contains(&transition.at_time)
                    && ctx.year_range.contains(&adjusted_transition_time)
                {
                    // Format handled here.
                    active_savings = transition.savings;
                    transition.format = entry.format.format_with_transition(&transition);
                    output.insert(transition);
                }
            }

            // Update our local "global" values.
            ctx.update_for_zone_entry(entry, output.last());
        }
    }
}

impl ZoneRecord {
    /// Parses a `ZoneTable` starting from the provided Zone line and
    /// ending on the final continuation line.
    pub fn parse_full_table(
        lines: &mut Peekable<Lines<'_>>,
        ctx: &mut LineParseContext,
    ) -> Result<(String, Self), ZoneInfoParseError> {
        ctx.enter("zone table");
        let mut table = Vec::default();
        ctx.line_number += 1;
        let header = lines
            .next()
            .ok_or(ZoneInfoParseError::unexpected_eol(ctx))?;
        let (identifier, entry) = Self::parse_header_line(header, ctx)?;
        let has_continuation_lines = entry.date.is_some();
        table.push(entry);
        if has_continuation_lines {
            #[allow(clippy::while_let_on_iterator)]
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
        }

        ctx.exit();
        Ok((
            identifier,
            Self {
                entries: table,
                associates: HashMap::default(),
            },
        ))
    }

    /// Parse a header line, i.e. the first zone record line.
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
        rule::{Rule, Rules},
        types::{
            AbbreviationFormat, Date, DayOfMonth, Month, QualifiedTime, RuleIdentifier, Sign, Time,
            ToYear, UntilDateTime, WeekDay,
        },
        zone::ZoneBuildContext,
    };

    use super::{ZoneEntry, ZoneRecord};

    const CHICAGO: &str = r#"Zone America/Chicago	-5:50:36 -	LMT	1883 Nov 18 18:00u
                    -6:00	US	C%sT	1920
                    -6:00	Chicago	C%sT	1936 Mar  1  2:00
                    -5:00	-	EST	1936 Nov 15  2:00
                    -6:00	Chicago	C%sT	1942
                    -6:00	US	C%sT	1946
                    -6:00	Chicago	C%sT	1967
                    -6:00	US	C%sT"#;

    fn parse_chicago() -> (String, ZoneRecord) {
        let mut lines = CHICAGO.lines().peekable();
        let mut ctx = LineParseContext::default();
        ZoneRecord::parse_full_table(&mut lines, &mut ctx).unwrap()
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
        let mut rules = Rules::initialize(Rule {
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
        build_context.update(
            1918,
            table
                .get_first_until_date()
                .expect("first date exists for America/Chicago"),
        );
        let mut transitions = BTreeSet::default();
        table.calculate_transitions_for_year(1918, &mut build_context, &mut transitions);
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
