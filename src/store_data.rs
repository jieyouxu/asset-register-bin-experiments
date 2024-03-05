use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::{eyre, Result as EResult};
use itertools::Itertools;
use tracing::*;

use crate::read::{read_array, Readable};
use crate::write::{write_array, Writable};

use crate::asset_registry_export_path::AssetRegistryExportPath;
use crate::unreal_types::{
    FDisplayNameEntryId, FName, FNumberedPair, FNumberlessExportPath, FNumberlessPair, FString,
};

/// See <https://github.com/EpicGames/UnrealEngine/blob/072300df18a94f18077ca20a14224b5d99fee872/Engine/Source/Runtime/CoreUObject/Public/AssetRegistry/AssetDataTagMapSerializationDetails.h#L97>.
///
/// ```cpp
/// struct FStoreData
///	{
///		TArray<FNumberedPair> Pairs;
///		TArray<FNumberlessPair> NumberlessPairs;
///
///		TArray<uint32> AnsiStringOffsets;
///		TArray<ANSICHAR> AnsiStrings;
///		TArray<uint32> WideStringOffsets;
///		TArray<WIDECHAR> WideStrings;
///		TArray<FDisplayNameEntryId> NumberlessNames;
///		TArray<FName> Names;
///		TArray<FNumberlessExportPath> NumberlessExportPaths;
///		TArray<FAssetRegistryExportPath> ExportPaths;
///		TArray<FMarshalledText> Texts;
///	};
/// ```
#[derive(Debug)]
pub struct StoreData {
    pub pairs: Vec<FNumberedPair>,
    pub numberless_pairs: Vec<FNumberlessPair>,

    pub ansi_strings: Vec<String>,
    pub wide_strings: Vec<String>,

    pub numberless_names: Vec<FDisplayNameEntryId>,
    pub names: Vec<FName>,
    pub numberless_export_paths: Vec<FNumberlessExportPath>,
    pub export_paths: Vec<AssetRegistryExportPath>,
    pub texts: Vec<FString>,
}

impl StoreData {
    pub const BEGIN_MAGIC: u32 = 0x12345679;
    pub const END_MAGIC: u32 = 0x87654321;
}

impl<W: Write> Writable<W> for StoreData {
    /// See <https://github.com/EpicGames/UnrealEngine/blob/release/Engine/Source/Runtime/CoreUObject/Private/AssetRegistry/AssetDataTagMap.cpp#L1146>.
    // SaveItem(BeginMagic);
    // VisitViews(Store, [&] (auto Array) { SaveItem(Array.Num()); });
    // SaveTextData(MakeArrayView(Store.Texts));
    // VisitViews<EOrder::SkipText>(Store, [&] (auto Array) { SaveViewData(MakeArrayView(Array)); });
    // SaveItem(EndMagic);
    #[instrument(skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        writer.write_u32::<LE>(Self::BEGIN_MAGIC)?;

        // `VisitViews(Store, [&] (auto Array) { SaveItem(Array.Num()); });` section

        // See <https://github.com/EpicGames/UnrealEngine/blob/release/Engine/Source/Runtime/CoreUObject/Private/AssetRegistry/AssetDataTagMap.cpp#L904>.
        writer.write_u32::<LE>(self.numberless_names.len() as u32)?;
        writer.write_u32::<LE>(self.names.len() as u32)?;
        writer.write_u32::<LE>(self.numberless_export_paths.len() as u32)?;
        writer.write_u32::<LE>(self.export_paths.len() as u32)?;
        writer.write_u32::<LE>(self.ansi_strings.len() as u32)?; // ansi string offsets
        writer.write_u32::<LE>(self.wide_strings.len() as u32)?; // wide string offsets
        writer.write_u32::<LE>(self.ansi_strings.len() as u32)?;
        writer.write_u32::<LE>(self.wide_strings.len() as u32)?;
        writer.write_u32::<LE>(self.numberless_pairs.len() as u32)?;
        writer.write_u32::<LE>(self.pairs.len() as u32)?;

        // `SaveTextData(MakeArrayView(Store.Texts));` section
        write_array(writer, &self.texts, |w, t| t.write(w))?;

        // `VisitViews<EOrder::SkipText>(Store, [&] (auto Array) { SaveViewData(MakeArrayView(Array)); });`
        write_array(writer, &self.numberless_names, |w, n| n.write(w))?;
        write_array(writer, &self.names, |w, n| n.write(w))?;
        write_array(writer, &self.numberless_export_paths, |w, p| p.write(w))?;
        write_array(writer, &self.export_paths, |w, p| p.write(w))?;

        let ansi_string_offsets = {
            let mut offsets = vec![];
            let mut offset = 0;
            for s in &self.ansi_strings {
                todo!()
            }
        };

