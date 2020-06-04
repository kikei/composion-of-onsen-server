use actix_web::{
    web, App, HttpResponse, HttpServer, Responder
};
use listenfd::ListenFd;
use serde::{Deserialize, Serialize};

use crate::models::{self, Models, analyses};
use crate::utils;
use crate::utils::elasticsearch::{DBConnectionPool, create_pool};
use crate::template::{Template, Render};
use crate::analysis::Analysis;

#[derive(Deserialize)]
struct ScrubQuery {
    title: String
}

#[derive(Debug, Deserialize)]
struct AnalysisPath {
    id: String
}

#[derive(Debug, Deserialize)]
struct AnalysisQuery {
    template: Option<String>
}

#[derive(Debug, Deserialize)]
struct AnalysisListQuery {
    /*
    #[serde(flatten)]
    pub select: analyses::SelectOptions
    */
    #[serde(rename = "q", default)]
    pub query: Option<String>,
    #[serde(rename = "p", default)]
    pub page: u32,
    #[serde(rename = "l", default = "default_limit")]
    pub limit: u32,
    #[serde(rename = "o", default = "default_order_by")]
    pub order_by: analyses::SortKey,
    #[serde(rename = "d", default = "default_direction")]
    pub direction: i32
}

fn default_limit() -> u32 { 20 }
fn default_order_by() -> analyses::SortKey { analyses::SortKey::LastModified }
fn default_direction() -> i32 { -1 }

impl From<&AnalysisListQuery> for analyses::SelectOptions {
    fn from(a: &AnalysisListQuery) -> Self {
        analyses::SelectOptions {
            query: a.query.clone(),
            skip: a.page * a.limit,
            limit: a.limit,
            order_by: a.order_by,
            direction: a.direction
        }
    }
}


#[derive(Serialize)]
struct AnalysisList {
    total: u32,
    page: u32,
    limit: u32,
    analysis: Vec<Analysis>
}

#[derive(Serialize)]
struct TemplateList {
    templates: Vec<Template>
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("こんにちは世界")
}

async fn add_analysis(json: web::Json<Analysis>,
                      pool: web::Data<DBConnectionPool>)
    -> impl Responder {
    println!("Start add_analysis");
    let models = Models::new(pool.get_ref());
    let a = &json.into_inner();
    match a.id {
        None => {
            // add_analysis is allowed when id is None
            let a = analyses::save(&models, &a).await;
            match a {
                Ok(a) => HttpResponse::Ok().json(a),
                Err(_) => HttpResponse::Forbidden().finish()
            }
        },
        Some(_) => HttpResponse::Forbidden().finish()
    }
}

async fn update_analysis(info: web::Path<AnalysisPath>,
                   json: web::Json<Analysis>,
                   pool: web::Data<DBConnectionPool>)
                   -> impl Responder {
    println!("Start add_analysis");
    let models = Models::new(pool.get_ref());
    let a = &json.into_inner();
    match &a.id {
        Some(id) if id.clone() == info.id => {
            // update_analysis is allowed when id matches with path
            let a = analyses::save(&models, &a).await;
            match a {
                Ok(a) => HttpResponse::Ok().json(a),
                Err(_) => HttpResponse::Forbidden().finish()
            }
        },
        _ => HttpResponse::Forbidden().finish()
    }
}



async fn list_analysis(query: web::Query<AnalysisListQuery>,
                 pool: web::Data<DBConnectionPool>) -> impl Responder {
    let models = Models::new(pool.get_ref());
    let query = &query.into_inner();
    let options = analyses::SelectOptions::from(query);
    let result = models::analyses::select(&models, &options).await;
    match result {
        Ok(ans) => {
            let json = AnalysisList {
                total: ans.total,
                page: query.page,
                limit: query.limit,
                analysis: ans.items.collect::<Vec<Analysis>>()
            };
            HttpResponse::Ok().json(json)
        },
        Err(e) => {
            warn!("Failed to list analyses, e: {}", e);
            HttpResponse::NotFound().finish()
        }
    }
}

