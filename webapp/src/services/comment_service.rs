use std::convert::TryFrom;
use std::str;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{json};

use actix_multipart::Multipart;
use actix_web::{
    web, HttpRequest, HttpResponse, Responder, Scope,
    http::{HeaderMap, header::AUTHORIZATION}
};
use futures_util::stream::StreamExt;

use crate::comment::{Comment};
use crate::token::{Authentication, TokenData, make_auth};
use crate::models::{
    Models,
    comments::{self, DeleteCommentOptions, SelectOptions, CommentIdGenerator},
    comment_photos::{self, PhotoPath, CommentPhotoIdGenerator}
};
use crate::utils::{
    identifier::Generate,
    elasticsearch::DBConnectionPool,   
    web::{read_content_length, save_uploaded_file,
          SaveUploadedFileOptions},
    image::ImagePath
};

// Constants

const IMAGE_BYTES_MIN: usize = 1024 * 10;

const DIRECTORY_UPLOAD_TMP: &str = "/data/comments/upload";

const NAME_IMAGE0: &str = "images0";
const NAME_IMAGE1: &str = "images1";
const NAME_IMAGE2: &str = "images2";
const NAME_IMAGE3: &str = "images3";
const NAME_IMAGE4: &str = "images4";

// Structures

#[derive(Deserialize, Debug)]
struct CommentRequest {
    id: Option<String>,
    #[serde(rename = "parentId")]
    parent_id: String,
    username: String,
    email: Option<String>,
    web: Option<String>,
    body: String,
    #[serde(rename = "lastModified")]
    last_modified: Option<f64>,
    #[serde(rename = "createdAt")]
    created_at: Option<f64>
}

impl CommentRequest {
    fn to_comment(self: Self, auth: &Authentication) -> Comment {
        Comment {
            id: self.id,
            parent_id: self.parent_id,
            username: self.username,
            email: self.email,
            web: self.web,
            body: self.body,
            images: Vec::new(),
            auth: auth.clone(),
            last_modified: now_nanos(),
            created_at: self.created_at.unwrap_or(now_nanos())
        }
    }
}

#[derive(Debug)]
struct ImageUploadRequest {
    images: Vec<PathBuf>,
}

impl ImageUploadRequest {
    fn paths(self: &Self, comment: &Comment) -> Vec<(PathBuf, PhotoPath)> {
        let photo_paths = self.images.iter().map(|i| {
            let id = i.file_stem().and_then(|s| s.to_str()).unwrap();
            (
                i.clone(),
                PhotoPath {
                    analysis: comment.parent_id.clone(),
                    comment: comment.id.clone().unwrap(),
                    id: id.to_string()
                }
            )
        }).collect();
        photo_paths
    }
}

#[derive(Debug, Deserialize)]
struct CommentPath {
    id: String
}

#[derive(Debug, Serialize, Deserialize)]
struct DeleteResult {
    token: String,
    id: String
}

#[derive(Debug, Serialize)]
struct UploadResult {
    token: String,
    comment: Comment
}

impl From<CommentPath> for DeleteCommentOptions {
    fn from(item: CommentPath) -> Self {
        DeleteCommentOptions {
            id: item.id
        }
    }
}

#[derive(Debug, Deserialize)]
struct CommentListQuery {
    #[serde(rename = "q", default)]
    pub query: Option<String>,
    #[serde(rename = "a", default)]
    pub parent_id: Option<String>,
    #[serde(rename = "l", default = "default_limit")]
    pub limit: u32,
    // #[serde(rename = "o", default = "default_order_by")]
    // pub order_by: analyses::SortKey,
    // #[serde(rename = "d", default = "default_direction")]
    // pub direction: i32
}

fn default_limit() -> u32 { 20 }

#[derive(Serialize)]
struct CommentList {
    total: u32,
    page: u32,
    limit: u32,
    comments: Vec<Comment>
}

impl From<&CommentListQuery> for SelectOptions {
    fn from(item: &CommentListQuery) -> Self {
        SelectOptions {
            query: match (item.query.as_ref(), item.parent_id.as_ref()) {
                (Some(q), _) =>
                    Some(comments::SelectQuery::Text(q.to_string())),
                (None, Some(p)) =>
                    Some(comments::SelectQuery::Parent(p.to_string())),
                _ => None
            },
            limit: item.limit
        }
    }
}

