//! Implementation of the `tzif` data struct.
//!
//! Please note this currently only supports the minimal required
//! fields in order to implement a TZif.

// TODO: Potentially add some serialization scheme?

use alloc::{collections::btree_set::BTreeSet, vec::Vec};
use hashbrown::HashSet;

use crate::types::Transition;

/// A version 2 TZif block.
///
/// Please note: implementation is very minimal
#[derive(Debug)]
pub struct TzifBlockV2 {
    pub transition_times: Vec<i64>,
    pub transition_types: Vec<u8>,
    pub local_time_types: Vec<LocalTimeRecord>, // TODO: Add other fields as needed
}

impl TzifBlockV2 {
    pub fn from_transition_set(set: &BTreeSet<Transition>) -> Self {
        let mut local_time_set = HashSet::new();
        let mut transition_times = Vec::default();
        let mut transition_types = Vec::default();
        for transition in set {
            let _ = local_time_set.insert(LocalTimeRecord {
                offset: transition.offset,
                is_dst: transition.dst,
            });

            transition_times.push(transition.at_time);
            for (index, time_type) in local_time_set.iter().enumerate() {
                if time_type.offset == transition.offset {
                    transition_types.push(index as u8);
                }
            }
        }

        let local_time_types: Vec<LocalTimeRecord> = local_time_set.iter().cloned().collect();

        Self {
            transition_times,
            transition_types,
            local_time_types,
        }
    }
}

// TODO: Add index field for abbr. if supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalTimeRecord {
    pub offset: i64,
    pub is_dst: bool,
}
