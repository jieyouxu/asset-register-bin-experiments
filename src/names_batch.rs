use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::{eyre, Result as EResult, WrapErr};
use tracing::*;

use crate::read::{read_array, Readable};
use crate::serialized_name_header::SerializedNameHeader;
use crate::write::{write_array, Writable};

#[derive(Debug, PartialEq, Clone)]
pub struct NamesBatch {
    // TODO: determine the actual hash and versions related to this.
    pub hash_version: u64,
    // FIXME: generate the hashes!
    pub hashes: Vec<u64>,
    // FIXME: generate the headers!
    pub headers: Vec<SerializedNameHeader>,
    pub strings: Vec<String>,
}

impl<W: Write> Writable<W> for NamesBatch {
    #[instrument(name = "NamesBatch_write", skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        assert_eq!(self.hashes.len(), self.headers.len());
        assert_eq!(self.headers.len(), self.strings.len());

        trace!(count = self.strings.len());
        writer.write_u32::<LE>(self.strings.len() as u32)?;

        // `s.len() + 1` for the terminating NUL-byte that we need to append.
        let string_bytes = self
            .strings
            .iter()
            .map(|s| (s.len() + 1) as u32)
            .sum::<u32>();
        writer.write_u32::<LE>(string_bytes)?;

        writer.write_u64::<LE>(self.hash_version)?;

        write_array(writer, &self.hashes, |w, h| w.write_u64::<LE>(*h))?;

        write_array(writer, &self.headers, |w, h| h.write(w))?;

        write_array(writer, &self.strings, |w, s| -> EResult<()> {
            w.write_all(s.as_bytes())?;
            w.write_u8(0)?;
            Ok(())
        })?;

        Ok(())
    }
}

impl<R: Read> Readable<R> for NamesBatch {
    #[instrument(name = "NamesBatch_read", skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let count = reader.read_u32::<LE>()?;
        debug!(count);

        let expected_string_bytes = reader.read_u32::<LE>()?;

        let hash_version = reader.read_u64::<LE>()?;
        let hashes = read_array(count, reader, |r| r.read_u64::<LE>())?;
        let headers = read_array(count, reader, SerializedNameHeader::read)?;

        let mut strings = Vec::with_capacity(count as usize);
        let mut processed_string_bytes = 0u32;
        for header @ SerializedNameHeader { is_utf16, len } in &headers {
            trace!(?header);
            if *len == 0 {
                return Err(eyre!(
                    "got unexpected zero-length NUL-terminated string, how did this happen?"
                ));
            }

            if processed_string_bytes.saturating_add(header.n_bytes()) > expected_string_bytes {
                return Err(eyre!(
                    "we got more string bytes than is expected from the NamesBatch header, what?"
                ));
            }

            if *is_utf16 {
                let mut buf = vec![0u16; *len as usize - 1];
                for _ in 0..len - 1 {
                    let b = reader.read_u16::<LE>()?;
                    buf.push(b);
                }
                // Assumes the NUL-byte is u16.
                let nul = reader.read_u16::<LE>()?;
                if nul != 0 {
                    return Err(eyre!("expected NUL-byte (u16), but got {nul:X}"));
                }
                let s = String::from_utf16(&buf)
                    .wrap_err_with(|| "failed to build a UTF-8 string from NamesBatch string")?;
                strings.push(s);
            } else {
                let mut buf = vec![0u8; *len as usize - 1];
                reader.read_exact(&mut buf)?;
                trace!(?buf);

                // Assumes the NUL-byte is u16.
                let nul = reader.read_u8()?;
                if nul != 0 {
                    return Err(eyre!("expected NUL-byte, but got {nul:X}"));
                }
                let s = String::from_utf8(buf)
                    .wrap_err_with(|| "failed to build a UTF-8 string from NamesBatch string")?;
                strings.push(s);
            }

            processed_string_bytes += header.n_bytes();
        }

        if processed_string_bytes != expected_string_bytes {
            return Err(eyre!("the NamesBatch header says to expect {:X} string bytes, but we processed {:X} string bytes", processed_string_bytes, expected_string_bytes));
        }

        Ok(NamesBatch {
            hash_version,
            hashes,
            headers,
            strings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_roundtrip() {
        let hash_version = 0xDEAD_BEEFu64;
        let hashes = vec![0xBAAAAAADu64, 0xDEADC0DE];
        let headers = vec![
            SerializedNameHeader {
                is_utf16: false,
                len: 3,
            },
            SerializedNameHeader {
                is_utf16: false,
                len: 2,
            },
        ];
        let strings = vec!["12".to_string(), "3".to_string()];

        let mut buf = vec![];
        let mut writer = Cursor::new(&mut buf);

        NamesBatch {
            hash_version: hash_version.clone(),
            hashes: hashes.clone(),
            headers: headers.clone(),
            strings: strings.clone(),
        }
        .write(&mut writer)
        .unwrap();

        let mut reader = Cursor::new(&buf);
        let names_batch = NamesBatch::read(&mut reader).unwrap();

        assert_eq!(names_batch.hash_version, hash_version);
        assert_eq!(names_batch.hashes, hashes);
        assert_eq!(names_batch.headers, headers);
        assert_eq!(names_batch.strings, strings);
    }
}
