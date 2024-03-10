use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use color_eyre::eyre::{eyre, Result as EResult};
use tracing::*;

use crate::read::Readable;
use crate::write::Writable;

use crate::unreal_types::FName;

use super::FAssetBundleEntry;

#[derive(Debug, PartialEq)]
pub struct AssetData {
    object_path: FName,
    package_path: FName,
    asset_class: FName,
    package_name: FName,
    asset_name: FName,
    tags: u64,
    bundles: Vec<FAssetBundleEntry>,
}
