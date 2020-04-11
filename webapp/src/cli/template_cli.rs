use crate::utils::mongodb;
use crate::models::{self, Models};

#[derive(Clone)]
pub enum Action {
    List,
    Add
}

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

fn template_add(_models: &Models, _args: &Args) {
    println!("Not implemented");
}

pub fn run(args: Args) {
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
