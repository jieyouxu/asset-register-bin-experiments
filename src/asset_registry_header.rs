use std::io::{Read, Write};

use color_eyre::eyre::Result as EResult;
use tracing::*;

use crate::read::Readable;
use crate::unreal_types::UnrealBool;
use crate::write::Writable;

use crate::asset_registry_version::AssetRegistryVersion;

#[derive(Debug)]
pub struct AssetRegistryHeader {
    pub version: AssetRegistryVersion,
    // WTF does this do? Do we want to set it for mint?
    pub filter_editor_only_data: bool,
}

impl<W: Write> Writable<W> for AssetRegistryHeader {
    /// See <https://github.com/EpicGames/UnrealEngine/blob/072300df18a94f18077ca20a14224b5d99fee872/Engine/Source/Runtime/CoreUObject/Private/AssetRegistry/AssetData.cpp#L849>.
    // - <https://github.com/EpicGames/UnrealEngine/blob/release/Engine/Source/Runtime/Core/Private/Serialization/Archive.cpp#L594>.
    // - <https://github.com/EpicGames/UnrealEngine/blob/072300df18a94f18077ca20a14224b5d99fee872/Engine/Source/Runtime/Core/Private/Serialization/CustomVersion.cpp#L285>
    // - <https://github.com/EpicGames/UnrealEngine/blob/072300df18a94f18077ca20a14224b5d99fee872/Engine/Source/Runtime/Core/Public/Serialization/StructuredArchiveSlots.h#L23>.
    // This sets `Ar.SetCustomVersion(Guid, VersionInt, TEXT("AssetRegistry"));` somehow.
    #[instrument(skip_all)]
    fn write(&self, writer: &mut W) -> EResult<()> {
        self.version.write(writer)?;
        if self.version >= AssetRegistryVersion::AddedHeader {
            UnrealBool(self.filter_editor_only_data).write(writer)?;
        }
        Ok(())
    }
}

impl<R: Read> Readable<R> for AssetRegistryHeader {
    #[instrument(skip_all)]
    fn read(reader: &mut R) -> EResult<Self> {
        let version = AssetRegistryVersion::read(reader)?;
        let filter_editor_only_data = UnrealBool::read(reader)?;

        Ok(AssetRegistryHeader {
            version,
            filter_editor_only_data: filter_editor_only_data.0,
        })
    }
}
