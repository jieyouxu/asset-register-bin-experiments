use std::io::{Read, Write};

use color_eyre::eyre::{Result as EResult};
use tracing::*;

use crate::read::Readable;
use crate::write::Writable;

use super::FName;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct FAssetRegistryExportPath {
    pub class: FName,
    pub object: FName,
    pub package: FName,
}

impl<W: Write> Writable<W> for FAssetRegistryExportPath {
    #[instrument(name = "FAssetRegistryExportPath_write", skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        self.class.write(writer)?;
        self.object.write(writer)?;
        self.package.write(writer)?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for FAssetRegistryExportPath {
    #[instrument(name = "FAssetRegistryExportPath_read", skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let class = FName::read(reader)?;
        let object = FName::read(reader)?;
        let package = FName::read(reader)?;
        Ok(FAssetRegistryExportPath {
            class,
            object,
            package,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_roundtrip() {
        let path = FAssetRegistryExportPath {
            class: FName {
                index: 123,
                number: 456,
            },
            object: FName {
                index: 583,
                number: 194,
            },
            package: FName {
                index: 789,
                number: 1012,
            },
        };
        let mut buf = vec![];
        let mut writer = Cursor::new(&mut buf);
        path.write(&mut writer).unwrap();
        let mut reader = Cursor::new(&buf);
        let read_path = FAssetRegistryExportPath::read(&mut reader).unwrap();
        assert_eq!(read_path, path);
    }
}
