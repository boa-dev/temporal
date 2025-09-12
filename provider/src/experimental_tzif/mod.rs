//! A compact, zero copy TZif file.
//!
//! NOTE: This representation does not follow the TZif specification
//! to full detail, but instead attempts to compress TZif data into
//! a functional, data driven equivalent.

use core::{cmp::Ordering, ops::Range};

use tzif::data::time::Seconds;
use zerofrom::ZeroFrom;
use zerotrie::ZeroAsciiIgnoreCaseTrie;
use zerovec::{vecs::Index32, VarZeroVec, ZeroVec};

use posix::PosixZone;

use crate::epoch_nanoseconds::NS_IN_S;
use crate::provider::TransitionDirection;
use crate::tzif::{DstTransitionInfoForYear, TimeZoneTransitionInfo};
use crate::{self as timezone_provider, utils, TimeZoneProviderError};
use crate::{
    epoch_nanoseconds::EpochNanoseconds,
    provider::{GapEntryOffsets, TimeZoneProviderResult, UtcOffsetSeconds},
    tzif::LocalTimeRecordResult,
};

compiled_zoneinfo_provider!(COMPILED_ZONEINFO_PROVIDER);

#[cfg(feature = "datagen")]
mod datagen;
pub mod posix;
mod provider;

pub use provider::{ZeroZoneInfoProvider, ZeroZoneInfo};

#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "datagen",
    derive(yoke::Yokeable, serde::Serialize, databake::Bake)
)]
#[cfg_attr(feature = "datagen", databake(path = timezone_provider::experimental_tzif))]
pub struct ZoneInfoProvider<'data> {
    // IANA identifier map to TZif index.
    pub ids: ZeroAsciiIgnoreCaseTrie<ZeroVec<'data, u8>>,
    // Vector of TZif data
    pub tzifs: VarZeroVec<'data, ZeroTzifULE, Index32>,
}

impl ZoneInfoProvider<'_> {
    pub fn get(&self, identifier: &str) -> Option<&ZeroTzifULE> {
        let idx = self.ids.get(identifier)?;
        self.tzifs.get(idx)
    }
}

#[zerovec::make_varule(ZeroTzifULE)]
#[derive(PartialEq, Debug, Clone, ZeroFrom)]
#[zerovec::skip_derive(Ord)]
#[zerovec::derive(Debug)]
#[cfg_attr(
    feature = "datagen",
    derive(yoke::Yokeable, serde::Serialize, databake::Bake)
)]
#[cfg_attr(feature = "datagen", zerovec::derive(Serialize))]
#[cfg_attr(feature = "datagen", databake(path = timezone_provider::experimental_tzif))]
pub struct ZeroTzif<'data> {
    pub transitions: ZeroVec<'data, i64>,
    pub transition_types: ZeroVec<'data, u8>,
    // NOTE: zoneinfo64 does a fun little bitmap str
    pub types: ZeroVec<'data, LocalTimeRecord>,
    pub posix: PosixZone,
}

#[zerovec::make_ule(LocalTimeRecordULE)]
#[derive(PartialEq, Eq, Debug, Default, Clone, Copy, PartialOrd, Ord)]
#[cfg_attr(
    feature = "datagen",
    derive(yoke::Yokeable, serde::Serialize, databake::Bake)
)]
#[cfg_attr(feature = "datagen", databake(path = timezone_provider::experimental_tzif))]
pub struct LocalTimeRecord {
    pub offset: i64,
}

impl LocalTimeRecord {
    pub fn as_utc_offset_seconds(&self) -> UtcOffsetSeconds {
        UtcOffsetSeconds(self.offset)
    }
}

// TODO (nekevss): It would be nice to unify these in the `tzif` crate so that the tzif logic
// is centralized whther using the "zero" or normal version.

impl<'data> ZeroTzif<'data> {
    pub fn get_first_time_zone_offset(&self) -> TimeZoneTransitionInfo {
        let offset = self.types.first();
        debug_assert!(offset.is_some(), "tzif internal invariant violated");
        TimeZoneTransitionInfo {
            // There was no transition into the first timezone
            transition_epoch: None,
            offset: offset.unwrap_or_default().as_utc_offset_seconds(),
        }
    }

