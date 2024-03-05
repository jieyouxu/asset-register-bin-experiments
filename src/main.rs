#![feature(let_chains)]
#![feature(str_from_utf16_endian)]

mod asset_registry;
mod asset_registry_export_path;
mod asset_registry_header;
mod asset_registry_version;
mod logging;
mod read;
mod store_data;
mod top_level_asset_path;
mod unreal_types;
mod write;

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
