//! Zone info compiler functionality
//!
//! This module contains the zone info compiler logic along
//! with output types.
//!

use core::ops::RangeInclusive;

use alloc::collections::BTreeSet;
use alloc::string::String;
use hashbrown::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct LocalTimeRecord {
    pub offset: i64,
    pub saving: Time,
    pub letter: Option<String>,
    pub designation: String, // AKA format / abbr
}

// TODO: improve `Transition` repr this type provides a lot of
// information by design, but the local time record data
// should be separated from the transition info with a clear
// separation.
//
// EX:
// pub struct Transition {
//     /// The time to transition at
//     pub at_time: i64,
//     /// The transition time kind.
//     pub time_type: QualifiedTimeKind,
//     /// LocalTimeRecord transitioned into
//     pub to_local: ZoneInfoLocalTimeRecord,
// }
//
/// The primary transition data.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Transition {
    /// The time to transition at
    ///
    /// This represents the time in Unix Epoch seconds
    /// at which a transition should occur.
    pub at_time: i64,
    /// The transition time kind.
    ///
    /// Whether the transition was specified in Local, Standard, or Universal time.
    pub time_type: QualifiedTimeKind,

    // TODO: Below are fields that should be split into a
    // currently non-existent LocalTime record.
    /// The offset of the transition.
    pub offset: i64,
    /// Whether the transition is a savings offset or not
    ///
    /// This flag corresponds to the `is_dst` flag
    pub dst: bool,
    /// The savings for the local time record
    ///
    /// This field represents the exact [`Time`] value
    /// used for savings.
    pub savings: Time,
    /// The letter designation for the local time record
    ///
    /// The LETTER designation used in the fully formatted
    /// abbreviation
    pub letter: Option<String>,
    /// The abbreviation format for the local time record.
    pub format: String,
}

impl Ord for Transition {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.at_time.cmp(&other.at_time)
    }
}

impl PartialOrd for Transition {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// `CompiledTransitions` is the complete compiled transition data
/// for one zone.
///
/// The compiled transition data contains an initial local time record, an ordered
/// set of transition data, and a POSIX time zone string.
///
/// In general, this struct offers the required data in a consummable format
/// for anyone who compiled zoneinfo data.
#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub struct CompiledTransitions {
    /// The initial local time record.
    ///
    /// This is used in the case where a time predates a transition time.
    pub initial_record: LocalTimeRecord,

    /// The full set of calculated time zone transitions
    pub transitions: BTreeSet<Transition>,

    /// The POSIX time zone string
    ///
    /// This string should be used to calculate the time zone beyond the last available transition.
    pub posix_time_zone: PosixTimeZone,
}

// NOTE: candidate for removal? Should this library offer TZif structs long term?
//
// I think I would prefer all of that live in the `tzif` crate, but that will
// be a process to update. So implement it here, and then upstream it?
impl CompiledTransitions {
    pub fn to_v2_data_block(&self) -> TzifBlockV2 {
        TzifBlockV2::from_transition_data(self)
    }
}

/// The `CompiledTransitionsMap` struct contains a mapping of zone identifiers (AKA IANA identifiers) to
/// the zone's `CompiledTransitions`
#[derive(Debug, Default)]
pub struct CompiledTransitionsMap {
    pub data: HashMap<String, CompiledTransitions>,
}

// ==== ZoneInfoCompiler build / compile methods ====

use crate::{
    posix::PosixTimeZone,
    rule::Rules,
    types::{QualifiedTimeKind, RuleIdentifier, Time},
    tzif::TzifBlockV2,
    zone::{self, ZoneBuildContext, ZoneEntry, ZoneRecord},
    ZoneInfoData,
};

/// The compiler for turning `ZoneInfoData` into `CompiledTransitionsData`
pub struct ZoneInfoCompiler {
    data: ZoneInfoData,
}

impl ZoneInfoCompiler {
    /// Create a new `ZoneInfoCompiler` instance with provided `ZoneInfoData`.
    pub fn new(data: ZoneInfoData) -> Self {
        Self { data }
    }

    /// Build transition data for a specific zone.
    pub fn build_zone(&mut self, target: &str) -> CompiledTransitions {
        if let Some(zone) = self.data.zones.get_mut(target) {
            zone.associate_rules(&self.data.rules);
        }
        self.build_zone_internal(target)
    }

