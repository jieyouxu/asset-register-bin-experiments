#![feature(let_chains)]
#![feature(str_from_utf16_endian)]
#![warn(unit_bindings)]

mod asset_registry_header;
mod asset_registry_version;
mod logging;
mod read;
mod write;
mod names_batch;
mod serialized_name_header;

use std::path::PathBuf;

use color_eyre::eyre::{eyre, Result as EResult};
use fs_err as fs;
use tracing::*;

fn main() -> EResult<()> {
    logging::setup();
    color_eyre::install()?;

    let Some(test_asset_register_path) = std::env::args().nth(1) else {
        return Err(eyre!("please specify path to test AssetRegister.bin"));
    };
    let test_asset_register_path = PathBuf::from(test_asset_register_path);
    if !test_asset_register_path.exists() {
        return Err(eyre!(
            "the path specified `{}` cannot be found, please double check your input",
            test_asset_register_path.display()
        ));
    }

    let raw = fs::read(test_asset_register_path)?;
    info!(asset_register_len = raw.len());

    let mut reader = std::io::Cursor::new(&raw);

    let _asset_registry: () =
        ser_hex::CounterSubscriber::read("trace.json", &mut reader, |reader| todo!());

    Ok(())
}
