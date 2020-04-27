use std::fs;
use structopt::StructOpt;

use crate::template::{Template};
use crate::utils::mongodb;
use crate::models::{self, Models};

#[derive(StructOpt, Debug)]
pub enum Action {
    /// Show template
    Show(ShowArgs),
    /// Add template
    Add(AddArgs)
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

fn template_show(models: &Models, args: &ShowArgs) {
    match &args.id {
        Some(id) => match models::templates::by_id(&models, &id) {
            Ok(Some(t)) => println!("{} {}\n{}",
                                    &t.id.unwrap_or("none".to_string()),
                                    &t.name, &t.body),
            Ok(None) => println!("No matched"),
            Err(e) =>
                println!("Failed to read templates: {}, error: {}", &id, e)
        },
        None => match models::templates::select(&models) {
            Ok(it) => for t in it {
                println!("{}: {} ({})",
                         &t.id.unwrap_or("none".to_string()),
                         &t.name, &t.body.len());
            },
            Err(e) => println!("Failed to read templates, error: {}", e)
        }
    }
}

fn template_add(models: &Models, args: &AddArgs) {
    if args.path.is_none() {
        println!("Path required, args: {:?}", args);
        return;
    };
    if args.name.is_none() {
        println!("Name required, args: {:?}", args);
        return;
    }
    let path = args.path.as_ref().unwrap();
    let name = args.name.as_ref().unwrap();
    let body = fs::read_to_string(path.as_str());
    if body.is_err() {
        println!("File does not exists, path: {}, error: {}",
                 &path, &body.unwrap_err());
        return;
    };
    
    let t = Template {
        id: args.id.clone(),
        name: name.to_string(),
        body: body.unwrap()
    };
    match models::templates::save(models, &t) {
        Ok(t) => println!("Successfully save template: {:?}", t),
        Err(e) => println!("Failed to save template, error: {}", e)
    }
}

pub fn run(args: &Action) {
    // TODO Use setup_logger
    env_logger::init();
    info!("Log initialized.");

    let db = mongodb::get_unpooled_connection();
    if db.is_err() {
        println!("Failed to get connection, error: {}", db.unwrap_err());
        return;
    }
    let models = models::models(&db.unwrap());
    match args {
        Action::Show(args) => template_show(&models, &args),
        Action::Add(args) => template_add(&models, &args)
    }
}
