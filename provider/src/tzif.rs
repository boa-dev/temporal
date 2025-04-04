//! A compact, zero copy TZif file.
//!
//! NOTE: This representation does not follow the TZif specification
//! to full detail, but instead attempts to compress TZif data into
//! a functional, data driven equivalent.

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use zerotrie::{ZeroTrieBuildError, ZeroTrieSimpleAscii};
use zerovec::{VarZeroVec, ZeroVec};
use zoneinfo_compiler::ZoneInfo;

use crate::tzdb::TzdbDataProvider;

#[derive(Debug, Clone, yoke::Yokeable, databake::Bake, serde::Serialize)]
#[databake(path = temporal_provider::tzif)]
pub struct ZeroZoneInfo<'data> {
    // Why u16? It would suck to have to refactor because there are > 256 TZifs
    ids: ZeroTrieSimpleAscii<ZeroVec<'data, u8>>,

    tzifs: VarZeroVec<'data, ZeroTzifULE>,
}

#[zerovec::make_varule(ZeroTzifULE)]
#[derive(PartialEq, Debug, Clone, yoke::Yokeable, serde::Serialize, databake::Bake)]
#[zerovec::skip_derive(Ord)]
#[zerovec::derive(Debug, Serialize)]
#[databake(path = temporal_provider::tzif)]
pub struct ZeroTzif<'data> {
    transitions: ZeroVec<'data, i64>,
    // NOTE: zoneinfo64 does a fun little bitmap str
    transition_types: ZeroVec<'data, u32>,
    types: ZeroVec<'data, i64>,
    posix: Cow<'data, str>,
}

pub enum ZoneInfoDataError {
    Build(ZeroTrieBuildError),
}

impl ZeroZoneInfo<'_> {
    pub fn build(tzdata: &Path) -> Result<Self, ZoneInfoDataError> {
        let provider = TzdbDataProvider::try_from_zoneinfo_directory(tzdata).unwrap();
        let mut identifiers = BTreeMap::default();
        let mut zones_set = BTreeSet::default();
        for zone_identifier in provider.zone_info.zones.keys() {
            let _ = zones_set.insert(zone_identifier.clone());
            identifiers.insert(zone_identifier.clone(), zone_identifier.clone());
        }
        for (link, zone) in provider.zone_info.links.iter() {
            identifiers.insert(link.clone(), zone.clone());
        }

        let zones: Vec<String> = zones_set.iter().cloned().collect();

        let identier_map: BTreeMap<Vec<u8>, usize> = identifiers
            .iter()
            .map(|(id, zoneid)| (id.as_bytes().to_vec(), zones.binary_search(zoneid).unwrap()))
            .collect();

        let tzifs: Vec<ZeroTzif<'_>> = zones
            .iter()
            .map(|id| ZeroTzif::build(&provider.zone_info, id))
            .collect();

        let tzifs_zerovec: VarZeroVec<'static, ZeroTzifULE> = tzifs.as_slice().into();

        let ids = ZeroTrieSimpleAscii::try_from(&identier_map)
            .map_err(ZoneInfoDataError::Build)?
            .convert_store();

        Ok(ZeroZoneInfo {
            ids,
            tzifs: tzifs_zerovec,
        })
    }
}

impl ZeroTzif<'_> {
    pub fn build(_data: &ZoneInfo, _id: &str) -> Self {
        todo!()
    }
}
