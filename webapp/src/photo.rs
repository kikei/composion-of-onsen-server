use std::fmt::{self, Formatter, Display};
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

impl Display for Profile {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Profile::ORIGINAL_JPG => "o",
            Profile::SCALE_1600_JPG => "p1600",
            Profile::THUMBNAIL_256_JPG => "t256"
        })
    }
}

#[derive(Clone, PartialEq, Serialize, Debug)]
pub struct Photo {
    #[serde(skip_serializing)]
    pub id: String,
    pub profile: Profile,
    pub path: PathBuf
}
