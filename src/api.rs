//! Logic for interacting with the GitHub API
//!
//! Whilst the actual API interaction is delegated to our Octocrab dependency,
//! this module contains both helpers but also queries we'll actually be
//! performing.
//!
//! `ghprod` uses a model that frontloads the actual data retrieval from the
//! GitHub API in an effort to minimise requests. As a result, all queries we
//! perform act on already-retrieved data.
use std::sync::Arc;

use log::{debug, info, warn};
use octocrab::{models::pulls::PullRequest, Octocrab};

use crate::error::GhProdError;

pub const MILLISECONDS_PER_SECOND: u64 = 1000;
pub const SECONDS_PER_MINUTE: u64 = 60;

/// Number of milliseconds to sleep for after each page is requested from the GitHub API.
///
/// The current rate limits for *unauthenticated* requests are 60 requests per
/// hour (obviously, this is one request per minute on average). Source:
/// [https://docs.github.com/en/rest/overview/resources-in-the-rest-api?apiVersion=2022-11-28#rate-limits-for-requests-from-personal-accounts](https://docs.github.com/en/rest/overview/resources-in-the-rest-api?apiVersion=2022-11-28#rate-limits-for-requests-from-personal-accounts)
pub const SLEEP_DURATION_MILLIS: u64 = MILLISECONDS_PER_SECOND * SECONDS_PER_MINUTE;

/// Maximum number of items to receive from the GitHub API per page.
///
/// The current maximum is 100.
pub const MAX_NUM_PER_PAGE: u8 = 100;

/// Returns all pull requests for the given repository
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
    prs.push(page.items.clone());

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

    info!("Retrieved {} PRs", prs.iter().flatten().count());

    Ok(prs.iter().flatten().cloned().collect())
}

/// Returns the net change of `pull_request`.
///
/// The net change of a PR is defined as the number of lines added subtract the
/// number of lines removed.
#[allow(dead_code)]
pub fn pull_request_net_change(pull_request: &PullRequest) -> Option<i64> {
    match (pull_request.additions, pull_request.deletions) {
        (Some(a), Some(d)) => Some(a as i64 - d as i64),
        (Some(a), None) => Some(a as i64),
        (None, Some(d)) => Some(0 - d as i64),
        _ => None,
    }
}
