use serde::{Deserialize, Serialize};
use std::{format, fs::read_to_string, path::Path, string::String, vec::Vec};
use zoneinfo_rs::{compiler::CompiledTransitions, ZoneInfoCompiler, ZoneInfoData};

#[derive(Debug, Serialize, Deserialize)]
struct TzifTestData {
    first_record: LocalRecord,
    transitions: Vec<TransitionRecord>,
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
    abbr: String,
}

fn read_test_data(test_data_dir: &Path, identifier: &str) -> TzifTestData {
    let test_json = identifier.replace("/", "-").to_ascii_lowercase();
    let path = test_data_dir.join(format!("{test_json}.json"));
    serde_json::from_str(&read_to_string(path).unwrap()).unwrap()
}

/// Zones where zic materializes transitions `zoneinfo_rs` does not generate.
///
/// `Rule Palestine 2072 max` is interleaved with Ramadan-derived `only` rules
/// through 2086. The compiler stops at the first `max` rule and defers the rest
/// to the POSIX footer, which has no Ramadan pause. The shared prefix is still
/// compared.
const UNDER_GENERATED: &[&str] = &["Asia/Gaza", "Asia/Hebron"];

/// Ground truth transitions with zic's trailing footer anchor removed.
///
/// glibc 2.43's zic ends the table with a no-op transition — same offset, DST
/// flag and designation as its predecessor — and omits the next real one,
/// which the footer already expresses; glibc 2.39's zic did the opposite.
///
/// The anchor sits on a zone-line boundary the compiler collapses, so only a
/// final repeat at an instant we do not generate qualifies. A repeat we do
/// generate is real, like `America/Nuuk`'s last transition.
fn significant<'a>(
    test_data: &'a TzifTestData,
    computed: &CompiledTransitions,
) -> &'a [TransitionRecord] {
    let transitions = &test_data.transitions[..];
    let [.., previous, last] = transitions else {
        return transitions;
    };
    let anchor = previous.record.offset == last.record.offset
        && previous.record.is_dst == last.record.is_dst
        && previous.record.abbr == last.record.abbr
        && !computed
            .transitions
            .iter()
            .any(|t| t.at_time == last.transition_time);
    if anchor {
        &transitions[..transitions.len() - 1]
    } else {
        transitions
    }
}

/// Compares a compiled zone against zic's table, one message per mismatch.
///
/// Ground truth may stop early — past its last entry is footer territory.
/// Running past *our* last transition is under-generation, and fails unless the
/// zone is a documented [`UNDER_GENERATED`] gap.
fn compare(
    identifier: &str,
    computed: &CompiledTransitions,
    test_data: &TzifTestData,
) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();

    if computed.initial_record.offset != test_data.first_record.offset {
        errors.push(format!(
            "initial offset {} != {}",
            computed.initial_record.offset, test_data.first_record.offset
        ));
    }
    if computed.initial_record.designation != test_data.first_record.abbr {
        errors.push(format!(
            "initial abbr {:?} != {:?}",
            computed.initial_record.designation, test_data.first_record.abbr
        ));
    }

    let expected = significant(test_data, computed);

    for (c, td) in computed.transitions.iter().zip(expected) {
        if c.at_time != td.transition_time {
            errors.push(format!(
                "transition time {} != {}",
                c.at_time, td.transition_time
            ));
            break; // The lists have desynchronized; later diffs are noise.
        }
        if c.offset != td.record.offset {
            errors.push(format!(
                "offset {} != {} (at {})",
                c.offset, td.record.offset, td.transition_time
            ));
        }
        if c.dst != td.record.is_dst {
            errors.push(format!(
                "dst {} != {} (at {})",
                c.dst, td.record.is_dst, td.transition_time
            ));
        }
        if c.format != td.record.abbr {
            errors.push(format!(
                "abbr {:?} != {:?} (at {})",
                c.format, td.record.abbr, td.transition_time
            ));
        }
    }

    let missing = expected.len().saturating_sub(computed.transitions.len());
    if missing != 0 && !UNDER_GENERATED.contains(&identifier) {
        errors.push(format!(
            "{missing} transitions missing after {} (zic ends at {})",
            computed.transitions.iter().last().map_or(-1, |t| t.at_time),
            expected.last().map_or(-1, |t| t.transition_time)
        ));
    }

    errors
}

fn run_assertions(identifier: &str, zoneinfo_data: ZoneInfoData, test_data: TzifTestData) {
    let mut compiler = ZoneInfoCompiler::new(zoneinfo_data);
    let computed = compiler.build_zone(identifier);
    let errors = compare(identifier, &computed, &test_data);
    assert!(errors.is_empty(), "{identifier}: {}", errors.join("; "));
}

