use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::{eyre, Result as EResult};
use tracing::*;

use crate::read::Readable;
use crate::write::Writable;

/// A [`FText`] is a NUL-terminated raw string with a len prepended when (de-)serializing.
#[derive(Debug, PartialEq, Clone)]
pub struct FText {
    raw: Vec<u8>,
}

impl From<String> for FText {
    fn from(value: String) -> Self {
        let mut bytes = value.into_bytes();
        bytes.push(b'\0');
        FText { raw: bytes }
    }
}

impl<'s> From<&'s str> for FText {
    fn from(value: &'s str) -> Self {
        let mut bytes = value.bytes().collect::<Vec<_>>();
        bytes.push(b'\0');
        FText { raw: bytes }
    }
}

impl FText {
    /// Try to convert a [`FText`] into a [`String`]. This will fail if the [`FText`]'s backing
    /// buffer is empty, does not contain a NUL-terminator, or if the [`FText`] contains invalid
    /// UTF-8 codepoints.
    pub fn try_into_string(&self) -> EResult<String> {
        if self.raw.is_empty() {
            return Err(eyre!("unexpected empty FText"));
        }

        let [start @ .., b'\0'] = &self.raw[..] else {
            return Err(eyre!("unexpected missing NUL-terminator"));
        };

        let s = String::from_utf8(start.to_vec())?;
        Ok(s)
    }
}

impl<W: Write> Writable<W> for FText {
    #[instrument(name = "FText_write", skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        writer.write_u32::<LE>(self.raw.len() as u32)?;
        writer.write_all(&self.raw)?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for FText {
    #[instrument(name = "FText_read", skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let len = reader.read_u32::<LE>()?;
        if len >= isize::MAX as u32 {
            return Err(eyre!("FText string length `{}` too large", len));
        }
        let mut buf = vec![0u8; len as usize];
        reader.read_exact(&mut buf)?;
        let last = buf.last().unwrap();
        if *last != b'\0' {
            return Err(eyre!("FText string isn't NUL-terminated!"));
        }
        Ok(FText { raw: buf })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_roundtrip() {
        let f = FText::from("Hello World!".to_string());
        let mut buf = vec![];
        let mut writer = Cursor::new(&mut buf);
        f.write(&mut writer).unwrap();
        let mut reader = Cursor::new(&buf);
        let read_f = FText::read(&mut reader).unwrap();
        assert_eq!(f, read_f);
    }
}
