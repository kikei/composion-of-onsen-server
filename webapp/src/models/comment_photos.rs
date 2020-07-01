use std::convert::TryFrom;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use image::{DynamicImage, ImageFormat, ImageResult, imageops::FilterType};

use crate::photo::{Photo, Profile};
use crate::models::Models;
use crate::utils::identifier::{IdGenerator, Generate};

type Image = DynamicImage;

pub const DIRECTORY_ORDER: &str = "/data/comments/order";
pub const DIRECTORY_UPLOAD: &str = "/data/comments/images";
pub const DIRECTORY_DELETED: &str = "/data/comments/images_deleted";

const PROFILES_UPLOAD: [Profile; 3] = [
    Profile::ORIGINAL_JPG,
    Profile::SCALE_1600_JPG,
    Profile::THUMBNAIL_256_JPG
];

const KEY_ID: &str = "id";
const KEY_PROFILE: &str = "prof";
const KEY_PATH: &str = "path";

#[derive(Clone, Copy)]
enum ResizeMethod {
    /// Keep aspect ratio and fit longer
    Scale(u32, u32, FilterType),

    /// Keep aspect ratio and fill size
    Fill(u32, u32, FilterType)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConvertOrder {
    id: String,
    /// Fullpath of source image file
    src: String,
    /// Relative path to convert
    dest: String,
    profile: Profile
}

impl ConvertOrder {
    pub fn process(self: &Self) -> Result<Photo, String> {
        // Load original image on temporary directory
        let src = Path::new(&self.src);
        let tmpimage = image::open(src)
            .map_err(|e| format!("Failed to open image, e: {}", &e))?;

        let dest = Path::new(&self.dest);
        let fullpath = Path::new(DIRECTORY_UPLOAD).join(&dest);

        std::fs::create_dir_all(fullpath.parent().unwrap())
            .map_err(|e| format!("Failed to create directory, \
                                    path: {:?}, e: {}", &fullpath, &e))?;
        ConvertRule::from(self.profile).convert(&tmpimage, &fullpath)
            .map_err(|e| format!("Failed to convert image, \
                                    profile: {:?}, e: {}", &self.profile, &e))?;
        Ok(Photo {
            id: self.id.clone(),
            profile: self.profile,
            path: dest.to_path_buf()
        })
    }

    fn dryrun(self: &Self) -> Photo {
        Photo {
            id: self.id.clone(),
            profile: self.profile.clone(),
            path: Path::new(&self.dest).to_path_buf()
        }
    }
}

struct ConvertRule {
    resize: Option<ResizeMethod>,
    format: ImageFormat
}

impl ConvertRule {
    fn convert(self: &Self, img: &Image, output: &Path) -> ImageResult<()> {
        match self.resize {
            None => img.save_with_format(output, self.format),
            Some(r) => (match r {
                ResizeMethod::Scale(w, h, f) => img.resize(w, h, f),
                ResizeMethod::Fill(w, h, f) => img.resize_to_fill(w, h, f)
            }).save_with_format(output, self.format)
        }
    }
}

impl TryFrom<Value> for Photo {
    type Error = String;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        // Value must be an object
        let obj = value.as_object()
            .ok_or(format!("Failed to get Photo from Value: {}", &value))?;
        // Extract each fields from the object.
        let pid = obj.get(KEY_ID).and_then(|v| v.as_str())
            .ok_or(format!("Maybe a bug: missing photo id"))?;
        let profile = obj.get(KEY_PROFILE)
            .and_then(|v| serde_json::from_value(v.to_owned()).ok())
            .ok_or(format!("Maybe a bug: missing profile"))?;
        let path = obj.get(KEY_PATH).and_then(|v| v.as_str())
            .ok_or(format!("Maybe a bug: missing path"))?;
        Ok(Photo {
            id: pid.to_string(),
            profile: profile,
            path: Path::new(path).to_owned()
        })
    }
}

impl From<Profile> for ConvertRule {
    fn from(profile: Profile) -> Self {
        match profile {
            Profile::ORIGINAL_JPG => {
                ConvertRule {
                    resize: None,
                    format: ImageFormat::Jpeg
                }
            },
            Profile::SCALE_1600_JPG => {
                ConvertRule {
                    resize: Some(ResizeMethod::Scale(1600, 1200,
                                                     FilterType::CatmullRom)),
                    format: ImageFormat::Jpeg
                }
            },
            Profile::THUMBNAIL_256_JPG => {
                ConvertRule {
                    resize: Some(ResizeMethod::Fill(256, 256,
                                                    FilterType::CatmullRom)),
                    format: ImageFormat::Jpeg
                }
            }
        }
    }
}

