use std::sync::Arc;

use clap::Parser;
use log::info;
use octocrab::{Octocrab, OctocrabBuilder};

use crate::{
    cli::{Commands, Opts},
    error::GhProdError,
};

mod api;
mod cli;
mod cmd;
mod error;
mod metrics;

#[tokio::main]
async fn main() -> Result<(), GhProdError> {
    pretty_env_logger::init();

    let opts: Opts = Opts::parse();

    let client: Arc<Octocrab> = if let Some(tok) = opts.clone().api_secret {
        Arc::new(OctocrabBuilder::default().personal_token(tok).build()?)
    } else {
        octocrab::instance()
    };

    match opts.clone().command {
        Commands::Solo(solo_opts) => cmd::solo(solo_opts, opts, client).await?,
    };

    info!("Initialised!");

    Ok(())
}
