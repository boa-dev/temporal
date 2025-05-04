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
#[cfg(all(feature = "std", not(target_os = "windows")))]
mod tests {
    use crate::{ZoneInfoCompiler, ZoneInfoData};
    use std::path::Path;

    #[test]
    fn test_chicago() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let zoneinfo_data =
            ZoneInfoData::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();
        let mut compiler = ZoneInfoCompiler::new(zoneinfo_data);

        // Association is needed.
        let computed_zoneinfo = compiler.build_zone("America/Chicago");

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
        let zoneinfo_data =
            ZoneInfoData::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();
        let mut compiler = ZoneInfoCompiler::new(zoneinfo_data);

        // Association is needed.
        let computed_zoneinfo = compiler.build_zone("America/New_York");

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
        let zoneinfo_data =
            ZoneInfoData::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();
        let mut compiler = ZoneInfoCompiler::new(zoneinfo_data);

        // Association is needed.
        let computed_zoneinfo = compiler.build_zone("America/Anchorage");

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
        let zoneinfo_data =
            ZoneInfoData::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();
        let mut compiler = ZoneInfoCompiler::new(zoneinfo_data);

        // Association is needed.
        let computed_zoneinfo = compiler.build_zone("Australia/Sydney");

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
        let zoneinfo_data =
            ZoneInfoData::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();
        let mut compiler = ZoneInfoCompiler::new(zoneinfo_data);
        // Association is needed.
        let computed_zoneinfo = compiler.build_zone("Australia/Lord_Howe");

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
        let zoneinfo_data =
            ZoneInfoData::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();
        let mut compiler = ZoneInfoCompiler::new(zoneinfo_data);
        // Association is needed.
        let computed_zoneinfo = compiler.build_zone("Antarctica/Troll");

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
        let zoneinfo_data =
            ZoneInfoData::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();
        let mut compiler = ZoneInfoCompiler::new(zoneinfo_data);
        // Association is needed.
        let computed_zoneinfo = compiler.build_zone("Europe/Dublin");

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
        let zoneinfo_data =
            ZoneInfoData::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();
        let mut compiler = ZoneInfoCompiler::new(zoneinfo_data);
        // Association is needed.
        let computed_zoneinfo = compiler.build_zone("Europe/Berlin");

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
        let zoneinfo_data =
            ZoneInfoData::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();
        let mut compiler = ZoneInfoCompiler::new(zoneinfo_data);
        // Association is needed.
        let computed_zoneinfo = compiler.build_zone("Europe/Paris");

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
        let zoneinfo_data =
            ZoneInfoData::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();
        let mut compiler = ZoneInfoCompiler::new(zoneinfo_data);
        // Association is needed.
        let computed_zoneinfo = compiler.build_zone("Europe/London");

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
        let zoneinfo_data =
            ZoneInfoData::from_filepath(manifest_dir.join("examples/zoneinfo")).unwrap();
        let mut compiler = ZoneInfoCompiler::new(zoneinfo_data);
        // Association is needed.
        let computed_zoneinfo = compiler.build_zone("Europe/Riga");

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
