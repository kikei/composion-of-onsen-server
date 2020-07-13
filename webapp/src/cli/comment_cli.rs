use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use structopt::StructOpt;

use crate::models::comment_photos::{DIRECTORY_ORDER, ConvertOrder};

const EXTENSION_LOCK: &str = "lock";

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

struct OrderFile {
    path: Option<PathBuf>
}

impl OrderFile {
    pub fn new(path: &Path) -> Self {
        OrderFile { path: Some(path.to_path_buf()) }
    }

    pub fn lock(&mut self) -> Result<(), String> {
        match &self.path {
            None => Err("OrderFile may be deleted".to_string()),
            Some(path) => {
                let lockpath = path.with_extension(EXTENSION_LOCK);
                std::fs::rename(path, &lockpath)
                    .map_err(|e| format!("Cannot lock file: {} -> {}, e: {}",
                                         &path.display(), &lockpath.display(), &e))?;
                self.path = Some(lockpath.to_path_buf());
                Ok(())
            }
        }
    }

    pub fn read(&self) -> Result<ConvertOrder, String> {
        match &self.path {
            None => Err("OrderFile may be deleted".to_string()),
            Some(path) => {
                let file = File::open(path)
                    .map_err(|e| format!("Cannot open file: {}, e: {}",
                                         &path.display(), &e))?;
                let reader = BufReader::new(file);

                let order = serde_json::from_reader(reader)
                    .map_err(|e| format!("Cannot parse order file: {}, e: {}",
                                         &path.display(), &e))?;
                Ok(order)
            }
        }
    }

    pub fn delete(&mut self) -> Result<(), String> {
        match &self.path {
            None => Ok(()),
            Some(path) => {
                std::fs::remove_file(&path)
                    .map_err(|e|
                             format!("Couldn't delete order file: {}, e: {}",
                                     &path.display(), &e))?;
                self.path = None;
                Ok(())
            }
        }
    }
}

impl fmt::Display for OrderFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.path {
            Some(path) => write!(f, "OrderFile({})", path.display()),
            None => write!(f, "OrderFile(deleted)")
        }
    }
}

fn get_order_files() -> impl Iterator<Item = OrderFile> {
    let path = Path::new(DIRECTORY_ORDER);
    path.read_dir()
        .expect("Failed to read order directory")
        .filter_map(|e| {
            let p = e.expect("Failed to read order directory entry").path();
            if p.ends_with(EXTENSION_LOCK) {
                None
            } else {
                Some(OrderFile::new(&p))
            }
        })
}

fn process_image(args: &ProcessImageArgs) {
    let mut count = 0;
    for mut order_file in get_order_files() {
        if args.number.filter(|n| &count >= n).is_some() {
            break;
        }
        debug!("Found order file: {}", &order_file);
        if let Err(e) = &order_file.lock() {
            error!("Failed to lock order file: {}, e: {}", &order_file, &e);
            continue;
        }
        debug!("Locked order file: {}", &order_file);
        let order = order_file.read();
        if let Err(e) = &order {
            error!("Failed to read order file: {}, e: {}", &order_file, &e);
            continue;
        }
        debug!("Processing order: {:?}", &order);
        match order.unwrap().process() {
            Ok(p) => info!("Successfully processed order, photo: {:?}", &p),
            Err(e) => error!("Failed to process order, e: {}", &e)
        }
        match order_file.delete() {
            Ok(()) => info!("Deleted order file"),
            Err(e) => error!("{}", &e)
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
