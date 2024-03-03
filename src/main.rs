#![feature(let_chains)]

mod asset_registry;
mod asset_registry_header;
mod asset_registry_version;
mod logging;
mod read;
mod write;
mod unreal_types;

use color_eyre::eyre::{Context, Result as EResult};
use fs_err as fs;

fn main() -> EResult<()> {
    logging::setup();
    color_eyre::install()?;

    let mut test_asset_register_path = std::env::current_dir()?;
    test_asset_register_path.push("test_assets");
    test_asset_register_path.push("minimal.bin");

    let _raw_bytes =
        fs::read(test_asset_register_path).wrap_err("failed to open test asset register")?;

    Ok(())
}
