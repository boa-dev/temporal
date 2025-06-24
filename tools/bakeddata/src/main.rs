use databake::{quote, Bake};
use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::Path,
};
use timezone_provider::{tzif::ZoneInfoProvider, IanaIdentifierNormalizer};

trait BakedDataProvider {
    fn write_data(&self, data_path: &Path) -> io::Result<()>;

    fn write_debug(&self, debug_path: &Path) -> io::Result<()>;
}

impl BakedDataProvider for ZoneInfoProvider<'_> {
    fn write_data(&self, data_path: &Path) -> io::Result<()> {
        fs::create_dir_all(data_path)?;
        let generated_file = data_path.join("zone_info_provider.rs.data");
        let baked = self.bake(&Default::default());

        let baked_macro = quote! {
            #[macro_export]
            macro_rules! zone_info_provider {
                () => {
                    pub const ZONE_INFO_PROVIDER: &'static temporal_provider::ZoneInfoProvider = &#baked;
                }
            }
        };
        let file = syn::parse_file(&baked_macro.to_string()).unwrap();
        let formatted = prettyplease::unparse(&file);
        let mut file = BufWriter::new(File::create(generated_file)?);
        write!(file, "//@generated\n// (by `bakeddata` binary in temporal_rs, using `databake`)\n\n{formatted}")
    }

    fn write_debug(&self, debug_path: &Path) -> io::Result<()> {
        let zoneinfo_debug_path = debug_path.join("zoneinfo");
        // Remove zoneinfo directory and recreate, so we can rely on diff of what is
        // changed / missing.
        if zoneinfo_debug_path.exists() {
            fs::remove_dir_all(zoneinfo_debug_path.clone())?;
        }
        // Recreate directory.
        fs::create_dir_all(zoneinfo_debug_path.clone())?;

        for (identifier, index) in self.ids.to_btreemap().iter() {
            let (directory, filename) = if identifier.contains('/') {
                let (directory, filename) = identifier.rsplit_once('/').expect("'/' must exist");
                let identifier_dir = zoneinfo_debug_path.join(directory);
                fs::create_dir_all(identifier_dir.clone())?;
                (identifier_dir, filename)
            } else {
                (zoneinfo_debug_path.clone(), identifier.as_str())
            };
            let mut filepath = directory.join(filename);
            filepath.set_extension("json");
            let json = serde_json::to_string_pretty(&self.tzifs[*index])?;
            fs::write(filepath, json)?;
        }

        // TODO: Add version
        Ok(())
    }
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
                    pub const SINGLETON_IANA_NORMALIZER: &'static timezone_provider::IanaIdentifierNormalizer = &#baked;
                }
            }
        };
        let file = syn::parse_file(&baked_macro.to_string()).unwrap();
        let formatted = prettyplease::unparse(&file);
        let mut file = BufWriter::new(File::create(generated_file)?);
        write!(file, "//@generated\n// (by `bakeddata` binary in temporal_rs, using `databake`)\n\n{formatted}")
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
    let tzdata_dir = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(tzdata_path);

    let provider = Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("provider/src");

    // Write identifiers
    write_data_file_with_debug(
        &provider.join("data"),
        &IanaIdentifierNormalizer::build(&tzdata_dir).unwrap(),
    )?;

    // Write tzif data
    write_data_file_with_debug(
        &provider.join("data"),
        &ZoneInfoProvider::build(&tzdata_dir).unwrap(),
    )
}
