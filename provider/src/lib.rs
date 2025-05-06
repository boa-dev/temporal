//! `temporal_provider` is a crate designed for data providers
//! intended for `temporal_rs`
//!

mod tzdb;

pub mod tzif;

pub use tzif::ZoneInfoProvider;

pub use tzdb::{IanaDataError, IanaIdentifierNormalizer};

/// A prelude of needed types for interacting with `temporal_provider` data.
pub mod prelude {
    pub use zerotrie;
    pub use zerovec;
}

include!("./data/mod.rs");

#[cfg(test)]
mod tests {
    use crate as temporal_provider;
    extern crate alloc;

    iana_normalizer_singleton!();
    zone_info_provider_baked!();

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
        let tzif = ZONE_INFO_PROVIDER.get("America/Chicago");
        assert!(tzif.is_some())
    }
}
