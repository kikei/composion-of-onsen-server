pub mod analyses;
pub mod templates;

use elasticsearch::Elasticsearch;

use crate::utils::elasticsearch::{Collection};

static INDEX_ANALYSES: &str = "analyses";
static INDEX_TEMPLATES: &str = "templates";

type Database = Elasticsearch;

pub struct Models<'a> {
    pub analyses: Collection<'a>,
    pub templates: Collection<'a>
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
            }
        }
    }

    pub async fn setup(self: &Self) {
        let result = analyses::setup(self).await;
        println!("Models::setup, result: {:?}", &result)
    }
}
