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
    pub images: Vec<Vec<Photo>>,

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
}
