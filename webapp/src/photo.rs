use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
#[allow(non_camel_case_types)]
pub enum Profile {
    ORIGINAL_JPG,
    SCALE_1600_JPG,
    THUMBNAIL_256_JPG
}

#[derive(Clone, PartialEq, Serialize, Debug)]
pub struct Photo {
    #[serde(skip_serializing)]
    pub id: String,
    pub profile: Profile,
    pub path: PathBuf
}
