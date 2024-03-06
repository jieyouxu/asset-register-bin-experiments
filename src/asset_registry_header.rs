use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::{eyre, Result as EResult, WrapErr};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use tracing::*;

use crate::read::Readable;
use crate::write::Writable;

use crate::asset_registry_version::AssetRegistryVersion;

pub const ASSET_REGISTRY_VERSION_GUID: [u8; 16] = [
    0xE7, 0x9E, 0x7F, 0x71, 0x3A, 0x49, 0xB0, 0xE9, 0x32, 0x91, 0xB3, 0x88, 0x07, 0x81, 0x38, 0x1B,
];

#[derive(Debug)]
pub struct AssetRegistryHeader {
    pub version: AssetRegistryVersion,
}

impl<W: Write> Writable<W> for AssetRegistryHeader {
    #[instrument(name = "AssetRegistryHeader_write", skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        writer.write_all(&ASSET_REGISTRY_VERSION_GUID)?;
        self.version.write(writer)?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for AssetRegistryHeader {
    #[instrument(name = "AssetRegistryHeader_read", skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let mut guid = [0u8; 16];
        reader
            .read_exact(&mut guid)
            .wrap_err_with(|| "failed to read GUID")?;
        if guid != ASSET_REGISTRY_VERSION_GUID {
            return Err(eyre!(
                "invalid guid: expected `{:x?}` but found `{:x?}`",
                ASSET_REGISTRY_VERSION_GUID,
                guid
            ));
        }

        let version = AssetRegistryVersion::read(reader)?;
        Ok(AssetRegistryHeader { version })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Cursor;

    #[test]
    fn test_roundtrip() {
        let mut buf = vec![];
        let mut writer = Cursor::new(&mut buf);
        AssetRegistryHeader {
            version: AssetRegistryVersion::LATEST_VERSION,
        }
        .write(&mut writer)
        .unwrap();
        let mut reader = Cursor::new(&buf);
        let header = AssetRegistryHeader::read(&mut reader).unwrap();
        assert_eq!(header.version, AssetRegistryVersion::LATEST_VERSION);
    }
}