impl From<Photo> for Value {
    fn from(item: Photo) -> Self {
        json!({
            KEY_ID: item.id,
            KEY_PROFILE: item.profile,
            KEY_PATH: item.path
        })
    }
}


pub async fn setup<'a>(_models: &Models<'a>) -> Result<(), String> {
    std::fs::create_dir_all(Path::new(DIRECTORY_UPLOAD))
        .map_err(|e| format!("Unable to setup comment_images, e: {}", &e))
}

#[derive(Debug)]
pub struct PhotoPath {
    pub analysis: String,
    pub comment: String,
    pub id: String
}

impl PhotoPath {
    fn as_path(self: &Self, profile: &Profile) -> PathBuf {
        Path::new(&self.analysis)
            .join(&self.comment)
            .join(&self.id)
            .join(format!("{}.jpg", profile))
    }
    fn directory(self: &Self) -> PathBuf {
        Path::new(&self.analysis)
            .join(&self.comment)
            .join(&self.id)
    }
}

impl TryFrom<PathBuf> for PhotoPath {
    type Error = String;

    fn try_from(item: PathBuf) -> Result<Self, Self::Error> {
        let parts = item.iter().map(|p| p.to_str().unwrap()).collect::<Vec<&str>>();
        if parts.len() < 4 {
            return Err(format!("Cannot convert to PhotoPath, parts: {:?}",
                               &parts));
        }
        Ok(PhotoPath {
            analysis: parts[parts.len() - 4].to_string(),
            comment: parts[parts.len() - 3].to_string(),
            id: parts[parts.len() - 2].to_string()
        })
    }
}

// Generate unique id for photo
pub struct CommentPhotoIdGenerator<'a>(IdGenerator<(&'static str, &'a str, &'a str)>);

impl<'a> CommentPhotoIdGenerator<'a> {
    pub fn new(analysis: &'a str, filename: &'a str) -> Self {
        CommentPhotoIdGenerator(IdGenerator::new(("comment", analysis, filename)))
    }
}

impl<'a> Generate for CommentPhotoIdGenerator<'a> {
    fn generate(self: &Self) -> String {
        self.0.generate()
    }
}

pub async fn save<'a>(_: &Models<'a>, src: &Path, dest: &PhotoPath) ->
    Result<Vec<Photo>, String>
{
    let mut orders = Vec::new();

    for profile in &PROFILES_UPLOAD {
        let path = (&dest).as_path(profile);
        let order = ConvertOrder {
            id: dest.id.clone(),
            src: src.to_str().unwrap().to_string(),
            dest: path.to_str().unwrap().to_string(),
            profile: *profile,
        };
        let order_file = format!("{}_{}.json", &order.id, &order.profile);
        let order_file = Path::new(DIRECTORY_ORDER)
            .join(Path::new(&order_file));
        debug!("Save order to {:?}, order: {:?}", &order_file, &order);
        std::fs::create_dir_all(DIRECTORY_ORDER)
            .map_err(|e| format!("Failed to create directory, \
                                  path: {:?}, e: {}", &DIRECTORY_ORDER, &e))?;
        std::fs::write(order_file,
                       serde_json::to_string(&order).unwrap().as_bytes())
            .unwrap_or_else(|e| {
                error!("Failed to save order: {:?}, e: {}", &order, &e);
            });
        orders.push(order);
    }

    let photos = orders.iter().map(|o| o.dryrun()).collect::<Vec<Photo>>();
    Ok(photos)
}

pub async fn delete<'a>(_: &Models<'a>, photos: &Vec<Photo>) -> Result<(), String>
{
    if photos.len() == 0 {
        debug!("No photo in the comment");
        Ok(())
    } else {
        let photo = &photos[0];
        let path = PhotoPath::try_from(photo.path.clone())
            .map_err(|e| format!("Cannot get PhotoPath from {:?}, ee: {}",
                                 &photo.path, &e))?;
        debug!("Delete path: {:?}, directory: {:?}",
               &path, &path.directory());
        let current = Path::new(DIRECTORY_UPLOAD)
            .join(path.directory().as_path());
        let dest = Path::new(DIRECTORY_DELETED)
            .join(path.directory().as_path());
        // TODO should be executed asynchronously?
        // Ensure destination comment directory to move images
        std::fs::create_dir_all(dest.parent().unwrap())
            .map_err(|e| format!("Failed to create directory, \
                                  path: {:?}, e: {}", &dest, &e))?;
        // Move image directory (comment directory will be left)
        std::fs::rename(&current, &dest)
            .map_err(|e| format!("Failed to rename {:?} -> {:?}, e: {}",
                                 &current, &dest, &e))
    }
}
