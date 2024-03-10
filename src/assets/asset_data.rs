use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::{Result as EResult};
use tracing::*;

use crate::read::{read_array, Readable};
use crate::unreal_types::FName;
use crate::write::{write_array, Writable};

use super::FAssetBundleEntry;

#[derive(Debug, PartialEq)]
pub struct AssetData {
    pub object_path: FName,
    pub package_path: FName,
    pub asset_class: FName,
    pub package_name: FName,
    pub asset_name: FName,
    pub tags: u64,
    pub bundles: Vec<FAssetBundleEntry>,
}

impl<W: Write> Writable<W> for AssetData {
    #[instrument(name = "AssetData_write", skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        self.object_path.write(writer)?;
        self.package_path.write(writer)?;
        self.asset_class.write(writer)?;
        self.package_name.write(writer)?;
        self.asset_name.write(writer)?;
        writer.write_u64::<LE>(self.tags)?;
        writer.write_u32::<LE>(self.bundles.len() as u32)?;
        write_array(writer, &self.bundles, |w, e| e.write(w))?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for AssetData {
    #[instrument(name = "AssetData_read", skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let object_path = FName::read(reader)?;
        let package_path = FName::read(reader)?;
        let asset_class = FName::read(reader)?;
        let package_name = FName::read(reader)?;
        let asset_name = FName::read(reader)?;
        let tags = reader.read_u64::<LE>()?;
        let bundles = read_array(reader.read_u32::<LE>()?, reader, FAssetBundleEntry::read)?;
        Ok(AssetData {
            object_path,
            package_path,
            asset_class,
            package_name,
            asset_name,
            tags,
            bundles,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::assets::FSoftObjectPath;
    use crate::unreal_types::FString;

    use super::*;
    use std::io::Cursor;

    #[test_log::test]
    fn test_roundtrip() {
        let data = AssetData {
            object_path: FName {
                index: 123,
                number: 456,
            },
            package_path: FName {
                index: 836,
                number: 136,
            },
            asset_class: FName {
                index: 58120912,
                number: 12873,
            },
            package_name: FName {
                index: 4723,
                number: 1,
            },
            asset_name: FName {
                index: 2,
                number: 3,
            },
            tags: 0xDEAD_BEEF,
            bundles: vec![FAssetBundleEntry {
                bundle_name: FName {
                    index: 621,
                    number: 921,
                },
                bundles: vec![FSoftObjectPath {
                    asset_path_name: FName {
                        index: 183,
                        number: 1749,
                    },
                    sub_path_string: FString::from("forklift"),
                }],
            }],
        };
        let mut buf = vec![];
        let mut writer = Cursor::new(&mut buf);
        data.write(&mut writer).unwrap();
        let mut reader = Cursor::new(&buf);
        let read_data = AssetData::read(&mut reader).unwrap();
        assert_eq!(read_data, data);
    }
}
