pub mod analyses;
pub mod templates;

use futures::{future::TryFutureExt};
use elasticsearch::{
    Elasticsearch,
    indices::IndicesCreateParts
};
use serde_json::json;

use crate::utils::elasticsearch::{Collection};

static INDEX_ANALYSES: &str = "analyses";
static INDEX_TEMPLATES: &str = "templates";

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
                "index": {"sort.field": "_lamo", "sort.order": "desc"},
            },
            "mappings": {
                "properties": {
                    "_lamo": {"type": "float"},
                    "no": {"type": "text", "analyzer": "kuromoji"},
                    "name": {"type": "text", "analyzer": "kuromoji"},
                    "location": {"type": "text", "analyzer": "kuromoji"},
                    "facilityName": {"type": "text", "analyzer": "kuromoji"},
                    "roomName": {"type": "text", "analyzer": "kuromoji"},
                    "applicantAddress": {"type": "text", "analyzer": "kuromoji"},
                    "applicantName": {"type": "text", "analyzer": "kuromoji"},
                    "quality": {"type": "text", "analyzer": "kuromoji"},
                    "investigator": {"type": "text", "analyzer": "kuromoji"},
                    "perception": {"type": "text", "analyzer": "kuromoji"},
                    "tester": {"type": "text", "analyzer": "kuromoji"},
                    "testedPerception": {"type": "text", "analyzer": "kuromoji"},
                    "heating": {"type": "text", "analyzer": "kuromoji"},
                    "water": {"type": "text", "analyzer": "kuromoji"},
                    "circulation": {"type": "text", "analyzer": "kuromoji"},
                    "chlorination": {"type": "text", "analyzer": "kuromoji"},
                    "additive": {"type": "text", "analyzer": "kuromoji"},
                    "header": {"type": "text", "analyzer": "kuromoji"},
                    "footer": {"type": "text", "analyzer": "kuromoji"},
                }
            }
        }))
        .send()
        .and_then(|r| async { r.text().await })
        .await; 
    debug!("Create indices, result: {:?}", &result);
    if let Err(e) = result {
        error!("Failed to setup index, error: {}", &e);
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

