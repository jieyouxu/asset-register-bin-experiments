use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::{eyre, Result as EResult, WrapErr};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use tracing::*;

use crate::read::Readable;
use crate::write::Writable;

use crate::asset_registry_version::AssetRegistryVersion;

#[derive(Debug)]
pub struct AssetRegistryHeader {
    pub version: AssetRegistryVersion,
}

impl<W: Write> Writable<W> for AssetRegistryHeader {
    #[instrument(name = "AssetRegistryHeader_write", skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        self.version.write(writer)?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for AssetRegistryHeader {
    #[instrument(name = "AssetRegistryHeader_read", skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let version = AssetRegistryVersion::read(reader)?;
        Ok(AssetRegistryHeader { version })
    }
}
