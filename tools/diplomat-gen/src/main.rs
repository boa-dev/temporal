use std::{io, path::Path};

use diplomat_tool::config::Config;

fn main() -> std::io::Result<()> {
    const LANGUAGES: [&str; 2] = ["c", "cpp"];

    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));

    let capi = manifest
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("temporal_capi");

    let library_config = Config::default();

    for lang in LANGUAGES {
        diplomat_tool::gen(
            &capi.join("src/lib.rs"),
            lang,
            &{
                let include = capi.join("bindings").join(lang);
                if let Err(err) = std::fs::remove_dir_all(&include) {
                    if err.kind() != io::ErrorKind::NotFound {
                        return Err(err);
                    }
                }
                std::fs::create_dir(&include)?;
                include
            },
            &Default::default(),
            library_config.clone(),
            false,
        )?;
    }

    Ok(())
}
