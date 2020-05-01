use r2d2_mongodb::mongodb::{bson, Bson, doc, Document};
use r2d2_mongodb::mongodb::coll::Collection;
use r2d2_mongodb::mongodb::coll::options::ReplaceOptions;

use crate::models::{Models, collection_templates};
use crate::template;
use crate::template::{Template};
use crate::utils::mongodb::{object_id, document_str};

impl From<&Document> for Template {
    fn from(item: &Document) -> Self {
        let id = item.get_object_id(template::KEY_ID)
            .map(|v| v.to_hex())
            .map_err(|e| error!("BUG: Couldn't get ObjectId, error:{}", e))
            .ok();
        let name = document_str(item, template::KEY_NAME).unwrap();
        let body = document_str(item, template::KEY_BODY).unwrap();
        Template {
            id: id,
            name: name,
            body: body
        }
    }
}
impl From<&Template> for Document {
    fn from(t: &Template) -> Self {
        let mut d = doc!{
            template::KEY_NAME: Bson::String(t.name.to_string()),
            template::KEY_BODY: Bson::String(t.body.to_string())
        };
        if let Some(Bson::ObjectId(id)) = object_id(&t.id) {
            d.insert_bson(template::KEY_ID.to_string(), Bson::ObjectId(id));
        };
        d
    }
}

/**
 * Operations for MongoDB.
 */
pub fn select(models: &Models) -> Result<impl Iterator<Item=Template>, String> {
    let coll: &Collection = collection_templates(models);
    let result = coll.find(None, None);
    match result {
        Ok(cur) => Ok(cur.filter_map(|row| {
            let item: Document = row.ok()?;
            let t: Template = Template::from(&item);
            Some(t)
        }).into_iter()),
        Err(e) => Err(String::from(format!("{}", e)))
    }
}

pub fn by_id(models: &Models, id: &String) -> Result<Option<Template>, String> {
    let coll = collection_templates(models);
    let oid = object_id(&Some(id.to_string()))
        .ok_or(format!("Invalie template id: {}", id))?;
    let filter = Some(doc! { template::KEY_ID: oid });
    match coll.find_one(filter, None) {
        Ok(Some(d)) => Ok(Some(Template::from(&d))),
        Ok(None) => Ok(None),
        Err(e) => Err(format!("Failed to find template, e: {}", &e))
    }
}

pub fn save(models: &Models, t: &Template) -> Result<Template, String> {
    // TODO Check template is valid in handlebars syntax.
    debug!("templates::save, template: {:?} name: {}", &t.id, &t.name);
    let coll = collection_templates(models);
    // Clone object
    let id = object_id(&t.id).ok_or("Invalid id".to_string())?;
    let d = Document::from(t);
    let result = match id {
        Bson::ObjectId(_) => { // Update exists object
            let options = ReplaceOptions {
                upsert: Some(true),
                ..Default::default()
            };
            let filter = doc! { template::KEY_ID: id };
            coll.replace_one(filter, d, Some(options))
                .map(|r| {
                    debug!("templates::save to update, result: {:?}", &r);
                    if r.modified_count >= 0 {
                        object_id(&t.id)
                    } else {
                        r.upserted_id
                    }
                })
        },
        _ => // Insert new object
            coll.insert_one(d, None).map(|r| {
                debug!("templates::save to create, result: {:?}", &r);
                r.inserted_id
            })
    };
    match result {
        Ok(Some(Bson::ObjectId(id))) => {
            let mut t: Template = t.clone();
            t.id = Some(id.to_hex());
            Ok(t)
        },
        Ok(e) =>
            Err(String::from(format!("unexpected result in template::save,
                                     e: {:?}", &e))),
        Err(e) => Err(String::from(format!("{}", e)))
    }
}
