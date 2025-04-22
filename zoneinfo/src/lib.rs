//! A library for parsing and compiling zoneinfo files into
//! time zone transition data that can be used to build
//! TZif files or any other desired time zone format.
//!
//! The `zoneinfo_compiler`* offers default parsing and compiling
//! of zoneinfo files into time zone transition data.
//!
//! Why `zoneinfo-compiler`?
//!
//! In general, this library seeks to maximally expose as much
//! data from the zoneinfo files as possible while also supporting
//! extra time zone database features like the zone.tab, PACKRATLIST,
//! and POSIX time zone strings.
//!
//! * TODO: bikeshed name

// TODO list:
//
//  - Support PACKRATLIST
//  - Support zone.tab
//  - Support leap second
//  - Support vanguard and rear guard parsing (potential backlog)
//  - Provide easy defaults for SLIM and FAT compiling.
//  - Support v1 TZif with conversion to i32.
//

// Implementation note: this library is NOT designed to be the most
// optimal speed. Instead invariance and clarity is preferred where
// need be.
//
// We can get away with any performance penalty primarily because
// this library is designed to aid with build time libraries, on
// a limited dataset, NOT at runtime on extremely large datasets.

#![no_std]

extern crate alloc;

use alloc::{collections::BTreeSet, string::String};
use parser::{ZoneInfoParseError, ZoneInfoParser};
use types::{Time, Transition};
use tzif::TzifBlockV2;
use utils::epoch_seconds_for_year;

use hashbrown::HashMap;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::{io, path::Path};

pub(crate) mod utils;

pub mod parser;
pub mod rule;
pub mod types;
pub mod tzif;
pub mod zone;

use rule::RuleTable;
use zone::{ZoneBuildContext, ZoneTable};

/// Well-known zone info file
pub const ZONEINFO_FILES: [&str; 9] = [
    "africa",
    "antarctica",
    "asia",
    "australasia",
    "backward",
    "etcetera",
    "europe",
    "northamerica",
    "southamerica",
];

/// The general error type for `ZoneInfo` operations
#[derive(Debug)]
pub enum ZoneInfoError {
    Parse(ZoneInfoParseError),
    #[cfg(feature = "std")]
    Io(io::Error),
}

#[cfg(feature = "std")]
impl From<io::Error> for ZoneInfoError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneInfoLocalTimeRecord {
    pub offset: i64,
    pub saving: Time,
    pub letter: Option<String>,
    pub designation: String, // AKA format
}

/// The `ZoneInfoTransitionData` is the complete compiled transition data
/// for one zone.
///
/// The transition data contains an initial local time record, an ordered
/// set of transition data, and a POSIX time zone string.
///
/// In general, this struct offers the required data in a consummable format
/// for anyone who compiled zoneinfo data.
#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub struct ZoneInfoTransitionData {
    /// The initial local time record.
    ///
    /// This is used in the case where a time predates a transition time.
    pub lmt: ZoneInfoLocalTimeRecord,
    /// The full set of calculated time zone transitions
    pub transitions: BTreeSet<Transition>,
    /// The POSIX time zone string
    ///
    /// This string should be used to calculate the time zone beyond the last available transition.
    pub posix_string: String, // TODO: Implement POSIX string building
}

// NOTE: candidate for removal? Should this library offer TZif structs long term?
//
// I think I would prefer all of that live in the `tzif` crate, but that will
// be a process to update. So implement it here, and then upstream it?
impl ZoneInfoTransitionData {
    pub fn to_v2_data_block(&self) -> TzifBlockV2 {
        TzifBlockV2::from_transition_data(self)
    }
}

/// The `ZoneInfo` contains a mapping of zone indentifiers (AKA IANA identifiers) to
/// the zone's transition data.
#[derive(Debug, Default)]
pub struct ZoneInfo {
    pub data: HashMap<String, ZoneInfoTransitionData>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct ZoneInfoCompiler {
    pub rules: HashMap<String, RuleTable>,
    pub zones: HashMap<String, ZoneTable>,
    pub links: HashMap<String, String>,
    pub pack_rat: HashMap<String, String>,
}

// ==== ZoneInfoCompiler parsing methods ====

impl ZoneInfoCompiler {
    /// Parse data from a path to a directory of zoneinfo files, using well known
    /// zoneinfo file names.
    ///
    /// This is usually pointed to a "tzdata" directory.
    #[cfg(feature = "std")]
    pub fn from_zoneinfo_directory<P: AsRef<Path>>(dir: P) -> Result<Self, ZoneInfoError> {
        let mut zoneinfo = Self::default();
        for filename in ZONEINFO_FILES {
            let file_path = dir.as_ref().join(filename);
            let parsed = Self::from_filepath(file_path)?;
            zoneinfo.extend(parsed);
        }
        Ok(zoneinfo)
    }

