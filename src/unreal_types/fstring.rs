use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::{eyre, Result as EResult};
use tracing::*;

use crate::read::Readable;
use crate::write::Writable;

#[derive(Debug, PartialEq)]
pub struct FString {
    inner: String,
}

impl From<String> for FString {
    fn from(value: String) -> Self {
        FString { inner: value }
    }
}

impl<'s> From<&'s str> for FString {
    fn from(value: &'s str) -> Self {
        FString {
            inner: value.to_string(),
        }
    }
}

impl<W: Write> Writable<W> for FString {
    #[instrument(name = "FString_write", skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        if self.inner.is_ascii() {
            writer.write_u32::<LE>(self.inner.len() as u32 + 1)?;
            writer.write_all(self.inner.as_bytes())?;
        } else {
            let buf = self.inner.encode_utf16().collect::<Vec<_>>();
            let len = buf.len() * std::mem::size_of::<u16>();
            writer.write_i32::<LE>(-(len as i32 + 1))?;

            for b in buf {
                writer.write_u16::<LE>(b)?;
            }
        }
        writer.write_u8(b'\0')?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for FString {
    #[instrument(name = "FString_read", skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let len = reader.read_i32::<LE>()?;
        debug!(%len);
        let s = match len {
            len if len > 0 => {
                let mut buf = vec![0u8; len as usize - 1];
                reader.read_exact(&mut buf)?;
                let nul = reader.read_u8()?;
                if nul != b'\0' {
                    return Err(eyre!("FString not NUL-terminated"));
                }
                String::from_utf8(buf)?
            }
            len if len == 0 => {
                return Err(eyre!("FString length cannot be 0"));
            }
            len if len < 0 => {
                let len = (-len) as usize;
                if (len - 1) % 2 != 0 {
                    return Err(eyre!(
                        "len without NUL byte not a multiple of 2, invalid FString"
                    ));
                }
                let mut buf = vec![0u8; len as usize - 1];
                reader.read_exact(&mut buf)?;
                let buf = buf
                    .chunks_exact(2)
                    .into_iter()
                    .map(|a| u16::from_le_bytes([a[0], a[1]]))
                    .collect::<Vec<_>>();

                let nul = reader.read_u8()?;
                if nul != b'\0' {
                    return Err(eyre!("FString not NUL-terminated"));
                }
                String::from_utf16(&buf)?
            }
            _ => unreachable!(),
        };
        Ok(FString { inner: s })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test_log::test]
    fn test_roundtrip_ansi() {
        let f = FString::from("OwO");
        let mut buf = vec![];
        let mut writer = Cursor::new(&mut buf);
        f.write(&mut writer).unwrap();
        let mut reader = Cursor::new(&buf);
        let read_f = FString::read(&mut reader).unwrap();
        assert_eq!(f, read_f);
    }

    #[test_log::test]
    fn test_roundtrip_unicode() {
        let f = FString::from("🙇");
        let mut buf = vec![];
        let mut writer = Cursor::new(&mut buf);
        f.write(&mut writer).unwrap();
        let mut reader = Cursor::new(&buf);
        let read_f = FString::read(&mut reader).unwrap();
        assert_eq!(f, read_f);
    }
}