/// Returns None on pass, Some(error) on failure; skips zones with no JSON.
fn check_zone(
    identifier: &str,
    compiler: &mut ZoneInfoCompiler,
    data_dir: &Path,
) -> Option<String> {
    let test_json = identifier.replace("/", "-").to_ascii_lowercase();
    let path = data_dir.join(format!("{test_json}.json"));
    let Ok(content) = std::fs::read_to_string(&path) else {
        return None; // no JSON yet — skip
    };
    let Ok(test_data) = serde_json::from_str::<TzifTestData>(&content) else {
        return Some(format!("{identifier}: malformed JSON"));
    };

    let computed = compiler.build_zone(identifier);
    let errors = compare(identifier, &computed, &test_data);

    if errors.is_empty() {
        None
    } else {
        Some(format!("{identifier}: {}", errors.join("; ")))
    }
}

/// Skips when TZDATA_DIR is not set; run `mise run zoneinfo:test` for the full suite.
fn test_data_for_id(identifier: &str) {
    let tzdata_dir = match std::env::var("TZDATA_DIR") {
        Ok(dir) => dir,
        Err(_) => {
            eprintln!("SKIP {identifier}: TZDATA_DIR not set (run `mise run zoneinfo:test`)");
            return;
        }
    };
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let test_data = read_test_data(&manifest_dir.join("tests").join("data"), identifier);
    let zoneinfo_data = ZoneInfoData::from_zoneinfo_directory(&tzdata_dir).unwrap();
    run_assertions(identifier, zoneinfo_data, test_data);
}

// Zone tests

#[test]
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

#[test]
fn test_sao_paulo() {
    test_data_for_id("America/Sao_Paulo");
}

// Regression tests — issue #602 (getTimeZoneTransition edge cases)

#[test]
fn test_regression_america_godthab() {
    test_data_for_id("America/Godthab");
}

#[test]
fn test_regression_america_grand_turk() {
    test_data_for_id("America/Grand_Turk");
}

#[test]
fn test_regression_america_resolute() {
    test_data_for_id("America/Resolute");
}

#[test]
fn test_regression_america_scoresbysund() {
    test_data_for_id("America/Scoresbysund");
}

#[test]
fn test_regression_europe_lisbon() {
    test_data_for_id("Europe/Lisbon");
}

// --- Past-time / same-time transition bugs ---

#[test]
fn test_regression_america_goose_bay() {
    test_data_for_id("America/Goose_Bay");
}

#[test]
fn test_regression_america_st_johns() {
    test_data_for_id("America/St_Johns");
}

/// A zone-line boundary landing on the same instant as a rule transition is
/// that rule's transition, so it must survive the merge even though its state
/// matches the preceding transition. Here the `4:00 1:00` period ends exactly
/// when `E-EurAsia`'s `Mar lastSun` rule fires, both sides being +05 DST.
#[test]
fn test_regression_asia_tbilisi() {
    test_data_for_id("Asia/Tbilisi");
}

#[test]
fn test_regression_america_indiana_petersburg() {
    test_data_for_id("America/Indiana/Petersburg");
}

#[test]
fn test_regression_america_indiana_vincennes() {
    test_data_for_id("America/Indiana/Vincennes");
}

#[test]
fn test_regression_america_north_dakota_beulah() {
    test_data_for_id("America/North_Dakota/Beulah");
}

#[test]
fn test_regression_america_ojinaga() {
    test_data_for_id("America/Ojinaga");
}

#[test]
fn test_regression_antarctica_macquarie() {
    test_data_for_id("Antarctica/Macquarie");
}

#[test]
fn test_regression_asia_nicosia() {
    test_data_for_id("Asia/Nicosia");
}

#[test]
fn test_regression_europe_bucharest() {
    test_data_for_id("Europe/Bucharest");
}

#[test]
fn test_regression_europe_chisinau() {
    test_data_for_id("Europe/Chisinau");
}

#[test]
fn test_regression_europe_kiev() {
    test_data_for_id("Europe/Kiev");
}

#[test]
fn test_regression_europe_sofia() {
    test_data_for_id("Europe/Sofia");
}

#[test]
fn test_regression_europe_vilnius() {
    test_data_for_id("Europe/Vilnius");
}

#[test]
fn test_regression_pacific_norfolk() {
    test_data_for_id("Pacific/Norfolk");
}

// Full IANA database coverage — failures collected and reported together.
#[test]
fn test_all_zones() {
    let tzdata_dir = match std::env::var("TZDATA_DIR") {
        Ok(dir) => dir,
        Err(_) => {
            eprintln!("SKIP test_all_zones: TZDATA_DIR not set (run `mise run zoneinfo:test`)");
            return;
        }
    };

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let data_dir = manifest_dir.join("tests").join("data");

    let zoneinfo_data = ZoneInfoData::from_zoneinfo_directory(&tzdata_dir).unwrap();
    let mut zone_names: Vec<String> = zoneinfo_data.zones.keys().cloned().collect();
    zone_names.sort();

    let mut compiler = ZoneInfoCompiler::new(zoneinfo_data);
    let mut failures: Vec<String> = Vec::new();

    for identifier in &zone_names {
        if let Some(error) = check_zone(identifier, &mut compiler, &data_dir) {
            failures.push(error);
        }
    }

    if !failures.is_empty() {
        panic!(
            "{} zone(s) failed:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}
