use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::{Result as EResult, WrapErr};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use tracing::*;

use crate::read::Readable;
use crate::write::Writable;

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum AssetRegistryVersion {
    PreVersioning = 0,     // From before file versioning was implemented
    HardSoftDependencies, // The first version of the runtime asset registry to include file versioning.
    AddAssetRegistryState, // Added FAssetRegistryState and support for piecemeal serialization
    ChangedAssetData, // AssetData serialization format changed, versions before this are not readable
    RemovedMD5Hash,   // Removed MD5 hash from package data
    AddedHardManage,  // Added hard/soft manage references
    AddedCookedMD5Hash, // Added MD5 hash of cooked package to package data
    AddedDependencyFlags, // Added UE::AssetRegistry::EDependencyProperty to each dependency
    FixedTags,        // Major tag format change that replaces USE_COMPACT_ASSET_REGISTRY:
}

impl AssetRegistryVersion {
    // For 4.27.2
    pub const LATEST_VERSION: AssetRegistryVersion = AssetRegistryVersion::FixedTags;
}

impl<W: Write> Writable<W> for AssetRegistryVersion {
    #[instrument(name = "AssetRegistryVersion_write", skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        writer.write_u32::<LE>((*self).into())?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for AssetRegistryVersion {
    #[instrument(name = "AssetRegistryVersion_read", skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let version_int = reader.read_u32::<LE>()?;
        let version = AssetRegistryVersion::try_from(version_int)
            .wrap_err_with(|| format!("unexpected AssetRegistryVersion: got {version_int}"))?;
        Ok(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Cursor;

    #[test]
    fn test_roundtrip() {
        let mut buf = [0u8; 4];
        let mut writer = Cursor::new(&mut buf[..]);
        AssetRegistryVersion::LATEST_VERSION
            .write(&mut writer)
            .unwrap();
        let mut reader = Cursor::new(&buf[..]);
        let version = AssetRegistryVersion::read(&mut reader).unwrap();
        assert_eq!(version, AssetRegistryVersion::LATEST_VERSION);
    }
}
