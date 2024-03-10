use crate::unreal_types::{FName, FString};

#[derive(Debug, PartialEq)]
pub struct FSoftObjectPath {
    asset_path_name: FName,
    sub_path_string: FString,
}
