//! Implementation of the `tzif` data struct
//!
//! Please note this currently only supports the minimal required
//! fields in order to implement a TZif.

// TODO: Look into upstreaming to `tzif`.
// TODO: Potentially add some serialization scheme?

use alloc::{string::String, vec::Vec};
use indexmap::IndexSet;

use crate::compiler::CompiledTransitions;

/// A version 2 TZif block.
///
/// Please note: implementation is very minimal
#[derive(Debug)]
pub struct TzifBlockV2 {
    pub transition_times: Vec<i64>,
    pub transition_types: Vec<u8>,
    pub local_time_types: Vec<LocalTimeRecord>, // TODO: Add other fields as needed
    pub designations: String,
}

impl TzifBlockV2 {
    pub fn from_transition_data(data: &CompiledTransitions) -> Self {
        let mut local_time_set = IndexSet::new();
        let mut designation_set = DesignationSet::default();
        let index =
            designation_set.insert_and_retrieve_index(data.initial_record.designation.clone());
        local_time_set.insert(LocalTimeRecord {
            offset: data.initial_record.offset,
            is_dst: data.initial_record.saving.as_secs() != 0,
            index: index as u8,
        });
        let mut transition_times = Vec::default();
        let mut transition_types = Vec::default();
        for transition in &data.transitions {
            let index = designation_set.insert_and_retrieve_index(transition.format.clone());
            let local_time_record = LocalTimeRecord {
                offset: transition.offset,
                is_dst: transition.dst,
                index: index as u8,
            };

            transition_times.push(transition.at_time);
            match local_time_set.get_index_of(&local_time_record) {
                Some(i) => transition_types.push(i as u8),
                None => {
                    let _ = local_time_set.insert(local_time_record);
                    transition_types.push(local_time_set.len() as u8 - 1);
                }
            }
        }

        let local_time_types = local_time_set.into_iter().collect::<Vec<LocalTimeRecord>>();

        let designations = designation_set.to_designations_string();

        Self {
            transition_times,
            transition_types,
            local_time_types,
            designations,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct DesignationSet {
    pub designations: IndexSet<String>,
    pub indices: Vec<usize>,
    pub next_index: usize,
}

impl DesignationSet {
    // Inserts the a designation if it doesn't exist, returns the designation index.
    pub fn insert_and_retrieve_index(&mut self, designation: String) -> usize {
        // Check if the designation already exists.
        let Some(index) = self.designations.get_index_of(&designation) else {
            // Add one for '\0'
            let designation_len = designation.len() + 1;

            // Insert the new designation into the set
            let _ = self.designations.insert(designation);

            // Get the designation index and cache it.
            let designation_index = self.next_index;
            self.indices.push(designation_index);

            // Calculate the next index to give out.
            self.next_index += designation_len;

            return designation_index;
        };
        self.indices[index]
    }

    pub fn to_designations_string(self) -> String {
        let mut output = String::new();
        for designation in self.designations {
            let nul_terminated_designation = designation + "\0";
            output.push_str(&nul_terminated_designation);
        }
        output
    }
}

// TODO: Add index field for abbr. if supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalTimeRecord {
    pub offset: i64,
    pub is_dst: bool,
    pub index: u8,
}
