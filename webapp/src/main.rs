extern crate getopts;
extern crate handlebars;
extern crate r2d2;
extern crate r2d2_mongodb;

#[macro_use]
extern crate log;

use getopts::{Options, Matches};
use std::env;

mod utils;
mod template;
mod analysis;
mod models;
mod server;

const COMMAND_TEMPLATE: &str = "template";
const COMMAND_APP: &str = "app";
const COMMAND_HELP: &str = "help";

enum Action {
    TemplatesList,
    AppStart
}

fn get_program_name() -> String {
    std::env::current_exe().ok()
        .and_then(|p| p.file_name()
                  .and_then(|p| p.to_str())
                  .map(|p| p.to_string()))
        .unwrap_or("main".to_string())
}

fn print_usage() {
    print!(r#"Usage: {program} <command> [<args>]

Commands:
    app        Control application server
    template   Create, read, update and delete template
    help       Display this help

Use `{program} <command> --help` to see help of each command.
"#, program = get_program_name());
}

fn print_command_usage(command: &str, opts: &Options) {
    let brief = format!("Usage: {} {} [<options>]",
                        get_program_name(), command);
    print!("{}", opts.usage(&brief));
}

// Helper to get option and show usage.
fn get_options(opts: &Options, args: &[String])
    -> Option<Matches> {
    match opts.parse(args) {
        Ok(m) => Some(m),
        Err(e) => {
            println!("{}", e.to_string());
            None
        }
    }
}

/**
 * Template actions
 */
fn get_template_action(args: &[String]) -> Option<Action> {
    let mut opts = Options::new();
    opts.optflag("l", "list", "List up all templates");
    opts.optflag("h", "help", "Display this help");
    let m = get_options(&opts, args).or_else(|| {
        print_command_usage(COMMAND_TEMPLATE, &opts);
        None
    })?;
    if m.opt_present("h") {
        print_command_usage(COMMAND_TEMPLATE, &opts);
        return None;
    }
    if m.opt_present("l") {
        return Some(Action::TemplatesList);
    }
    print_command_usage(COMMAND_TEMPLATE, &opts);
    None
}

/**
 * App actions
 */
fn get_app_action(args: &[String]) -> Option<Action> {
    let mut opts = Options::new();
    opts.optflag("h", "--help", "Display this help");
    let m = get_options(&opts, args).or_else(|| {
        print_command_usage(COMMAND_APP, &opts);
        None
    })?;
    if m.opt_present("h") {
        print_command_usage(COMMAND_APP, &opts);
        return None;
    }
    Some(Action::AppStart)
}

fn get_action(args: &[String]) -> Option<Action> {
    if args.len() == 0 {
        print_usage();
        return None;
    }
    match args[0].as_str() {
        COMMAND_TEMPLATE => get_template_action(&args[1..]),
        COMMAND_APP => get_app_action(&args[1..]),
        COMMAND_HELP => {
            print_usage();
            None
        },
        _ => None
    }
}

fn template_list() {
    println!("Not Implemented");
}

fn app_start() {
    server::start();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let action = get_action(&args[1..]);
    match action {
        Some(a) => match a {
            Action::TemplatesList => template_list(),
            Action::AppStart => app_start()
        },
        None => std::process::exit(1)
    }
}
