//! `temporal_provider` is a crate designed for data providers
//! intended for `temporal_rs`
//!

mod tzdb;

pub mod tzif;

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
