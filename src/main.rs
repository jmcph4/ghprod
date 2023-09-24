use clap::Parser;
use log::info;

use crate::{cli::Opts, error::GhProdError};

mod cli;
mod error;

#[tokio::main]
async fn main() -> Result<(), GhProdError> {
    pretty_env_logger::init();

    let _opts: Opts = Opts::parse();

    info!("Initialised!");

    Ok(())
}
