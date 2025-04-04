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
use zoneinfo_compiler::{TransitionData, ZoneInfoCompileSettings};

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
    transition_types: ZeroVec<'data, u8>,
    // NOTE: zoneinfo64 does a fun little bitmap str
    types: ZeroVec<'data, LocalTimeRecord>,
    posix: Cow<'data, str>,
}

#[zerovec::make_ule(LocalTimeRecordULE)]
#[derive(
    PartialEq,
    Eq,
    Debug,
    Clone,
    Copy,
    PartialOrd,
    Ord,
    yoke::Yokeable,
    serde::Serialize,
    databake::Bake,
)]
#[databake(path = temporal_provider::tzif)]
pub struct LocalTimeRecord {
    offset: i64,
    is_dst: bool,
}

impl From<&zoneinfo_compiler::tzif::LocalTimeRecord> for LocalTimeRecord {
    fn from(value: &zoneinfo_compiler::tzif::LocalTimeRecord) -> Self {
        Self {
            offset: value.offset,
            is_dst: value.is_dst,
        }
    }
}

impl ZeroTzif<'_> {
    fn from_transition_data(data: &TransitionData) -> Self {
        let tzif = data.to_v2_data_block();
        let transitions = ZeroVec::alloc_from_slice(&tzif.transition_times);
        let transition_types = ZeroVec::alloc_from_slice(&tzif.transition_types);
        let mapped_local_records: Vec<LocalTimeRecord> =
            tzif.local_time_types.iter().map(Into::into).collect();
        let types = ZeroVec::alloc_from_slice(&mapped_local_records);
        let posix = String::from("TODO").into();

        Self {
            transitions,
            transition_types,
            types,
            posix,
        }
    }
}

pub enum ZoneInfoDataError {
    Build(ZeroTrieBuildError),
}

impl ZeroZoneInfo<'_> {
    pub fn build(tzdata: &Path) -> Result<Self, ZoneInfoDataError> {
        let mut provider = TzdbDataProvider::try_from_zoneinfo_directory(tzdata).unwrap();
        let mut identifiers = BTreeMap::default();
        let mut zones_set = BTreeSet::default();

        let zoneinfo_compiled = provider
            .zone_info
            .associate_and_build(ZoneInfoCompileSettings::default());

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
            .map(|id| {
                let data = zoneinfo_compiled
                    .data
                    .get(id)
                    .expect("all zones should be built");
                ZeroTzif::from_transition_data(data)
            })
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
