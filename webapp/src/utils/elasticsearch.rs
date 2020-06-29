use async_trait::async_trait;
use futures::prelude::*;
use elasticsearch::{
    self,
    Elasticsearch,
    CountParts, GetParts, SearchParts, ScrollParts,
    CreateParts, DeleteParts, IndexParts,
    indices::IndicesCreateParts,
    http::{
        headers::{CONTENT_TYPE, HeaderValue},
        transport::Transport
    }
};
use serde::{self, Deserialize, Serialize};
use serde_json::{json, Value};

pub type DBConnectionPool = Elasticsearch;
pub type DBConnection = Elasticsearch;

pub struct Collection<'a> {
    pub name: &'static str,
    pub client: &'a Elasticsearch
}

// Constants
lazy_static! {
    static ref APPLICATION_JSON: HeaderValue =
        HeaderValue::from_static("application/json");
}

/**
 * Connection
 */
pub fn create_pool() -> Result<DBConnectionPool, elasticsearch::Error> {
    let address = "http://elasticsearch:9200";
    let transport = Transport::single_node(&address)?;
    let client = Elasticsearch::new(transport);
    Ok(client)
}

pub fn get_unpooled_connection() -> Result<DBConnection, elasticsearch::Error>  {
    create_pool()
}

/**
 * Operations
 */
#[async_trait]
pub trait Operations {
    type Error;
    type Client;

    type GetOptions;
    type GetResult;

    type SelectOptions;
    type SelectResult;

    type InsertOptions;
    type InsertResult;

    type UpdateOptions;
    type UpdateResult;

    type DeleteOptions;
    type DeleteResult;

    async fn get(&self, o: GetOptions) -> Result<Self::GetResult, Self::Error>;
    async fn count(&self) -> Result<u64, Self::Error>;
    async fn select(&self, o: Self::SelectOptions)
                    -> Result<Self::SelectResult, Self::Error>;
    async fn insert(&self, v: &Value, o: Self::InsertOptions)
                    -> Result<Self::InsertResult, Self::Error>;
    async fn update(&self, v: &Value, o: Self::UpdateOptions)
                    -> Result<Self::UpdateResult, Self::Error>;
    async fn delete(&self, o: Self::DeleteOptions)
                    -> Result<Self::DeleteResult, Self::Error>;
}

pub struct GetOptions {
    id: String
}

impl GetOptions {
    pub fn new(id: &str) -> Self {
        GetOptions {
            id: id.to_string()
        }
    }
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct GetResult {
    pub _id: String,
    pub _index: String,
    pub _primary_term: u64,
    pub _seq_no: u64,
    pub _source: Value,
}

#[derive(Serialize, Clone, Debug)]
pub struct SearchOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>
}

impl Default for SearchOptions {
    fn default() -> Self {
        SearchOptions {
            query: None,
            sort: None,
            from: None,
            size: None
        }
    }
}

#[derive(Serialize)]
pub struct InsertOptions {
    #[serde(skip)]
    id: Option<String>
}

impl InsertOptions {
    pub fn new(id: Option<&str>) -> Self {
        InsertOptions {
            id: id.map(|s| s.to_string())
        }
    }
}

