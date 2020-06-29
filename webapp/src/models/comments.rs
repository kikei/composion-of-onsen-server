use std::convert::TryFrom;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::token::{Authentication};
use crate::comment::{Comment};
use crate::photo::{Photo};
use crate::models::Models;
use crate::utils::{
    identifier::{IdGenerator, Generate},
    elasticsearch::{
        GetResult, SearchResultItem, OperationResultType,
        Setup, SetupOptions,
        Operations, GetOptions, SearchOptions, InsertOptions, UpdateOptions,
        DeleteOptions
    }
};

const KEY_ID: &str = "id";
const KEY_PARENT_ID: &str = "pid";
const KEY_USERNAME: &str = "name";
const KEY_EMAIL: &str = "mail";
const KEY_WEB: &str = "web";
const KEY_BODY: &str = "body";
const KEY_IMAGES: &str = "imag";
const KEY_LAST_MODIFIED: &str = "_lamo";
const KEY_CREATED_AT: &str = "_crat";
const KEY_AUTH: &str = "auth";

const KEY_AUTH_GUESTID: &str = "user";
const KEY_AUTH_USERID: &str = "user";
const KEY_AUTH_TYPE: &str = "t";
const VAL_AUTH_GUEST: &str = "gust";
const VAL_AUTH_SIGNIN: &str = "sign";

const KEY_FIELDS_SEARCHABLE: &'static [&'static str] = &["name", "comm"];

/// Conversion from Comment to Database object
impl From<&Comment> for Value {
    fn from(item: &Comment) -> Self {
        json!({
            KEY_ID: item.id.as_ref()
                .map_or(Value::Null, |s| Value::from(s.as_str())),
            KEY_PARENT_ID: Value::from(item.parent_id.as_str()),
            KEY_USERNAME: Value::from(item.username.as_str()),
            KEY_EMAIL: item.email.as_ref()
                .map_or_else(|| Value::Null, |s| Value::from(s.as_str())),
            KEY_WEB: item.web.as_ref()
                .map_or_else(|| Value::Null, |s| Value::from(s.as_str())),
            KEY_BODY: Value::from(item.body.as_str()),
            KEY_IMAGES: Value::from(item.images.as_slice()),
            // KEY_IMAGES: Value::from(&item.images.map(|s| s.as_str().unwrap().to_string())),
            KEY_AUTH: Value::from(&item.auth),
            KEY_CREATED_AT: Value::from(item.created_at),
            KEY_LAST_MODIFIED: Value::from(item.last_modified)
        })
    }
}

/// Conversion from Comment to Database object
impl From<&Authentication> for Value {
    fn from(item: &Authentication) -> Self {
        match item {
            Authentication::Guest { guestid } => json!({
                KEY_AUTH_TYPE: Value::from(VAL_AUTH_GUEST),
                KEY_AUTH_GUESTID: Value::from(guestid.as_str())
            }),
            Authentication::Signin { userid } => json!({
                KEY_AUTH_TYPE: Value::from(VAL_AUTH_SIGNIN),
                KEY_AUTH_USERID: Value::from(userid.as_str())
            })
        }
    }
}

impl From<VecPhoto> for Value {
    fn from(item: VecPhoto) -> Self {
        let item = item.0;
        json!(item)
    }
}

impl TryFrom<GetResult> for Comment {
    type Error = String;
    fn try_from(value: GetResult) -> Result<Self, Self::Error> {
        let mut a = Comment::try_from(value._source)?;
        a.id = Some(value._id.to_string());
        Ok(a)
    }
}

impl TryFrom<SearchResultItem> for Comment {
    type Error = String;
    fn try_from(value: SearchResultItem) -> Result<Self, Self::Error> {
        let mut a = Comment::try_from(value._source)?;
        a.id = Some(value._id.to_string());
        Ok(a)
    }
}

/// Conversion from Database object.
impl TryFrom<Value> for Comment {
    type Error = String;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        // Value must be an object
        let obj = value.as_object()
            .ok_or(format!("Failed to get Comment from Value: {}", &value))?;

