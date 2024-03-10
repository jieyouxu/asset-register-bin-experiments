use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::{eyre, Result as EResult, WrapErr};
use itertools::Itertools;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use tracing::*;

use crate::read::{read_array, Readable};
use crate::unreal_types::*;
use crate::write::Writable;

pub const START_MAGIC: u32 = 0x12345679;
pub const END_MAGIC: u32 = 0x87654321;

#[derive(Debug, PartialEq)]
pub struct StoreData {
    pub text_data: Vec<FText>,
    pub numberless_names: Vec<FName>,
    pub names: Vec<FName>,
    pub numberless_export_paths: Vec<FAssetRegistryExportPath>,
    pub ansi_strings: Vec<String>,
    pub wide_strings: Vec<String>,
    pub numberless_pairs: Vec<FNumberedPair>,
    pub pairs: Vec<FNumberedPair>,
}

impl<W: Write> Writable<W> for StoreData {
    #[instrument(name = "StoreData_write", skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        writer.write_u32::<LE>(START_MAGIC)?;

        // === Counts (of elements and bytes) header ===
        writer.write_u32::<LE>(self.numberless_names.len() as u32)?;
        writer.write_u32::<LE>(self.names.len() as u32)?;
        writer.write_u32::<LE>(self.numberless_export_paths.len() as u32)?;
        writer.write_u32::<LE>(self.text_data.len() as u32)?;
        writer.write_u32::<LE>(self.ansi_strings.len() as u32)?;
        writer.write_u32::<LE>(self.wide_strings.len() as u32)?;

        {
            let mut ansi_string_bytes = 0u32;
            self.ansi_strings
                .iter()
                .for_each(|s| ansi_string_bytes += s.len() as u32 + 1);
            writer.write_u32::<LE>(ansi_string_bytes)?;
        }

        {
            let mut wide_string_bytes = 0u32;
            self.wide_strings
                .iter()
                .for_each(|s| wide_string_bytes += s.len() as u32 + 1);
            writer.write_u32::<LE>(wide_string_bytes)?;
        }

        writer.write_u32::<LE>(self.numberless_pairs.len() as u32)?;
        writer.write_u32::<LE>(self.pairs.len() as u32)?;

        // === Content ===
        write_array_content(writer, &self.text_data)?;
        write_array_content(writer, &self.numberless_names)?;
        write_array_content(writer, &self.names)?;
        write_array_content(writer, &self.numberless_export_paths)?;

        {
            let mut offset = 0u32;
            for s in &self.ansi_strings {
                writer.write_u32::<LE>(offset)?;
                offset += s.len() as u32 + 1;
            }
        }

        {
            let mut offset = 0u32;
            for s in &self.wide_strings {
                writer.write_u32::<LE>(offset)?;
                offset += s.len() as u32 + 1;
            }
        }

        self.ansi_strings.iter().try_for_each(|s| {
            writer.write_all(s.as_bytes())?;
            writer.write_u8(b'\0')
        })?;
        self.wide_strings.iter().try_for_each(|s| {
            let elems = s.encode_utf16().collect::<Vec<_>>();
            elems
                .into_iter()
                .try_for_each(|e| writer.write_u16::<LE>(e))?;
            writer.write_u8(b'\0')
        })?;

        write_array_content(writer, &self.numberless_pairs)?;
        write_array_content(writer, &self.pairs)?;

        writer.write_u32::<LE>(END_MAGIC)?;
        Ok(())
    }
}

fn write_array_content<W: Write, T: Writable<W>>(writer: &mut W, elements: &[T]) -> EResult<()> {
    elements.iter().try_for_each(|e| e.write(writer))
}

