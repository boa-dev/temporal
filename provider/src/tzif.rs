//! A compact, zero copy TZif file.
//!
//! NOTE: This representation does not follow the TZif specification
//! to full detail, but instead attempts to compress TZif data into
//! a functional, data driven equivalent.

use std::{borrow::Cow, collections::BTreeMap, path::Path};
use zerotrie::{ZeroAsciiIgnoreCaseTrie, ZeroTrieBuildError};
use zerovec::{vecs::Index32, VarZeroVec, ZeroVec};
use zoneinfo_compiler::ZoneInfoTransitionData;

use crate::tzdb::TzdbDataSource;

#[derive(Debug, Clone, yoke::Yokeable, databake::Bake, serde::Serialize)]
#[databake(path = temporal_provider::tzif)]
pub struct ZoneInfoProvider<'data> {
    // IANA identifier map to TZif index.
    pub ids: ZeroAsciiIgnoreCaseTrie<ZeroVec<'data, u8>>,
    // Vector of TZif data
    pub tzifs: VarZeroVec<'data, ZeroTzifULE, Index32>,
}

#[zerovec::make_varule(ZeroTzifULE)]
#[derive(PartialEq, Debug, Clone, yoke::Yokeable, serde::Serialize, databake::Bake)]
#[zerovec::skip_derive(Ord)]
#[zerovec::derive(Debug, Serialize)]
#[databake(path = temporal_provider::tzif)]
pub struct ZeroTzif<'data> {
    pub transitions: ZeroVec<'data, i64>,
    pub transition_types: ZeroVec<'data, u8>,
    // NOTE: zoneinfo64 does a fun little bitmap str
    pub types: ZeroVec<'data, LocalTimeRecord>,
    pub posix: Cow<'data, str>,
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
    pub offset: i64,
    pub is_dst: bool,
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
    fn from_transition_data(data: &ZoneInfoTransitionData) -> Self {
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

#[derive(Debug)]
pub enum ZoneInfoDataError {
    Build(ZeroTrieBuildError),
}

impl ZoneInfoProvider<'_> {
    pub fn build(tzdata: &Path) -> Result<Self, ZoneInfoDataError> {
        let mut tzdb_source = TzdbDataSource::try_from_zoneinfo_directory(tzdata).unwrap();
        let compiled_transitions = tzdb_source.compiler.build();

        let mut identifiers = BTreeMap::default();
        let mut zones = Vec::default();

        // Create a Map of <ZoneId | Link, ZoneId>, this is used later to index
        for zone_identifier in tzdb_source.compiler.zones.keys() {
            zones.push(zone_identifier.clone());
            identifiers.insert(zone_identifier.clone(), zone_identifier.clone());
        }
        for (link, zone) in tzdb_source.compiler.links.iter() {
            identifiers.insert(link.clone(), zone.clone());
        }

        let identier_map: BTreeMap<Vec<u8>, usize> = identifiers
            .iter()
            .map(|(id, zoneid)| {
                (
                    id.to_ascii_lowercase().as_bytes().to_vec(),
                    zones.binary_search(zoneid).unwrap(),
                )
            })
            .collect();

        let tzifs: Vec<ZeroTzif<'_>> = zones
            .iter()
            .map(|id| {
                let data = compiled_transitions
                    .data
                    .get(id)
                    .expect("all zones should be built");
                ZeroTzif::from_transition_data(data)
            })
            .collect();

        let tzifs_zerovec: VarZeroVec<'static, ZeroTzifULE, Index32> = tzifs.as_slice().into();

        let ids = ZeroAsciiIgnoreCaseTrie::try_from(&identier_map)
            .map_err(ZoneInfoDataError::Build)?
            .convert_store();

        Ok(ZoneInfoProvider {
            ids,
            tzifs: tzifs_zerovec,
        })
    }
}
