use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::{eyre, Result as EResult};
use tracing::*;

use crate::read::{read_array, Readable};
use crate::unreal_types::FName;
use crate::write::{write_array, Writable};

use super::FSoftObjectPath;

#[derive(Debug, PartialEq)]
pub struct FAssetBundleEntry {
    bundle_name: FName,
    bundles: Vec<FSoftObjectPath>,
}

impl<W: Write> Writable<W> for FAssetBundleEntry {
    #[instrument(name = "FAssetBundleEntry_write", skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        self.bundle_name.write(writer)?;
        writer.write_u32::<LE>(self.bundles.len() as u32)?;
        write_array(writer, &self.bundles, |w, e| e.write(w))?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for FAssetBundleEntry {
    #[instrument(name = "FAssetBundleEntry_read", skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let bundle_name = FName::read(reader)?;
        debug!(?bundle_name);
        let len = reader.read_u32::<LE>()?;
        debug!(?len);
        let bundles = read_array(len, reader, FSoftObjectPath::read)?;
        Ok(FAssetBundleEntry {
            bundle_name,
            bundles,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::unreal_types::FString;

    use super::*;
    use std::io::Cursor;

    #[test_log::test]
    fn test_roundtrip() {
        let entry = FAssetBundleEntry {
            bundle_name: FName {
                index: 123,
                number: 456,
            },
            bundles: vec![FSoftObjectPath {
                asset_path_name: FName {
                    index: 789,
                    number: 120,
                },
                sub_path_string: FString::from("KEKW"),
            }],
        };
        let mut buf = vec![];
        let mut writer = Cursor::new(&mut buf);
        entry.write(&mut writer).unwrap();
        let mut reader = Cursor::new(&buf);
        let read_entry = FAssetBundleEntry::read(&mut reader).unwrap();
        assert_eq!(read_entry, entry);
    }
}