        write_array(writer, &self.ansi_string_offsets, |w, o| {
            w.write_u32::<LE>(*o)
        })?;
        write_array(writer, &self.wide_string_offsets, |w, o| {
            w.write_u32::<LE>(*o)
        })?;
        write_array(writer, &self.ansi_strings, |w, s| -> EResult<()> {
            w.write_all(s.as_bytes())?;
            w.write_u8(0)?;
            Ok(())
        })?;
        write_array(writer, &self.wide_strings, |w, s| -> EResult<()> {
            for e in s.encode_utf16() {
                w.write_u16::<LE>(e)?;
            }
            w.write_u16::<LE>(0)?;
            Ok(())
        })?;
        write_array(writer, &self.numberless_pairs, |w, p| p.write(w))?;
        write_array(writer, &self.pairs, |w, p| p.write(w))?;

        writer.write_u32::<LE>(Self::END_MAGIC)?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for StoreData {
    #[instrument(skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let begin_magic = reader.read_u32::<LE>()?;
        if begin_magic != Self::BEGIN_MAGIC {
            return Err(eyre!(
                "StoreData begin match mismatch: expected {:X} got {:X}",
                Self::BEGIN_MAGIC,
                begin_magic
            ));
        }

        let numberless_names_len = reader.read_u32::<LE>()?;
        let names_len = reader.read_u32::<LE>()?;
        let numberless_export_paths_len = reader.read_u32::<LE>()?;
        let export_paths_len = reader.read_u32::<LE>()?;
        let ansi_string_offsets_len = reader.read_u32::<LE>()?;
        let wide_string_offsets_len = reader.read_u32::<LE>()?;
        let ansi_strings_len = reader.read_u32::<LE>()?;
        let wide_strings_len = reader.read_u32::<LE>()?;
        let numberless_pairs_len = reader.read_u32::<LE>()?;
        let pairs_len = reader.read_u32::<LE>()?;

        let texts = read_array(reader.read_u32::<LE>()?, reader, FString::read)?;

        let numberless_names = read_array(numberless_names_len, reader, FDisplayNameEntryId::read)?;
        let names = read_array(names_len, reader, FName::read)?;
        let numberless_export_paths = read_array(
            numberless_export_paths_len,
            reader,
            FNumberlessExportPath::read,
        )?;
        let export_paths = read_array(export_paths_len, reader, AssetRegistryExportPath::read)?;
        let ansi_string_offsets = read_array(ansi_string_offsets_len, reader, |reader| {
            reader.read_u32::<LE>()
        })?;
        let wide_string_offsets = read_array(wide_string_offsets_len, reader, |reader| {
            reader.read_u32::<LE>()
        })?;

        let ansi_strings = {
            let mut ansi_strings = vec![];
            for (current_offset, next_offset) in ansi_string_offsets
                .iter()
                .chain(std::iter::once(&ansi_strings_len))
                .tuple_windows()
            {
                if current_offset >= next_offset {
                    return Err(eyre!("unexpected ANSI string offset: next offset {next_offset} is smaller than or equal to current offset {current_offset}"));
                }

                let expected_len = next_offset - current_offset;

                let mut buf = Vec::with_capacity(expected_len as usize);
                reader.read_exact(&mut buf[..])?;

                if buf[(expected_len as usize) - 1] != 0 {
                    return Err(eyre!("ANSI string is not NUL-terminated"));
                }

                let s = String::from_utf8_lossy(&buf[..(expected_len as usize) - 1]);
                ansi_strings.push(s.into_owned());
            }
            ansi_strings
        };

        let wide_strings = {
            let mut wide_strings = vec![];
            for (current_offset, next_offset) in wide_string_offsets
                .iter()
                .chain(std::iter::once(&wide_strings_len))
                .tuple_windows()
            {
                if current_offset >= next_offset {
                    return Err(eyre!("unexpected wide string offset: next offset {next_offset} is smaller than or equal to current offset {current_offset}"));
                }

                let expected_len = next_offset - current_offset;

                let mut buf = Vec::with_capacity(expected_len as usize);
                reader.read_exact(&mut buf[..])?;

                if buf[(expected_len as usize) - 1] != 0 {
                    return Err(eyre!("wide string is not NUL-terminated"));
                }

                let s = String::from_utf16le_lossy(&buf[..(expected_len as usize) - 1]);
                wide_strings.push(s);
            }
            wide_strings
        };

        let numberless_pairs = read_array(numberless_pairs_len, reader, FNumberlessPair::read)?;
        let pairs = read_array(pairs_len, reader, FNumberedPair::read)?;

        let end_magic = reader.read_u32::<LE>()?;
        if end_magic != Self::END_MAGIC {
            return Err(eyre!(
                "StoreData begin match mismatch: expected {:X} got {:X}",
                Self::END_MAGIC,
                end_magic
            ));
        }

        Ok(StoreData {
            pairs,
            numberless_pairs,
            ansi_strings,
            wide_strings,
            numberless_names,
            names,
            numberless_export_paths,
            export_paths,
            texts,
        })
    }
}
