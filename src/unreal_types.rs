use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::{eyre, Result as EResult};
use tracing::*;

use crate::read::Readable;
use crate::write::Writable;

/// Serializes to 32-bits... when in legacy Unreal bool type.
pub struct UnrealBool(pub bool);

impl<W: Write> Writable<W> for UnrealBool {
    // This sets `Ar.SetCustomVersion(Guid, VersionInt, TEXT("AssetRegistry"));` somehow.
    #[instrument(skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        writer.write_u32::<LE>(match self.0 {
            true => 1,
            false => 0,
        })?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for UnrealBool {
    #[instrument(skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        match reader.read_u32::<LE>()? {
            0 => Ok(UnrealBool(false)),
            1 => Ok(UnrealBool(true)),
            v => Err(eyre!("failed to serialize {} as legacy unreal bool", v)),
        }
    }
}
