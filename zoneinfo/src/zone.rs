//! Implementation of zone info's [`ZoneRecord`]

use core::{iter::Peekable, ops::Range, str::Lines};

use alloc::{borrow::ToOwned, collections::BTreeSet, string::String, vec::Vec};
use hashbrown::HashMap;

use crate::{
    compiler::{CompiledTransitions, LocalTimeRecord, Transition},
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
    pub(crate) use_start_year: i32,
    pub(crate) start_kind: QualifiedTimeKind,
    pub(crate) previous_offset: i64,
    pub(crate) previous_rule: RuleIdentifier,
    pub(crate) previous_letter: Option<String>,
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
            use_start_year: 0,
            start_kind: QualifiedTimeKind::Universal,
            previous_offset: 0,
            previous_rule: RuleIdentifier::None,
            previous_letter: None,
            previous_format: String::default(),
        }
    }
}

impl ZoneBuildContext {
    pub(crate) fn new_v2(first_zone_line: &ZoneEntry) -> Self {
        let (use_start, use_start_year) = first_zone_line
            .date
            .map(|dt| {
                (
                    dt.as_precise_ut_time(first_zone_line.std_offset.as_secs(), 0),
                    dt.date.year,
                )
            })
            .unwrap_or((i64::MIN, 0));
        Self {
            use_start, // NOTE: use_start would be the initial transition
            use_start_year,
            saving: Time::default(),
            previous_offset: first_zone_line.std_offset.as_secs(),
            previous_rule: RuleIdentifier::None,
            previous_format: first_zone_line.format.format(
                first_zone_line.std_offset.as_secs(),
                None,
                false,
            ),
            ..Default::default()
        }
    }

