
use std::env;
use temporal_rs::{TimeZone, ZonedDateTime};
use timezone_provider::{experimental_tzif::{ZeroZoneInfo, ZeroZoneInfoProvider}, provider::TimeZoneResolver};

macro_rules! format_line(
    ($arr:ident[$i:expr], $($args:expr),*) => {
        let string = stringify!($arr);
        let array = format!("{}[{}]", string, $i);
        format_line!(array, $($args),*)
    };
    ($a:expr, $b:expr, $c: expr, $d: expr) => {
        println!("{:<25} {:<20} {:<5} {}", $a, $b, $c, $d)
    };
    ($a:expr, $b:expr, $c: expr) => {
        println!("{:<25} {:<20} {}", $a, $b, $c)
    };
    ($a:expr, $b:expr) => {
        println!("{:<25} {}", $a, $b)
    };
);

fn main() {
    let tz = env::args().nth(1).expect("Needs one argument");
    let provider = ZeroZoneInfoProvider::default();
    // Create zoneinfo
    let zoneinfo = ZeroZoneInfo::default();

    // Get tzif data
    let resolved_id = zoneinfo.get_id(tz.as_bytes()).unwrap();
    let tzif = zoneinfo.zero_tzif(resolved_id).unwrap();

    format_line!("Index", "Transition", "Local type", "Datetime");
    for (index, transition) in tzif.transitions.iter().enumerate() {
        let type_index = tzif.transition_types.get(index).expect("must exist");
        
        let time_zone = TimeZone::try_from_identifier_str_with_provider(&tz, &provider).unwrap();
        let zdt = ZonedDateTime::try_new_iso_with_provider(transition as i128 * 1_000_000_000, time_zone, &provider).unwrap();

        format_line!(format!("transition[{index}]"), transition, type_index, zdt.to_string_with_provider(&provider).unwrap())
    }

    println!("");

    let mut index = 0;
    for designation in tzif.designations.into_owned().split('\0') {
        // Ignore the hanging nul terminator
        if !designation.is_empty() {
            println!("designations[{index}]: {designation}");
            index += designation.len() + 1
        }
    }

    println!("");

    for (index, local_type) in tzif.types.iter().enumerate() {
        println!("local_type[{index}]");
        println!("{local_type:#?}\n");
    }
}

