use clap::{Args, Parser, Subcommand};

use crate::metrics::{Metric, PullRequestTerminatingState};

#[derive(Clone, Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Opts {
    pub owner: String,
    pub repo: String,
    #[clap(short, long)]
    pub api_secret: Option<String>,

    #[clap(short, long)]
    pub pull_request_terminating_state: Option<PullRequestTerminatingState>,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Commands {
    Solo(SoloOpts),
}

#[derive(Args, Clone, Debug)]
pub struct SoloOpts {
    pub user: String,
    pub metric: Option<Metric>,
}
