use structopt::StructOpt;

use crate::utils::scrub::scrub;

#[derive(StructOpt, Debug)]
pub enum Action {
    /// Try scrub
    Scrub(ScrubArgs)
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

pub fn run(args: &Action) {
    // TODO Use setup_logger
    env_logger::init();
    info!("Log initialized.");

    match args {
        Action::Scrub(args) => analysis_scrub(&args),
    }
}
