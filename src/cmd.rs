use std::sync::Arc;

use log::{debug, info, warn};
use octocrab::{models::pulls::PullRequest, Octocrab};

use crate::{
    cli::{Opts, SoloOpts},
    error::GhProdError,
    metrics::{mean_pr_duration, median_pr_duration, Metric, PullRequestTerminatingState},
};

pub const SLEEP_DURATION_MILLIS: u64 = 10;
pub const MAX_NUM_PER_PAGE: u8 = 100;

pub async fn fetch_all_pull_requests(
    owner: &str,
    repo: &str,
    client: Arc<Octocrab>,
) -> Result<Vec<PullRequest>, GhProdError> {
    let mut prs: Vec<Vec<PullRequest>> = vec![];
    let mut num_pages: usize = 0;
    let mut page = client
        .pulls(owner, repo)
        .list()
        .state(octocrab::params::State::All)
        .per_page(MAX_NUM_PER_PAGE)
        .send()
        .await?;

    loop {
        info!("Fetching page {}...", num_pages);
        num_pages += 1;

        match client
            .get_page::<octocrab::models::pulls::PullRequest>(&page.next)
            .await?
        {
            Some(next_page) => {
                if next_page.items.is_empty() {
                    warn!("Received empty page");
                }
                prs.push(next_page.items.clone());
                page = next_page;
            }
            None => break,
        }

        debug!("Sleeping for {} milliseconds...", SLEEP_DURATION_MILLIS);
        tokio::time::sleep(tokio::time::Duration::from_millis(SLEEP_DURATION_MILLIS)).await;
    }

    Ok(prs.iter().flatten().cloned().collect())
}

pub async fn solo(
    opts: SoloOpts,
    global_opts: Opts,
    client: Arc<Octocrab>,
) -> Result<(), GhProdError> {
    let owner: String = global_opts.owner;
    let repo: String = global_opts.repo;
    let user: String = opts.user;
    let prs: Vec<PullRequest> =
        fetch_all_pull_requests(owner.as_str(), repo.as_str(), client).await?;

    let pr_terminal_state: PullRequestTerminatingState =
        if let Some(t) = global_opts.pull_request_terminating_state {
            t
        } else {
            PullRequestTerminatingState::default()
        };

    if let Some(metric) = opts.metric {
        match metric {
            Metric::MeanPrDuration => match mean_pr_duration(user, prs, pr_terminal_state) {
                Some(mean_duration) => println!("{}", mean_duration),
                None => println!("(null)"),
            },
            Metric::MedianPrDuration => match median_pr_duration(user, prs, pr_terminal_state) {
                Some(median_duration) => println!("{}", median_duration),
                None => println!("(null)"),
            },
        }
    } else {
        todo!()
    }

    Ok(())
}