    pub fn build(&mut self) -> CompiledTransitionsMap {
        // Associate the necessary rules with the ZoneTable
        self.associate();
        // TODO: Validate and resolve settings here.
        let mut zoneinfo = CompiledTransitionsMap::default();
        for identifier in self.data.zones.keys() {
            let transition_data = self.build_zone_internal(identifier);
            let _ = zoneinfo.data.insert(identifier.clone(), transition_data);
        }
        zoneinfo
    }

    pub(crate) fn build_zone_internal(&self, target: &str) -> CompiledTransitions {
        let zone_table = self
            .data
            .zones
            .get(target)
            .expect("Invalid identifier provided.");
        zone_table.compile()
    }

    /// Builds the `ZoneInfoTransitionData` for a provided zone identifier (AKA IANA identifier)
    ///
    /// NOTE: Make sure to associate first!
    /*
    pub(crate) fn build_for_zone(&self, target: &str) -> CompiledTransitions {
        let zone_table = self
            .data
            .zones
            .get(target)
            .expect("Invalid identifier provided.");
        let initial_record = zone_table.get_first_local_record();
        let mut transitions = BTreeSet::default();
        if let Some(until_date) = zone_table.get_first_until_date() {
            // Arbitrary end year, expose as option?
            // TODO: Handle max year better.
            let range = until_date.date.year..=2050;

            let mut build_context = ZoneBuildContext::new(&initial_record);
            for year in range {
                build_context.update(year, until_date);
                zone_table.calculate_transitions_for_year(
                    year,
                    &mut build_context,
                    &mut transitions,
                );
            }
        }

        let posix_time_zone = zone_table.get_posix_time_zone();

        // First entry must exist.
        let first_zone_line = &zone_table.entries[0];
        let range = first_zone_line.date.map(|date| {
            let savings_time = match &first_zone_line.rule {
                RuleIdentifier::None => 0,
                RuleIdentifier::Numeric(num) => num.as_secs(),
                // All time zones with a transition should begin with a LMT line
                RuleIdentifier::Named(name) => unreachable!("No zone begins with a Rule offset.")
                ,
            };
            let first_transition_time = date.as_precise_ut_time(first_zone_line.std_offset.as_secs(), savings_time);

            // We want the second to last zone line, because the last zone line will not have an until / end date.
            let second_to_last_zone_line = &zone_table.entries[zone_table.entries.len() - 2];
            let final_until_date = second_to_last_zone_line.date.expect("UNTIL must exist if not on last zone line");
            let savings_time = match &first_zone_line.rule {
                RuleIdentifier::None => 0,
                RuleIdentifier::Numeric(num) => num.as_secs(),
                // All time zones with a transition should begin with a LMT line
                RuleIdentifier::Named(name) => {
                    let associated_rules = zone_table.associates.get(name).expect("Rules must be associated");
                    let last_rules = associated_rules.get_last_rules();
                    if let Some(dst_rule) = last_rules.saving {
                        let _dst_transition_timestamp = dst_rule
                            .transition_time_for_year(final_until_date.date.year, &second_to_last_zone_line.std_offset, &Time::default());
                        let _std_transition_timestamp = last_rules
                            .standard
                            .transition_time_for_year(final_until_date.date.year, &second_to_last_zone_line.std_offset, &dst_rule.save);
                    } else {

                    }
                    todo!()
                }
            };
            let date = second_to_last_zone_line.date.expect("UntilDateTime must exist if multiple zone entries exists");
            let last_transition_time = date.as_precise_ut_time(second_to_last_zone_line.std_offset.as_secs(), savings_time);
            first_transition_time..=last_transition_time
        });

        CompiledTransitions {
            initial_record,
            transitions,
            posix_time_zone,
        }
    }
    */

    pub fn get_posix_time_zone(&mut self, target: &str) -> Option<PosixTimeZone> {
        self.associate();
        self.data
            .zones
            .get(target)
            .map(ZoneRecord::get_posix_time_zone)
    }

    /// Associates the current `ZoneTables` with their applicable rules.
    pub fn associate(&mut self) {
        for zones in self.data.zones.values_mut() {
            zones.associate_rules(&self.data.rules);
        }
    }
}

/// A compiler to compile all the transitions for a zone line.
pub struct ZoneLineCompiler;

impl ZoneLineCompiler {}