    pub fn get_time_zone_offset(&self, idx: usize) -> TimeZoneTransitionInfo {
        // NOTE: Transition type can be empty. If no transition_type exists,
        // then use 0 as the default index of local_time_type_records.
        let offset = self
            .types
            .get(self.transition_types.get(idx).unwrap_or(0) as usize);
        debug_assert!(offset.is_some(), "tzif internal invariant violated");
        TimeZoneTransitionInfo {
            transition_epoch: self.transitions.get(idx),
            offset: offset.unwrap_or_default().as_utc_offset_seconds(),
        }
    }

    pub fn get(&self, epoch_seconds: i64) -> TimeZoneProviderResult<TimeZoneTransitionInfo> {
        let result = self.transitions.binary_search(&epoch_seconds);

        match result {
            // The transition time was given. The transition entries *start* at their
            // transition time, so we use the same index
            Ok(idx) => Ok(self.get_time_zone_offset(idx)),
            // TODO: Double check how the below is handled by zoneinfo_rs
            // <https://datatracker.ietf.org/doc/html/rfc8536#section-3.2>
            // If there are no transitions, local time for all timestamps is specified by the TZ
            // string in the footer if present and nonempty; otherwise, it is
            // specified by time type 0.
            Err(_) if self.transitions.is_empty() => {
                if self.types.len() == 1 {
                    let local_time_record = self
                        .types
                        .first()
                        .ok_or(TimeZoneProviderError::Assert("Out of transition range"))?;
                    let offset = local_time_record.as_utc_offset_seconds();
                    Ok(TimeZoneTransitionInfo {
                        offset,
                        transition_epoch: None,
                    })
                } else {
                    // Resolve the POSIX time zone.
                    self.posix.resolve_for_epoch_seconds(epoch_seconds)
                }
            }
            // Our time is before the first transition.
            // Get the first timezone offset
            Err(0) => Ok(self.get_first_time_zone_offset()),
            // Our time is after some transition.
            Err(idx) => {
                if self.transitions.len() <= idx {
                    // The transition time provided is beyond the length of
                    // the available transition time, so the time zone is
                    // resolved with the POSIX tz string.
                    let mut offset = self.posix.resolve_for_epoch_seconds(epoch_seconds)?;
                    if offset.transition_epoch.is_none() {
                        offset.transition_epoch = Some(
                            self.transitions
                                .get(idx - 1)
                                .ok_or(TimeZoneProviderError::Assert("Out of transition range"))?,
                        )
                    }
                    return Ok(offset);
                }
                // binary_search returns the insertion index, which is one after the
                // index of the closest lower bound. Fetch that bound.
                Ok(self.get_time_zone_offset(idx - 1))
            }
        }
    }

