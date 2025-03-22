//! `temporal_provider` is the core data provider implementations for `temporal_rs`

// TODO: What are we even doing here?
//
// Two core data sources need to be accounted for:
//
//   - IANA identifier normalization (hopefully, semi easy)
//   - IANA TZif data (much harder)
//

// ==== Data Marker implementation ====

// NOTE: A data_struct and data_marker would typically be in
// a different crate. This may need to be moved into `temporal_rs`,
// but that remains to be determined.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
};

use parse_zoneinfo::{
    line::{Line, LineParser},
    table::{Table, TableBuilder},
};
use zerotrie::{ZeroAsciiIgnoreCaseTrie, ZeroTrieBuildError};
use zerovec::{VarZeroVec, ZeroVec};

// TODO: Potentially update; may require further cfg attributes
#[derive(PartialEq, Debug, Clone, yoke::Yokeable, serde::Serialize, databake::Bake)]
#[databake(path = temporal_provider)]
#[derive(serde::Deserialize)]
pub struct IanaIdentifierNormalizer<'data> {
    // Q: Can ZeroAsciiIgnoreCaseTrie have an inner store that is `VarZeroVec`
    /// An index to the location of the normal identifier.
    #[serde(borrow)]
    pub available_id_index: ZeroAsciiIgnoreCaseTrie<ZeroVec<'data, u8>>,

    /// The normalized IANA identifier
    #[serde(borrow)]
    pub normalized_identifiers: VarZeroVec<'data, str>,
}

// ==== End Data marker implementation ====

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

pub struct TzdbDataProvider {
    data: Table,
}

impl TzdbDataProvider {
    pub fn new() -> Result<Self, io::Error> {
        let parser = LineParser::default();
        let mut builder = TableBuilder::default();

        for filename in ZONE_INFO_FILES {
            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            let file_path = format!("{manifest_dir}/tzdata/{filename}");
            let file = fs::read_to_string(file_path)?;

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

        Ok(Self {
            data: builder.build(),
        })
    }
}

// ==== Begin DataProvider impl ====

#[derive(Debug)]
pub enum IanaDataError {
    Io(io::Error),
    Build(ZeroTrieBuildError),
}

impl IanaIdentifierNormalizer<'_> {
    pub fn build() -> Result<Self, IanaDataError> {
        let provider = TzdbDataProvider::new().unwrap();
        let mut identifiers = BTreeSet::default();
        for zoneset_id in provider.data.zonesets.keys() {
            // Add canonical identifiers.
            let _ = identifiers.insert(zoneset_id.clone());
        }
        for links in provider.data.links.keys() {
            // Add link / non-canonical identifiers
            let _ = identifiers.insert(links.clone());
        }

        // Create trie and bin search the index from Vec
        let norm_vec: Vec<String> = identifiers.iter().cloned().collect();
        let norm_zerovec: VarZeroVec<'static, str> = norm_vec.as_slice().into();

        let identier_map: BTreeMap<Vec<u8>, usize> = identifiers
            .iter()
            .map(|id| {
                (
                    id.to_ascii_lowercase().as_bytes().to_vec(),
                    norm_vec.binary_search(id).unwrap(),
                )
            })
            .collect();

        Ok(IanaIdentifierNormalizer {
            available_id_index: ZeroAsciiIgnoreCaseTrie::try_from(&identier_map)
                .map_err(IanaDataError::Build)?
                .convert_store(),
            normalized_identifiers: norm_zerovec,
        })
    }
}

// ==== End DataProvider impl ====
