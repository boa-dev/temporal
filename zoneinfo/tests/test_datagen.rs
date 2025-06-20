#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use std::{
    format,
    fs::{self, read_to_string},
    path::{Path, PathBuf},
    string::String,
    vec::Vec,
};
#[cfg(feature = "std")]
use zoneinfo_rs::{ZoneInfoCompiler, ZoneInfoData};

#[cfg(feature = "std")]
#[derive(Debug, Serialize, Deserialize)]
struct TzifTestData {
    first_record: LocalRecord,
    transitions: Vec<TransitionRecord>,
}

#[cfg(feature = "std")]
#[derive(Debug, Serialize, Deserialize)]
struct TransitionRecord {
    transition_time: i64,
    record: LocalRecord,
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalRecord {
    offset: i64,
    is_dst: bool,
    abbr: String,
}

// Utility function for generating example files
#[allow(unused)]
#[cfg(feature = "std")]
fn generate_test_data(tzdata_dir: PathBuf, identifier: &str) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let test_dir = manifest_dir.join("tests");
    let filename = identifier.to_lowercase().replace("/", "-");
    let test_data_path = test_dir.join(format!("{filename}.json"));

    let tzdata = manifest_dir.join("tzdata/test/");
    let tzif = tzif::parse_tzif_file(&tzdata_dir.join(identifier)).unwrap();
    let tzif_block_v2 = tzif.data_block2.unwrap();

    let first_record_data = tzif_block_v2.local_time_type_records[0];
    let first_record = LocalRecord {
        offset: first_record_data.utoff.0,
        is_dst: first_record_data.is_dst,
        abbr: tzif_block_v2.time_zone_designations[0].clone(),
    };

    print!("{:#?}", tzif_block_v2);

    let local_records = tzif_block_v2
        .local_time_type_records
        .iter()
        .enumerate()
        .map(|(idx, r)| LocalRecord {
            offset: r.utoff.0,
            is_dst: r.is_dst,
            abbr: tzif_block_v2
                .time_zone_designations
                .get(r.idx / 4)
                .cloned()
                .unwrap_or(String::from("unknown")),
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

// Uncomment and adjust path to generate any new testing data
//
// #[test]
// #[cfg(feature = "std")]
// fn gen() {
//     let path = Path::new("Europe/Dublin")
//     generate_test_data(path);
// }

#[cfg(feature = "std")]
fn test_data_for_id(identifier: &str) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let test_dir = manifest_dir.join("tests");
    let test_data_dir = test_dir.join("data");

    // Get test data
    let test_json = identifier.replace("/", "-").to_ascii_lowercase();
    let test_data_path = test_data_dir.join(format!("{test_json}.json"));
    let test_data: TzifTestData =
        serde_json::from_str(&read_to_string(test_data_path).unwrap()).unwrap();

    // Compile zoneinfo file.
    let zoneinfo_data = ZoneInfoData::from_filepath(test_dir.join("zoneinfo")).unwrap();
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
        assert_eq!(computed.dst, test_data.record.is_dst); // TODO stabilize dst flags / vanguard/rearguard parsing
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
#[cfg(feature = "std")]
fn test_new_york() {
    test_data_for_id("America/New_York");
}

#[test]
#[cfg(feature = "std")]
fn test_anchorage() {
    test_data_for_id("America/Anchorage");
}

#[test]
#[cfg(feature = "std")]
fn test_sydney() {
    test_data_for_id("Australia/Sydney");
}

#[test]
#[cfg(feature = "std")]
fn test_lord_howe() {
    test_data_for_id("Australia/Lord_Howe");
}

#[test]
#[cfg(feature = "std")]
fn test_troll() {
    test_data_for_id("Antarctica/Troll");
}

// TODO: test_dublin_rearguard
#[test]
#[cfg(feature = "std")]
fn test_dublin() {
    test_data_for_id("Europe/Dublin");
}

#[test]
#[cfg(feature = "std")]
fn test_berlin() {
    test_data_for_id("Europe/Berlin");
}

#[test]
#[cfg(feature = "std")]
fn test_paris() {
    test_data_for_id("Europe/Paris");
}

#[test]
#[cfg(feature = "std")]
fn test_london() {
    test_data_for_id("Europe/London");
}

#[test]
#[cfg(feature = "std")]
fn test_riga() {
    test_data_for_id("Europe/Riga");
}
