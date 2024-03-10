use crate::unreal_types::FName;

use super::FSoftObjectPath;

#[derive(Debug, PartialEq)]
pub struct FAssetBundleEntry {
    bundle_name: FName,
    bundles: Vec<FSoftObjectPath>,
}
