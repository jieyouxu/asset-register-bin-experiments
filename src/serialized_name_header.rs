use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::{eyre, Result as EResult, WrapErr};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use tracing::*;

use crate::read::Readable;
use crate::write::Writable;

#[derive(Debug, PartialEq, Clone)]
pub struct SerializedNameHeader {
    pub is_utf16: bool,
    /// Number of `u8` or `u16` elements; use [`SerializedNameHeader::n_bytes`] to get the number
    /// of corresponding bytes.
    pub len: u16,
}

impl SerializedNameHeader {
    pub fn n_bytes(&self) -> u32 {
        match self.is_utf16 {
            true => self.len as u32 * std::mem::size_of::<u16>() as u32,
            false => self.len as u32 * std::mem::size_of::<u8>() as u32,
        }
    }
}

impl<W: Write> Writable<W> for SerializedNameHeader {
    #[instrument(name = "SerializedNameHeader_write", skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        let b0 = ((self.is_utf16 as u16) << 7 | self.len >> 8) as u8;
        let b1 = self.len as u8;
        writer.write_u8(b0)?;
        writer.write_u8(b1)?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for SerializedNameHeader {
    #[instrument(name = "SerializedNameHeader_read", skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let packed = reader.read_u16::<LE>()?;
        let bytes = packed.to_le_bytes();
        Ok(SerializedNameHeader {
            is_utf16: bytes[0] & 0x80 != 0,
            len: ((bytes[0] as u16 & 0x7F) << 8) + bytes[1] as u16,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_roundtrip() {
        let len = 0x1234u16;
        let is_utf16 = true;

        let mut buf = [0u8; 2];
        let mut writer = Cursor::new(&mut buf[..]);
        SerializedNameHeader { len, is_utf16 }
            .write(&mut writer)
            .unwrap();

        let mut reader = Cursor::new(&buf[..]);
        let header = SerializedNameHeader::read(&mut reader).unwrap();
        assert_eq!(header.len, len);
        assert_eq!(header.is_utf16, is_utf16);
    }
}
