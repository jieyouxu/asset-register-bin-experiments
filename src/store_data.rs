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
    /// TODO
    pub pairs: (),
    /// TODO
    pub numberless_pairs: (),

    pub ansi_string_offsets: u32,
    /// TODO
    pub ansi_strings: Vec<u8>,

    pub wide_string_offsets: u32,
    /// TODO
    pub wide_strings: Vec<u16>,

    /// TODO
    pub numberless_names: (),
    /// TODO
    pub names: (),
    /// TODO
    pub numberless_export_paths: (),
    /// TODO
    pub export_paths: (),
    /// TODO
    pub texts: (),
}
