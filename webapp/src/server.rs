use actix_web::{
    web, App, HttpResponse, HttpServer, Responder
};
use listenfd::ListenFd;
use serde::{Deserialize, Serialize};

use r2d2::Pool;
use r2d2_mongodb::{ConnectionOptions, MongodbConnectionManager};

use crate::models;
use crate::utils;
use crate::template::{Template, Render};
use crate::analysis::Analysis;

type DBConnectionPool = Pool<MongodbConnectionManager>;

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

#[derive(Serialize)]
struct AnalysisList {
    analysis: Vec<Analysis>
}

#[derive(Serialize)]
struct TemplateList {
    templates: Vec<Template>
}

pub fn index() -> impl Responder {
    HttpResponse::Ok().body("こんにちは世界")
}

fn add_analysis(json: web::Json<Analysis>,
                pool: web::Data<DBConnectionPool>)
    -> impl Responder {
    println!("Start add_analysis");
    let db = pool.get().unwrap();
    let models = models::models(&db);
    let a = models::analyses::save(&models, &json.into_inner());
    match a {
        Ok(a) => HttpResponse::Ok().json(a),
        Err(_) => HttpResponse::Forbidden().finish() // TODO forbidden?
    }
}

/*
fn post_analysis(body: web::Payload,
                 pool: web::Data<DBConnectionPool>)
                 -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    async_json(body).and_then(|res| {
        match res {
            Ok(v) => {
                let a = Analysis::from(&v);
                HttpResponse::Ok().json(a)
            },
            Err(e) => HttpResponse::BadRequest().body(format!("{}", &e))
        }
    })
}
 */

fn list_analysis(pool: web::Data<DBConnectionPool>) -> impl Responder {
    let db = pool.get().unwrap();
    let models = models::models(&db);
    let result = models::analyses::select(&models);
    match result {
        Ok(ans) => {
            let json = AnalysisList {
                analysis: ans.collect::<Vec<Analysis>>()
            };
            HttpResponse::Ok().json(json)
        },
        Err(e) => {
            println!("Error {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

fn get_analysis(info: web::Path<AnalysisPath>,
                query: web::Query<AnalysisQuery>,
                pool: web::Data<DBConnectionPool>)
                -> impl Responder {
    println!("Start get_analysis, info: {:?}", &info);
    let db = pool.get().unwrap();
    let models = models::models(&db);
    let result = models::analyses::by_id(&models, &info.id);
    match result {
        Ok(Some(a)) => {
            match &query.template {
                Some(template_id) => {
                    let template =
                        models::templates::by_id(&models, &template_id);
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
fn add_template(json: web::Json<Template>,
                pool: web::Data<DBConnectionPool>)
                -> impl Responder {
    println!("Start add_template");
    let db = pool.get().unwrap();
    let models = models::models(&db);
    let t = models::templates::save(&models, &json.into_inner());
    match t {
        Ok(a) => HttpResponse::Ok().json(&a),
        Err(e) => {
            println!("Error {}", e);
            HttpResponse::Forbidden().finish() // TODO forbidden?
        }
    }
}

// GET /templates/
fn list_templates(pool: web::Data<DBConnectionPool>) -> impl Responder {
    let db = pool.get().unwrap();
    let models = models::models(&db);
    let result = models::templates::select(&models);
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
fn get_template(info: web::Path<AnalysisPath>,
                pool: web::Data<DBConnectionPool>)
                -> impl Responder {
    println!("Start get_template, info: {:?}", &info);
    let db = pool.get().unwrap();
    let models = models::models(&db);
    let result = models::templates::by_id(&models, &info.id);
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
fn debug_scrub(query: web::Query<ScrubQuery>) -> String {
    utils::scrub::scrub(&query.title)
}

fn create_pool() -> DBConnectionPool {
    let dbuser = "webapp";
    let dbpassword = "PASSWORD";
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

fn setup_logger() {
    env_logger::init();
    info!("Log initialized.");
}

pub fn start() {
    let address = "0.0.0.0:8088";
    /*
     * $ systemfd --no-pid -s http::0.0.0.0:8088 -- cargo watch -x run
     * https://github.com/mitsuhiko/systemfd
     */
    setup_logger();
    let mut listenfd = ListenFd::from_env();
    println!("Starting HTTP server ({})", &address);
    let mut server = HttpServer::new(|| {
        let pool = create_pool();
        App::new()
            .data(pool.clone())
            .route("/", web::get().to(index))
            .service(
                web::scope("/analysis")
                    .route("/", web::post().to(add_analysis))
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
    // .workers(4)
        .run()
        .unwrap();
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
