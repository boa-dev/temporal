//! The Temporal Now component

use num_bigint::BigInt;
use num_traits::FromPrimitive;

use crate::{sys, TemporalResult, TemporalUnwrap};

use super::Instant;

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
}

fn system_instant() -> TemporalResult<Instant> {
    let nanos = sys::get_system_nanoseconds()?;
    Instant::new(BigInt::from_u128(nanos).temporal_unwrap()?)
}
