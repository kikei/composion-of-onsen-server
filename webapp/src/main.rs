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

use cli::template_cli;

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
    App
}


/**
 * Template actions
 */
// fn get_template_action(args: &[String]) -> Option<Action> {
//     let mut a = template_cli::Args::new();
//     let mut opts = Options::new();
//     opts.optflag("s", "show", "Show all templates or one with given id");
//     opts.optflag("a", "add", "Add or update a template");
//     opts.optopt("i", "id", "Template id", "<template_id>");
//     opts.optopt("n", "name", "Template name", "<name>");
//     opts.optopt("p", "path", "Template path", "<path>");
//     opts.optflag("h", "help", "Display this help");
//     let m = get_options(&opts, &args).or_else(|| {
//         print_command_usage(COMMAND_TEMPLATE, &opts);
//         None
//     })?;
//     if m.opt_present("h") {
//         print_command_usage(COMMAND_TEMPLATE, &opts);
//         return None;
//     }
//     if m.opt_present("s") {
//         a.action = template_cli::Action::Show;
//     }
//     if m.opt_present("a") {
//         a.action = template_cli::Action::Add;
//     }
//     a.id = m.opt_str("i");
//     a.name = m.opt_str("n");
//     a.path = m.opt_str("p");
//     Some(Action::Template(a))
// }

/**
 * App actions
 */
// fn get_app_action(args: &[String]) -> Option<Action> {
//     let mut opts = Options::new();
//     opts.optflag("h", "--help", "Display this help");
//     let m = get_options(&opts, args).or_else(|| {
//         print_command_usage(COMMAND_APP, &opts);
//         None
//     })?;
//     if m.opt_present("h") {
//         print_command_usage(COMMAND_APP, &opts);
//         return None;
//     }
//     Some(Action::AppStart)
// }

// fn get_action(args: &[String]) -> Option<Action> {
//     if args.len() == 0 {
//         print_usage();
//         return None;
//     }
//     match args[0].as_str() {
//         COMMAND_TEMPLATE => get_template_action(&args[1..]),
//         COMMAND_APP => get_app_action(&args[1..]),
//         COMMAND_HELP => {
//             print_usage();
//             None
//         },
//         _ => None
//     }
// }

fn app_start() {
    server::start();
}

fn dispatch(args: &Args) {
    match &args.action {
        Action::App => app_start(),
        Action::Template(args) => template_cli::run(args)
    }
}

fn main() {
    // let args: Vec<String> = env::args().collect();
    // let action = get_action(&args[1..]);
    // match action {
    //     Some(a) => match a {
    //         Action::Template(args) => template_cli::run(args),
    //         Action::AppStart => app_start()
    //     },
    //     None => std::process::exit(1)
    // }
    let args = Args::from_args();
    dispatch(&args)
}
