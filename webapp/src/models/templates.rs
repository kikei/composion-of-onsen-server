use std::convert::TryFrom;
use serde_json::{json, Value};

use crate::models::{Models};
use crate::template::{self, Template};
use crate::utils::elasticsearch::{
    GetResult, SearchResultItem,
    Operations, GetOptions, SearchOptions, InsertOptions, UpdateOptions
};

impl TryFrom<&GetResult> for Template {
    type Error = String;
    fn try_from(value: &GetResult) -> Result<Self, Self::Error> {
        let mut t = Template::try_from(&value._source)?;
        t.id = Some(value._id.to_string());
        Ok(t)
    }
}

impl TryFrom<&SearchResultItem> for Template {
    type Error = String;
    fn try_from(value: &SearchResultItem) -> Result<Self, Self::Error> {
        let mut t = Template::try_from(&value._source)?;
        t.id = Some(value._id.to_string());
        Ok(t)
    }
}

impl TryFrom<&Value> for Template {
    type Error = String;
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let obj = value.as_object()
            .ok_or(format!("Failed to get Template from Value: {}", &value))?;
        let id =
            obj.get(template::KEY_ID)
            .and_then(|v| v.as_str()).map(|s| s.to_string());
        let name =
            obj.get(template::KEY_NAME).and_then(|v| v.as_str()).unwrap();
        let body =
            obj.get(template::KEY_BODY).and_then(|v| v.as_str()).unwrap();
        Ok(Template {
            id: id,
            name: name.to_string(),
            body: body.to_string()
        })
    }
}

impl From<&Template> for Value {
    fn from(t: &Template) -> Self {
        let mut v = json!({
            template::KEY_NAME: Value::from(t.name.as_str()),
            template::KEY_BODY: Value::from(t.body.as_str())
        });
        if let Some(id) = &t.id {
            v.as_object_mut().unwrap().insert(template::KEY_ID.to_string(),
                                              Value::from(id.as_str()));
        }
        v
    }
}

/**
 * Operations for MongoDB.
 */
pub async fn select<'a>(models: &Models<'a>)
    -> Result<impl Iterator<Item=Template>, String>
{
    let result = models.templates.select(SearchOptions {
        ..Default::default()
    }).await;
    match result {
        Ok(result) => Ok(result.hits.hits.into_iter().filter_map(|row| {
            Template::try_from(&row).ok()
        }).into_iter()),
        Err(e) => Err(String::from(format!("{}", e)))
    }
}

pub async fn by_id<'a>(models: &Models<'a>, id: &String)
    -> Result<Option<Template>, String>
{
    let result = models.templates
        .get(GetOptions::new(id))
        .await;
    match result {
        Ok(row) => Template::try_from(&row).map(Some),
        Err(e) => Err(format!("Failed to find template, e: {}", &e))
    }
}

pub async fn save<'a>(models: &Models<'a>, t: &Template) -> Result<Template, String> {
    // TODO Check template is valid in handlebars syntax.
    debug!("templates::save, template: {:?} name: {}", &t.id, &t.name);
    // Clone object
    let mut v = Value::from(t);
    v.as_object_mut().unwrap().remove(template::KEY_ID);
    let result = match &t.id {
        Some(id) => { // Update exists object
            models.templates
                .update(&v, UpdateOptions::new(id))
                .await
                .map(|r| {
                    debug!("templates::save to update, result: {:?}", &r);
                    id.clone()
                    /*
                    if r.modified_count >= 0 {
                        object_id(&t.id)
                    } else {
                        r.upserted_id
                    }
                    */
                })
        },
        _ => { // Insert new object
            models.templates
                .insert(&v, InsertOptions::new(None))
                .await
                .map(|r| {
                    debug!("templates::save to create, result: {:?}", &r);
                    "TODO".to_string()
                })
        }
    };
    match result {
        Ok(id) => {
            let mut t: Template = t.clone();
            t.id = Some(id);
            Ok(t)
        },
        // Ok(e) =>
        //     Err(String::from(format!("unexpected result in template::save,
        //                              e: {:?}", &e))),
        Err(e) => Err(String::from(format!("{}", e)))
    }
}