// Utilities

fn now_nanos() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or_else(|e| {
            warn!("Failed to calulate epoch!? {}", &e);
            0.0
        })
}

async fn read_image_payload<'a>(comment_id: &str,
                                req: &HttpRequest, payload: &mut Multipart)
    -> Result<ImageUploadRequest, String>
{
    debug!("read_image_payload, start");
    // while let Ok(Some(mut field)) = payload.try_next().await {

    let content_length = read_content_length(req.headers())
        .ok_or("Content-Length must be set".to_string())?;

    let mut images = Vec::new();

    while let Some(item) = payload.next().await {
        if let Err(e) = item {
            info!("could not read payload, e:{}", &e);
            Err(format!("Could not read payload, e: {}", &e))?;
            continue;
        }
        let mut field = item.unwrap();
        let mimetype = field.content_type();
        let content = field.content_disposition().unwrap();
        let name = content.get_name();
        let filename = content.get_filename();

        debug!("Read field, name: {:?}, filename: {:?}, mimetype: {:?}",
               &name, &filename, &mimetype);

        match (name, filename) {
            // File upload
            (Some(NAME_IMAGE0), Some(filename)) |
            (Some(NAME_IMAGE1), Some(filename)) |
            (Some(NAME_IMAGE2), Some(filename)) |
            (Some(NAME_IMAGE3), Some(filename)) |
            (Some(NAME_IMAGE4), Some(filename)) => {
                let image_id =
                    CommentPhotoIdGenerator::new(comment_id, filename)
                    .generate();
                let tmp = ImagePath {
                    name: image_id,
                    mimetype: mimetype.clone(),
                    dirname: Some(DIRECTORY_UPLOAD_TMP.to_string())
                };
                if tmp.extension_str().is_none() {
                    Err(format!("Unsupported mimetype: {}", &tmp.mimetype))?;
                }

                info!("Try writing uploaded file: {}",
                      &tmp.filename_full().unwrap());
                save_uploaded_file(
                    &mut field,
                    SaveUploadedFileOptions {
                        directory: &tmp.dirname.as_ref().unwrap(),
                        path: &tmp.filename().unwrap(),
                        // Must have exactly same size with Content-Length header
                        bytes_max: Some(content_length as usize),
                        bytes_min: Some(IMAGE_BYTES_MIN)
                    }
                ).await?;
                info!("Completed writing uploaded file: {}",
                      &tmp.filename_full().unwrap());
                images.push(Path::new(&tmp.filename_full().unwrap()).to_owned());
            },
            _ => continue
        }
    } 
    Ok(ImageUploadRequest {
        images: images
    })
}

fn read_authentication_bearer(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(AUTHORIZATION)?;
    let bearer = value.to_str().ok()?;
    let v = bearer.splitn(2, ' ').collect::<Vec<&str>>();
    match v.len() {
        2 => Some(v.get(1).unwrap()),
        _ => None
    }
}

async fn add_comment(req: HttpRequest,
                     mut json: web::Json<CommentRequest>,
                     pool: web::Data<DBConnectionPool>) -> impl Responder {
    println!("start adding comment");
    let models = Models::new(pool.get_ref());
    // Read Authentication header
    let auth = read_authentication_bearer(&req.headers());
    let auth = match make_auth(auth) {
        Ok(a) => a,
        Err(_) => make_auth(None).unwrap() // Create new guest
    };
    match json.id {
        // add_comments is allowed when id is None
        Some(_) => HttpResponse::Forbidden().finish(),
        None => {
            // Assign new comment id
            let comment_id =
                CommentIdGenerator::new(json.parent_id.as_ref(),
                                        json.username.as_ref()).generate();
            json.id = Some(comment_id);
            json.created_at = None;
            let comment = json.into_inner().to_comment(&auth);
            let a = comments::save(&models, &comment).await;
            match a {
                Err(e) => {
                    info!("Couldn't save comment, e: {}", &e);
                    HttpResponse::Forbidden().finish()
                }
                Ok(a) => {
                    let token = String::from(auth.clone());
                    let (auth_type, userid) = match &auth {
                        Authentication::Guest { guestid } => ("guest", guestid),
                        Authentication::Signin { userid } => ("siginin", userid)
                    };
                    HttpResponse::Ok()
                        .json(json!({
                            "auth_type": auth_type,
                            "userid": userid,
                            "token": token,
                            "comment": a
                        }))
                }
            }
        }
    }
}

