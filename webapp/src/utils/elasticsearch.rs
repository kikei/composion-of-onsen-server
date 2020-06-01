use async_trait::async_trait;
use futures::{future::TryFutureExt};
use elasticsearch::{
    self,
    Elasticsearch,
    CountParts, GetParts, SearchParts,
    CreateParts, UpdateParts, DeleteParts, IndexParts,
    http::{
        headers::{CONTENT_TYPE, HeaderValue},
        transport::Transport
    }
};
use serde::{self, Deserialize, Serialize};
use serde_json::value::{Value};

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

    type SelectOptions;
    type SelectResult;

    type InsertOptions;
    type UpdateOptions;
    type DeleteOptions;
    async fn get(&self, o: GetOptions) -> Result<Value, Self::Error>;
    async fn count(&self) -> Result<u64, Self::Error>;
    async fn select(&self, o: Self::SelectOptions)
                    -> Result<Self::SelectResult, Self::Error>;
    async fn insert(&self, v: &Value, o: Self::InsertOptions)
                    -> Result<Value, Self::Error>;
    async fn update(&self, v: &Value, o: Self::UpdateOptions)
                    -> Result<Value, Self::Error>;
    async fn delete(&self, o: Self::DeleteOptions)
                    -> Result<Value, Self::Error>;
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

#[derive(Serialize)]
pub struct SearchOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>
}

impl Default for SearchOptions {
    fn default() -> Self {
        SearchOptions {
            q: None,
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

#[derive(Deserialize)]
struct CountResult {
    count: u64
}

// Search API

#[derive(Deserialize)]
pub struct SearchResult {
    #[allow(dead_code)]
    pub took: u64,
    #[allow(dead_code)]
    pub timed_out: bool,
    pub hits: SearchResultHits
}

#[derive(Deserialize)]
pub struct SearchResultHits {
    #[allow(dead_code)]
    pub total: SearchResultHitsTotal,
    pub hits: Vec<SearchResultItem>
}

#[derive(Deserialize)]
pub struct SearchResultHitsTotal {
    #[allow(dead_code)]
    pub value: u64,
    #[allow(dead_code)]
    pub relation: String
}

#[derive(Deserialize)]
pub struct SearchResultItem {
    #[allow(dead_code)]
    pub _index: String,
    #[allow(dead_code)]
    pub _id: String,
    #[allow(dead_code)]
    pub _score: f64,
    pub _source: Value
}

// Operations
#[async_trait]
impl<'a> Operations for Collection<'a> {
    type Error = elasticsearch::Error;
    type Client = Elasticsearch;

    type SelectOptions = SearchOptions;
    type SelectResult = SearchResult;

    type GetOptions = GetOptions;
    type InsertOptions = InsertOptions;
    type UpdateOptions = UpdateOptions;
    type DeleteOptions = DeleteOptions;


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
                 -> Result<Value, Self::Error> {
        self.client
            .get(GetParts::IndexId(self.name, &options.id))
            .send()
            .and_then(|r| async {
                r.json::<Value>().await
            })
            .await
    }

    async fn select(&self, options: Self::SelectOptions)
        -> Result<Self::SelectResult, Self::Error>
    {
        let body = serde_json::to_string(&options).unwrap();
        debug!("Search elasticsearch, body: {}, headers: {}: {:?}",
               &body, CONTENT_TYPE, APPLICATION_JSON.clone());
        self.client
            .search(SearchParts::Index(&[self.name]))
            .header(CONTENT_TYPE, APPLICATION_JSON.clone())
            // .body(body)
            .send()
            .and_then(|r| async {
                r.error_for_status_code_ref()?;
                r.json::<Self::SelectResult>().await
            })
            .await
    }

    async fn insert(&self, value: &Value, options: Self::InsertOptions)
        -> Result<Value, Self::Error>
    {
        match options.id {
            None => {
                self.client
                    .index(IndexParts::Index(self.name))
                    .body(value)
                    .send()
                    .and_then(|r| async {
                        r.json::<Value>().await
                    })
                    .await
            },
            Some(id) => {
                self.client
                    .create(CreateParts::IndexId(self.name, &id))
                    .body(value)
                    .send()
                    .and_then(|r| async {
                        r.json::<Value>().await

                    })
                    .await
            }
        }
    }

    async fn update(&self, value: &Value, options: Self::UpdateOptions)
        -> Result<Value, Self::Error>
    {
        self.client
            .update(UpdateParts::IndexId(self.name, &options.id))
            .body(value)
            .send()
            .and_then(|r| async {
                r.json::<Value>().await
            })
            .await
    }

    async fn delete(&self, options: Self::DeleteOptions)
        -> Result<Value, Self::Error>
    {
        self.client
            .delete(DeleteParts::IndexId(self.name, &options.id))
            .send()
            .and_then(|r| async {
                r.json::<Value>().await

            })
            .await
    }
}
