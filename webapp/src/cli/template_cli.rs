use std::fs;

use crate::template::{Template};
use crate::utils::mongodb;
use crate::models::{self, Models};

#[derive(Clone, Debug)]
pub enum Action {
    List,
    Add
}

#[derive(Debug)]
pub struct Args {
    pub action: Action,
    pub id: Option<String>,
    pub name: Option<String>,
    pub path: Option<String>
}

impl Args {
    pub fn new() -> Self {
        Args {
            action: Action::List,
            id: None,
            name: None,
            path: None
        }
    }
}

fn template_list(models: &Models, _args: &Args) {
    match models::templates::select(&models) {
        Ok(it) => for t in it {
            println!("{}: {} ({})",
                     &t.id.unwrap_or("none".to_string()),
                     &t.name, &t.body.len());
        },
        Err(e) => println!("Failed to read templates, error: {}", e)
    }
}

fn template_add(models: &Models, args: &Args) {
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

pub fn run(args: Args) {
    // TODO Use setup_logger
    env_logger::init();
    info!("Log initialized.");

    let db = mongodb::get_unpooled_connection();
    if db.is_err() {
        println!("Failed to get connection, error: {}", db.unwrap_err());
        return;
    }
    let models = models::models(&db.unwrap());
    match args.action {
        Action::List => template_list(&models, &args),
        Action::Add => template_add(&models, &args)
    }
}
