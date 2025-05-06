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

// #![no_std]

extern crate alloc;

use alloc::string::String;
use parser::ZoneInfoParseError;
use utils::epoch_seconds_for_year;

use hashbrown::HashMap;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::{io, path::Path};

pub(crate) mod utils;

pub mod compiler;
pub mod parser;
pub mod posix;
pub mod rule;
pub mod types;
pub mod tzif;
pub mod zone;

#[doc(inline)]
pub use compiler::ZoneInfoCompiler;

#[doc(inline)]
pub use parser::ZoneInfoParser;

use rule::Rules;
use zone::ZoneRecord;

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

/// `ZoneInfoData` represents raw unprocessed zone info data
/// as parsed from a zone info file.
///
/// See [`ZoneInfoCompiler`] if transitions are required.
#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct ZoneInfoData {
    /// Data parsed from zone info Rule lines keyed by Rule name
    pub rules: HashMap<String, Rules>,
    /// Data parsed from zone info Zone records.
    pub zones: HashMap<String, ZoneRecord>,
    /// Data parsed from Link lines
    pub links: HashMap<String, String>,
    /// Data parsed from `#PACKRATLIST` lines
    pub pack_rat: HashMap<String, String>,
}

// ==== ZoneInfoData parsing methods ====

impl ZoneInfoData {
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

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::{ZoneInfoCompiler, ZoneInfoData};
    use std::{
        format,
        fs::{self, read_to_string},
        path::Path,
        vec::Vec,
    };

    #[derive(Debug, Serialize, Deserialize)]
    struct TzifTestData {
        first_record: LocalRecord,
        transitions: alloc::vec::Vec<TransitionRecord>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct TransitionRecord {
        transition_time: i64,
        record: LocalRecord,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct LocalRecord {
        offset: i64,
        is_dst: bool,
        abbr: alloc::string::String,
    }

    // Utility function for generating example files
    #[allow(unused)]
    fn generate_test_data(identifier: &str) {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let examples_dir = manifest_dir.join("examples");
        let filename = identifier.to_lowercase().replace("/", "-");
        let test_data_path = examples_dir.join(format!("{filename}.json"));

        let tzif =
            tzif::parse_tzif_file(Path::new(&format!("/usr/share/zoneinfo/{identifier}"))).unwrap();
        let tzif_block_v2 = tzif.data_block2.unwrap();

        let first_record_data = tzif_block_v2.local_time_type_records[0];
        let first_record = LocalRecord {
            offset: first_record_data.utoff.0,
            is_dst: first_record_data.is_dst,
            abbr: tzif_block_v2.time_zone_designations[0].clone(),
        };

        let local_records = tzif_block_v2
            .local_time_type_records
            .iter()
            .enumerate()
            .map(|(idx, r)| LocalRecord {
                offset: r.utoff.0,
                is_dst: r.is_dst,
                abbr: tzif_block_v2.time_zone_designations[idx].clone(),
            })
            .collect::<Vec<_>>();

        let transitions = tzif_block_v2
            .transition_times
            .iter()
            .zip(tzif_block_v2.transition_types)
            .map(|(time, time_type)| TransitionRecord {
                transition_time: time.0,
                record: local_records[time_type].clone(),
            })
            .collect::<Vec<TransitionRecord>>();

        let tzif_data = TzifTestData {
            first_record,
            transitions,
        };

        std::println!("Writing generated example data to {:?}", test_data_path);
        fs::write(
            test_data_path,
            serde_json::to_string_pretty(&tzif_data).unwrap(),
        )
        .unwrap();
    }

    fn test_data_for_id(identifier: &str) {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let examples_dir = manifest_dir.join("examples");

        // Get test data
        let test_json = identifier.replace("/", "-").to_ascii_lowercase();
        let test_data_path = examples_dir.join(format!("{test_json}.json"));
        let test_data: TzifTestData =
            serde_json::from_str(&read_to_string(test_data_path).unwrap()).unwrap();

        // Compile zoneinfo file.
        let zoneinfo_data = ZoneInfoData::from_filepath(examples_dir.join("zoneinfo")).unwrap();
        let mut compiler = ZoneInfoCompiler::new(zoneinfo_data);
        let computed_zoneinfo = compiler.build_zone(identifier);

        assert_eq!(
            computed_zoneinfo.initial_record.offset,
            test_data.first_record.offset
        );
        assert_eq!(
            computed_zoneinfo.initial_record.designation,
            test_data.first_record.abbr
        );

        for (computed, test_data) in computed_zoneinfo
            .transitions
            .iter()
            .zip(test_data.transitions)
        {
            assert_eq!(computed.at_time, test_data.transition_time);
            assert_eq!(computed.offset, test_data.record.offset);
            // Test data is currently in rearguard, not vanguard. Would need to add
            // support for rearguard and to test dst for Europe/Dublin
            //
            // That or the tzif source for the data is wrong ...
            // assert_eq!(computed.dst, test_data.record.is_dst); // TODO stabilize dst flags / vanguard/rearguard parsing
            // TODO: Fix bug with first transition formatting.
            //
            // When in named rule before any transition has happened,
            // value is initialized to first letter of save == 0
            // assert_eq!(computed.format, test_data.record.abbr); // TODO stabilize abbr
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_chicago() {
        test_data_for_id("America/Chicago");
    }

    #[test]
    fn test_new_york() {
        test_data_for_id("America/New_York");
    }

    #[test]
    fn test_anchorage() {
        test_data_for_id("America/Anchorage");
    }

    #[test]
    fn test_sydney() {
        test_data_for_id("Australia/Sydney");
    }

    #[test]
    fn test_lord_howe() {
        test_data_for_id("Australia/Lord_Howe");
    }

    #[test]
    fn test_troll() {
        test_data_for_id("Antarctica/Troll");
    }

    #[test]
    fn test_dublin() {
        test_data_for_id("Europe/Dublin");
    }

    #[test]
    fn test_berlin() {
        test_data_for_id("Europe/Berlin");
    }

    #[test]
    fn test_paris() {
        test_data_for_id("Europe/Paris");
    }

    #[test]
    fn test_london() {
        test_data_for_id("Europe/London");
    }

    #[test]
    fn test_riga() {
        test_data_for_id("Europe/Riga");
    }
}
