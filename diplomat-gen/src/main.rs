use std::path::Path;

fn main() -> std::io::Result<()> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));

    let capi = manifest.parent().unwrap().join("temporal_capi");

    let library_config = Default::default();

    diplomat_tool::gen(
        &capi.join("src/lib.rs"),
        "cpp",
        &{
            let include = capi.join("bindings").join("cpp");
            std::fs::remove_dir_all(&include)?;
            std::fs::create_dir(&include)?;
            include
        },
        &Default::default(),
        library_config,
        false,
    )
}
