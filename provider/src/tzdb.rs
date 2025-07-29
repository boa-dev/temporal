//! `timezone_provider` is the core data provider implementations for `temporal_rs`

// What are we even doing here? Why are providers needed?
//
// Two core data sources need to be accounted for:
//
//   - IANA identifier normalization (hopefully, semi easy)
//   - IANA TZif data (much harder)
//

use alloc::borrow::Cow;

#[cfg(feature = "datagen")]
use alloc::string::String;
#[cfg(feature = "datagen")]
use alloc::vec::Vec;

#[cfg(feature = "datagen")]
use std::{
    borrow::ToOwned,
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
};

use zerotrie::ZeroAsciiIgnoreCaseTrie;
use zerovec::{VarZeroVec, ZeroVec};
#[cfg(feature = "datagen")]
use zoneinfo_rs::{ZoneInfoData, ZoneInfoError};

/// A data struct for IANA identifier normalization
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(
    feature = "datagen",
    derive(serde::Serialize, yoke::Yokeable, serde::Deserialize, databake::Bake)
)]
#[cfg_attr(feature = "datagen", databake(path = timezone_provider))]
pub struct IanaIdentifierNormalizer<'data> {
    /// TZDB version
    pub version: Cow<'data, str>,
    /// An index to the location of the normal identifier.
    #[cfg_attr(feature = "datagen", serde(borrow))]
    pub available_id_index: ZeroAsciiIgnoreCaseTrie<ZeroVec<'data, u8>>,

    /// The normalized IANA identifier
    #[cfg_attr(feature = "datagen", serde(borrow))]
    pub normalized_identifiers: VarZeroVec<'data, str>,
}

// ==== End Data marker implementation ====

#[derive(Debug)]
#[cfg(feature = "datagen")]
pub enum TzdbDataSourceError {
    Io(io::Error),
    ZoneInfo(ZoneInfoError),
}

#[cfg(feature = "datagen")]
impl From<io::Error> for TzdbDataSourceError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

#[cfg(feature = "datagen")]
impl From<ZoneInfoError> for TzdbDataSourceError {
    fn from(value: ZoneInfoError) -> Self {
        Self::ZoneInfo(value)
    }
}

#[cfg(feature = "datagen")]
pub struct TzdbDataSource {
    pub version: String,
    pub data: ZoneInfoData,
}

#[cfg(feature = "datagen")]
impl TzdbDataSource {
    /// Try to create a tzdb source from a tzdata directory.
    pub fn try_from_zoneinfo_directory(tzdata_path: &Path) -> Result<Self, TzdbDataSourceError> {
        let version_file = tzdata_path.join("version");
        let version = fs::read_to_string(version_file)?.trim().to_owned();
        let data = ZoneInfoData::from_zoneinfo_directory(tzdata_path)?;
        Ok(Self { version, data })
    }

    /// Try to create a tzdb source from a tzdata rearguard.zi
    ///
    /// To generate a rearguard.zi, download tzdata from IANA. Run `make rearguard.zi`
    pub fn try_from_rearguard_zoneinfo_dir(tzdata_path: &Path) -> Result<Self, TzdbDataSourceError> {
        let version_file = tzdata_path.join("version");
        let version = fs::read_to_string(version_file)?.trim().to_owned();
        let rearguard_zoneinfo = tzdata_path.join("rearguard.zi");
        let data = ZoneInfoData::from_filepath(rearguard_zoneinfo)?;
        Ok(Self { version, data })
    }
}

// ==== Begin DataProvider impl ====

#[derive(Debug)]
#[cfg(feature = "datagen")]
pub enum IanaDataError {
    Io(io::Error),
    Provider(TzdbDataSourceError),
    Build(zerotrie::ZeroTrieBuildError),
}

#[cfg(feature = "datagen")]
impl IanaIdentifierNormalizer<'_> {
    pub fn build(tzdata_path: &Path) -> Result<Self, IanaDataError> {
        let provider = TzdbDataSource::try_from_zoneinfo_directory(tzdata_path)
            .map_err(IanaDataError::Provider)?;
        let mut canonical = BTreeSet::default();
        for zone_id in provider.data.zones.keys() {
            // Add canonical identifiers.
            let _ = canonical.insert(zone_id.clone());
        }
        let mut all_identifiers = canonical.clone();

        for link_from in provider.data.links.keys() {
            // Add link / non-canonical identifiers
            let _ = all_identifiers.insert(link_from.clone());
        }
        // Make a sorted list of canonical timezones
        let norm_vec: Vec<String> = canonical.iter().cloned().collect();
        let norm_zerovec: VarZeroVec<'static, str> = norm_vec.as_slice().into();

        let identifier_map: BTreeMap<Vec<u8>, usize> = all_identifiers
            .iter()
            .map(|id| {
                // Either this is already canonical
                let normalized_id = norm_vec.binary_search(id);
                // ... or it is an alias from the links table, fetch the
                // canonical id corresponding to it
                let normalized_id = normalized_id
                    .unwrap_or_else(|_| norm_vec.binary_search(&provider.data.links[id]).unwrap());
                (id.to_ascii_lowercase().as_bytes().to_vec(), normalized_id)
            })
            .collect();

        Ok(IanaIdentifierNormalizer {
            version: provider.version.into(),
            available_id_index: ZeroAsciiIgnoreCaseTrie::try_from(&identifier_map)
                .map_err(IanaDataError::Build)?
                .convert_store(),
            normalized_identifiers: norm_zerovec,
        })
    }
}

// ==== End DataProvider impl ====
