//! The Temporal Now component

use crate::{sys, TemporalResult};
use alloc::string::String;

use num_traits::FromPrimitive;

use crate::{iso::IsoDateTime, TemporalUnwrap};

use super::{
    calendar::Calendar,
    timezone::{TimeZone, TzProvider},
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

fn system_date_time(
    tz: Option<TimeZone>,
    provider: &impl TzProvider,
) -> TemporalResult<IsoDateTime> {
    // 1. If temporalTimeZoneLike is undefined, then
    // a. Let timeZone be SystemTimeZoneIdentifier().
    // 2. Else,
    // a. Let timeZone be ? ToTemporalTimeZoneIdentifier(temporalTimeZoneLike).
    let tz = tz.unwrap_or(TimeZone::IanaIdentifier(sys::get_system_tz_identifier()?));
    // 3. Let epochNs be SystemUTCEpochNanoseconds().
    // TODO: Handle u128 -> i128 better for system nanoseconds
    let epoch_ns = EpochNanoseconds::try_from(sys::get_system_nanoseconds()?)?;
    // 4. Return GetISODateTimeFor(timeZone, epochNs).
    tz.get_iso_datetime_for(&Instant::from(epoch_ns), provider)
}

fn system_instant() -> TemporalResult<Instant> {
    let nanos = sys::get_system_nanoseconds()?;
    Instant::try_new(i128::from_u128(nanos).temporal_unwrap()?)
}

#[cfg(feature = "tzdb")]
#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration as StdDuration;

    use crate::{options::DifferenceSettings, tzdb::FsTzdbProvider, Now};

    #[test]
    fn now_datetime_test() {
        let provider = &FsTzdbProvider::default();
        let sleep = 2;

        let before = Now::plain_date_time_with_provider(None, provider).unwrap();
        thread::sleep(StdDuration::from_secs(sleep));
        let after = Now::plain_date_time_with_provider(None, provider).unwrap();

        let diff = after.since(&before, DifferenceSettings::default()).unwrap();

        let sleep_base = sleep as f64;
        let tolerable_range = sleep_base..=sleep_base + 5.0;

        // We assert a tolerable range of sleep + 5 because std::thread::sleep
        // is only guaranteed to be >= the value to sleep. So to prevent sporadic
        // errors, we only assert a range.
        assert!(tolerable_range.contains(&diff.seconds().as_inner()));
        assert!(diff.hours().is_zero());
        assert!(diff.minutes().is_zero());
    }
}
