use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::{eyre, Result as EResult};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use tracing::*;

use crate::asset_registry_export_path::AssetRegistryExportPath;
use crate::read::Readable;
use crate::write::Writable;

/// <https://docs.unrealengine.com/5.0/en-US/API/Runtime/Core/GenericPlatform/FGenericPlatformTypes/ANSICHAR/>.
pub type Ansichar = u8;

/// Serializes to 32-bits... when in legacy Unreal bool type.
pub struct UnrealBool(pub bool);

impl<W: Write> Writable<W> for UnrealBool {
    // This sets `Ar.SetCustomVersion(Guid, VersionInt, TEXT("AssetRegistry"));` somehow.
    #[instrument(skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        writer.write_u32::<LE>(match self.0 {
            true => 1,
            false => 0,
        })?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for UnrealBool {
    #[instrument(skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        match reader.read_u32::<LE>()? {
            0 => Ok(UnrealBool(false)),
            1 => Ok(UnrealBool(true)),
            v => Err(eyre!("failed to serialize {:X} as legacy unreal bool", v)),
        }
    }
}

/// [`FString`]s are *not* guaranteed to be UTF-8: they can be UTF-16, they can be ANSI.
/// - If the underlying bytes are not pure-ANSI, then Unreal serializes the bytes into a
///   NUL-terminated UTF-16 string, with its number of bytes encoded by a negative sign to indicate
///   it is a UTF-16 string.
/// - If the underlying bytes are pure-ANSI, then Unreal serializes the bytes as-is with no
///   NUL-terminator. The number of bytes are encoded by a positive sign to indicate it is a
///   ANSI string.
///
/// The UTF-16 variant string **must** be NUL-terminated. Favor `FString::from(s)` over directly
/// constructing this type.
#[derive(Debug, PartialEq)]
pub enum FString {
    Utf16(Vec<u16>),
    /// ASCII encoding contains 128 characters. ANSI encoding contains 256 characters.
    Ansi(Vec<u8>),
}

impl FString {
    pub fn len(&self) -> usize {
        match self {
            FString::Utf16(b) => b.len(),
            FString::Ansi(b) => b.len(),
        }
    }

    /// This method is fallible because ANSI strings are not necessarily valid UTF-8 strings, and
    /// the UTF-16 string might be invalid.
    pub fn to_string(&self) -> EResult<String> {
        if self.len() == 0 {
            return Err(eyre!("UTF-16 NUL-terminated strings cannot be empty"));
        }

        match self {
            // We can slice [0..b.len() - 1] to skip the NUL-terminator without panicking now
            // because we checked for the `len() == 0` case.
            FString::Utf16(b) => Ok(String::from_utf16(&b[0..b.len() - 1])?),
            FString::Ansi(b) => Ok(String::from_utf8(b[0..b.len() - 1].to_vec())?),
        }
    }
}

impl From<String> for FString {
    fn from(value: String) -> Self {
        if is_pure_ansi(&value) {
            let mut buf = value.into_bytes();
            buf.push(0);
            FString::Ansi(buf)
        } else {
            let mut buf: Vec<u16> = value.encode_utf16().collect();
            // NUL-terminator
            buf.push(0);
            FString::Utf16(buf)
        }
    }
}

/// static bool IsPureAnsi(const WIDECHAR* Str, const int32 Len)
/// {
/// 	// Consider SSE version if this function takes significant amount of time
/// 	uint32 Result = 0;
/// 	for (int32 I = 0; I < Len; ++I)
/// 	{
/// 		Result |= TChar<WIDECHAR>::ToUnsigned(Str[I]);
/// 	}
/// 	return !(Result & 0xffffff80u);
/// }
fn is_pure_ansi(s: &str) -> bool {
    let mut res = 0u32;
    for b in s.chars() {
        let b = b as u32;
        res |= b;
    }
    !(res & 0xFFFF_FF80) != 0
}

impl<W: Write> Writable<W> for FString {
    // This sets `Ar.SetCustomVersion(Guid, VersionInt, TEXT("AssetRegistry"));` somehow.
    #[instrument(skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        match self {
            FString::Utf16(s) => {
                // We assume from `FString` invariant that the NUL-terminator is already there.
                writer.write_i32::<LE>(-(s.len() as i32))?;
                for c in s {
                    writer.write_u16::<LE>(*c)?;
                }
                Ok(())
            }
            FString::Ansi(s) => {
                writer.write_i32::<LE>(s.len() as i32)?;
                writer.write_all(s)?;
                Ok(())
            }
        }
    }
}

