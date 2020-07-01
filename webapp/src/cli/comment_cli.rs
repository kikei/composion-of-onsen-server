use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use structopt::StructOpt;

use crate::models::comment_photos::{DIRECTORY_ORDER, ConvertOrder};

#[derive(StructOpt, Debug)]
pub enum Action {
    /// Process uploaded comment images
    ProcessImage(ProcessImageArgs)
}

#[derive(StructOpt, Debug)]
pub struct ProcessImageArgs {
    /// How many images to convert
    #[structopt(short, long)]
    pub number: Option<u32>
}

fn get_order_file() -> Option<PathBuf> {
    let path = Path::new(DIRECTORY_ORDER);
    for entry in path.read_dir().expect("Failed to read order directory") {
        if let Ok(e) = entry {
            return Some(e.path())
        }
    }
    None
}

fn read_order_file(path: &Path) -> Result<ConvertOrder, String> {
    let file = File::open(path)
        .map_err(|e| format!("Cannot open file: {}, e: {}",
                             &path.display(), &e))?;
    let reader = BufReader::new(file);

    let order = serde_json::from_reader(reader)
        .map_err(|e| format!("Cannot parse order file: {}, e: {}",
                             &path.display(), &e))?;
    Ok(order)
}

fn process_image(args: &ProcessImageArgs) {
    let mut count = 0;
    while let Some(path) = get_order_file() {
        if args.number.filter(|n| &count >= n).is_some() {
            break;
        }
        debug!("Found order file: {}", &path.display());
        let order = read_order_file(&path);
        if let Err(e) = &order {
            error!("Failed to read order file: {}, e: {}", &path.display(), &e);
            continue;
        }
        debug!("Processing order: {:?}", &order);
        match order.unwrap().process() {
            Ok(p) => info!("Successfully processed order, photo: {:?}", &p),
            Err(e) => error!("Failed to process order, e: {}", &e)
        }
        match std::fs::remove_file(&path) {
            Ok(()) => info!("Deleted order file: {}", &path.display()),
            Err(e) => error!("Couldn't delete order file: {}, e: {}",
                             &path.display(), &e)
        }
        count += 1;
    }
    debug!("{} orders processed", &count);
}

pub fn run(args: &Action) {
    // TODO Use setup_logger
    env_logger::init();
    info!("Log initialized.");

    match args {
        Action::ProcessImage(args) => process_image(&args)
    }

}
