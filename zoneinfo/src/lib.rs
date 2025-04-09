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
use core::ops::RangeInclusive;
use parser::{ZoneInfoParseError, ZoneInfoParser};
use types::Transition;
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

// NOTE: RangeInclusive<i32> here is excessive. Would be nice to have a
// range type that enforced a max-min
#[derive(Debug, Clone)]
pub struct ZoneInfoCompileSettings {
    range: RangeInclusive<i32>,
}

impl Default for ZoneInfoCompileSettings {
    fn default() -> Self {
        Self { range: 1901..=2038 }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SingleLineZone {
    offset: i64,
    identifier: String,
}

/// Intermediate zoneinfo data type that contains an ordered
/// set of transition data along with a POSIX time zone
/// string
///
/// But the point here is to provide the required data in a
/// consummable format for anyone who needs zoneinfo data.
#[derive(Debug, PartialEq)]
pub struct ZoneInfoTransitionData {
    pub transitions: BTreeSet<Transition>,
    pub single_line_zone: Option<SingleLineZone>,
    pub posix_string: String, // TODO: Implement POSIX string building
}

impl ZoneInfoTransitionData {
    pub fn to_v2_data_block(&self) -> TzifBlockV2 {
        if let Some(single_line) = &self.single_line_zone {
            TzifBlockV2::from_single_line_zone(single_line)
        } else {
            TzifBlockV2::from_transition_set(&self.transitions)
        }
    }
}

/// `ZoneInfo` is a struct of that maps a IANA identifier to
/// its ordered transition data.
#[derive(Debug, Default)]
pub struct ZoneInfo {
    pub data: HashMap<String, ZoneInfoTransitionData>,
}

pub enum CompiledZone {
    Single(SingleLineZone),
    Transitions(BTreeSet<Transition>),
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct ZoneInfoCompiler {
    pub rules: HashMap<String, RuleTable>,
    pub zones: HashMap<String, ZoneTable>,
    pub links: HashMap<String, String>,
    pub pack_rat: HashMap<String, String>,
}

impl ZoneInfoCompiler {
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

    #[cfg(feature = "std")]
    pub fn from_filepath<P: AsRef<Path> + core::fmt::Debug>(
        path: P,
    ) -> Result<Self, ZoneInfoError> {
        Self::from_zoneinfo_str(&std::fs::read_to_string(path)?)
    }

    pub fn from_zoneinfo_str(src: &str) -> Result<Self, ZoneInfoError> {
        ZoneInfoParser::from_zoneinfo_str(src)
            .parse()
            .map_err(ZoneInfoError::Parse)
    }

    pub fn extend(&mut self, other: Self) {
        self.rules.extend(other.rules);
        self.zones.extend(other.zones);
        self.links.extend(other.links);
        self.pack_rat.extend(other.pack_rat);
    }
}

impl ZoneInfoCompiler {
    pub fn associate_and_build(&mut self, settings: ZoneInfoCompileSettings) -> ZoneInfo {
        // Associate the necessary rules with the ZoneTable
        self.associate();
        self.build(settings)
    }

    pub fn associate_and_build_for_zone(
        &mut self,
        target: &str,
        settings: &ZoneInfoCompileSettings,
    ) -> CompiledZone {
        self.associate();
        self.build_for_zone(target, settings)
    }

    pub fn build(&mut self, settings: ZoneInfoCompileSettings) -> ZoneInfo {
        // TODO: Validate and resolve settings here.
        let mut zoneinfo = ZoneInfo::default();
        for identifier in self.zones.keys() {
            let transition_data = self.build_for_zone(identifier, &settings);
            let (transitions, single_line_zone) = match transition_data {
                CompiledZone::Single(d) => (BTreeSet::default(), Some(d)),
                CompiledZone::Transitions(d) => (d, None),
            };
            let tzif = ZoneInfoTransitionData {
                transitions,
                single_line_zone,
                // TODO: Handle POSIX tz string
                posix_string: String::default(),
            };
            let _ = zoneinfo.data.insert(identifier.clone(), tzif);
        }
        zoneinfo
    }

    /// Make sure to associate first!
    pub fn build_for_zone(&self, target: &str, settings: &ZoneInfoCompileSettings) -> CompiledZone {
        let table = self
            .zones
            .get(target)
            .expect("Invalid identifier provided.");
        let mut output = BTreeSet::default();
        let Some(transition) = table.get_first_transition() else {
            let line = &table.table[0];
            return CompiledZone::Single(SingleLineZone {
                offset: line.std_offset.as_secs(),
                identifier: line.format.format(line.std_offset.as_secs(), None, false),
            });
        };
        output.insert(transition);
        // Move this into a build from range function.
        let mut build_context = ZoneBuildContext::default();
        for year in settings.range.clone() {
            build_context.update(year);
            let year_transitions = table.calculate_transitions_for_year(year, &mut build_context);
            output.extend(year_transitions);
        }
        CompiledZone::Transitions(output)
    }

    pub fn associate(&mut self) {
        for zones in self.zones.values_mut() {
            zones.associate_rules(&self.rules);
        }
    }
}

#[cfg(test)]
#[cfg(all(feature = "std", not(target_os = "windows")))]
mod tests {
    use crate::{CompiledZone, ZoneInfoCompileSettings, ZoneInfoCompiler};
    use std::path::Path;

    // Use this function for tests until we handle the i32::MAX
    fn test_default_settings() -> ZoneInfoCompileSettings {
        ZoneInfoCompileSettings { range: 1901..=2037 }
    }

    #[test]
    fn test_chicago() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut zoneinfo =
            ZoneInfoCompiler::from_filepath(manifest_dir.join("examples/northamerica")).unwrap();

        // Association is needed.
        let computed_transitions = match zoneinfo
            .associate_and_build_for_zone("America/Chicago", &test_default_settings())
        {
            CompiledZone::Single(_) => unreachable!(),
            CompiledZone::Transitions(set) => set,
        };

        let data = tzif::parse_tzif_file(Path::new("/usr/share/zoneinfo/America/Chicago")).unwrap();
        let fs_transitions = data.data_block2.unwrap().transition_times;

        for (computed, fs) in computed_transitions.iter().zip(fs_transitions.iter()) {
            assert_eq!(computed.at_time, fs.0);
        }
    }

    #[test]
    fn test_new_york() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut zoneinfo =
            ZoneInfoCompiler::from_filepath(manifest_dir.join("examples/northamerica")).unwrap();

        // Association is needed.
        let computed_transitions = match zoneinfo
            .associate_and_build_for_zone("America/New_York", &test_default_settings())
        {
            CompiledZone::Single(_) => unreachable!(),
            CompiledZone::Transitions(set) => set,
        };

        let data =
            tzif::parse_tzif_file(Path::new("/usr/share/zoneinfo/America/New_York")).unwrap();
        let fs_transitions = data.data_block2.unwrap().transition_times;

        for (computed, fs) in computed_transitions.iter().zip(fs_transitions.iter()) {
            assert_eq!(computed.at_time, fs.0);
        }
    }

    #[test]
    fn test_sydney() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut zoneinfo =
            ZoneInfoCompiler::from_filepath(manifest_dir.join("examples/australasia")).unwrap();
        // Association is needed.
        let computed_transitions = match zoneinfo
            .associate_and_build_for_zone("Australia/Sydney", &test_default_settings())
        {
            CompiledZone::Single(_) => unreachable!(),
            CompiledZone::Transitions(set) => set,
        };

        let data =
            tzif::parse_tzif_file(Path::new("/usr/share/zoneinfo/Australia/Sydney")).unwrap();
        let fs_transitions = data.data_block2.unwrap().transition_times;

        for (computed, fs) in computed_transitions.iter().zip(fs_transitions.iter()) {
            assert_eq!(computed.at_time, fs.0);
        }
    }

