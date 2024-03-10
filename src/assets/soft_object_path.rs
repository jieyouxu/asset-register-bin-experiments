use std::io::{Read, Write};

use color_eyre::eyre::{Result as EResult};
use tracing::*;

use crate::read::Readable;
use crate::unreal_types::{FName, FString};
use crate::write::Writable;

#[derive(Debug, PartialEq)]
pub struct FSoftObjectPath {
    pub asset_path_name: FName,
    pub sub_path_string: FString,
}

impl<W: Write> Writable<W> for FSoftObjectPath {
    #[instrument(name = "FSoftObjectPath_write", skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        self.asset_path_name.write(writer)?;
        self.sub_path_string.write(writer)?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for FSoftObjectPath {
    #[instrument(name = "FSoftObjectPath_read", skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let asset_path_name = FName::read(reader)?;
        debug!(?asset_path_name);
        let sub_path_string = FString::read(reader)?;
        debug!(?sub_path_string);
        Ok(FSoftObjectPath {
            asset_path_name,
            sub_path_string,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_roundtrip() {
        let path = FSoftObjectPath {
            asset_path_name: FName {
                index: 123,
                number: 456,
            },
            sub_path_string: FString::from("catJAM"),
        };
        let mut buf = vec![];
        let mut writer = Cursor::new(&mut buf);
        path.write(&mut writer).unwrap();
        let mut reader = Cursor::new(&buf);
        let read_path = FSoftObjectPath::read(&mut reader).unwrap();
        assert_eq!(read_path, path);
    }
}