async fn add_comment_images(req: HttpRequest,
                            info: web::Path<CommentPath>,
                            mut payload: Multipart,
                            pool: web::Data<DBConnectionPool>)
    -> impl Responder
{
    println!("start adding images on comment {}", &info.id);

    // Load token
    let auth = read_authentication_bearer(&req.headers())
        .and_then(|a| TokenData::try_from(a).ok());
    if auth.is_none() {
        debug!("Token required");
        return HttpResponse::Unauthorized().finish();
    }

    let token = auth.unwrap();
    let models = Models::new(pool.get_ref());

    // Check if comment owner
    let mut comment = match comments::by_id(&models, &info.id).await {
        Ok(Some(comment)) if !comment.is_editable(&token) => {
            info!("Authorization unmatch, comment: {:?}, token: {:?}",
                  &comment, &token);
            return HttpResponse::Unauthorized().finish()
        },
        Ok(Some(comment)) => comment,
        Err(e) => {
            warn!("Failed to get comment, e: {}", &e);
            return HttpResponse::NotFound().finish()
        },
        _ => return HttpResponse::NotFound().finish()
    };

    // Read multipart formdata
    let form = read_image_payload((&comment).id.as_ref().unwrap(),
                                  &req, &mut payload).await;
    if let Err(e) = form {
        warn!("Failed to read payload, e: {}", &e);
        return HttpResponse::BadRequest().finish();
    }
    let form = form.unwrap();
    let photos = form.paths(&comment);

    let mut images = Vec::new();
    // Save and convert
    for (src, dest) in &photos {
        let photo = comment_photos::save(&models, &src, dest).await;
        match photo {
            Ok(photos) => images.push(photos),
            Err(e) => warn!("Skipped photo, e: {}", &e)
        }
    }

    // Save comment
    for profiles in images {
        comment.add_image(profiles);
    }
    let result = comments::save(&models, &comment).await;

    // Response
    match result {
        Ok(comment) => {
            let token = String::from(Authentication::from(token));
            HttpResponse::Ok()
                .json(UploadResult { token: token, comment: comment })
        },
        Err(e) => {
            warn!("Failed to add images on comment, e: {}", &e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// async fn update_analysis(info: web::Path<AnalysisPath>,
//                    json: web::Json<Analysis>,
//                    pool: web::Data<DBConnectionPool>)
//                    -> impl Responder {
//     println!("Start add_analysis");
//     let models = Models::new(pool.get_ref());
//     let a = &json.into_inner();
//     match &a.id {
//         Some(id) if id.clone() == info.id => {
//             // update_analysis is allowed when id matches with path
//             let a = analyses::save(&models, &a).await;
//             match a {
//                 Ok(a) => HttpResponse::Ok().json(a),
//                 Err(_) => HttpResponse::Forbidden().finish()
//             }
//         },
//         _ => HttpResponse::Forbidden().finish()
//     }
// }
// 
// 
async fn list_comments(query: web::Query<CommentListQuery>,
                       pool: web::Data<DBConnectionPool>) -> impl Responder {
    let models = Models::new(pool.get_ref());
    let query = &query.into_inner();
    let options = SelectOptions::from(query);
    let result = comments::select(&models, &options).await;
    match result {
        Ok(cs) => {
            let json = CommentList {
                total: cs.total,
                page: 0,
                limit: query.limit,
                comments: cs.items.collect::<Vec<Comment>>()
            };
            HttpResponse::Ok().json(json)
        },
        Err(e) => {
            warn!("Failed to list comments, e: {}", e);
            HttpResponse::NotFound().finish()
        }
    }
}

async fn delete_comment(req: HttpRequest,
                        info: web::Path<CommentPath>,
                        pool: web::Data<DBConnectionPool>) -> impl Responder
{
    println!("Start delete_comment, info: {:?}", &info);

    // Load token
    let auth = read_authentication_bearer(&req.headers())
        .and_then(|a| TokenData::try_from(a).ok());
    if auth.is_none() {
        debug!("Token required");
        return HttpResponse::Unauthorized().finish();
    }

    let token = auth.unwrap();
    let models = Models::new(pool.get_ref());

    // Check if comment owner
    let original = match comments::by_id(&models, &info.id).await {
        Ok(Some(comment)) if !comment.is_editable(&token) => {
            info!("Authorization unmatch, comment: {:?}, token: {:?}",
                  &comment, &token);
            return HttpResponse::Unauthorized().finish()
        },
        Ok(Some(comment)) => comment,
        Err(e) => {
            warn!("Failed to get comment, e: {}", &e);
            return HttpResponse::NotFound().finish()
        },
        _ => return HttpResponse::NotFound().finish()
    };

    // Delete comment
    let options = DeleteCommentOptions::from(info.into_inner());
    let result = comments::delete(&models, options).await;

    // Delete images on the comment
    for photo in original.images {
        match comment_photos::delete(&models, &photo).await {
            Ok(()) => debug!("Successfully deleted photos on comment {:?}",
                             &original.id),
            Err(e) => warn!("{}, comment: {:?}", &e, &original.id)
        }
    }

    // Response
    match result {
        Ok(id) => {
            let token = String::from(Authentication::from(token));
            HttpResponse::Ok()
                .json(DeleteResult { token: token, id: id })
        },
        Err(e) => {
            warn!("Failed to delete comment, e: {}", &e);
            HttpResponse::NotFound().finish()
        }
    }
}

// 
// async fn get_analysis(info: web::Path<AnalysisPath>,
//                 query: web::Query<AnalysisQuery>,
//                 pool: web::Data<DBConnectionPool>)
//                 -> impl Responder {
//     println!("Start get_analysis, info: {:?}", &info);
//     let models = Models::new(pool.get_ref());
//     let result = models::analyses::by_id(&models, &info.id).await;
//     match result {
//         Ok(Some(a)) => {
//             match &query.template {
//                 Some(template_id) => {
//                     let template =
//                         models::templates::by_id(&models, &template_id).await;
//                     match template {
//                         Ok(Some(template)) =>
//                             HttpResponse::Ok().body(match &a.render(&template) {
//                                 Ok(body) => body,
//                                 Err(e) => e
//                             }),
//                         Ok(None) => HttpResponse::NotFound().finish(),
//                         Err(e) => {
//                             println!("main::get_analysis, error: {}", e);
//                             HttpResponse::InternalServerError().finish()
//                         }
//                     }
//                 },
//                 None => // Return by JSON
//                    HttpResponse::Ok().json(a)
//             }
//         },
//         Ok(None) => HttpResponse::NotFound().finish(),
//         Err(e) => {
//             println!("Error {}", e);
//             HttpResponse::InternalServerError().finish()
//         }
//     }
// }

pub fn service(scope: Scope) -> Scope {
    scope
        .route("/", web::post().to(add_comment))
        // .route("/{id}", web::post().to(update_comment))
        .route("/", web::get().to(list_comments))
        // .route("/{id}", web::get().to(get_comment))
        .route("/{id}", web::delete().to(delete_comment))
        .route("/{id}/images/", web::post().to(add_comment_images))
}

#[derive(Deserialize)]
struct StaticPath {
    filename: String
}

async fn get_static(path: web::Path<StaticPath>)
                    -> actix_web::Result<actix_files::NamedFile>
{
    let p = format!("{}/{}", comment_photos::DIRECTORY_UPLOAD, &path.filename);
    actix_web::Result::Ok(actix_files::NamedFile::open(p.clone())?)
}

pub fn service_static(scope: Scope) -> Scope {
    scope
        .route("/image/{filename:.*}", web::get().to(get_static))
}
