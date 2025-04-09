//! `temporal_provider` is the core data provider implementations for `temporal_rs`

// What are we even doing here? Why are providers needed?
//
// Two core data sources need to be accounted for:
//
//   - IANA identifier normalization (hopefully, semi easy)
//   - IANA TZif data (much harder)
//

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
};

use zerotrie::{ZeroAsciiIgnoreCaseTrie, ZeroTrieBuildError};
use zerovec::{VarZeroVec, ZeroVec};
use zoneinfo_compiler::{ZoneInfoCompiler, ZoneInfoError};

/// A data struct for IANA identifier normalization
#[derive(PartialEq, Debug, Clone, yoke::Yokeable, serde::Serialize, databake::Bake)]
#[databake(path = temporal_provider)]
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

// ==== End Data marker implementation ====

#[derive(Debug)]
pub enum TzdbDataProviderError {
    Io(io::Error),
    ZoneInfo(ZoneInfoError),
}

impl From<io::Error> for TzdbDataProviderError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<ZoneInfoError> for TzdbDataProviderError {
    fn from(value: ZoneInfoError) -> Self {
        Self::ZoneInfo(value)
    }
}

pub struct TzdbDataProvider {
    pub version: String,
    pub zone_info: ZoneInfoCompiler,
}

impl TzdbDataProvider {
    pub fn try_from_zoneinfo_directory(tzdata_path: &Path) -> Result<Self, TzdbDataProviderError> {
        let version_file = tzdata_path.join("version");
        let version = fs::read_to_string(version_file)?.trim().to_owned();
        let zone_info = ZoneInfoCompiler::from_zoneinfo_directory(tzdata_path)?;
        Ok(Self { version, zone_info })
    }
}

// ==== Begin DataProvider impl ====

#[derive(Debug)]
pub enum IanaDataError {
    Io(io::Error),
    Build(ZeroTrieBuildError),
    Provider(TzdbDataProviderError),
}

impl IanaIdentifierNormalizer<'_> {
    pub fn build(tzdata_path: &Path) -> Result<Self, IanaDataError> {
        let provider = TzdbDataProvider::try_from_zoneinfo_directory(tzdata_path)
            .map_err(IanaDataError::Provider)?;
        let mut identifiers = BTreeSet::default();
        for zone_id in provider.zone_info.zones.keys() {
            // Add canonical identifiers.
            let _ = identifiers.insert(zone_id.clone());
        }
        for links in provider.zone_info.links.keys() {
            // Add link / non-canonical identifiers
            let _ = identifiers.insert(links.clone());
        }
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
