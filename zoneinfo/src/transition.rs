//! Zone transition data and accumulation.
//!
//! [`TransitionAccumulator`] collects transitions during zone compilation and
//! [`TransitionAccumulator::merge`] produces the final [`MergedTransitions`].

use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
};

use crate::types::{QualifiedTimeKind, Time};

// TODO: improve `Transition` repr this type provides a lot of
// information by design, but the local time record data
// should be separated from the transition info with a clear
// separation.
//
// EX:
// pub struct Transition {
//     /// The time to transition at
//     pub at_time: i64,
//     /// The transition time kind.
//     pub time_type: QualifiedTimeKind,
//     /// LocalTimeRecord transitioned into
//     pub to_local: ZoneInfoLocalTimeRecord,
// }
//
/// The primary transition data.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Transition {
    /// The time to transition at
    ///
    /// This represents the time in Unix Epoch seconds
    /// at which a transition should occur.
    pub at_time: i64,
    /// The transition time kind.
    ///
    /// Whether the transition was specified in Local, Standard, or Universal time.
    pub time_type: QualifiedTimeKind,

    // TODO: Below are fields that should be split into a
    // currently non-existent LocalTime record.
    /// The offset of the transition.
    pub offset: i64,
    /// Whether the transition is a savings offset or not
    ///
    /// This flag corresponds to the `is_dst` flag
    pub dst: bool,
    /// The savings for the local time record
    ///
    /// This field represents the exact [`Time`] value
    /// used for savings.
    pub savings: Time,
    /// The letter designation for the local time record
    ///
    /// The LETTER designation used in the fully formatted
    /// abbreviation
    pub letter: Option<String>,
    /// The abbreviation format for the local time record.
    pub format: String,
}

impl Ord for Transition {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.at_time.cmp(&other.at_time)
    }
}

impl PartialOrd for Transition {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Merged transitions: sorted by `at_time`, with no two consecutive entries
/// sharing the same `(offset, dst, format)` unless pinned.
///
/// Only constructible via [`TransitionAccumulator::merge`].
#[non_exhaustive]
#[derive(Debug, Default, PartialEq)]
pub struct MergedTransitions(BTreeSet<Transition>);

impl MergedTransitions {
    pub fn iter(&self) -> impl Iterator<Item = &Transition> {
        self.0.iter()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a> IntoIterator for &'a MergedTransitions {
    type Item = &'a Transition;
    type IntoIter = alloc::collections::btree_set::Iter<'a, Transition>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// Whether a transition may be collapsed by the merge step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MergeEligibility {
    /// A zone-line boundary transition. Collapsible when its state matches
    /// the preceding transition.
    Eligible,
    /// A rule transition, or a zone-line boundary whose instant coincides with
    /// a rule transition. Always retained.
    Pinned,
}

/// Accumulates raw transitions during zone compilation.
///
/// Keyed by `at_time`, so transitions stay sorted and no two share an instant
/// (first insert wins). Consecutive same-state entries are permitted; call
/// [`merge`](Self::merge) to produce [`MergedTransitions`].
pub(crate) struct TransitionAccumulator(BTreeMap<i64, (Transition, MergeEligibility)>);

impl TransitionAccumulator {
    pub(crate) fn new() -> Self {
        Self(BTreeMap::default())
    }

    pub(crate) fn insert(&mut self, t: Transition, eligibility: MergeEligibility) {
        let _ = self.0.entry(t.at_time).or_insert((t, eligibility));
    }

    pub(crate) fn last(&self) -> Option<&Transition> {
        self.0.last_key_value().map(|(_, (t, _))| t)
    }

    /// Collapse runs of merge-eligible transitions that share the preceding
    /// `(offset, dst, format)`.
    pub(crate) fn merge(self) -> MergedTransitions {
        let mut inner = BTreeSet::default();
        let mut prev: Option<(i64, bool, String)> = None;
        for (_, (t, eligibility)) in self.0 {
            let same = prev.as_ref().is_some_and(|(off, dst, fmt)| {
                *off == t.offset && *dst == t.dst && fmt == &t.format
            });
            if eligibility == MergeEligibility::Pinned || !same {
                prev = Some((t.offset, t.dst, t.format.clone()));
                inner.insert(t);
            }
        }
        MergedTransitions(inner)
    }
}