        // Extract each fields from the object.
        let id = obj.get(KEY_ID).and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let parent_id = obj.get(KEY_PARENT_ID).and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or("Maybe a bug: missing parent id")?;
        let name = obj.get(KEY_USERNAME).and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or("Maybe a bug: missing username")?;
        let email = obj.get(KEY_EMAIL).and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let web = obj.get(KEY_WEB).and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let body = obj.get(KEY_BODY).and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or("Maybe a bug: missing comment body")?;
        let images = obj.get(KEY_IMAGES).and_then(|v| v.as_array())
            .map(|a| a.into_iter()
                 .filter_map(|v| VecPhoto::try_from(v.to_owned())
                             .ok().map(|o| o.0))
                 .collect())
            .ok_or("Maybe a bug: missing images")?;
        let auth = obj.get(KEY_AUTH)
            .and_then(|v| Authentication::try_from(v).ok())
            .ok_or("Maybe a bug: missing auth")?;
        let created_at = obj.get(KEY_CREATED_AT).and_then(|v| v.as_f64())
            .ok_or("Maybe a bug: missing created at")?;
        let last_modified = obj.get(KEY_LAST_MODIFIED).and_then(|v| v.as_f64())
            .ok_or("Maybe a bug: missing last modified")?;
        Ok(Comment {
            id: id,
            parent_id: parent_id,
            username: name,
            email: email,
            web: web,
            body: body,
            images: images,
            auth: auth,
            created_at: created_at,
            last_modified: last_modified
        })
    }
}

struct VecPhoto(Vec<Photo>);

impl TryFrom<Value> for VecPhoto {
    type Error = String;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let a = value.as_array()
            .ok_or(format!("Failed to get Vec<Photo> from value: {}", &value))?;
        let v = a.into_iter()
            .filter_map(|v| Photo::try_from(v.to_owned()).ok())
            .collect();
        Ok(VecPhoto(v))
    }
}

/// Conversion from Database object.
impl TryFrom<&Value> for Authentication {
    type Error = String;
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        // Value must be an object
        let obj = value.as_object()
            .ok_or(format!("Failed to get Authentication from Value: {}",
                           &value))?;
        
        // Extract each fields from the object.
        let ty = obj.get(KEY_AUTH_TYPE).and_then(|v| v.as_str())
            .ok_or(format!("Maybe a bug: missing auth type"))?;
        match ty {
            VAL_AUTH_GUEST => {
                let guestid = obj.get(KEY_AUTH_GUESTID)
                    .and_then(|v| v.as_str())
                    .ok_or(format!("Maybe a bug: missing username"))?;
                Ok(Authentication::Guest { guestid: guestid.to_string() })
            },
            VAL_AUTH_SIGNIN => {
                let userid = obj.get(KEY_AUTH_USERID)
                    .and_then(|v| v.as_str())
                    .ok_or(format!("Maybe a bug: missing userid"))?;
                Ok(Authentication::Signin { userid: userid.to_string() })
            },
            _ => Err(format!("Unknown authentication type: {}", &ty))
        }
    }
}

pub enum SelectQuery {
    Text(String),
    Parent(String)
}

pub struct SelectOptions {
    pub query: Option<SelectQuery>,
    pub limit: u32
}

pub struct SelectResult {
    /// Total found
    pub total: u32,

    /// Comments hit
    pub items: Box<dyn Iterator<Item = Comment>>
}

pub struct DeleteCommentOptions {
    /// Comment id
    pub id: String
}

pub async fn setup<'a>(models: &Models<'a>) -> Result<String, String> {
    models.comments
        .setup(SetupOptions::new(json!({
            "settings": {
                "index": {
                    "sort.field": [KEY_PARENT_ID, KEY_CREATED_AT],
                    "sort.order": ["desc", "desc"]
                }
            },
            "mappings": {
                "properties": {
                    KEY_PARENT_ID: {"type": "text"},
                    KEY_CREATED_AT: {"type": "text"},
                    KEY_USERNAME: {"type": "text", "analyzer": "kuromoji"},
                    KEY_BODY: {"type": "text", "analyzer": "kuromoji"}
                }
            }
        })))
        .await
        .map_err(|e| String::from(format!("{}", &e)))
}