// You got big problems if your single string is larger than 16MB...
const MAX_STRING_SERIALIZATION_SIZE: usize = 16 * 1024 * 1024;

impl<R: Read> Readable<R> for FString {
    #[instrument(skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let len = reader.read_i32::<LE>()?;
        // `len < 0` means UTF-16 with NUL-terminator.
        let load_utf16 = len < 0;
        let n_bytes = if load_utf16 {
            // `len` cannot be negated due to integer overflow, archive is definitely
            // corrupted.
            if len == i32::MIN {
                return Err(eyre!("failed to read a FString with len {len:x}"));
            }

            len.wrapping_abs() as usize
        } else {
            len as usize
        };

        if n_bytes > MAX_STRING_SERIALIZATION_SIZE {
            return Err(eyre!("FString too large: max serialization size is {MAX_STRING_SERIALIZATION_SIZE} but got len {n_bytes}"));
        }

        let s = match load_utf16 {
            true => {
                let mut buf = Vec::with_capacity(n_bytes.next_multiple_of(2));
                reader.read_u16_into::<LE>(&mut buf)?;
                FString::Utf16(buf)
            }
            false => {
                let mut buf = Vec::with_capacity(n_bytes);
                reader.read_exact(&mut buf[..])?;
                FString::Ansi(buf)
            }
        };

        Ok(s)
    }
}

/// Public name, available to the world.  Names are stored as a combination of
/// an index into a table of unique strings and an instance number.
/// Names are case-insensitive, but case-preserving (when WITH_CASE_PRESERVING_NAME is 1)
///
//// See <https://github.com/EpicGames/UnrealEngine/blob/072300df18a94f18077ca20a14224b5d99fee872/Engine/Source/Runtime/Core/Public/UObject/NameTypes.h#L568>.
#[derive(Debug, Copy, Clone)]
pub struct FName {
    /// Index into the Names array (used to find String portion of the string/number pair used for comparison).
    pub comparison_index: FNameEntryId,
    /// Number portion of the string/number pair (stored internally as 1 more than actual, so zero'd
    /// memory will be the default, no-instance case).
    pub number: u32,
}

impl<W: Write> Writable<W> for FName {
    // This sets `Ar.SetCustomVersion(Guid, VersionInt, TEXT("AssetRegistry"));` somehow.
    #[instrument(skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        self.comparison_index.write(writer)?;
        writer.write_u32::<LE>(self.number)?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for FName {
    #[instrument(skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let comparison_index = FNameEntryId::read(reader)?;
        let number = reader.read_u32::<LE>()?;
        Ok(FName {
            comparison_index,
            number,
        })
    }
}

/// See <https://github.com/EpicGames/UnrealEngine/blob/072300df18a94f18077ca20a14224b5d99fee872/Engine/Source/Runtime/Core/Public/UObject/NameTypes.h#L1705>.
///
/// Stores 32-bit display entry id with an unused bit to indicate if
/// `FName::GetComparisonIdFromDisplayId` lookup is needed.
/// Note that only display entries should be saved to make output deterministic.
#[derive(Debug)]
pub struct FDisplayNameEntryId(pub FNameEntryId);

impl<W: Write> Writable<W> for FDisplayNameEntryId {
    // This sets `Ar.SetCustomVersion(Guid, VersionInt, TEXT("AssetRegistry"));` somehow.
    #[instrument(skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        self.0.write(writer)
    }
}

impl<R: Read> Readable<R> for FDisplayNameEntryId {
    #[instrument(skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        Ok(FDisplayNameEntryId(FNameEntryId::read(reader)?))
    }
}

/// See <https://github.com/EpicGames/UnrealEngine/blob/072300df18a94f18077ca20a14224b5d99fee872/Engine/Source/Runtime/Core/Public/UObject/NameTypes.h#L51>.
/// See <https://github.com/EpicGames/UnrealEngine/blob/072300df18a94f18077ca20a14224b5d99fee872/Engine/Source/Runtime/Core/Private/UObject/UnrealNames.cpp#L281>.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct FNameEntryId(pub u32);

impl<W: Write> Writable<W> for FNameEntryId {
    // This sets `Ar.SetCustomVersion(Guid, VersionInt, TEXT("AssetRegistry"));` somehow.
    #[instrument(skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        writer.write_u32::<LE>(self.0)?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for FNameEntryId {
    #[instrument(skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        Ok(FNameEntryId(reader.read_u32::<LE>()?))
    }
}

/// See <https://github.com/EpicGames/UnrealEngine/blob/072300df18a94f18077ca20a14224b5d99fee872/Engine/Source/Developer/ToolMenus/Public/ToolMenuOwner.h#L22>.
#[derive(Debug, Copy, Clone, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum ValueType {
    None,
    Pointer,
    Name,
}

