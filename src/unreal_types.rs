use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::{eyre, Result as EResult};
use tracing::*;

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

pub struct TArray<T>(pub Vec<T>);

impl<W: Write, T: Writable<W>> Writable<W> for TArray<T> {
    // This sets `Ar.SetCustomVersion(Guid, VersionInt, TEXT("AssetRegistry"));` somehow.
    #[instrument(skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        // Not sure what ZSTs do...
        assert!(std::mem::size_of::<T>() >= 1);

        writer.write_u32::<LE>(self.0.len().try_into()?)?;

        for elem in &self.0 {
            elem.write(writer)?;
        }

        Ok(())
    }
}

impl<R: Read, T: Readable<R>> Readable<R> for TArray<T> {
    #[instrument(skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        // Not sure what ZSTs do...
        assert!(std::mem::size_of::<T>() >= 1);

        let len = reader.read_u32::<LE>()?;

        // Heuristic to check of the `len` may be crafted to trigger insane allocations / OOM.
        // Try to follow Unreal here and limit to 16MiB.
        let elem_size = std::mem::size_of::<T>() as u32;
        let max_capacity = (16 * 1024 * 1024) * elem_size;
        let expected_size = len.saturating_mul(elem_size);
        if expected_size > max_capacity {
            return Err(eyre!(
                "TArray too large for {len} elements of size {elem_size}"
            ));
        }
        let mut buf = Vec::with_capacity(expected_size as usize);
        for _ in 0..len {
            let elem = T::read(reader)?;
            buf.push(elem);
        }

        Ok(TArray(buf))
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
            match self {
                FString::Utf16(_) => {
                    return Err(eyre!("UTF-16 NUL-terminated strings cannot be empty"))
                }
                FString::Ansi(_) => return Ok(String::new()),
            }
        }

        match self {
            // We can slice [0..b.len() - 1] to skip the NUL-terminator without panicking now
            // because we checked for the `len() == 0` case.
            FString::Utf16(b) => Ok(String::from_utf16(&b[0..b.len() - 1])?),
            FString::Ansi(b) => Ok(String::from_utf8(b.to_vec())?),
        }
    }
}

impl From<String> for FString {
    fn from(value: String) -> Self {
        if is_pure_ansi(&value) {
            FString::Ansi(value.into_bytes())
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