    pub fn search_candidate_offset(
        &self,
        local_seconds: i64,
    ) -> TimeZoneProviderResult<LocalTimeRecordResult> {
        let b_search_result = self.transitions.binary_search(&local_seconds);

        let mut estimated_idx = match b_search_result {
            Ok(idx) => idx,
            Err(idx) => idx,
        };

        if estimated_idx + 1 >= self.transitions.len() {
            // If we are *well past* the last transition time, we want
            // to use the posix tz string
            let mut use_posix = true;
            if !self.transitions.is_empty() {
                // In case idx was out of bounds, bring it back in
                estimated_idx = self.transitions.len() - 1;
                let transition_info = self.get_transition_info(estimated_idx);

                // I'm not fully sure if this is correct.
                // Is the next_offset valid for the last transition time in its
                // vicinity? Probably? It does not seem pleasant to try and do this
                // math using half of the transition info and half of the posix info.
                //
                // TODO(manishearth, nekevss): https://github.com/boa-dev/temporal/issues/469
                if transition_info.transition_time_prev_epoch() > local_seconds
                    || transition_info.transition_time_next_epoch() > local_seconds
                {
                    // We're before the transition fully ends; we should resolve
                    // with the regular transition time instead of use_posix
                    use_posix = false;
                }
            }
            if use_posix {
                // The transition time provided is beyond the length of
                // the available transition time, so the time zone is
                // resolved with the POSIX tz string.
                return self.posix.resolve_for_local_seconds(local_seconds);
            }
        }

        debug_assert!(estimated_idx < self.transitions.len());

        let transition_info = self.get_transition_info(estimated_idx);

        let range = transition_info.offset_range_local();

        if range.contains(&local_seconds) {
            return Ok(transition_info.record_for_contains());
        } else if local_seconds < range.start {
            if estimated_idx == 0 {
                // We're at the beginning, there are no timezones before us
                // So we just return the first offset
                return Ok(LocalTimeRecordResult::Single(
                    transition_info.prev.as_utc_offset_seconds(),
                ));
            }
            // Otherwise, try the previous offset
            estimated_idx -= 1;
        } else {
            if estimated_idx + 1 == self.transitions.len() {
                // We're at the end, return posix instead
                return self.posix.resolve_for_local_seconds(local_seconds);
            }
            // Otherwise, try the next offset
            estimated_idx += 1;
        }

        let transition_info = self.get_transition_info(estimated_idx);
        let range = transition_info.offset_range_local();

        if range.contains(&local_seconds) {
            Ok(transition_info.record_for_contains())
        } else if local_seconds < range.start {
            // Note that get_transition_info will correctly fetch the first offset
            // into .prev when working with the first transition.
            Ok(LocalTimeRecordResult::Single(
                transition_info.prev.as_utc_offset_seconds(),
            ))
        } else {
            // We're at the end, return posix instead
            if estimated_idx + 1 == self.transitions.len() {
                return self.posix.resolve_for_local_seconds(local_seconds);
            }
            Ok(LocalTimeRecordResult::Single(
                transition_info.next.as_utc_offset_seconds(),
            ))
        }
    }

