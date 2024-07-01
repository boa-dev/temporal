//! This module implements the Temporal `TimeZone` and components.

use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::{
    components::{calendar::TemporalCalendar, DateTime, Instant},
    TemporalError, TemporalResult,
};

/// Any object that implements the `TzProtocol` must implement the below methods/properties.
pub const TIME_ZONE_PROPERTIES: [&str; 3] =
    ["getOffsetNanosecondsFor", "getPossibleInstantsFor", "id"];

/// A Temporal `TimeZone`.
#[derive(Debug, Clone)]
#[allow(unused)]
pub struct TimeZone {
    pub(crate) iana: Option<String>, // TODO: ICU4X IANA TimeZone support.
    pub(crate) offset: Option<i16>,
}

impl TimeZone {
    pub(crate) fn get_datetime_for(
        &self,
        instant: &Instant,
        calendar: &TemporalCalendar,
    ) -> TemporalResult<DateTime> {
        let nanos = self.get_offset_nanos_for()?;
        DateTime::from_instant(instant, nanos.to_f64().unwrap_or(0.0), calendar.clone())
    }
}

impl TimeZone {
    /// Get the offset for this current `TimeZoneSlot`.
    pub fn get_offset_nanos_for(&self) -> TemporalResult<BigInt> {
        // 1. Let timeZone be the this value.
        // 2. Perform ? RequireInternalSlot(timeZone, [[InitializedTemporalTimeZone]]).
        // 3. Set instant to ? ToTemporalInstant(instant).
        // 4. If timeZone.[[OffsetMinutes]] is not empty, return 𝔽(timeZone.[[OffsetMinutes]] × (60 × 10^9)).
        if let Some(offset) = &self.offset {
            return Ok(BigInt::from(i64::from(*offset) * 60_000_000_000i64));
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