    /// Parse data from a filepath to a zoneinfo file.
    #[cfg(feature = "std")]
    pub fn from_filepath<P: AsRef<Path> + core::fmt::Debug>(
        path: P,
    ) -> Result<Self, ZoneInfoError> {
        Self::from_zoneinfo_file(&std::fs::read_to_string(path)?)
    }

    /// Parses data from a zoneinfo file as a string slice.
    pub fn from_zoneinfo_file(src: &str) -> Result<Self, ZoneInfoError> {
        ZoneInfoParser::from_zoneinfo_str(src)
            .parse()
            .map_err(ZoneInfoError::Parse)
    }

    /// Extend the current `ZoneInfoCompiler` data from another `ZoneInfoCompiler`.
    pub fn extend(&mut self, other: Self) {
        self.rules.extend(other.rules);
        self.zones.extend(other.zones);
        self.links.extend(other.links);
        self.pack_rat.extend(other.pack_rat);
    }
}

// ==== ZoneInfoCompiler build / compile methods ====

impl ZoneInfoCompiler {
    /// Build transition data for a specific zone.
    pub fn build_zone(&mut self, target: &str) -> ZoneInfoTransitionData {
        if let Some(zone) = self.zones.get_mut(target) {
            zone.associate_rules(&self.rules);
        }
        self.build_for_zone(target)
    }

    pub fn build(&mut self) -> ZoneInfo {
        // Associate the necessary rules with the ZoneTable
        self.associate();
        // TODO: Validate and resolve settings here.
        let mut zoneinfo = ZoneInfo::default();
        for identifier in self.zones.keys() {
            let transition_data = self.build_for_zone(identifier);
            let _ = zoneinfo.data.insert(identifier.clone(), transition_data);
        }
        zoneinfo
    }

    /// Builds the `ZoneInfoTransitionData` for a provided zone identifier (AKA IANA identifier)
    ///
    /// NOTE: Make sure to associate first!
    pub(crate) fn build_for_zone(&self, target: &str) -> ZoneInfoTransitionData {
        let zone_table = self
            .zones
            .get(target)
            .expect("Invalid identifier provided.");
        let lmt = zone_table.get_first_local_record();
        let mut transitions = BTreeSet::default();
        if let Some(until_date) = zone_table.get_first_until_date() {
            // TODO: Handle max year better.
            let range = until_date.date.year..=2037;

            let mut build_context = ZoneBuildContext::new(&lmt);
            for year in range {
                build_context.update(year, until_date);
                zone_table.calculate_transitions_for_year(
                    year,
                    &mut build_context,
                    &mut transitions,
                );
            }
        }

        // TODO: POSIX tz string handling

        ZoneInfoTransitionData {
            lmt,
            transitions,
            posix_string: String::default(),
        }
    }

    /// Associates the current `ZoneTables` with their applicable rules.
    pub fn associate(&mut self) {
        for zones in self.zones.values_mut() {
            zones.associate_rules(&self.rules);
        }
    }
}

#[cfg(test)]
#[cfg(all(feature = "std", not(target_os = "windows")))]
mod tests {
    use crate::ZoneInfoCompiler;
    use std::path::Path;

    #[test]
    fn test_chicago() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut zoneinfo =
            ZoneInfoCompiler::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();

        // Association is needed.
        let computed_zoneinfo = zoneinfo.build_zone("America/Chicago");

        let data = tzif::parse_tzif_file(Path::new("/usr/share/zoneinfo/America/Chicago")).unwrap();
        let data_block_v2 = data.data_block2.unwrap();
        let fs_transitions = data_block_v2.transition_times;

        for (computed, (idx, fs)) in computed_zoneinfo
            .transitions
            .iter()
            .zip(fs_transitions.iter().enumerate())
        {
            assert_eq!(computed.at_time, fs.0);
            let type_index = data_block_v2.transition_types[idx];
            assert_eq!(
                computed.offset,
                data_block_v2.local_time_type_records[type_index].utoff.0
            );
        }
    }

