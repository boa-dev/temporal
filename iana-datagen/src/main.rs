use std::fs;

use parse_zoneinfo::{line::{Line, LineParser}, table::TableBuilder};

const ZONE_INFO_FILES: [&str; 9] =  [
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

const PROJECT_BASE: &str = "iana-datagen/tzdata/";

fn main() {

    let parser = LineParser::default();
    let mut builder = TableBuilder::default();

    for filename in ZONE_INFO_FILES {
        let file_path = format!("{PROJECT_BASE}{filename}");
        let file = fs::read_to_string(file_path).unwrap();

        for line in file.lines() {
            match parser.parse_str(line) {
                Ok(Line::Zone(zone)) => builder.add_zone_line(zone).unwrap(),
                Ok(Line::Continuation(cont)) => builder.add_continuation_line(cont).unwrap(),
                Ok(Line::Rule(rule)) => builder.add_rule_line(rule).unwrap(),
                Ok(Line::Link(link)) => builder.add_link_line(link).unwrap(),
                Ok(Line::Space) => {}
                Err(e) => eprintln!("{e}"),
            }
        }

    }
    let table = builder.build();

    for (identifier, zoneinfo) in table.zonesets {
        println!("Identifier: {identifier}");
        println!("{:#?}", zoneinfo);
    }
    // println!("Zonesets");
    // for (identifier, zoneinfo) in table.zonesets {
    //     // println!("{identifier:?}");
    //     println!("{:?}", zoneinfo[0])
    // }

    println!("rulesets");
    for (identifier, ruleinfo) in table.rulesets {
        println!("{identifier:?}");
        println!("{:#?}", ruleinfo)
    }

    // println!("links");
    // for (identifier, file) in table.links {
    //     println!("{identifier:?} {file:?}");
    // }
}
