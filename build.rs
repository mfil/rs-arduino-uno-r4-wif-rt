use std::{env, error::Error, fs::File, io::Write, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    // Build directory for this crate.
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // Extend the library search path.
    println!("cargo:rustc-link-search={}", out_dir.display());

    // Put `link.x` in the build directory.
    File::create(out_dir.join("link.x"))?.write_all(include_bytes!("link.x"))?;

    Ok(())
}