    /// Create a new zone build context with the initial local time record
    /// from prior to the first transition.
    pub(crate) fn new(lmt: &LocalTimeRecord) -> Self {
        // TODO: Need a better name, but this is the beginning of the active range.
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
        let (savings, format, letter) = last
            .map(|transition| {
                (
                    transition.savings,
                    zone.format.format(
                        zone.std_offset.as_secs(),
                        transition.letter.as_deref(),
                        transition.savings != Time::default(),
                    ),
                    transition.letter.clone(),
                )
            })
            .unwrap_or((
                Time::default(),
                zone.format.format(zone.std_offset.as_secs(), None, false),
                None,
            ));
        self.saving = savings;
        self.previous_offset = zone.std_offset.as_secs();
        self.previous_rule = zone.rule.clone();
        self.previous_letter = letter;
        self.previous_format = format;

        if let Some(use_until) = zone.date {
            self.start_kind = use_until.time.time_kind();
            self.use_start =
                use_until.as_precise_ut_time(zone.std_offset.as_secs(), savings.as_secs());
            self.use_start_year = use_until.date.year;
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

    pub(crate) fn update_previous_transition(
        &mut self,
        zone: &ZoneEntry,
        last: Option<&Transition>,
    ) {
        if let Some(transition) = last {
            self.saving = transition.savings;
            self.previous_offset = transition.offset;
            self.previous_rule = zone.rule.clone();
            self.previous_letter = transition.letter.clone();
            self.previous_format = transition.format.clone();
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

    /// Creates a `LocalTimeRecord` from a LMT `ZoneEntry`
    ///
    /// Note: Calling this on a non-LMT zone line is GIGO
    pub(crate) fn into_first_local_time_record(&self) -> LocalTimeRecord {
        LocalTimeRecord {
            offset: self.std_offset.as_secs(),
            // An assumption
            saving: Time::default(),
            letter: None,
            designation: self.format.format(self.std_offset.as_secs(), None, false),
        }
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
        lmt_entry.into_first_local_time_record()
    }

    pub(crate) fn get_first_until_date(&self) -> Option<&UntilDateTime> {
        self.entries[0].date.as_ref()
    }

    // TODO: Return struct CompiledZoneRecord
    pub(crate) fn compile(&self) -> CompiledTransitions {
        let mut zone_line_iter = self.entries.iter();
        let first = zone_line_iter
            .next()
            .expect("A well formed zone table must contain one line");
        let initial_record = first.into_first_local_time_record();
        let mut context = ZoneBuildContext::new_v2(first);
        let mut transitions = BTreeSet::default();
        // We iterate through the zone lines.
        while let Some(zone_line) = zone_line_iter.next() {
            // We iterate through the continuation lines. The final continuation
            // line will not have an UntilDateTime value.
            // Check if we are on a continutation line
            if let Some(until_date) = zone_line.date {
                // We need to compute a range of timestamps that are in
                // the range of use_start..until_datetime. Where use_start
                // is the first transition and until_datetime is the first
                // transition time for the next line.

                // First, compute the initial transition.
                //
                // NOTE: This will have differing behavior depending on pre-existing transitions
                //
                // Per `tz-how-to.hmtl`:
                //
                // > One wrinkle, not fully explained in zic.8.txt, is what happens when
                // > switching to a named rule. To what values should the SAVE and LETTER data be initialized?
                // >  - If at least one transition has happened, use the SAVE and LETTER
                // >    data from the most recent.
                // >  - If switching to a named rule before any transition has happened,
                // >    assume standard time (SAVE zero), and use the LETTER data from
                // >    the earliest transition with a SAVE of zero.
                let is_first_transition = transitions.len() == 0;
                // Add the initial_transition for this line.
                let transition =
                    self.handle_zone_line_transition(zone_line, is_first_transition, &mut context);

                // Check whether the transition would be a true change from the previous
                // transition.
                let is_different_rule = transition.offset != context.previous_offset
                    || transition.format != context.previous_format
                    || transition.dst != (context.saving != Time::default());
                if is_different_rule {
                    // Add transition and update the running state.
                    transitions.insert(transition);
                    context.update_previous_transition(zone_line, transitions.last());
                }

                // If the zone line is a steady state, i.e. it has no named Rule, we can
                // move on early.
                let RuleIdentifier::Named(rule_identifier) = &zone_line.rule else {
                    // We need to update our context with current information.
                    context.update_for_zone_entry(zone_line, transitions.last());
                    continue;
                };

                let rules = self
                    .associates
                    .get(rule_identifier)
                    .expect("rules must be associated prior to compilation");

                // We've calculated our first transition, so now we need to determine
                // the range of years we are operating in. Why years? Because a Rule's
                // active FROM and TO fields are stored in years (with the obvious
                // caveats).
                let zone_line_year_range = context.use_start_year..=until_date.date.year;
                for year in zone_line_year_range {
                    // Assumption: Rules are returned in historical order, i.e. oldest
                    // to youngest. With this assumption, we assume that processing the
                    // rules in order should return a valid set of transitions that do
                    // not need to be filtered beyond checking it is within the bounds
                    // of the UNTIL datetime.
                    let mut rules_for_year = rules.rules_for_year(year);

                    // Sort the rules by their transition time in that year.
                    //
                    // We are simply doing a rough calculation of the datetime
                    // seconds and comparing them.
                    rules_for_year.sort_by(|r1, r2| {
                        let r1_time = r1.transition_time_for_year(
                            year,
                            &zone_line.std_offset,
                            &Time::default(),
                        );
                        let r2_time = r2.transition_time_for_year(
                            year,
                            &zone_line.std_offset,
                            &Time::default(),
                        );
                        r1_time.cmp(&r2_time)
                    });

                    // We need to sort our rules for the time they would take
                    // place in the year.

                    // NOTE: we can consume this vec as it is a clone.
                    for rule in rules_for_year {
                        let transition_time = rule.transition_time_for_year(
                            year,
                            &zone_line.std_offset,
                            &context.saving,
                        );
                        // We calculate the UNTIL date time seconds with the active savings value
                        let contextual_until_dt_secs = until_date.as_precise_ut_time(
                            zone_line.std_offset.as_secs(),
                            context.saving.as_secs(),
                        );
                        // If the transition time is in a valid range with the contextual_until_dt_secs,
                        // then we have a valid transition.
                        let offset = zone_line.std_offset.as_secs() + rule.save.as_secs();
                        let format =
                            zone_line
                                .format
                                .format(offset, rule.letter.as_deref(), rule.is_dst());
                        // Check with the transition time is in a valid range.
                        let within_range = (context.use_start..contextual_until_dt_secs)
                            .contains(&transition_time);
                        // Check whether the transition values consitute an transition, i.e.
                        // whether the local time record is different.
                        let is_different_rule = offset != context.previous_offset
                            || format != context.previous_format
                            || rule.is_dst() != (context.saving != Time::default());

                        if within_range && is_different_rule {
                            let transition = Transition {
                                at_time: transition_time,
                                offset,
                                dst: rule.is_dst(),
                                savings: rule.save,
                                letter: rule.letter,
                                time_type: rule.at.time_kind(),
                                format,
                            };
                            transitions.insert(transition);
                            context.update_previous_transition(zone_line, transitions.last());
                        }
                    }
                }

                // We've reached the end of our year range, so we need to update our state
                // and find our final use_start.
                context.use_start = until_date
                    .as_precise_ut_time(zone_line.std_offset.as_secs(), context.saving.as_secs());
                context.use_start_year = until_date.date.year;
                context.start_kind = until_date.time.time_kind();
            } else {
                // We have entered into a continuation line that does not have
                // an UNTIL datetime
                //
                // There are two primary tasks:
                //
                // 1. Compute the final transition from the previous UNTIL datetime.
                // 2. Compute any transitions up until the last Rule change
                let transition = self.handle_zone_line_transition(zone_line, false, &mut context);
                // Check whether the transition would be a true change from the previous
                // transition.
                let is_different_rule = transition.offset != context.previous_offset
                    || transition.format != context.previous_format
                    || transition.dst != (context.saving != Time::default());

                if is_different_rule {
                    transitions.insert(transition);
                    context.update_previous_transition(zone_line, transitions.last());
                }

                // If the zone line is a steady state, i.e. it has no named Rule, we can
                // move on early.
                let RuleIdentifier::Named(rule_identifier) = &zone_line.rule else {
                    // We need to update our context with current information.
                    context.update_for_zone_entry(zone_line, transitions.last());
                    continue;
                };

                // Get the rules being used.
                //
                // NOTE: This will panic if the zonetable has not been associated with
                // its rules.
                let rules = self
                    .associates
                    .get(rule_identifier)
                    .expect("rules must be associated prior to compilation");

                // Find the last applicable rules. That represents the final POSIX time zone
                let last_rules = rules.get_last_rules();

                // Try to find the largest maximum FROM year. This will be the base for
                // which transitions should be precomputed.
                let final_year = last_rules
                    .standard
                    .from
                    .max(last_rules.saving.map(|r| r.from).unwrap_or(0))
                    as i32;
                let zone_line_year_range = context.use_start_year..=final_year;

                for year in zone_line_year_range {
                    // Assumption: Rules are returned in historical order, i.e. oldest
                    // to youngest. With this assumption, we assume that processing the
                    // rules in order should return a valid set of transitions that do
                    // not need to be filtered beyond checking it is within the bounds
                    // of the UNTIL datetime.
                    let mut rules_for_year = rules.rules_for_year(year);

                    rules_for_year.sort_by(|r1, r2| {
                        let r1_time = r1.transition_time_for_year(
                            year,
                            &zone_line.std_offset,
                            &Time::default(),
                        );
                        let r2_time = r2.transition_time_for_year(
                            year,
                            &zone_line.std_offset,
                            &Time::default(),
                        );
                        r1_time.cmp(&r2_time)
                    });

                    // NOTE: we can consume this vec as it is a clone.
                    for rule in rules_for_year {
                        let transition_time = rule.transition_time_for_year(
                            year,
                            &zone_line.std_offset,
                            &context.saving,
                        );
                        let offset = zone_line.std_offset.as_secs() + rule.save.as_secs();
                        let format =
                            zone_line
                                .format
                                .format(offset, rule.letter.as_deref(), rule.is_dst());
                        let within_range = (context.use_start..i64::MAX).contains(&transition_time);
                        let is_different_rule =
                            offset != context.previous_offset || format != context.previous_format;
                        if within_range && is_different_rule {
                            let offset = zone_line.std_offset.as_secs() + rule.save.as_secs();
                            let format = zone_line.format.format(
                                offset,
                                rule.letter.as_deref(),
                                rule.is_dst(),
                            );
                            let transition = Transition {
                                at_time: transition_time,
                                offset,
                                dst: rule.is_dst(),
                                savings: rule.save,
                                letter: rule.letter,
                                time_type: rule.at.time_kind(),
                                format,
                            };
                            transitions.insert(transition);
                            context.update_previous_transition(zone_line, transitions.last());
                        }
                    }
                }
            }
        }
        let posix_time_zone = self.get_posix_time_zone();
        CompiledTransitions {
            initial_record,
            transitions,
            posix_time_zone,
        }
    }

    pub(crate) fn handle_zone_line_transition(
        &self,
        zone_line: &ZoneEntry,
        is_first_transition: bool,
        context: &mut ZoneBuildContext,
    ) -> Transition {
        match &zone_line.rule {
            // If the zone line has no identifier, then it is a standard
            // transition. Return the transition value
            RuleIdentifier::None => {
                let offset = zone_line.std_offset.as_secs();
                Transition {
                    at_time: context.use_start,
                    offset,
                    dst: false,
                    savings: Time::default(),
                    letter: None,
                    time_type: context.start_kind,
                    format: zone_line.format.format(offset, None, false),
                }
            }
            // If the zone line has a numeric identifier, then it is a savings
            // transition. Return the transition
            RuleIdentifier::Numeric(t) => {
                let offset = zone_line.std_offset.as_secs() + t.as_secs();
                Transition {
                    at_time: context.use_start,
                    offset,
                    dst: true,
                    savings: *t,
                    letter: None,
                    time_type: context.start_kind,
                    format: zone_line.format.format(offset, None, true),
                }
            }
            // The Rule is named, so we need to resolve the rule.
            RuleIdentifier::Named(identifier) => {
                let rules = self
                    .associates
                    .get(identifier)
                    .expect("rules were not associated.");
                // This is not a first transition so we need to know what the active rule
                // is when the transition occurs.
                let mut rules_for_year = rules.rules_for_year(context.use_start_year);
                // Sort rules to be properly in order for the year based on the approximate
                // transition time in that year.
                rules_for_year.sort_by(|r1, r2| {
                    let r1_time = r1.transition_time_for_year(
                        context.use_start_year,
                        &zone_line.std_offset,
                        &Time::default(),
                    );
                    let r2_time = r2.transition_time_for_year(
                        context.use_start_year,
                        &zone_line.std_offset,
                        &Time::default(),
                    );
                    r1_time.cmp(&r2_time)
                });
                // Determine whether start time is before or after any of the rule
                // transitions to set a baseline.
                let mut index = None;
                for (i, rule) in rules_for_year.iter().enumerate() {
                    // We make a guess for the savings value.
                    //
                    // We could use the context savings for the previous, but that
                    // would be applying a savings value from a different Rule to
                    // the current rule.
                    //
                    // The primary issue is that without backtracking to year - 1.
                    // The current savings value is unknown.
                    let savings = rules_for_year
                        .get(i.wrapping_sub(1))
                        .map_or(context.saving, |r| r.save);
                    let imprecise_transition_time = rule.transition_time_for_year(
                        context.use_start_year,
                        &zone_line.std_offset,
                        &savings,
                    );
                    if imprecise_transition_time <= context.use_start {
                        index = Some(i);
                    }
                }

                // TODO: Can this be removed in favor of searching for the last rule?
                // Set the transition based off whether there is an active zone or not.
                if let Some(active_rule) =
                    rules_for_year.get(index.unwrap_or(rules_for_year.len()))
                {
                    let offset = zone_line.std_offset.as_secs() + active_rule.save.as_secs();
                    let format = zone_line.format.format(
                        offset,
                        active_rule.letter.as_deref(),
                        active_rule.is_dst(),
                    );
                    Transition {
                        at_time: context.use_start,
                        offset,
                        dst: active_rule.is_dst(),
                        letter: active_rule.letter.clone(),
                        savings: active_rule.save,
                        format,
                        time_type: context.start_kind,
                    }
                } else {
                    // We have a transition that is occuring before any Rules have
                    // occurred in the year.
                    //
                    // The wording in how-to is slightly ambiguous, but has been quoted
                    // elsewhere in this code:
                    //
                    // > - If at least one transition has happened, use the SAVE
                    // > and LETTER data from the most recent.
                    // > - If switching to a named rule before any transition has happened,
                    // > assume standard time (SAVE zero), and use the LETTER data
                    // > from the earliest transition with a SAVE of zero.
                    //
                    // This is especially important in this scenario. If we are able to find
                    // a last active rule, then we use data from that rule. If not, we assume
                    // a SAVE of zero and search for an intial transition letter.
                    let (savings, letter, dst) =
                        if let Some(rule) = rules.search_last_active_rule(context.use_start) {
                            (rule.save, rule.letter.clone(), rule.is_dst())
                        } else {
                            (
                                Time::default(),
                                rules.find_initial_transition_letter(),
                                false,
                            )
                        };
                    let offset = zone_line.std_offset.as_secs() + savings.as_secs();
                    let format = zone_line.format.format(offset, letter.as_deref(), false);
                    Transition {
                        at_time: context.use_start,
                        offset,
                        dst,
                        letter: letter.clone(),
                        savings,
                        format,
                        time_type: context.start_kind,
                    }
                }
            }
        }
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
