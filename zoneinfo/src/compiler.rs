//! Zone info compiler functionality
//!
//! This module contains the zone info compiler logic along
//! with output types.
//!

use alloc::{borrow::ToOwned, string::String};

#[cfg(feature = "unstable")]
use crate::tzif::TzifBlockV2;
use crate::{
    posix::PosixTimeZone,
    transition::MergedTransitions,
    types::Time,
    zone::ZoneRecord,
    ZoneInfoData,
};

use hashbrown::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct LocalTimeRecord {
    pub offset: i64,
    pub saving: Time,
    pub letter: Option<String>,
    pub designation: String, // AKA format / abbr
}

/// Compiled transition data for one zone.
#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub struct CompiledTransitions {
    /// The initial local time record (used when a time predates all transitions).
    pub initial_record: LocalTimeRecord,
    /// Merged transitions satisfying the slim-format invariant.
    pub transitions: MergedTransitions,
    /// POSIX TZ string for times beyond the last explicit transition.
    pub posix_time_zone: PosixTimeZone,
}

// NOTE: candidate for removal? Should this library offer TZif structs long term?
//
// I think I would prefer all of that live in the `tzif` crate, but that will
// be a process to update. So implement it here, and then upstream it?
#[cfg(feature = "unstable")]
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

/// The compiler for turning `ZoneInfoData` into `CompiledTransitionsData`
pub struct ZoneInfoCompiler {
    data: ZoneInfoData,
}

impl ZoneInfoCompiler {
    /// Create a new `ZoneInfoCompiler` instance with provided `ZoneInfoData`.
    pub fn new(data: ZoneInfoData) -> Self {
        Self { data }
    }

    /// Build transition data for a specific zone, following Link records if
    /// the identifier is a renamed/aliased zone (e.g. `America/Godthab` →
    /// `America/Nuuk`).
    pub fn build_zone(&mut self, target: &str) -> CompiledTransitions {
        let resolved = self.resolve_link(target).to_owned();
        if let Some(zone) = self.data.zones.get_mut(&resolved) {
            zone.associate_rules(&self.data.rules);
        }
        self.build_zone_internal(&resolved)
    }

    /// Resolve a zone identifier through `Link` records transitively.
    /// Stops after 16 hops to guard against cycles.
    fn resolve_link<'a>(&'a self, target: &'a str) -> &'a str {
        let mut current = target;
        for _ in 0..16 {
            match self.data.links.get(current) {
                Some(next) => current = next.as_str(),
                None => return current,
            }
        }
        current
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

    /// The internal method for retrieving a zone table and compiling it.
    pub(crate) fn build_zone_internal(&self, target: &str) -> CompiledTransitions {
        let zone_table = self
            .data
            .zones
            .get(target)
            .expect("Invalid identifier provided.");
        zone_table.compile()
    }

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
