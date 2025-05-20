//! `timezone_provider` is the core data provider implementations for `temporal_rs`

// What are we even doing here? Why are providers needed?
//
// Two core data sources need to be accounted for:
//
//   - IANA identifier normalization (hopefully, semi easy)
//   - IANA TZif data (much harder)
//

use alloc::string::String;

use std::{fs, io, path::Path};

use parse_zoneinfo::{
    line::{Line, LineParser},
    table::{Table, TableBuilder},
};

const ZONE_INFO_FILES: [&str; 9] = [
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

pub struct TzdbDataSource {
    pub version: String,
    pub data: Table,
}

impl TzdbDataSource {
    pub fn new(tzdata: &Path) -> Result<Self, io::Error> {
        let parser = LineParser::default();
        let mut builder = TableBuilder::default();

        let version_file = tzdata.join("version");
        let version = fs::read_to_string(version_file)?.trim().into();

        for filename in ZONE_INFO_FILES {
            let file_path = tzdata.join(filename);
            let file = fs::read_to_string(file_path)?;

            for line in file.lines() {
                match parser.parse_str(line) {
                    Ok(Line::Zone(zone)) => builder.add_zone_line(zone).unwrap(),
                    Ok(Line::Continuation(cont)) => builder.add_continuation_line(cont).unwrap(),
                    Ok(Line::Rule(rule)) => builder.add_rule_line(rule).unwrap(),
                    Ok(Line::Link(link)) => builder.add_link_line(link).unwrap(),
                    Ok(Line::Space) => {}
                    Err(e) => std::eprintln!("{e}"),
                }
            }
        }

        Ok(Self {
            version,
            data: builder.build(),
        })
    }
}
