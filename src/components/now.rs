//! The Temporal Now component

use alloc::string::String;
use num_traits::FromPrimitive;

use crate::{iso::IsoDateTime, sys, TemporalResult, TemporalUnwrap};

use super::{
    calendar::Calendar,
    tz::{TimeZone, TzProvider},
    Instant, PlainDateTime,
};

/// The Temporal Now object.
pub struct Now;

impl Now {
    /// Returns the current time zone.
    pub fn time_zone_id() -> TemporalResult<String> {
        sys::get_system_tz_identifier()
    }

    /// Returns the current instant
    pub fn instant() -> TemporalResult<Instant> {
        system_instant()
    }

    pub fn plain_date_time_with_provider(
        tz: Option<TimeZone>,
        provider: &mut impl TzProvider,
    ) -> TemporalResult<PlainDateTime> {
        let iso = system_date_time(tz, provider)?;
        Ok(PlainDateTime::new_unchecked(iso, Calendar::default()))
    }
}

fn system_date_time(
    tz: Option<TimeZone>,
    provider: &mut impl TzProvider,
) -> TemporalResult<IsoDateTime> {
    // 1. If temporalTimeZoneLike is undefined, then
    // a. Let timeZone be SystemTimeZoneIdentifier().
    // 2. Else,
    // a. Let timeZone be ? ToTemporalTimeZoneIdentifier(temporalTimeZoneLike).
    let tz = tz.unwrap_or(sys::get_system_tz_identifier()?.into());
    // 3. Let epochNs be SystemUTCEpochNanoseconds().
    // TODO: Handle u128 -> i128 better for system nanoseconds
    let epoch_ns = sys::get_system_nanoseconds()?;
    // 4. Return GetISODateTimeFor(timeZone, epochNs).
    tz.get_iso_datetime_for(
        &Instant {
            epoch_nanos: epoch_ns as i128,
        },
        provider,
    )
}

fn system_instant() -> TemporalResult<Instant> {
    let nanos = sys::get_system_nanoseconds()?;
    Instant::new(i128::from_u128(nanos).temporal_unwrap()?)
}