    pub fn get_time_zone_transition(
        &self,
        epoch_nanoseconds: i128,
        direction: TransitionDirection,
    ) -> TimeZoneProviderResult<Option<EpochNanoseconds>> {
        // First search the tzif data

        let epoch_seconds = (epoch_nanoseconds / NS_IN_S) as i64;
        // When *exactly* on a transition the spec wants you to
        // get the next one, so it's important to know if we are
        // actually on epoch_seconds or a couple nanoseconds before/after it
        // to handle the exact match case
        let seconds_is_exact = (epoch_nanoseconds % NS_IN_S) == 0;
        let seconds_is_negative = epoch_nanoseconds < 0;

        let b_search_result = self.transitions.binary_search(&epoch_seconds);

        let mut transition_idx = match b_search_result {
            Ok(idx) => {
                match (direction, seconds_is_exact, seconds_is_negative) {
                    // If we are N.001 for negative N, the next transition is idx
                    (TransitionDirection::Next, false, true) => idx,
                    // If we are exactly N, or N.001 for positive N, the next transition is idx + 1
                    (TransitionDirection::Next, true, _)
                    | (TransitionDirection::Next, false, false) => idx + 1,
                    // If we are N.001 for positive N, the previous transition the one at idx (= N)
                    (TransitionDirection::Previous, false, false) => idx,
                    // If we are exactly N, or N.0001 for negative N, the previous transition is idx - 1
                    (TransitionDirection::Previous, true, _)
                    | (TransitionDirection::Previous, false, true) => {
                        if let Some(idx) = idx.checked_sub(1) {
                            idx
                        } else {
                            // If we found the first transition, there is no previous one,
                            // return None
                            return Ok(None);
                        }
                    }
                }
            }
            // idx is insertion index here, so it is the index of the closest upper
            // transition
            Err(idx) => match direction {
                TransitionDirection::Next => idx,
                // Special case, we're after the end of the array, we want to make
                // sure we hit the POSIX tz handling and we should not subtract one.
                TransitionDirection::Previous if idx == self.transitions.len() => idx,
                TransitionDirection::Previous => {
                    // Go one previous
                    if let Some(idx) = idx.checked_sub(1) {
                        idx
                    } else {
                        // If we found the first transition, there is no previous one,
                        // return None
                        return Ok(None);
                    }
                }
            },
        };

        while let Some(tzif_transition) = self.maybe_get_transition_info(transition_idx) {
            // This is not a real transition. Skip it.
            if tzif_transition.prev.offset == tzif_transition.next.offset {
                match direction {
                    TransitionDirection::Next => transition_idx += 1,
                    TransitionDirection::Previous if transition_idx == 0 => return Ok(None),
                    TransitionDirection::Previous => transition_idx -= 1,
                }
            } else {
                return Ok(Some(EpochNanoseconds::from_seconds(
                    tzif_transition.transition_time,
                )));
            }
        }

        // We went past the Tzif transitions. We need to handle the posix time zone instead.
        let posix = self.posix;

        // The last transition in the tzif tables.
        // We should not go back beyond this
        let last_tzif_transition = self.transitions.last();

        // We need to do a similar backwards iteration to find the last real transition.
        // Do it only when needed, this case will only show up when walking Previous for a date
        // just after the last tzif transition but before the first posix transition.
        let last_real_tzif_transition = || {
            debug_assert!(direction == TransitionDirection::Previous);
            for last_transition_idx in (0..self.transitions.len()).rev() {
                if let Some(tzif_transition) = self.maybe_get_transition_info(last_transition_idx) {
                    if tzif_transition.prev.offset == tzif_transition.next.offset {
                        continue;
                    }
                    return Some(tzif_transition.transition_time);
                }
                break;
            }
            None
        };

        let Some(dst_variant) = &self.posix.transition else {
            // There are no further transitions.
            match direction {
                TransitionDirection::Next => return Ok(None),
                TransitionDirection::Previous => {
                    // Go back to the last tzif transition
                    if last_tzif_transition.is_some() {
                        if let Some(last_real_tzif_transition) = last_real_tzif_transition() {
                            return Ok(Some(EpochNanoseconds::from_seconds(
                                last_real_tzif_transition,
                            )));
                        }
                    }
                    return Ok(None);
                }
            }
        };

        // Calculate year, but clamp it to the last transition
        // We do not want to try and apply the posix string to earlier years!
        //
        // Antarctica/Troll is an example of a timezone that has a posix string
        // but no meaningful previous transitions.
        let mut epoch_seconds_for_year_calculation = epoch_seconds;
        if let Some(last_tzif_transition) = last_tzif_transition {
            if epoch_seconds < last_tzif_transition {
                epoch_seconds_for_year_calculation = last_tzif_transition;
            }
        }
        let year = utils::epoch_time_to_iso_year(epoch_seconds_for_year_calculation * 1000);

        let transition_info =
            DstTransitionInfoForYear::compute_zero_transition(posix.offset, dst_variant, year);

        let range = transition_info.transition_range();

        let mut seconds = match direction {
            TransitionDirection::Next => {
                // Inexact seconds in the negative case means that (seconds == foo) is actually
                // seconds < foo
                //
                // This code will likely not actually be hit: the current Tzif database has no
                // entries with DST offset posix strings where the posix string starts
                // before the unix epoch.
                let seconds_is_inexact_for_negative = seconds_is_negative && !seconds_is_exact;
                // We're before the first transition
                if epoch_seconds < range.start.0
                    || (epoch_seconds == range.start.0 && seconds_is_inexact_for_negative)
                {
                    range.start
                } else if epoch_seconds < range.end.0
                    || (epoch_seconds == range.end.0 && seconds_is_inexact_for_negative)
                {
                    // We're between the first and second transition
                    range.end
                } else {
                    // We're after the second transition
                    let transition_info = DstTransitionInfoForYear::compute_zero_transition(
                        posix.offset,
                        dst_variant,
                        year + 1,
                    );

                    transition_info.transition_range().start
                }
            }
            TransitionDirection::Previous => {
                // Inexact seconds in the positive case means that (seconds == foo) is actually
                // seconds > foo
                let seconds_is_ineexact_for_positive = !seconds_is_negative && !seconds_is_exact;
                // We're after the second transition
                // (note that seconds_is_exact means that epoch_seconds == range.end actually means equality)
                if epoch_seconds > range.end.0
                    || (epoch_seconds == range.end.0 && seconds_is_ineexact_for_positive)
                {
                    range.end
                } else if epoch_seconds > range.start.0
                    || (epoch_seconds == range.start.0 && seconds_is_ineexact_for_positive)
                {
                    // We're after the first transition
                    range.start
                } else {
                    // We're before the first transition
                    let transition_info = DstTransitionInfoForYear::compute_zero_transition(
                        posix.offset,
                        dst_variant,
                        year - 1,
                    );

                    transition_info.transition_range().end
                }
            }
        };

        if let Some(last_tzif_transition) = last_tzif_transition {
            // When going Previous, we went back into the area of tzif transition
            if seconds.0 < last_tzif_transition {
                if let Some(last_real_tzif_transition) = last_real_tzif_transition() {
                    seconds = Seconds(last_real_tzif_transition);
                } else {
                    return Ok(None);
                }
            }
        }

        Ok(Some(seconds.into()))
    }

