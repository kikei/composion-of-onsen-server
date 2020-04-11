extern crate r2d2;
extern crate r2d2_mongodb;
extern crate handlebars;

#[macro_use]
extern crate log;

mod utils;
mod template;
mod analysis;
mod models;
mod server;

fn main() {
    server::start();
}
