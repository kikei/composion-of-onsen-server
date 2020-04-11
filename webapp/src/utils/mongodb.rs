use r2d2_mongodb::mongodb::Document;
use r2d2_mongodb::{ConnectionOptions, MongodbConnectionManager};
use r2d2::{self, Pool, PooledConnection};

pub type DBConnectionPool = Pool<MongodbConnectionManager>;
pub type DBConnection = PooledConnection<MongodbConnectionManager>;

/**
 * Connection
 */
pub fn create_pool() -> DBConnectionPool {
    let dbuser = "webapp";
    let dbpassword = "webapp";
    let address = "mongodb";
    let dbname = "onsen";
    let manager = MongodbConnectionManager::new(
        ConnectionOptions::builder()
            .with_host(&address, 27017)
            .with_db(&dbname)
            .with_auth(&dbuser, &dbpassword)
            .build());
    let pool = Pool::builder()
        .max_size(2)
        .build(manager)
        .unwrap();
    pool
}

pub fn get_unpooled_connection() -> Result<DBConnection, r2d2::Error>  {
    let pool = create_pool();
    pool.get()
}

/**
 * Extractor for Document
 */
pub fn document_str(item: &Document, key: &str) -> Option<String> {
    item.get_str(key).ok().map(|v| String::from(v))
}

pub fn document_number(item: &Document, key: &str) -> Option<f64> {
    item.get_f64(key).ok()
}
