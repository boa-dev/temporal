//! This module implements the Temporal `TimeZone` and components.

use alloc::string::String;
use alloc::vec::Vec;
use num_traits::ToPrimitive;

use crate::{
    components::{calendar::Calendar, Instant, PlainDateTime},
    TemporalError, TemporalResult,
};

/// A Temporal `TimeZone`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeZone {
    /// The IANA identifier for this time zone.
    pub(crate) iana: Option<String>, // TODO: ICU4X IANA TimeZone support.
    /// The offset minutes of a time zone.
    pub(crate) offset: Option<i16>,
}

impl TimeZone {
    pub(crate) fn get_datetime_for(
        &self,
        instant: &Instant,
        calendar: &Calendar,
    ) -> TemporalResult<PlainDateTime> {
        let nanos = self.get_offset_nanos_for()?;
        PlainDateTime::from_instant(instant, nanos.to_f64().unwrap_or(0.0), calendar.clone())
    }
}

impl TimeZone {
    /// Get the offset for this current `TimeZoneSlot`.
    pub fn get_offset_nanos_for(&self) -> TemporalResult<i128> {
        // 1. Let timeZone be the this value.
        // 2. Perform ? RequireInternalSlot(timeZone, [[InitializedTemporalTimeZone]]).
        // 3. Set instant to ? ToTemporalInstant(instant).
        // 4. If timeZone.[[OffsetMinutes]] is not empty, return 𝔽(timeZone.[[OffsetMinutes]] × (60 × 10^9)).
        if let Some(offset) = &self.offset {
            return Ok(i128::from(*offset) * 60_000_000_000);
        }
        // 5. Return 𝔽(GetNamedTimeZoneOffsetNanoseconds(timeZone.[[Identifier]], instant.[[Nanoseconds]])).
        Err(TemporalError::range().with_message("IANA TimeZone names not yet implemented."))
    }

    /// Get the possible `Instant`s for this `TimeZoneSlot`.
    pub fn get_possible_instant_for(&self) -> TemporalResult<Vec<Instant>> {
        Err(TemporalError::general("Not yet implemented."))
    }

    /// Returns the current `TimeZoneSlot`'s identifier.
    pub fn id(&self) -> TemporalResult<String> {
        Err(TemporalError::range().with_message("Not yet implemented."))
    }
}
