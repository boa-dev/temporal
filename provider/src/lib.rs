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

mod tzdb;
#[cfg(feature = "experimental_tzif")]
pub mod tzif;

pub use tzdb::IanaIdentifierNormalizer;

/// A prelude of needed types for interacting with `timezone_provider` data.
pub mod prelude {
    pub use zerotrie;
    pub use zerovec;
}

include!("./data/mod.rs");

#[cfg(test)]
#[cfg(feature = "experimental_tzif")]
mod tests {
    use crate as timezone_provider;
    extern crate alloc;

    iana_normalizer_singleton!();
    compiled_zoneinfo_provider!();

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
    fn zone_info_basic() {
        let tzif = COMPILED_ZONEINFO_PROVIDER.get("America/Chicago");
        assert!(tzif.is_some())
    }
}
