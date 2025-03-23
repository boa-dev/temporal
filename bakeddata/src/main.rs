use databake::{quote, Bake};
use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::Path,
};
use temporal_provider::IanaIdentifierNormalizer;

trait BakedDataProvider {
    fn write_data(&self, data_path: &Path) -> io::Result<()>;

    fn write_debug(&self, debug_path: &Path) -> io::Result<()>;
}

impl BakedDataProvider for IanaIdentifierNormalizer<'_> {
    fn write_data(&self, data_path: &Path) -> io::Result<()> {
        fs::create_dir_all(data_path)?;
        let generated_file = data_path.join("iana_normalizer.rs.data");
        let baked = self.bake(&Default::default());

        let baked_macro = quote! {
            #[macro_export]
            macro_rules! iana_normalizer_singleton {
                () => {
                    pub const SINGLETON_IANA_NORMALIZER: &'static temporal_provider::IanaIdentifierNormalizer = &#baked;
                }
            }
        };
        let generated = baked_macro.to_string();
        let mut file = BufWriter::new(File::create(generated_file)?);
        write!(file, "//@generated\n\n{generated}")
    }

    fn write_debug(&self, debug_path: &Path) -> io::Result<()> {
        fs::create_dir_all(debug_path)?;
        let debug_filename = debug_path.join("iana_normalizer.json");
        let json = serde_json::to_string_pretty(self).unwrap();
        fs::write(debug_filename, json)
    }
}

fn write_data_file_with_debug(
    data_path: &Path,
    provider: &impl BakedDataProvider,
) -> io::Result<()> {
    let debug_path = data_path.join("debug");
    provider.write_debug(&debug_path)?;
    provider.write_data(data_path)
}

fn main() -> io::Result<()> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tzdata_input = std::env::var("TZDATA_DIR").unwrap_or("tzdata".into());
    let tzdata_path = Path::new(&tzdata_input);
    let tzdata_dir = manifest_dir.parent().unwrap().join(tzdata_path);


    let provider = Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("provider/src");
    write_data_file_with_debug(
        &provider.join("data"),
        &IanaIdentifierNormalizer::build(&tzdata_dir).unwrap(),
    )
}