    fn get_transition_info(&self, idx: usize) -> TransitionInfo {
        let info = self.maybe_get_transition_info(idx);
        debug_assert!(info.is_some(), "tzif internal invariant violated");
        info.unwrap_or_default()
    }

    fn maybe_get_transition_info(&self, idx: usize) -> Option<TransitionInfo> {
        let next = self.get_local_time_record(idx);
        let transition_time = self.transitions.get(idx)?;
        let prev = if idx == 0 {
            self.types.first()?
        } else {
            self.get_local_time_record(idx - 1)
        };
        Some(TransitionInfo {
            prev,
            next,
            transition_time,
        })
    }

    fn get_local_time_record(&self, idx: usize) -> LocalTimeRecord {
        // NOTE: Transition type can be empty. If no transition_type exists,
        // then use 0 as the default index of local_time_type_records.
        let idx = self.transition_types.get(idx).unwrap_or(0);

        let get = self.types.get(idx as usize);
        debug_assert!(get.is_some(), "tzif internal invariant violated");
        get.unwrap_or_default()
    }
}

#[derive(Debug, Default)]
pub struct TransitionInfo {
    pub next: LocalTimeRecord,
    pub prev: LocalTimeRecord,
    pub transition_time: i64,
}

impl TransitionInfo {
    fn transition_time_prev_epoch(&self) -> i64 {
        self.transition_time + self.prev.offset
    }

    fn transition_time_next_epoch(&self) -> i64 {
        self.transition_time + self.next.offset
    }

    /// Gets the range of local times where this transition is active
    ///
    /// Note that this will always be start..end, NOT prev..next: if the next
    /// offset is before prev (e.g. for a TransitionKind::Overlap) year,
    /// it will be next..prev.
    ///
    /// You should use .kind() to understand how to interpret this
    fn offset_range_local(&self) -> Range<i64> {
        let prev = self.transition_time_prev_epoch();
        let next = self.transition_time_next_epoch();
        match self.kind() {
            TransitionKind::Overlap => next..prev,
            _ => prev..next,
        }
    }

    /// What is the kind of the transition?
    fn kind(&self) -> TransitionKind {
        match self.prev.offset.cmp(&self.next.offset) {
            Ordering::Less => TransitionKind::Gap,
            Ordering::Greater => TransitionKind::Overlap,
            Ordering::Equal => TransitionKind::Smooth,
        }
    }

    /// If a time is found to be within self.offset_range_local(),
    /// what is the corresponding LocalTimeRecordResult?
    fn record_for_contains(&self) -> LocalTimeRecordResult {
        match self.kind() {
            TransitionKind::Gap => LocalTimeRecordResult::Empty(GapEntryOffsets {
                offset_before: self.prev.as_utc_offset_seconds(),
                offset_after: self.next.as_utc_offset_seconds(),
                transition_epoch: EpochNanoseconds::from_seconds(self.transition_time),
            }),
            TransitionKind::Overlap => LocalTimeRecordResult::Ambiguous {
                first: self.prev.as_utc_offset_seconds(),
                second: self.next.as_utc_offset_seconds(),
            },
            TransitionKind::Smooth => {
                LocalTimeRecordResult::Single(self.prev.as_utc_offset_seconds())
            }
        }
    }
}

#[derive(Debug)]
enum TransitionKind {
    // The offsets didn't change (happens when abbreviations/savings values change)
    Smooth,
    // The offsets changed in a way that leaves a gap
    Gap,
    // The offsets changed in a way that produces overlapping time.
    Overlap,
}
