use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::{eyre, Result as EResult, WrapErr};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use tracing::*;

use crate::asset_registry_header::AssetRegistryHeader;
use crate::asset_registry_version::AssetRegistryVersion;
use crate::names_batch::NamesBatch;
use crate::read::Readable;
use crate::write::Writable;

#[derive(Debug)]
pub struct AssetRegistry {
    pub names: NamesBatch,
}

impl<W: Write> Writable<W> for AssetRegistry {
    #[instrument(name = "AssetRegistry_write", skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        AssetRegistryHeader {
            version: AssetRegistryVersion::LATEST_VERSION,
        }
        .write(writer)?;
        self.names.write(writer)?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for AssetRegistry {
    #[instrument(name = "AssetRegistry_read", skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let _ = AssetRegistryHeader::read(reader)?;
        let names = NamesBatch::read(reader)?;
        Ok(AssetRegistry { names })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::names_batch::NamesBatch;
    use crate::serialized_name_header::SerializedNameHeader;
    use std::io::Cursor;

    #[test]
    fn test_roundtrip() {
        let names = NamesBatch {
            hash_version: 0,
            hashes: vec![0xDEAD_BEEF],
            headers: vec![SerializedNameHeader {
                is_utf16: false,
                len: 2,
            }],
            strings: vec!["a".to_string()],
        };

        let mut buf = vec![];
        let mut writer = Cursor::new(&mut buf);
        AssetRegistry {
            names: names.clone(),
        }
        .write(&mut writer)
        .unwrap();

        let mut reader = Cursor::new(&buf);
        let asset_registry = AssetRegistry::read(&mut reader).unwrap();

        assert_eq!(asset_registry.names, names);
    }
}