impl<R: Read> Readable<R> for StoreData {
    #[instrument(name = "StoreData_read", skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        {
            let start_magic = reader.read_u32::<LE>()?;
            if start_magic != START_MAGIC {
                return Err(eyre!(
                    "store data start magic mismatch: expected {:X} but found {:X}",
                    START_MAGIC,
                    start_magic,
                ));
            }
        }

        // === Header ===
        let numberless_names_count = reader.read_u32::<LE>()?;
        let names_count = reader.read_u32::<LE>()?;
        let numberless_export_paths_count = reader.read_u32::<LE>()?;
        let text_data_count = reader.read_u32::<LE>()?;
        let ansi_string_offsets_count = reader.read_u32::<LE>()?;
        let wide_string_offsets_count = reader.read_u32::<LE>()?;
        let ansi_string_bytes = reader.read_u32::<LE>()?;
        let wide_string_bytes = reader.read_u32::<LE>()?;
        let numberless_pairs_count = reader.read_u32::<LE>()?;
        let pairs_count = reader.read_u32::<LE>()?;

        // === Content ===
        let text_data = read_array(text_data_count, reader, FText::read)?;
        let numberless_names = read_array(numberless_names_count, reader, FName::read)?;
        let names = read_array(names_count, reader, FName::read)?;
        let numberless_export_paths = read_array(
            numberless_export_paths_count,
            reader,
            FAssetRegistryExportPath::read,
        )?;
        let ansi_string_offsets = read_array(ansi_string_offsets_count, reader, |reader| {
            reader.read_u32::<LE>()
        })?;
        let wide_string_offsets = read_array(wide_string_offsets_count, reader, |reader| {
            reader.read_u32::<LE>()
        })?;

        let ansi_strings = {
            // Packed ANSI strings
            let mut strings = vec![];
            for (offset, next_offset) in ansi_string_offsets
                .iter()
                .chain(std::iter::once(&ansi_string_bytes))
                .tuple_windows()
            {
                if offset >= next_offset {
                    return Err(eyre!(
                        "offset {:X} >= next offset {:X}, ANSI string offset is bad",
                        offset,
                        next_offset
                    ));
                }

                if *offset > ansi_string_bytes || *next_offset > ansi_string_bytes {
                    return Err(eyre!("offset exceeds claimed number of ANSI string bytes, invalid offsets or number of ANSI string bytes"));
                }

                let len = next_offset - offset;

                if len == 0 {
                    return Err(eyre!("ANSI string len is 0, ANSI string offset is bad",));
                } else if len > isize::MAX as u32 {
                    return Err(eyre!(
                        "ANSI string len {:X} larger than {:X}, ANSI string offset is bad",
                        len,
                        isize::MAX
                    ));
                }

                let mut buf = vec![0u8; len as usize - 1];
                reader.read_exact(&mut buf)?;
                let nul = reader.read_u8()?;
                if nul != b'\0' {
                    return Err(eyre!("ANSI string is not NUL-terminated"));
                }
                let s = String::from_utf8(buf)?;
                strings.push(s);
            }

            strings
        };

        let wide_strings = {
            // Packed wide strings
            let mut strings = vec![];
            for (offset, next_offset) in wide_string_offsets
                .iter()
                .chain(std::iter::once(&wide_string_bytes))
                .tuple_windows()
            {
                if offset >= next_offset {
                    return Err(eyre!(
                        "offset {:X} >= next offset {:X}, wide string offset is bad",
                        offset,
                        next_offset
                    ));
                }

                let len = next_offset - offset;

                if len == 0 {
                    return Err(eyre!("ANSI string len is 0, wide string offset is bad",));
                } else if len > isize::MAX as u32 {
                    return Err(eyre!(
                        "ANSI string len {:X} larger than {:X}, wide string offset is bad",
                        len,
                        isize::MAX
                    ));
                }

                let mut buf = vec![0u8; len as usize - 1];
                reader.read_exact(&mut buf)?;
                let nul = reader.read_u8()?;
                if nul != b'\0' {
                    return Err(eyre!("wide string is not NUL-terminated"));
                }
                let s = String::from_utf16le(&buf)?;
                strings.push(s);
            }

            strings
        };

        let numberless_pairs = read_array(numberless_pairs_count, reader, FNumberedPair::read)?;
        let pairs = read_array(pairs_count, reader, FNumberedPair::read)?;

        {
            let end_magic = reader.read_u32::<LE>()?;
            if end_magic != END_MAGIC {
                return Err(eyre!(
                    "store data end magic mismatch: expected {:X} but found {:X}",
                    END_MAGIC,
                    end_magic,
                ));
            }
        }

        Ok(StoreData {
            text_data,
            numberless_names,
            names,
            numberless_export_paths,
            ansi_strings,
            wide_strings,
            numberless_pairs,
            pairs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Cursor;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_roundtrip() {
        let store = StoreData {
            text_data: vec![FText::from("OwO")],
            numberless_names: vec![FName {
                index: 0,
                number: 567,
            }],
            names: vec![
                FName {
                    index: 134,
                    number: 536,
                },
                FName {
                    index: 621,
                    number: 999,
                },
            ],
            numberless_export_paths: vec![FAssetRegistryExportPath {
                class: FName {
                    index: 123,
                    number: 456,
                },
                object: FName {
                    index: 789,
                    number: 101,
                },
                package: FName {
                    index: 194,
                    number: 249,
                },
            }],
            ansi_strings: vec!["hewwo world".to_string(), "a".to_string()],
            wide_strings: vec![],
            numberless_pairs: vec![FNumberedPair {
                key: FName {
                    index: 192,
                    number: 795,
                },
                value: 3492,
            }],
            pairs: vec![],
        };
        let mut buf = vec![];
        let mut writer = Cursor::new(&mut buf);
        store.write(&mut writer).unwrap();
        let mut reader = Cursor::new(&buf);
        let read_store = StoreData::read(&mut reader).unwrap();
        assert_eq!(read_store, store);
    }
}
