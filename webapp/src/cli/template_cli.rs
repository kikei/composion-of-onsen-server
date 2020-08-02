use std::fs;
use structopt::StructOpt;
use tokio::runtime::Runtime;

use crate::utils::elasticsearch;
use crate::template::{Template};
use crate::models::{self, templates, Models};

#[derive(StructOpt, Debug)]
pub enum Action {
    /// Show template
    Show(ShowArgs),
    /// Add template
    Add(AddArgs),
    /// Delete template
    Delete(DeleteArgs)
}

#[derive(StructOpt, Debug)]
pub struct ShowArgs {
    /// Template Id
    #[structopt(short, long)]
    pub id: Option<String>,
}

#[derive(StructOpt, Debug)]
pub struct AddArgs {
    /// Template Id
    #[structopt(short, long)]
    pub id: Option<String>,

    /// Template name
    #[structopt(short, long)]
    pub name: Option<String>,

    /// Path to template file
    #[structopt(short, long)]
    pub path: Option<String>
}

#[derive(StructOpt, Debug)]
pub struct DeleteArgs {
    /// Template Id
    #[structopt(short, long)]
    pub id: String
}

impl From<&DeleteArgs> for templates::DeleteTemplateOptions {
    fn from(a: &DeleteArgs) -> Self {
        templates::DeleteTemplateOptions {
            id: a.id.clone()
        }
    }
}

// impl Args {
//     pub fn new() -> Self {
//         Args {
//             action: Action::Show,
//             id: None,
//             name: None,
//             path: None
//         }
//     }
// }

async fn template_show<'a>(models: &Models<'a>, args: &ShowArgs) {
    match &args.id {
        Some(id) => match models::templates::by_id(&models, &id).await {
            Ok(Some(t)) => println!("{} {}\n{}",
                                    &t.id.unwrap_or("none".to_string()),
                                    &t.name, &t.body),
            Ok(None) => println!("No matched"),
            Err(e) =>
                println!("Failed to read templates: {}, error: {}", &id, e)
        },
        None => match models::templates::select(&models).await {
            Ok(it) => for t in it {
                println!("{}: {} ({})",
                         &t.id.unwrap_or("none".to_string()),
                         &t.name, &t.body.len())
            },
            Err(e) => println!("Failed to read templates, error: {}", e)
        }
    }
}

async fn template_add<'a>(models: &Models<'a>, args: &AddArgs) {
    if args.id.is_none() {
        println!("ID required, args: {:?}", args);
        return;
    }
    if args.path.is_none() {
        println!("Path required, args: {:?}", args);
        return;
    };
    if args.name.is_none() {
        println!("Name required, args: {:?}", args);
        return;
    }
    let id = args.id.as_ref().unwrap();
    let path = args.path.as_ref().unwrap();
    let name = args.name.as_ref().unwrap();
    let body = fs::read_to_string(path.as_str());
    if body.is_err() {
        println!("File does not exists, path: {}, error: {}",
                 &path, &body.unwrap_err());
        return;
    };
    
    let t = Template {
        id: Some(id.to_string()),
        name: name.to_string(),
        body: body.unwrap()
    };
    match models::templates::save(models, &t).await {
        Ok(t) => println!("Successfully save template: {:?}", t),
        Err(e) => println!("Failed to save template, error: {}", e)
    }
}

async fn template_delete<'a>(models: &Models<'a>, args: &DeleteArgs) {
    let options = templates::DeleteTemplateOptions::from(args);
    match models::templates::delete(models, options).await {
        Ok(t) => println!("Successfully delete template: {:?}", t),
        Err(e) => println!("Failed to delete template, error: {}", e)
    }
}

pub fn run(args: &Action) {
    // TODO Use setup_logger
    env_logger::init();
    info!("Log initialized.");

    let db = elasticsearch::get_unpooled_connection();
    if db.is_err() {
        println!("Failed to get connection, error: {}", db.unwrap_err());
        return;
    }
    Runtime::new().unwrap().block_on(async {
        let db = db.unwrap();
        let models = Models::new(&db);
        match args {
            Action::Add(args) => template_add(&models, &args).await,
            Action::Show(args) => template_show(&models, &args).await,
            Action::Delete(args) => template_delete(&models, &args).await
        }
    })
}

