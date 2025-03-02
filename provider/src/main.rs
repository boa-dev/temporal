use icu_provider_export::baked_exporter::*;
use icu_provider_export::prelude::*;
use tzdb::IanaIdentifierNormalizerV1;

extern crate alloc;

pub mod tzdb;

fn main() {
    let demo_path = std::env::temp_dir().join("icu4x_baked_demo");

    // Set up the exporter
    let exporter =
        BakedExporter::new(demo_path.clone(), Default::default()).unwrap();

    let tzdb_provider = tzdb::TzdbDataProvider::new().unwrap();

    // Export something. Make sure to use the same fallback data at runtime!
    ExportDriver::new(
        [],
        DeduplicationStrategy::Maximal.into(),
        LocaleFallbacker::new_without_data(),
    )
    .with_markers([IanaIdentifierNormalizerV1::INFO])
    .export(&tzdb_provider, exporter)
    .unwrap();
}