/// {
///     "_id": "hinodeonsenkinokonosato",
///     "_index": "analyses",
///     "_primary_term": 1,
///     "_seq_no": 3,
///     "_shards": {
///         "failed": 0,
///         "successful": 1,
///         "total": 2
///     },
///     "_type": "_doc",
///     "_version": 1,
///     "result": "created"
/// }
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct OperationResult {
    pub _id: String,
    pub _index: String,
    pub _primary_term: u64,
    pub _seq_no: u64,
    pub _shards: ResultShards,
    pub _type: String,
    pub _version: u64,
    pub result: OperationResultType,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct ResultShards {
    pub failed: u64,
    pub successful: u64,
    pub total: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum OperationResultType {
    Created,
    Updated,
    Deleted,
    Noop,
}


#[derive(Serialize)]
pub struct UpdateOptions {
    #[serde(skip)]
    id: String
}

impl UpdateOptions {
    pub fn new(id: &str) -> Self {
        UpdateOptions {
            id: id.to_string()
        }
    }
}

#[derive(Serialize)]
pub struct DeleteOptions {
    #[serde(skip)]
    id: String
}

impl DeleteOptions {
    pub fn new(id: &str) -> Self {
        DeleteOptions {
            id: id.to_string()
        }
    }
}

#[derive(Deserialize)]
struct CountResult {
    count: u64
}

// Search API

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct SearchResult {
    pub took: u64,
    pub timed_out: bool,
    pub hits: SearchResultHits,
    pub _scroll_id: Option<String>
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct SearchResultHits {
    pub total: SearchResultHitsTotal,
    pub hits: Vec<SearchResultItem>
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct SearchResultHitsTotal {
    pub value: u64,
    pub relation: String
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct SearchResultItem {
    pub _index: String,
    pub _id: String,
    pub _score: Option<f64>,
    pub _source: Value
}

#[async_trait]
pub trait Setup {
    type Error;
    type Client;
    type SetupOptions;
    type SetupResult;
    
    async fn setup(&self, options: Self::SetupOptions)
                   -> Result<Self::SetupResult, Self::Error>;
}

pub struct SetupOptions {
    value: Value
}

impl SetupOptions {
    pub fn new(value: Value) -> Self {
        Self {
            value: value
        }
    }
}

#[async_trait]
impl<'a> Setup for Collection<'a> {
    type Error = elasticsearch::Error;
    type Client = Elasticsearch;
    type SetupOptions = SetupOptions;
    type SetupResult = String;
    
    async fn setup(&self, options: Self::SetupOptions)
                   -> Result<Self::SetupResult, Self::Error>
    {
        self.client.indices()
            .create(IndicesCreateParts::Index(self.name))
            .body(options.value)
            .send()
            .and_then(|r| async {
                r.text().await
            })
            .await
    }
}


// Operations
#[async_trait]
impl<'a> Operations for Collection<'a> {
    type Error = elasticsearch::Error;
    type Client = Elasticsearch;

    type GetOptions = GetOptions;
    type GetResult = GetResult;

    type SelectOptions = SearchOptions;
    type SelectResult = SearchResult;

    type InsertOptions = InsertOptions;
    type InsertResult = OperationResult;

    type UpdateOptions = UpdateOptions;
    type UpdateResult = OperationResult;

    type DeleteOptions = DeleteOptions;
    type DeleteResult = OperationResult;

    async fn count(&self) -> Result<u64, Self::Error> {
        self.client
            .count(CountParts::Index(&[self.name]))
            .send()
            .and_then(|r| async {
                r.error_for_status_code_ref()?;
                r.json::<CountResult>().await.map(|v| v.count)
            })
            .await
    }

    async fn get(&self, options: Self::GetOptions)
                 -> Result<Self::GetResult, Self::Error> {
        self.client
            .get(GetParts::IndexId(self.name, &options.id))
            .send()
            .and_then(|r| async {
                r.error_for_status_code_ref()?;
                r.json::<Self::GetResult>().await
            })
            .await
    }

    async fn select(&self, options: Self::SelectOptions)
        -> Result<Self::SelectResult, Self::Error>
    {
        debug!("Search elasticsearch, body: {}, headers: {}: {:?}",
               serde_json::to_string(&options).unwrap(),
               CONTENT_TYPE, APPLICATION_JSON.clone());
        self.client
            .search(SearchParts::Index(&[self.name]))
            .header(CONTENT_TYPE, APPLICATION_JSON.clone())
            .body(options)
            .send()
            .and_then(|r| async {
                r.error_for_status_code_ref()?;
                r.json::<Self::SelectResult>().await
            })
            .await
    }

    async fn insert(&self, value: &Value, options: Self::InsertOptions)
        -> Result<Self::InsertResult, Self::Error>
    {
        match options.id {
            None => {
                self.client
                    .index(IndexParts::Index(self.name))
                    .body(value)
                    .send()
                    .and_then(|r| async {
                        r.json::<Self::InsertResult>().await
                    })
                    .await
            },
            Some(id) => {
                self.client
                    .create(CreateParts::IndexId(self.name, &id))
                    .body(value)
                    .send()
                    .and_then(|r| async {
                        r.json::<Self::InsertResult>().await
                    })
                    .await
            }
        }
    }

    async fn update(&self, value: &Value, options: Self::UpdateOptions)
        -> Result<Self::UpdateResult, Self::Error>
    {
        self.client
            .index(IndexParts::IndexId(self.name, &options.id))
            .body(value)
            .send()
            .and_then(|r| async {
                r.error_for_status_code_ref()?;
                r.json::<Self::UpdateResult>().await
            })
            .await
    }

    async fn delete(&self, options: Self::DeleteOptions)
        -> Result<Self::DeleteResult, Self::Error>
    {
        self.client
            .delete(DeleteParts::IndexId(self.name, &options.id))
            .send()
            .and_then(|r| async {
                r.json::<Self::DeleteResult>().await

            })
            .await
    }
}

#[async_trait]
pub trait Scroll {
    type Error;
    type Client;
    type Item;

    // TODO want to return Result<impl Stream<Item = Self::Item>, Self::Error>>
    async fn scroll(&self, scroll_id: Option<&str>)
                    -> Result<SearchResult, Self::Error>;
}

#[async_trait]
impl<'a> Scroll for Collection<'a> {
    type Error = elasticsearch::Error;
    type Client = Elasticsearch;
    type Item = SearchResultItem;

    async fn scroll(&self, scroll_id: Option<&str>)
                    -> Result<SearchResult, Self::Error>
    {
        let index_name = &[self.name];
        match scroll_id {
            None => {
                self.client
                    .search(SearchParts::Index(index_name))
                    .scroll("1m")
                    .send()
                    .and_then(|r| async {
                        r.error_for_status_code_ref()?;
                        r.json::<SearchResult>().await
                    }).await
            },
            Some(scroll_id) => {
                self.client
                    .scroll(ScrollParts::None)
                    .body(json!({
                        "scroll": "1m",
                        "scroll_id": scroll_id
                    }))
                    .send()
                    .and_then(|r| async {
                        r.error_for_status_code_ref()?;
                        r.json::<SearchResult>().await
                    }).await
            }
        }
    }
}