    #[test]
    fn test_lord_howe() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut zoneinfo =
            ZoneInfoCompiler::from_filepath(manifest_dir.join("examples/australasia")).unwrap();
        // Association is needed.
        let computed_transitions = match zoneinfo
            .associate_and_build_for_zone("Australia/Lord_Howe", &test_default_settings())
        {
            CompiledZone::Single(_) => unreachable!(),
            CompiledZone::Transitions(set) => set,
        };

        let data =
            tzif::parse_tzif_file(Path::new("/usr/share/zoneinfo/Australia/Lord_Howe")).unwrap();
        let fs_transitions = data.data_block2.unwrap().transition_times;

        for (computed, fs) in computed_transitions.iter().zip(fs_transitions.iter()) {
            assert_eq!(computed.at_time, fs.0);
        }
    }

    #[test]
    fn test_troll() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut zoneinfo =
            ZoneInfoCompiler::from_filepath(manifest_dir.join("examples/antarctica")).unwrap();
        // Association is needed.
        let computed_transitions = match zoneinfo
            .associate_and_build_for_zone("Antarctica/Troll", &test_default_settings())
        {
            CompiledZone::Single(_) => unreachable!(),
            CompiledZone::Transitions(set) => set,
        };

        let data =
            tzif::parse_tzif_file(Path::new("/usr/share/zoneinfo/Antarctica/Troll")).unwrap();
        let fs_transitions = data.data_block2.unwrap().transition_times;

        for (computed, fs) in computed_transitions.iter().zip(fs_transitions.iter()) {
            assert_eq!(computed.at_time, fs.0);
        }
    }

    #[test]
    fn test_dublin() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut zoneinfo =
            ZoneInfoCompiler::from_filepath(manifest_dir.join("examples/europe")).unwrap();
        // Association is needed.
        let computed_transitions = match zoneinfo
            .associate_and_build_for_zone("Europe/Dublin", &test_default_settings())
        {
            CompiledZone::Single(_) => unreachable!(),
            CompiledZone::Transitions(set) => set,
        };

        let data = tzif::parse_tzif_file(Path::new("/usr/share/zoneinfo/Europe/Dublin")).unwrap();
        let fs_transitions = data.data_block2.unwrap().transition_times;

        for (computed, fs) in computed_transitions.iter().zip(fs_transitions.iter()) {
            assert_eq!(computed.at_time, fs.0);
        }
    }

    #[test]
    fn test_berlin() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut zoneinfo =
            ZoneInfoCompiler::from_filepath(manifest_dir.join("examples/europe")).unwrap();
        // Association is needed.
        let computed_transitions = match zoneinfo
            .associate_and_build_for_zone("Europe/Berlin", &test_default_settings())
        {
            CompiledZone::Single(_) => unreachable!(),
            CompiledZone::Transitions(set) => set,
        };

        let data = tzif::parse_tzif_file(Path::new("/usr/share/zoneinfo/Europe/Berlin")).unwrap();
        let fs_transitions = data.data_block2.unwrap().transition_times;

        for (computed, fs) in computed_transitions.iter().zip(fs_transitions.iter()) {
            assert_eq!(computed.at_time, fs.0);
        }
    }

    #[test]
    fn test_paris() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut zoneinfo =
            ZoneInfoCompiler::from_filepath(manifest_dir.join("examples/europe")).unwrap();
        // Association is needed.
        let computed_transitions =
            match zoneinfo.associate_and_build_for_zone("Europe/Paris", &test_default_settings()) {
                CompiledZone::Single(_) => unreachable!(),
                CompiledZone::Transitions(set) => set,
            };

        let data = tzif::parse_tzif_file(Path::new("/usr/share/zoneinfo/Europe/Paris")).unwrap();
        let fs_transitions = data.data_block2.unwrap().transition_times;

        for (computed, fs) in computed_transitions.iter().zip(fs_transitions.iter()) {
            assert_eq!(computed.at_time, fs.0);
        }
    }

    #[test]
    fn test_london() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut zoneinfo =
            ZoneInfoCompiler::from_filepath(manifest_dir.join("examples/europe")).unwrap();
        // Association is needed.
        let computed_transitions = match zoneinfo
            .associate_and_build_for_zone("Europe/London", &test_default_settings())
        {
            CompiledZone::Single(_) => unreachable!(),
            CompiledZone::Transitions(set) => set,
        };

        let data = tzif::parse_tzif_file(Path::new("/usr/share/zoneinfo/Europe/London")).unwrap();
        let fs_transitions = data.data_block2.unwrap().transition_times;

        for (computed, fs) in computed_transitions.iter().zip(fs_transitions.iter()) {
            assert_eq!(computed.at_time, fs.0);
        }
    }

    #[test]
    fn test_riga() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut zoneinfo =
            ZoneInfoCompiler::from_filepath(manifest_dir.join("examples/europe")).unwrap();
        // Association is needed.
        let computed_transitions =
            match zoneinfo.associate_and_build_for_zone("Europe/Riga", &test_default_settings()) {
                CompiledZone::Single(_) => unreachable!(),
                CompiledZone::Transitions(set) => set,
            };

        let data = tzif::parse_tzif_file(Path::new("/usr/share/zoneinfo/Europe/Riga")).unwrap();
        let fs_transitions = data.data_block2.unwrap().transition_times;

        for (computed, fs) in computed_transitions.iter().zip(fs_transitions.iter()) {
            assert_eq!(computed.at_time, fs.0);
        }
    }
}