/// [`FValueId`] is packed with two components: a type and an index.\
/// 31       2     0
/// Index    Type
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct FValueId(u32);

impl FValueId {
    pub const TYPE_BITS: u32 = 3;
    pub const INDEX_BITS: u32 = 32 - Self::TYPE_BITS;
}

impl FValueId {
    pub fn value_type(&self) -> ValueType {
        ValueType::try_from(self.0 & 0b111).expect("invalid ValueType")
    }

    pub fn index(&self) -> u32 {
        self.0 >> Self::TYPE_BITS
    }

    pub fn try_from_u32(n: u32) -> EResult<Self> {
        // Check value type.
        let _ = ValueType::try_from(n & 0b111)?;
        Ok(FValueId(n))
    }
}

impl<W: Write> Writable<W> for FValueId {
    // This sets `Ar.SetCustomVersion(Guid, VersionInt, TEXT("AssetRegistry"));` somehow.
    #[instrument(skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        Ok(writer.write_u32::<LE>(self.0)?)
    }
}

impl<R: Read> Readable<R> for FValueId {
    #[instrument(skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let n = reader.read_u32::<LE>()?;
        Ok(FValueId::try_from_u32(n)?)
    }
}

/// See <https://github.com/EpicGames/UnrealEngine/blob/release/Engine/Source/Runtime/CoreUObject/Private/AssetRegistry/AssetDataTagMap.cpp#L1057>.
#[derive(Debug)]
pub struct FNumberedPair {
    pub key: FName,
    pub value: FValueId,
}

impl<W: Write> Writable<W> for FNumberedPair {
    // This sets `Ar.SetCustomVersion(Guid, VersionInt, TEXT("AssetRegistry"));` somehow.
    #[instrument(skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        self.key.write(writer)?;
        self.value.write(writer)?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for FNumberedPair {
    #[instrument(skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let key = FName::read(reader)?;
        let value = FValueId::read(reader)?;
        Ok(FNumberedPair { key, value })
    }
}

/// See <https://github.com/EpicGames/UnrealEngine/blob/release/Engine/Source/Runtime/CoreUObject/Private/AssetRegistry/AssetDataTagMap.cpp#L1045>.
#[derive(Debug)]
pub struct FNumberlessPair {
    pub key: FDisplayNameEntryId,
    pub value: FValueId,
}

impl<W: Write> Writable<W> for FNumberlessPair {
    // This sets `Ar.SetCustomVersion(Guid, VersionInt, TEXT("AssetRegistry"));` somehow.
    #[instrument(skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        self.key.write(writer)?;
        self.value.write(writer)?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for FNumberlessPair {
    #[instrument(skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let key = FDisplayNameEntryId::read(reader)?;
        let value = FValueId::read(reader)?;
        Ok(FNumberlessPair { key, value })
    }
}

#[derive(Debug)]
pub struct FTopLevelAssetPath {
    pub package_name: FName,
    pub asset_name: FName,
}

impl<W: Write> Writable<W> for FTopLevelAssetPath {
    // This sets `Ar.SetCustomVersion(Guid, VersionInt, TEXT("AssetRegistry"));` somehow.
    #[instrument(skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        self.package_name.write(writer)?;
        self.asset_name.write(writer)?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for FTopLevelAssetPath {
    #[instrument(skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let package_name = FName::read(reader)?;
        let asset_name = FName::read(reader)?;
        Ok(FTopLevelAssetPath {
            package_name,
            asset_name,
        })
    }
}

/// See <https://github.com/EpicGames/UnrealEngine/blob/072300df18a94f18077ca20a14224b5d99fee872/Engine/Source/Runtime/CoreUObject/Public/AssetRegistry/AssetDataTagMap.h#L68>.
/// See <https://github.com/EpicGames/UnrealEngine/blob/release/Engine/Source/Runtime/CoreUObject/Private/AssetRegistry/AssetDataTagMap.cpp#L299>.
#[derive(Debug)]
pub struct FNumberlessExportPath(pub AssetRegistryExportPath);

impl<W: Write> Writable<W> for FNumberlessExportPath {
    // This sets `Ar.SetCustomVersion(Guid, VersionInt, TEXT("AssetRegistry"));` somehow.
    #[instrument(skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        self.0.write(writer)
    }
}

impl<R: Read> Readable<R> for FNumberlessExportPath {
    #[instrument(skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let inner = AssetRegistryExportPath::read(reader)?;
        Ok(FNumberlessExportPath(inner))
    }
}
