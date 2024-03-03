use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::{eyre, Result as EResult, WrapErr};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use tracing::*;

use crate::read::Readable;
use crate::write::Writable;

/// See <https://github.com/EpicGames/UnrealEngine/blob/072300df18a94f18077ca20a14224b5d99fee872/Engine/Source/Runtime/CoreUObject/Private/AssetRegistry/AssetData.cpp#L28>.
pub const ASSET_REGISTRY_VERSION_GUID: [u8; 16] = [
    0xE7, 0x9E, 0x7F, 0x71, 0x3A, 0x49, 0xB0, 0xE9, 0x32, 0x91, 0xB3, 0x88, 0x07, 0x81, 0x38, 0x1B,
];

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
    // * Target tag INI settings cooked into tag data
    // * Instead of FString values are stored directly as one of:
    //		- Narrow / wide string
    //		- [Numberless] FName
    //		- [Numberless] export path
    //		- Localized string
    // * All value types are deduplicated
    // * All key-value maps are cooked into a single contiguous range
    // * Switched from FName table to seek-free and more optimized FName batch loading
    // * Removed global tag storage, a tag map reference-counts one store per asset registry
    // * All configs can mix fixed and loose tag maps
    WorkspaceDomain,                 // Added Version information to AssetPackageData
    PackageImportedClasses,          // Added ImportedClasses to AssetPackageData
    PackageFileSummaryVersionChange, // A new version number of UE5 was added to FPackageFileSummary
    ObjectResourceOptionalVersionChange, // Change to linker export/import resource serialization
    AddedChunkHashes, // Added FIoHash for each FIoChunkId in the package to the AssetPackageData.
    ClassPaths, // Classes are serialized as path names rather than short object names, e.g. /Script/Engine.StaticMesh
    RemoveAssetPathFNames, // Asset bundles are serialized as FTopLevelAssetPath instead of FSoftObjectPath, deprecated FAssetData::ObjectPath
    AddedHeader,           // Added header with bFilterEditorOnlyData flag
    AssetPackageDataHasExtension, // Added Extension to AssetPackageData.
}

impl AssetRegistryVersion {
    pub const LATEST_VERSION: AssetRegistryVersion =
        AssetRegistryVersion::AssetPackageDataHasExtension;
}

impl<W: Write> Writable<W> for AssetRegistryVersion {
    /// See <https://github.com/EpicGames/UnrealEngine/blob/072300df18a94f18077ca20a14224b5d99fee872/Engine/Source/Runtime/CoreUObject/Private/AssetRegistry/AssetData.cpp#L849>.
    // - <https://github.com/EpicGames/UnrealEngine/blob/release/Engine/Source/Runtime/Core/Private/Serialization/Archive.cpp#L594>.
    // - <https://github.com/EpicGames/UnrealEngine/blob/072300df18a94f18077ca20a14224b5d99fee872/Engine/Source/Runtime/Core/Private/Serialization/CustomVersion.cpp#L285>
    // - <https://github.com/EpicGames/UnrealEngine/blob/072300df18a94f18077ca20a14224b5d99fee872/Engine/Source/Runtime/Core/Public/Serialization/StructuredArchiveSlots.h#L23>.
    // This sets `Ar.SetCustomVersion(Guid, VersionInt, TEXT("AssetRegistry"));` somehow.
    #[instrument(skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        writer.write_all(&ASSET_REGISTRY_VERSION_GUID)?;
        writer.write_u32::<LE>((*self).into())?;

        Ok(())
    }
}

impl<R: Read> Readable<R> for AssetRegistryVersion {
    #[instrument(skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let mut guid = [0u8; 16];
        reader
            .read_exact(&mut guid)
            .wrap_err_with(|| "failed to read GUID")?;
        if guid != ASSET_REGISTRY_VERSION_GUID {
            return Err(eyre!(
                "invalid guid: expected `{:x?}` but found `{:x?}`",
                ASSET_REGISTRY_VERSION_GUID,
                guid
            ));
        }
        let version_int = reader.read_u32::<LE>()?;
        let version = AssetRegistryVersion::try_from(version_int)
            .wrap_err_with(|| format!("unexpected AssetRegistryVersion: got {version_int}"))?;
        Ok(version)
    }
}