async fn get_analysis(info: web::Path<AnalysisPath>,
                query: web::Query<AnalysisQuery>,
                pool: web::Data<DBConnectionPool>)
                -> impl Responder {
    println!("Start get_analysis, info: {:?}", &info);
    let models = Models::new(pool.get_ref());
    let result = models::analyses::by_id(&models, &info.id).await;
    match result {
        Ok(Some(a)) => {
            match &query.template {
                Some(template_id) => {
                    let template =
                        models::templates::by_id(&models, &template_id).await;
                    match template {
                        Ok(Some(template)) =>
                            HttpResponse::Ok().body(match &a.render(&template) {
                                Ok(body) => body,
                                Err(e) => e
                            }),
                        Ok(None) => HttpResponse::NotFound().finish(),
                        Err(e) => {
                            println!("main::get_analysis, error: {}", e);
                            HttpResponse::InternalServerError().finish()
                        }
                    }
                },
                None => // Return by JSON
                   HttpResponse::Ok().json(a)
            }
        },
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            println!("Error {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// POST /templates/
async fn add_template(json: web::Json<Template>,
                pool: web::Data<DBConnectionPool>)
                -> impl Responder {
    println!("Start add_template");
    let models = Models::new(pool.get_ref());
    let t = models::templates::save(&models, &json.into_inner()).await;
    match t {
        Ok(a) => HttpResponse::Ok().json(&a),
        Err(e) => {
            println!("Error {}", e);
            HttpResponse::Forbidden().finish() // TODO forbidden?
        }
    }
}

// GET /templates/
async fn list_templates(pool: web::Data<DBConnectionPool>) -> impl Responder {
    let models = Models::new(pool.get_ref());
    let result = models::templates::select(&models).await;
    match result {
        Ok(ts) => {
            HttpResponse::Ok().json(TemplateList {
                templates: ts.collect::<Vec<Template>>()
            })
        },
        Err(e) => {
            println!("Error {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// GET /templates/{id}
async fn get_template(info: web::Path<AnalysisPath>,
                pool: web::Data<DBConnectionPool>)
                -> impl Responder {
    println!("Start get_template, info: {:?}", &info);
    let models = Models::new(pool.get_ref());
    let result = models::templates::by_id(&models, &info.id).await;
    match result {
        Ok(Some(i)) => HttpResponse::Ok().json(i),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            println!("Error {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// /debug/scrube
async fn debug_scrub(query: web::Query<ScrubQuery>) -> String {
    utils::scrub::scrub(&query.title)
}

fn setup_logger() {
    env_logger::init();
    info!("Log initialized.");
}

#[actix_rt::main]
// pub async fn start() -> std::io::Result<()> {
// pub async fn start() -> std::result::Result<(), std::io::Error> {
pub async fn start() -> () {
    let address = "0.0.0.0:8088";
    /*
     * $ systemfd --no-pid -s http::0.0.0.0:8088 -- cargo watch -x run
     * https://github.com/mitsuhiko/systemfd
     */
    setup_logger();
    let mut listenfd = ListenFd::from_env();
    println!("Starting HTTP server ({})", &address);
    // Connection
    let pool = create_pool();
    if let Err(e) = pool {
        println!("Failed to create database connection, {}", &e);
        return;
    }
    let pool = pool.unwrap();
    Models::new(&pool).setup().await;
    // Setup server
    let mut server = HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .route("/", web::get().to(index))
            .service(
                web::scope("/analysis")
                    .route("/", web::put().to(add_analysis))
                    .route("/{id}", web::post().to(update_analysis))
                    .route("/", web::get().to(list_analysis))
                    .route("/{id}", web::get().to(get_analysis))
            )
            .service(
                web::scope("/templates")
                    .route("/", web::post().to(add_template))
                    .route("/", web::get().to(list_templates))
                    .route("/{id}", web::get().to(get_template))
            )
            .service(
                web::scope("/debug")
                    .route("/scrub", web::get().to(debug_scrub))
            )
    });
    server = match listenfd.take_tcp_listener(0).unwrap() {
        Some(l) => server.listen(l).unwrap(),
        None => server.bind(&address).unwrap()
    };
    server
        .workers(1)
        .run()
        .await;
}

#[cfg(test)]
mod tests {
    /*
    use super::*;
    use actix_web::dev::Service;
    use actix_web::{test, App};

    fn create_test_service() -> impl Service {
        let mut app = test::init_service({
            let pool = create_pool();
            App::new()
                .data(pool.clone())
                .route("/", web::get().to(index))
                .service(
                    web::scope("/analysis")
                        .route("/", web::put().to_async(add_analysis))
                        .route("/", web::get().to(list_analysis))
                        .route("/{id}", web::get().to(get_analysis))
                        // .route("/{id}", web::post().to(update_analysis))
                )
        });
        app
    }

    #[test]
    fn test_index_ok() {
        let req = test::TestRequest::get().uri("/").to_request();
        // let res = test::block_on(app.call(req)).unwrap(); // actix_web::dev::ServiceResponse
        // assert_eq!(&res.status(), http::StatusCode::OK);
        let mut app = test::init_service({
            let pool = create_pool();
            App::new()
                .data(pool.clone())
                .route("/", web::get().to(index))
                .service(
                    web::scope("/analysis")
                        .route("/", web::put().to_async(add_analysis))
                        .route("/", web::get().to(list_analysis))
                        .route("/{id}", web::get().to(get_analysis))
                    // .route("/{id}", web::post().to(update_analysis))
                )
        });
        // let mut app = create_test_service();
        let res = test::call_service(&mut app, req);
        let result = test::read_body(res);
        assert_eq!(result, "こんにちは世界");
    }
     */
}
