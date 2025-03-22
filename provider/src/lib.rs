mod tzdb;

pub use tzdb::{IanaDataError, IanaIdentifierNormalizer};

pub mod prelude {
    pub use zerotrie;
    pub use zerovec;
}

include!("./data/mod.rs");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_normalization() {
        let iana_normalizer = IanaIdentifierNormalizer::build().unwrap();
        let index = iana_normalizer
            .available_id_index
            .get(&"America/CHICAGO")
            .unwrap();
        assert_eq!(
            iana_normalizer.normalized_identifiers.get(index),
            Some("America/Chicago")
        );

        let index = iana_normalizer.available_id_index.get(&"uTc").unwrap();
        assert_eq!(
            iana_normalizer.normalized_identifiers.get(index),
            Some("UTC")
        );

        let index = iana_normalizer.available_id_index.get(&"eTC/uTc").unwrap();
        assert_eq!(
            iana_normalizer.normalized_identifiers.get(index),
            Some("Etc/UTC")
        );
    }
}
