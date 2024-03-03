use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::Result as EResult;
use tracing::*;

use crate::read::Readable;
use crate::write::Writable;

use crate::asset_registry_header::AssetRegistryHeader;

#[derive(Debug)]
pub struct AssetRegistry {
    pub header: AssetRegistryHeader,
    pub asset_count: u32,
}

impl<W: Write> Writable<W> for AssetRegistry {
    /// See <https://github.com/EpicGames/UnrealEngine/blob/072300df18a94f18077ca20a14224b5d99fee872/Engine/Source/Runtime/AssetRegistry/Private/AssetRegistryState.cpp#L1138>.
    #[instrument(skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        self.header.write(writer)?;
        writer.write_u32::<LE>(self.asset_count)?;

        todo!()
    }
}

impl<R: Read> Readable<R> for AssetRegistry {
    #[instrument(skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let _header = AssetRegistryHeader::read(reader)?;
        let _asset_count = reader.read_u32::<LE>()?;

        todo!()
    }
}

