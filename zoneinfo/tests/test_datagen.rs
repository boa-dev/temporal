#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use std::{format, fs::read_to_string, path::Path, string::String, vec::Vec};
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

/// Load JSON ground truth for `identifier` from `test_data_dir`.
#[cfg(feature = "std")]
fn read_test_data(test_data_dir: &Path, identifier: &str) -> TzifTestData {
    let test_json = identifier.replace("/", "-").to_ascii_lowercase();
    let path = test_data_dir.join(format!("{test_json}.json"));
    serde_json::from_str(&read_to_string(path).unwrap()).unwrap()
}

/// Assert that `zoneinfo_data` compiles `identifier` to transitions matching `test_data`.
///
/// `strict` additionally asserts that the transition counts are equal (not just that every
/// compiled transition has a matching ground-truth entry via `zip`).
#[cfg(feature = "std")]
fn run_assertions(identifier: &str, strict: bool, zoneinfo_data: ZoneInfoData, test_data: TzifTestData) {
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
    if strict {
        assert_eq!(
            computed_zoneinfo.transitions.len(),
            test_data.transitions.len()
        );
    }

    for (computed, td) in computed_zoneinfo
        .transitions
        .iter()
        .zip(test_data.transitions)
    {
        assert_eq!(
            computed.at_time, td.transition_time,
            "Transition times are not aligned for {}",
            td.transition_time,
        );
        assert_eq!(
            computed.offset, td.record.offset,
            "Offsets are not aligned for {}",
            td.transition_time
        );
        // Test data is currently in rearguard, not vanguard. Would need to add
        // support for rearguard and to test dst for Europe/Dublin
        assert_eq!(
            computed.dst, td.record.is_dst,
            "DST flag is not aligned for {}",
            td.transition_time
        );
        // When in named rule before any transition has happened,
        // value is initialized to first letter of save == 0
        assert_eq!(
            computed.format, td.record.abbr,
            "Designation is not aligned for {}",
            td.transition_time
        );
    }
}

/// Test a zone against the hand-crafted `tests/zoneinfo` source file.
///
/// All zones tested this way must be defined in that file.
#[cfg(feature = "std")]
fn test_data_for_id(identifier: &str, strict: bool) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let test_dir = manifest_dir.join("tests");

    let test_data = read_test_data(&test_dir.join("data"), identifier);
    let zoneinfo_data = ZoneInfoData::from_filepath(test_dir.join("zoneinfo")).unwrap();
    run_assertions(identifier, strict, zoneinfo_data, test_data);
}

