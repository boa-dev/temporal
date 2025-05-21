//! The available name provider

use alloc::borrow::Cow;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use zerotrie::{ZeroAsciiIgnoreCaseTrie, ZeroTrieBuildError};
use zerovec::{VarZeroVec, ZeroVec};

/// A data struct for IANA identifier normalization
#[derive(PartialEq, Debug, Clone, yoke::Yokeable, serde::Serialize, databake::Bake)]
#[databake(path = timezone_provider)]
#[derive(serde::Deserialize)]
pub struct IanaIdentifierNormalizer<'data> {
    /// TZDB version
    pub version: Cow<'data, str>,
    /// An index to the location of the normal identifier.
    #[serde(borrow)]
    pub available_id_index: ZeroAsciiIgnoreCaseTrie<ZeroVec<'data, u8>>,

    /// The normalized IANA identifier
    #[serde(borrow)]
    pub normalized_identifiers: VarZeroVec<'data, str>,
}

// ==== Begin DataProvider impl ====

#[derive(Debug)]
pub enum IanaDataError {
    #[cfg(feature = "std")]
    Io(std::io::Error),
    Build(ZeroTrieBuildError),
}

impl IanaIdentifierNormalizer<'_> {
    #[cfg(feature = "std")]
    pub fn build(tzdata: &std::path::Path) -> Result<Self, IanaDataError> {
        let provider = crate::tzdb::TzdbDataSource::new(tzdata).unwrap();
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
            version: provider.version.into(),
            available_id_index: ZeroAsciiIgnoreCaseTrie::try_from(&identier_map)
                .map_err(IanaDataError::Build)?
                .convert_store(),
            normalized_identifiers: norm_zerovec,
        })
    }
}

// ==== End DataProvider impl ====
