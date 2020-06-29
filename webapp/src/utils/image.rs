use mime::{self, Mime};

pub struct ImagePath {
    pub name: String,
    pub mimetype: Mime,
    pub dirname: Option<String>
}

impl ImagePath {
    // pub fn new(name: &str, mimetype: &Mime) -> Option<Self> {
    //     let a = ImagePath {
    //         name: name.to_string(),
    //         mimetype: mimetype.clone(),
    //         dirname: None
    //     };
    //     match a.extension_str() {
    //         Some(_) => Some(a),
    //         None => None
    //     }
    // }

    pub fn extension_str(self: &Self) -> Option<&'static str> {
        match self.mimetype.subtype() {
            mime::JPEG => Some("jpg"),
            mime::PNG => Some("png"),
            _ => None
        }
    }

    // pub fn is_image_of(self: &Self, mime: &mime::Name) -> bool {
    //     &self.mimetype.subtype() == mime
    // }

    pub fn filename(self: &Self) -> Option<String> {
        self.extension_str()
            .map(|ext| format!("{}.{}", self.name.as_str(), &ext).to_string())
    }

    pub fn filename_full(self: &Self) -> Option<String> {
        self.filename().map(|name| {
            let dir = self.dirname.as_ref().map(|s| s.as_str()).unwrap_or(".");
            format!("{}/{}", &dir, &name)
        })
    }
}

