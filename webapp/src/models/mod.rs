pub mod analyses;
pub mod templates;
pub mod comments;
pub mod comment_photos;

use elasticsearch::Elasticsearch;

use crate::utils::elasticsearch::{Collection};

static INDEX_ANALYSES: &str = "analyses";
static INDEX_TEMPLATES: &str = "templates";
static INDEX_COMMENTS: &str = "comments";

type Database = Elasticsearch;

pub struct Models<'a> {
    pub analyses: Collection<'a>,
    pub templates: Collection<'a>,
    pub comments: Collection<'a>
}

impl<'a> Models<'a> {
    pub fn new(db: &'a Database) -> Self {
        Models {
            analyses : Collection {
                client: db,
                name: INDEX_ANALYSES
            },
            templates: Collection {
                client: db,
                name: INDEX_TEMPLATES
            },
            comments: Collection {
                client: db,
                name: INDEX_COMMENTS
            }
        }
    }

    pub async fn setup(self: &Self) {
        let result = analyses::setup(self).await;
        println!("Models::setup, result: {:?}", &result);
        let result = comments::setup(self).await;
        println!("Models::setup, result: {:?}", &result);
        let result = comment_photos::setup(self).await;
        println!("Models::setup, result: {:?}", &result);
    }
}
