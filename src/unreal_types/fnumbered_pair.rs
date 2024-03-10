//! `FNumberlessPair` is a special case of `FNumberedPair`.

use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::{eyre, Result as EResult};
use tracing::*;

use crate::read::Readable;
use crate::write::Writable;

use super::FName;

#[derive(Debug, PartialEq)]
pub struct FNumberedPair {
    pub key: FName,
    pub value: u32, // FValueId
}

impl<W: Write> Writable<W> for FNumberedPair {
    #[instrument(name = "FNumberedPair_write", skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        self.key.write(writer)?;
        writer.write_u32::<LE>(self.value)?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for FNumberedPair {
    #[instrument(name = "FNumberedPair_read", skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let key = FName::read(reader)?;
        let value = reader.read_u32::<LE>()?;
        Ok(FNumberedPair { key, value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_roundtrip() {
        let pair = FNumberedPair {
            key: FName {
                index: 123,
                number: 456,
            },
            value: 789,
        };
        let mut buf = vec![];
        let mut writer = Cursor::new(&mut buf);
        pair.write(&mut writer).unwrap();
        let mut reader = Cursor::new(&buf);
        let read_pair = FNumberedPair::read(&mut reader).unwrap();
        assert_eq!(read_pair, pair);
    }
}
