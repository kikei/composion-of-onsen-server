extern crate handlebars;
extern crate r2d2;
extern crate r2d2_mongodb;

#[macro_use]
extern crate log;

use structopt::StructOpt;

mod utils;
mod template;
mod analysis;
mod models;
mod server;
mod cli;

use cli::{template_cli, analysis_cli};

#[derive(StructOpt, Debug)]
#[structopt(name = "onsen-compo", about = "CLI tool for onsen app")]
struct Args {
    #[structopt(subcommand)]
    action: Action
}

#[derive(StructOpt, Debug)]
enum Action {
    /// Control template
    Template(template_cli::Action),
    /// Start web application
    App,
    /// Control analysis
    Analysis(analysis_cli::Action)
}

fn app_start() {
    server::start();
}

fn dispatch(args: &Args) {
    match &args.action {
        Action::App => app_start(),
        Action::Template(args) => template_cli::run(args),
        Action::Analysis(args) => analysis_cli::run(args)
    }
}

fn main() {
    let args = Args::from_args();
    dispatch(&args)
}
