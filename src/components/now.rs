//! The Temporal Now component

use crate::{sys, TemporalResult};
use alloc::string::String;

#[cfg(feature = "std")]
use num_traits::FromPrimitive;

#[cfg(feature = "std")]
use crate::{iso::IsoDateTime, TemporalUnwrap};

#[cfg(feature = "std")]
use super::{
    calendar::Calendar,
    tz::{TimeZone, TzProvider},
    EpochNanoseconds, Instant, PlainDateTime,
};

/// The Temporal Now object.
pub struct Now;

impl Now {
    /// Returns the current time zone.
    pub fn time_zone_id() -> TemporalResult<String> {
        sys::get_system_tz_identifier()
    }
}

#[cfg(feature = "std")]
impl Now {
    /// Returns the current instant
    pub fn instant() -> TemporalResult<Instant> {
        system_instant()
    }

    pub fn plain_date_time_with_provider(
        tz: Option<TimeZone>,
        provider: &impl TzProvider,
    ) -> TemporalResult<PlainDateTime> {
        let iso = system_date_time(tz, provider)?;
        Ok(PlainDateTime::new_unchecked(iso, Calendar::default()))
    }
}

#[cfg(feature = "std")]
fn system_date_time(
    tz: Option<TimeZone>,
    provider: &impl TzProvider,
) -> TemporalResult<IsoDateTime> {
    // 1. If temporalTimeZoneLike is undefined, then
    // a. Let timeZone be SystemTimeZoneIdentifier().
    // 2. Else,
    // a. Let timeZone be ? ToTemporalTimeZoneIdentifier(temporalTimeZoneLike).
    let tz = tz.unwrap_or(sys::get_system_tz_identifier()?.into());
    // 3. Let epochNs be SystemUTCEpochNanoseconds().
    // TODO: Handle u128 -> i128 better for system nanoseconds
    let epoch_ns = EpochNanoseconds::try_from(sys::get_system_nanoseconds()?)?;
    // 4. Return GetISODateTimeFor(timeZone, epochNs).
    tz.get_iso_datetime_for(&Instant::from(epoch_ns), provider)
}

#[cfg(feature = "std")]
fn system_instant() -> TemporalResult<Instant> {
    let nanos = sys::get_system_nanoseconds()?;
    Instant::try_new(i128::from_u128(nanos).temporal_unwrap()?)
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration as StdDuration;

    use crate::{partial::PartialDuration, tzdb::FsTzdbProvider, Duration, Now};

    #[cfg(feature = "tzdb")]
    #[test]
    fn now_datetime_test() {
        let provider = &FsTzdbProvider::default();

        let now = Now::plain_date_time_with_provider(None, provider).unwrap();
        thread::sleep(StdDuration::from_secs(2));
        let then = Now::plain_date_time_with_provider(None, provider).unwrap();

        let two_seconds = Duration::from_partial_duration(PartialDuration {
            seconds: Some(2.into()),
            ..Default::default()
        })
        .unwrap();

        let now_plus_two = now.add(&two_seconds, None).unwrap();

        assert_eq!(now_plus_two.second(), then.second());
        assert_eq!(now_plus_two.minute(), then.minute());
        assert_eq!(now_plus_two.hour(), then.hour());
    }
}