    #[test]
    fn test_new_york() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut zoneinfo =
            ZoneInfoCompiler::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();

        // Association is needed.
        let computed_zoneinfo = zoneinfo.build_zone("America/New_York");

        let data =
            tzif::parse_tzif_file(Path::new("/usr/share/zoneinfo/America/New_York")).unwrap();
        let data_block_v2 = data.data_block2.unwrap();
        let fs_transitions = data_block_v2.transition_times;

        for (computed, (idx, fs)) in computed_zoneinfo
            .transitions
            .iter()
            .zip(fs_transitions.iter().enumerate())
        {
            assert_eq!(computed.at_time, fs.0);
            let type_index = data_block_v2.transition_types[idx];
            assert_eq!(
                computed.offset,
                data_block_v2.local_time_type_records[type_index].utoff.0
            );
        }
    }

    #[test]
    fn test_anchorage() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut zoneinfo =
            ZoneInfoCompiler::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();

        // Association is needed.
        let computed_zoneinfo = zoneinfo.build_zone("America/Anchorage");

        let data =
            tzif::parse_tzif_file(Path::new("/usr/share/zoneinfo/America/Anchorage")).unwrap();
        let data_block_v2 = data.data_block2.unwrap();
        let fs_transitions = data_block_v2.transition_times;

        for (computed, (idx, fs)) in computed_zoneinfo
            .transitions
            .iter()
            .zip(fs_transitions.iter().enumerate())
        {
            assert_eq!(computed.at_time, fs.0);
            let type_index = data_block_v2.transition_types[idx];
            assert_eq!(
                computed.offset,
                data_block_v2.local_time_type_records[type_index].utoff.0
            );
        }
    }

    #[test]
    fn test_sydney() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut zoneinfo =
            ZoneInfoCompiler::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();
        // Association is needed.
        let computed_zoneinfo = zoneinfo.build_zone("Australia/Sydney");

        let data =
            tzif::parse_tzif_file(Path::new("/usr/share/zoneinfo/Australia/Sydney")).unwrap();
        let data_block_v2 = data.data_block2.unwrap();
        let fs_transitions = data_block_v2.transition_times;

        for (computed, (idx, fs)) in computed_zoneinfo
            .transitions
            .iter()
            .zip(fs_transitions.iter().enumerate())
        {
            assert_eq!(computed.at_time, fs.0);
            let type_index = data_block_v2.transition_types[idx];
            assert_eq!(
                computed.offset,
                data_block_v2.local_time_type_records[type_index].utoff.0
            );
        }
    }

    #[test]
    fn test_lord_howe() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut zoneinfo =
            ZoneInfoCompiler::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();
        // Association is needed.
        let computed_zoneinfo = zoneinfo.build_zone("Australia/Lord_Howe");

        let data =
            tzif::parse_tzif_file(Path::new("/usr/share/zoneinfo/Australia/Lord_Howe")).unwrap();
        let data_block_v2 = data.data_block2.unwrap();
        let fs_transitions = data_block_v2.transition_times;

        for (computed, (idx, fs)) in computed_zoneinfo
            .transitions
            .iter()
            .zip(fs_transitions.iter().enumerate())
        {
            assert_eq!(computed.at_time, fs.0);
            let type_index = data_block_v2.transition_types[idx];
            assert_eq!(
                computed.offset,
                data_block_v2.local_time_type_records[type_index].utoff.0
            );
        }
    }

    #[test]
    fn test_troll() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut zoneinfo =
            ZoneInfoCompiler::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();
        // Association is needed.
        let computed_zoneinfo = zoneinfo.build_zone("Antarctica/Troll");

        let data =
            tzif::parse_tzif_file(Path::new("/usr/share/zoneinfo/Antarctica/Troll")).unwrap();
        let data_block_v2 = data.data_block2.unwrap();
        let fs_transitions = data_block_v2.transition_times;

        for (computed, (idx, fs)) in computed_zoneinfo
            .transitions
            .iter()
            .zip(fs_transitions.iter().enumerate())
        {
            assert_eq!(computed.at_time, fs.0);
            let type_index = data_block_v2.transition_types[idx];
            assert_eq!(
                computed.offset,
                data_block_v2.local_time_type_records[type_index].utoff.0
            );
        }
    }

    #[test]
    fn test_dublin() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut zoneinfo =
            ZoneInfoCompiler::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();
        // Association is needed.
        let computed_zoneinfo = zoneinfo.build_zone("Europe/Dublin");

        let data = tzif::parse_tzif_file(Path::new("/usr/share/zoneinfo/Europe/Dublin")).unwrap();
        let data_block_v2 = data.data_block2.unwrap();
        let fs_transitions = data_block_v2.transition_times;

        for (computed, (idx, fs)) in computed_zoneinfo
            .transitions
            .iter()
            .zip(fs_transitions.iter().enumerate())
        {
            assert_eq!(computed.at_time, fs.0);
            let type_index = data_block_v2.transition_types[idx];
            assert_eq!(
                computed.offset,
                data_block_v2.local_time_type_records[type_index].utoff.0
            );
        }
    }

    #[test]
    fn test_berlin() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut zoneinfo =
            ZoneInfoCompiler::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();
        // Association is needed.
        let computed_zoneinfo = zoneinfo.build_zone("Europe/Berlin");

        let data = tzif::parse_tzif_file(Path::new("/usr/share/zoneinfo/Europe/Berlin")).unwrap();
        let data_block_v2 = data.data_block2.unwrap();
        let fs_transitions = data_block_v2.transition_times;

        for (computed, (idx, fs)) in computed_zoneinfo
            .transitions
            .iter()
            .zip(fs_transitions.iter().enumerate())
        {
            assert_eq!(computed.at_time, fs.0);
            let type_index = data_block_v2.transition_types[idx];
            assert_eq!(
                computed.offset,
                data_block_v2.local_time_type_records[type_index].utoff.0
            );
        }
    }

    #[test]
    fn test_paris() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut zoneinfo =
            ZoneInfoCompiler::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();
        // Association is needed.
        let computed_zoneinfo = zoneinfo.build_zone("Europe/Paris");

        let data = tzif::parse_tzif_file(Path::new("/usr/share/zoneinfo/Europe/Paris")).unwrap();
        let data_block_v2 = data.data_block2.unwrap();
        let fs_transitions = data_block_v2.transition_times;

        for (computed, (idx, fs)) in computed_zoneinfo
            .transitions
            .iter()
            .zip(fs_transitions.iter().enumerate())
        {
            assert_eq!(computed.at_time, fs.0);
            let type_index = data_block_v2.transition_types[idx];
            assert_eq!(
                computed.offset,
                data_block_v2.local_time_type_records[type_index].utoff.0
            );
        }
    }

    #[test]
    fn test_london() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut zoneinfo =
            ZoneInfoCompiler::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();
        // Association is needed.
        let computed_zoneinfo = zoneinfo.build_zone("Europe/London");

        let data = tzif::parse_tzif_file(Path::new("/usr/share/zoneinfo/Europe/London")).unwrap();
        let data_block_v2 = data.data_block2.unwrap();
        let fs_transitions = data_block_v2.transition_times;

        for (computed, (idx, fs)) in computed_zoneinfo
            .transitions
            .iter()
            .zip(fs_transitions.iter().enumerate())
        {
            let type_index = data_block_v2.transition_types[idx];
            assert_eq!(
                (computed.at_time, computed.offset),
                (
                    fs.0,
                    data_block_v2.local_time_type_records[type_index].utoff.0
                )
            );
        }
    }

    #[test]
    fn test_riga() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut zoneinfo =
            ZoneInfoCompiler::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();
        // Association is needed.
        let computed_zoneinfo = zoneinfo.build_zone("Europe/Riga");

        let data = tzif::parse_tzif_file(Path::new("/usr/share/zoneinfo/Europe/Riga")).unwrap();
        let data_block_v2 = data.data_block2.unwrap();
        let fs_transitions = data_block_v2.transition_times;

        for (computed, (idx, fs)) in computed_zoneinfo
            .transitions
            .iter()
            .zip(fs_transitions.iter().enumerate())
        {
            let type_index = data_block_v2.transition_types[idx];
            assert_eq!(
                (computed.at_time, computed.offset),
                (
                    fs.0,
                    data_block_v2.local_time_type_records[type_index].utoff.0
                )
            );
        }
    }
}
