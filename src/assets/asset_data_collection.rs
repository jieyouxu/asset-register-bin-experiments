use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::Result as EResult;
use tracing::*;

use crate::read::{read_array, Readable};
use crate::write::{write_array, Writable};

use super::AssetData;

#[derive(Debug, PartialEq)]
pub struct AssetDataCollection {
    pub assets: Vec<AssetData>,
}

impl<W: Write> Writable<W> for AssetDataCollection {
    #[instrument(name = "AssetDataCollection_write", skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        writer.write_u32::<LE>(self.assets.len() as u32)?;
        write_array(writer, &self.assets, |w, a| a.write(w))?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for AssetDataCollection {
    #[instrument(name = "AssetDataCollection_read", skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let assets = read_array(reader.read_u32::<LE>()?, reader, AssetData::read)?;
        Ok(AssetDataCollection { assets })
    }
}

#[cfg(test)]
mod tests {
    use crate::assets::{FAssetBundleEntry, FSoftObjectPath};
    use crate::unreal_types::{FName, FString};

    use super::*;
    use std::io::Cursor;

    #[test_log::test]
    fn test_roundtrip() {
        let data = AssetDataCollection {
            assets: vec![AssetData {
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
            }],
        };
        let mut buf = vec![];
        let mut writer = Cursor::new(&mut buf);
        data.write(&mut writer).unwrap();
        let mut reader = Cursor::new(&buf);
        let read_data = AssetDataCollection::read(&mut reader).unwrap();
        assert_eq!(read_data, data);
    }
}
