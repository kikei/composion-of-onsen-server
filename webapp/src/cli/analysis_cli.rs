use std::convert::TryFrom;
use structopt::StructOpt;
use tokio::runtime::Runtime;

use crate::analysis::Analysis;
use crate::models;
use crate::utils::elasticsearch::{self, Scroll};
use crate::utils::scrub::scrub;

#[derive(StructOpt, Debug)]
pub enum Action {
    /// Try scrub
    Scrub(ScrubArgs),
    All
}

#[derive(StructOpt, Debug)]
pub struct ScrubArgs {
    /// Name
    #[structopt(short, long)]
    pub name: String
}

fn analysis_scrub(args: &ScrubArgs) {
    let s = scrub(&args.name);
    println!("name: {n}, scrub: {s}", n=&args.name, s=&s);
}

fn analysis_all() {
    let db = elasticsearch::get_unpooled_connection();
    if db.is_err() {
        println!("Failed to get connection, error: {}", db.unwrap_err());
        return;
    }
    Runtime::new().unwrap().block_on(async {
        let db = db.unwrap();
        let models = models::models(&db).await;
        let mut count = 0;
        debug!("analysis_cli::analysis_all, fetching all rows");
        let mut result = models.analyses.scroll(None).await.unwrap();
        while result.hits.hits.len() > 0 {
            let len = result.hits.hits.len();
            debug!("analysis_cli::analysis_all, fetched: {}-{}/{}",
                   count + 1,
                   count + len,
                   &result.hits.total.value);
            count += len;
            while let Some(a) = result.hits.hits.pop() {
                let a = Analysis::try_from(&a._source).unwrap();
                println!("{}", serde_json::to_string(&a).unwrap());
            }
            result = models.analyses
                .scroll(Some(result._scroll_id.unwrap().as_str()))
                .await
                .unwrap();
        }
    })
}

pub fn run(args: &Action) {
    // TODO Use setup_logger
    env_logger::init();
    info!("Log initialized.");

    match args {
        Action::Scrub(args) => analysis_scrub(&args),
        Action::All => analysis_all()
    }
}
