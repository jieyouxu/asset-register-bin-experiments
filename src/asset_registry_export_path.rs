use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::{eyre, Result as EResult};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use tracing::*;

use crate::read::Readable;
use crate::write::Writable;

use crate::unreal_types::{FName, FTopLevelAssetPath};

/// ```cpp
/// FTopLevelAssetPath ClassPath;
/// FName Package;
/// FName Object;
///
/// Ar << Path.ClassPath << Path.Object << Path.Package
/// ```
#[derive(Debug)]
pub struct AssetRegistryExportPath {
    pub class_path: FTopLevelAssetPath,
    pub package: FName,
    pub object: FName,
}

impl<W: Write> Writable<W> for AssetRegistryExportPath {
    // This sets `Ar.SetCustomVersion(Guid, VersionInt, TEXT("AssetRegistry"));` somehow.
    #[instrument(skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        self.class_path.write(writer)?;
        self.package.write(writer)?;
        self.object.write(writer)?;
        Ok(())
    }
}

impl<R: Read> Readable<R> for AssetRegistryExportPath {
    #[instrument(skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let class_path = FTopLevelAssetPath::read(reader)?;
        let package = FName::read(reader)?;
        let object = FName::read(reader)?;
        Ok(AssetRegistryExportPath {
            class_path,
            package,
            object,
        })
    }
}