/// Test a zone against the full IANA tzdata source directory.
///
/// Requires the `TZDATA_DIR` environment variable to point at an extracted IANA `tzdata`
/// tarball (a directory containing the standard source files: `northamerica`, `europe`,
/// etc.).  When `TZDATA_DIR` is not set the test emits a diagnostic and exits without
/// failing so that the suite passes locally without the download.
///
/// Run `mise run tz:fetch` to populate `.tzdata/src/`, then set
/// `TZDATA_DIR=$PWD/.tzdata/src` before invoking `cargo test`.
#[cfg(feature = "std")]
fn test_data_for_id_iana(identifier: &str) {
    let tzdata_dir = match std::env::var("TZDATA_DIR") {
        Ok(dir) => dir,
        Err(_) => {
            eprintln!(
                "SKIP {identifier}: TZDATA_DIR not set \
                 (run `mise run tz:fetch` and set TZDATA_DIR=$PWD/.tzdata/src)"
            );
            return;
        }
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let test_data = read_test_data(&manifest_dir.join("tests").join("data"), identifier);
    let zoneinfo_data = ZoneInfoData::from_zoneinfo_directory(&tzdata_dir).unwrap();
    run_assertions(identifier, false, zoneinfo_data, test_data);
}

// =============================================================================
// Existing zone tests (hand-crafted zoneinfo file)
// =============================================================================

#[test]
#[cfg(feature = "std")]
fn test_chicago() {
    test_data_for_id("America/Chicago", false);
}

#[test]
#[cfg(feature = "std")]
fn test_new_york() {
    test_data_for_id("America/New_York", false);
}

#[test]
#[cfg(feature = "std")]
fn test_anchorage() {
    test_data_for_id("America/Anchorage", false);
}

#[test]
#[cfg(feature = "std")]
fn test_sydney() {
    test_data_for_id("Australia/Sydney", false);
}

#[test]
#[cfg(feature = "std")]
fn test_lord_howe() {
    test_data_for_id("Australia/Lord_Howe", false);
}

#[test]
#[cfg(feature = "std")]
fn test_troll() {
    test_data_for_id("Antarctica/Troll", false);
}

// TODO: test_dublin_rearguard
#[test]
#[cfg(feature = "std")]
fn test_dublin() {
    test_data_for_id("Europe/Dublin", false);
}

#[test]
#[cfg(feature = "std")]
fn test_berlin() {
    test_data_for_id("Europe/Berlin", false);
}

#[test]
#[cfg(feature = "std")]
fn test_paris() {
    test_data_for_id("Europe/Paris", false);
}

#[test]
#[cfg(feature = "std")]
fn test_london() {
    test_data_for_id("Europe/London", false);
}

#[test]
#[cfg(feature = "std")]
fn test_riga() {
    test_data_for_id("Europe/Riga", false);
}

#[test]
#[cfg(feature = "std")]
fn test_sao_paulo() {
    test_data_for_id("America/Sao_Paulo", true);
}

// =============================================================================
// Regression tests for issue #602 — getTimeZoneTransition incorrect results
//
// These zones require IANA tzdata source (TZDATA_DIR).  The JSON ground truth
// is pre-generated from TZif and checked in; only the zoneinfo compiler path
// needs the full source.
// =============================================================================

// --- Same-offset transition bugs ---

#[test]
#[cfg(feature = "std")]
fn test_regression_america_godthab() {
    test_data_for_id_iana("America/Godthab");
}

#[test]
#[cfg(feature = "std")]
fn test_regression_america_grand_turk() {
    test_data_for_id_iana("America/Grand_Turk");
}

#[test]
#[cfg(feature = "std")]
fn test_regression_america_resolute() {
    test_data_for_id_iana("America/Resolute");
}

#[test]
#[cfg(feature = "std")]
fn test_regression_america_scoresbysund() {
    test_data_for_id_iana("America/Scoresbysund");
}

#[test]
#[cfg(feature = "std")]
fn test_regression_europe_lisbon() {
    test_data_for_id_iana("Europe/Lisbon");
}

// --- Past-time / same-time transition bugs ---

#[test]
#[cfg(feature = "std")]
fn test_regression_america_goose_bay() {
    test_data_for_id_iana("America/Goose_Bay");
}

#[test]
#[cfg(feature = "std")]
fn test_regression_america_st_johns() {
    test_data_for_id_iana("America/St_Johns");
}

#[test]
#[cfg(feature = "std")]
fn test_regression_america_indiana_petersburg() {
    test_data_for_id_iana("America/Indiana/Petersburg");
}

#[test]
#[cfg(feature = "std")]
fn test_regression_america_indiana_vincennes() {
    test_data_for_id_iana("America/Indiana/Vincennes");
}

#[test]
#[cfg(feature = "std")]
fn test_regression_america_north_dakota_beulah() {
    test_data_for_id_iana("America/North_Dakota/Beulah");
}

#[test]
#[cfg(feature = "std")]
fn test_regression_america_ojinaga() {
    test_data_for_id_iana("America/Ojinaga");
}

#[test]
#[cfg(feature = "std")]
fn test_regression_antarctica_macquarie() {
    test_data_for_id_iana("Antarctica/Macquarie");
}

#[test]
#[cfg(feature = "std")]
fn test_regression_asia_nicosia() {
    test_data_for_id_iana("Asia/Nicosia");
}

#[test]
#[cfg(feature = "std")]
fn test_regression_europe_bucharest() {
    test_data_for_id_iana("Europe/Bucharest");
}

#[test]
#[cfg(feature = "std")]
fn test_regression_europe_chisinau() {
    test_data_for_id_iana("Europe/Chisinau");
}

#[test]
#[cfg(feature = "std")]
fn test_regression_europe_kiev() {
    test_data_for_id_iana("Europe/Kiev");
}

#[test]
#[cfg(feature = "std")]
fn test_regression_europe_sofia() {
    test_data_for_id_iana("Europe/Sofia");
}

#[test]
#[cfg(feature = "std")]
fn test_regression_europe_vilnius() {
    test_data_for_id_iana("Europe/Vilnius");
}

#[test]
#[cfg(feature = "std")]
fn test_regression_pacific_norfolk() {
    test_data_for_id_iana("Pacific/Norfolk");
}
