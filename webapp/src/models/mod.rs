pub mod analyses;
pub mod templates;

use futures::{future::TryFutureExt};
use elasticsearch::{
    Elasticsearch,
    indices::IndicesCreateParts
};
use serde_json::json;

use crate::utils::elasticsearch::{Collection};

static INDEX_ANALYSES: &str = "onsen.analyses";
static INDEX_TEMPLATES: &str = "onsen.templates";

type Database = Elasticsearch;

pub struct Models<'a> {
    pub analyses: Collection<'a>,
    pub templates: Collection<'a>
}

pub async fn models<'a>(db: &'a Database) -> Models<'a> {
    let result = db.indices()
        .create(IndicesCreateParts::Index(INDEX_ANALYSES))
        .body(json!({
            "settings": {
                "index": {"sort.field": "_lamo", "sort.order": "desc"}
            },
            "mappings": {
                "properties": {
                    "name": {"type": "text", "analyzer": "kuromoji"},
                    "address": {"type": "text", "analyzer": "kuromoji"}
                }
            }
        }))
        .send()
        .and_then(|r| async { r.text().await })
        .await; 
    if let Err(e) = result {
        println!("Failed to setup index, error: {}", &e);
    }
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