pub async fn select<'a>(models: &Models<'a>, options: &SelectOptions) ->
    Result<SelectResult, String>
{
    let query = match &options.query {
        None => None,
        Some(SelectQuery::Text(text)) => Some(json!({
            "multi_match": {
                "query": text.as_str(),
                "fields": KEY_FIELDS_SEARCHABLE
            }
        })),
        Some(SelectQuery::Parent(parent_id)) => Some(json!({
            "term": {
                KEY_PARENT_ID: parent_id.as_str()
            }
        }))
    };
    let result = models.comments.select(SearchOptions {
        query: query,
        sort: Some(json!([{
            KEY_CREATED_AT: "desc"
        }])),
        ..Default::default()
    }).await;
    debug!("comments::select, result: {:?}", &result);
    match result {
        Ok(result) => Ok(SelectResult {
            total: result.hits.total.value as u32,
            items: Box::new(result.hits.hits
                            .into_iter()
                            .filter_map(|row| {
                                match Comment::try_from(row) {
                                    Ok(c) => Some(c),
                                    Err(e) => {
                                        debug!("{}", &e);
                                        None
                                    }
                                }
                            }).into_iter())
        }),
        Err(e) => Err(String::from(format!("{}", e)))
    }
}

pub async fn by_id<'a>(models: &Models<'a>, id: &str)
    -> Result<Option<Comment>, String>
{
    debug!("comments:by_id, id:{}", id);
    let result = models.comments.get(GetOptions::new(id)).await;
    debug!("comments::by_id, result: {:?}", &result);
    match result {
        Ok(row) => Comment::try_from(row).map(|a| Some(a)),
        Err(e) => Err(String::from(format!("{}", e)))
    }
}

pub struct CommentIdGenerator<'a>(IdGenerator<(&'a str, &'a str)>);

impl<'a> CommentIdGenerator<'a> {
    pub fn new(analysis: &'a str, content: &'a str) -> Self {
        CommentIdGenerator(IdGenerator::new((analysis, content)))
    }
}

impl<'a> Generate for CommentIdGenerator<'a> {
    fn generate(self: &Self) -> String {
        self.0.generate()
    }
}

pub async fn delete<'a>(models: &Models<'a>, options: DeleteCommentOptions)
    -> Result<String, String>
{
    let result = models.comments
        .delete(DeleteOptions::new(options.id.as_str()))
        .await
        .map(|r| {
            debug!("comments::delete, result: {:?}", &r);
            match r.result {
                OperationResultType::Deleted => Some(r._id),
                _ => None
            }
        });
    match result {
        Ok(Some(id)) => Ok(id),
        Ok(None) => Err(String::from("unexpected result in comments::delete")),
        Err(e) => Err(String::from(format!("{}", &e)))
    }
}

pub async fn save<'a>(models: &Models<'a>, a: &Comment)
                      -> Result<Comment, String>
{
    // Clone object
    let mut a: Comment = a.clone();
    // Update created_at
    let epoch = SystemTime::now().duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or_else(|e| {
            warn!("Failed to calulate epoch!? {}", &e);
            0.0
        });
    // Update last_modified
    a.last_modified = epoch;
    let value = Value::from(&a);
    debug!("comments::save, value: {}", &value);
    let result = match &a.id {
        // Create document
        None => {
            models.comments
                .insert(&value, InsertOptions::new(None))
                .await
                .map(|r| {
                    debug!("comments::save, created, result: {:?}", &r);
                    match r.result {
                        OperationResultType::Created => Some(a),
                        _ => None
                    }
                })
        },
        Some(id) => {
            models.comments
                .update(&value, UpdateOptions::new(id.as_str()))
                .await
                .map(|r| {
                    debug!("comments::save, updated, result: {:?}", &r);
                    match r.result {
                        OperationResultType::Updated => Some(a),
                        OperationResultType::Created => Some(a),
                        _ => None
                    }
                })
        }
    };
    match result {
        Ok(Some(a)) => Ok(a),
        Ok(None) => Err(String::from(format!("unexpected result in \
                                              comments::save"))),
        Err(e) => Err(String::from(format!("{}", e)))
    }
}

