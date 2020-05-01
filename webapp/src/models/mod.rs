pub mod analyses;
pub mod templates;

use r2d2_mongodb::mongodb::{bson, doc};
use r2d2_mongodb::mongodb::db::{Database, ThreadedDatabase};
use r2d2_mongodb::mongodb::coll::Collection;
use r2d2_mongodb::mongodb::coll::options::IndexOptions;

static COLLECTION_ANALYSES: &str = "analyses";
static COLLECTION_TEMPLATES: &str = "templates";

pub struct Models {
    pub analyses: Collection,
    pub templates: Collection
}

pub fn models(db: &Database) -> Models {
    let analyses = db.collection(COLLECTION_ANALYSES);
    let templates = db.collection(COLLECTION_TEMPLATES);
    // Ensure index for analyses
    let keys = doc!{ "id": -1 };
    if let Err(e) = &analyses.create_index(keys, Some(IndexOptions {
        unique: Some(true),
        ..Default::default()
    })) {
        println!("Failed to create index: {}", e) 
    }
    // Ensure index for templates
    let keys = doc!{ "id": -1 };
    if let Err(e) = &templates.create_index(keys, Some(IndexOptions {
        unique: Some(true),
        ..Default::default()
    })) {
        println!("Failed to create index: {}", e) 
    }
    Models {
        analyses : analyses,
        templates: templates
    }
}

pub fn collection_analyses(models: &Models) -> &Collection {
    &models.analyses
}

pub fn collection_templates(models: &Models) -> &Collection {
    &models.templates
}
