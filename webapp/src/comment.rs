use serde::Serialize;

use crate::token::{Authentication, TokenData};
use crate::photo::Photo;

#[derive(Clone, Serialize, Debug)]
pub struct Comment {
    /// Comment ID. New comment doesn't have id.
    pub id: Option<String>,

    /// Parent analysis ID
    #[serde(rename = "parentId")]
    pub parent_id: String,

    /// Username who posted a comment
    pub username: String,

    /// E-mail address
    pub email: Option<String>,

    /// URL for websites
    pub web: Option<String>,

    /// Comment body
    pub body: String,

    /// Images
    pub images: Vec<Vec<Photo>>, // [photo][profile]

    /// Authentication data
    pub auth: Authentication,

    /// Last modified by epoch [ms]
    #[serde(rename = "lastModified")]
    pub last_modified: f64,

    /// Create at by epoch [ms]
    #[serde(rename = "createdAt")]
    pub created_at: f64,
}

impl Comment {
    pub fn is_editable(self: &Self, token: &TokenData) -> bool {
        match (&self.auth, token.is_guest()) {
            (Authentication::Guest { guestid }, true) =>
                guestid == token.get_id(),
            (Authentication::Signin { userid }, false) =>
                userid == token.get_id(),
            _ => false
        }
    }

    #[allow(dead_code)]
    pub fn has_image(self: &Self, profiles: &Vec<Photo>) -> bool {
        if profiles.len() == 0 {
            false
        } else {
            let photo = &profiles[0];
            self.images.iter().find(|v| {
                v.len() > 0 && v[0].id == photo.id
            }).is_some()
        }
    }

    pub fn add_image(self: &mut Self, profiles: Vec<Photo>) {
        if profiles.len() == 0 {
            warn!("No profiles unexpectedly?");
        } else {
            // NOTE Assume photos id will never conflict
            self.images.push(profiles);
        }
    }
}
