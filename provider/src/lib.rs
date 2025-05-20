//! Data providers for time zone data
//!
//! This crate aims to provide a variety of data providers
//! for time zone data.
//!

#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod names;
mod tzdb;
pub(crate) mod utils;

#[cfg(all(feature = "file_system_provider", not(target_os = "windows")))]
mod fs;

#[cfg(all(feature = "file_system_provider", not(target_os = "windows")))]
pub use fs::{FsProviderError, FsTzdbProvider};

use fs::LocalTimeRecord;
pub use names::{IanaDataError, IanaIdentifierNormalizer};

/// A prelude of needed types for interacting with `timezone_provider` data.
pub mod prelude {
    pub use zerotrie;
    pub use zerovec;
}

include!("./data/mod.rs");

mod identifiers {
    use crate as timezone_provider;
    iana_normalizer_singleton!();
}

/// `TimeZoneOffset` represents the number of seconds to be added to UT in order to determine local time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeZoneOffset {
    /// The transition time epoch at which the offset needs to be applied.
    pub transition_epoch: Option<i64>,
    /// The time zone offset in seconds.
    pub offset: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransitionDirection {
    Next,
    Previous,
}

// TODO: What is this named?
#[derive(Debug, PartialEq)]
pub enum PotentialLocalTime {
    Empty,
    Single(LocalTimeRecord),
    Ambiguous {
        dst: LocalTimeRecord,
        std: LocalTimeRecord,
    },
}

// TODO: What lives on the time zone provider trait.

/// The core `TimeZoneProvider` trait
pub trait TimeZoneProvider {
    type Error;
    fn check_identifier(&self, identifier: &str) -> bool;

    fn get_possible_local_time_seconds(
        &self,
        identifier: &str,
        date_time_seconds: i64,
    ) -> Result<PotentialLocalTime, Self::Error>;

    fn get_time_zone_offset(
        &self,
        identifier: &str,
        epoch_seconds: i64,
    ) -> Result<TimeZoneOffset, Self::Error>;

    // TODO: implement and stabalize
    fn get_time_zone_transition(
        &self,
        identifier: &str,
        epoch_seconds: i64,
        direction: TransitionDirection,
    ) -> Result<Option<i64>, Self::Error>;
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    #[cfg(feature = "file_system_provider")]
    use crate::identifiers::SINGLETON_IANA_NORMALIZER;

    #[cfg(not(feature = "file_system_provider"))]
    iana_normalizer_singleton!();

    #[test]
    fn basic_normalization() {
        let index = SINGLETON_IANA_NORMALIZER
            .available_id_index
            .get("America/CHICAGO")
            .unwrap();
        assert_eq!(
            SINGLETON_IANA_NORMALIZER.normalized_identifiers.get(index),
            Some("America/Chicago")
        );

        let index = SINGLETON_IANA_NORMALIZER
            .available_id_index
            .get("uTc")
            .unwrap();
        assert_eq!(
            SINGLETON_IANA_NORMALIZER.normalized_identifiers.get(index),
            Some("UTC")
        );

        let index = SINGLETON_IANA_NORMALIZER
            .available_id_index
            .get("eTC/uTc")
            .unwrap();
        assert_eq!(
            SINGLETON_IANA_NORMALIZER.normalized_identifiers.get(index),
            Some("Etc/UTC")
        );
    }
}
