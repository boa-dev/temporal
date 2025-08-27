//! Data providers for time zone data
//!
//! This crate aims to provide a variety of data providers
//! for time zone data.
//!

#![no_std]
#![cfg_attr(
    not(test),
    warn(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)
)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[macro_use]
mod private {
    include!("./data/mod.rs");
}

mod tzdb;
#[cfg(feature = "experimental_tzif")]
pub mod tzif;

pub use tzdb::IanaIdentifierNormalizer;

use crate as timezone_provider;
iana_normalizer_singleton!(SINGLETON_IANA_NORMALIZER);

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::SINGLETON_IANA_NORMALIZER;

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

    #[test]
    #[cfg(feature = "experimental_tzif")]
    fn zone_info_basic() {
        let tzif = crate::tzif::COMPILED_ZONEINFO_PROVIDER.get("America/Chicago");
        assert!(tzif.is_some())
    }
}
