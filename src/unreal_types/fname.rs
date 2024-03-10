use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::{eyre, Result as EResult};
use tracing::*;

use crate::read::Readable;
use crate::write::Writable;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct FName {
    pub index: u32,
    pub number: u32,
}

impl<W: Write> Writable<W> for FName {
    #[instrument(name = "FName_write", skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        writer.write_u32::<LE>(self.index)?;
        writer.write_u32::<LE>(self.number)?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for FName {
    #[instrument(name = "FName_read", skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let index = reader.read_u32::<LE>()?;
        let number = reader.read_u32::<LE>()?;
        Ok(FName { index, number })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_roundtrip() {
        let name = FName { index: 123, number: 456 };
        let mut buf = vec![];
        let mut writer = Cursor::new(&mut buf);
        name.write(&mut writer).unwrap();
        let mut reader = Cursor::new(&buf);
        let read_name = FName::read(&mut reader).unwrap();
        assert_eq!(read_name, name);
    }
}
