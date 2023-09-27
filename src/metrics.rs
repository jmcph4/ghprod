//! Implementations of the metrics used for reporting
use std::str::FromStr;

use chrono::{DateTime, Duration, Utc};
use octocrab::models::pulls::PullRequest;

use crate::api::pull_request_net_change;

/// Represents a particular statistic
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Metric {
    MeanPrDuration,
    MedianPrDuration,
    MeanNetChange,
    TotalPullRequests,
}

impl FromStr for Metric {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mean_pr_duration" => Ok(Self::MeanPrDuration),
            "median_pr_duration" => Ok(Self::MedianPrDuration),
            "mean_net_change" => Ok(Self::MeanNetChange),
            "total_num_prs" => Ok(Self::TotalPullRequests),
            _ => Err("Unknown metric"),
        }
    }
}

/// Represents the state a PR is in iff. it is completed
///
/// Some projects have workflows such that a PR that successfully completes and
/// ends up being integrated into the codebase may never be merged -- but
/// rather closed. One example is the use of the Bors merge bot.
#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub enum PullRequestTerminatingState {
    Merged,
    Closed,
}

impl Default for PullRequestTerminatingState {
    fn default() -> Self {
        Self::Merged
    }
}

impl FromStr for PullRequestTerminatingState {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "merged" | "Merged" | "MERGED" | "m" | "M" | "merge" | "Merge" | "MERGE" => {
                Ok(Self::Merged)
            }
            "closed" | "Closed" | "CLOSED" | "c" | "C" | "close" | "Close" | "CLOSE" => {
                Ok(Self::Merged)
            }
            _ => Err("Unknown terminating state"),
        }
    }
}

/// Determines whether `pull_request` has terminated (based on `terminal_state`)
pub fn pull_request_terminated(
    pull_request: &PullRequest,
    terminal_state: PullRequestTerminatingState,
) -> bool {
    match terminal_state {
        PullRequestTerminatingState::Closed => pull_request.closed_at.is_some(),
        PullRequestTerminatingState::Merged => pull_request.merged_at.is_some(),
    }
}

/// Returns the length of time this PR took from its creation until its termination.
///
/// "Termination" here is defined by `pull_request_terminated`.
pub fn pull_request_duration(
    pull_request: PullRequest,
    terminal_state: PullRequestTerminatingState,
) -> Duration {
    let start_time: DateTime<Utc> = pull_request.created_at.unwrap();
    let end_time: DateTime<Utc> = if pull_request_terminated(&pull_request, terminal_state) {
        match terminal_state {
            PullRequestTerminatingState::Merged => pull_request.merged_at.unwrap(),
            PullRequestTerminatingState::Closed => pull_request.closed_at.unwrap(),
        }
    } else {
        Utc::now()
    };

    end_time - start_time
}

/// Returns the subset of PRs that are authored by `author`
pub fn pull_requests_by_author(author: &str, pull_requests: &Vec<PullRequest>) -> Vec<PullRequest> {
    pull_requests
        .iter()
        .filter(|pr| pr.user.as_ref().is_some_and(|user| user.login == author))
        .cloned()
        .collect()
}

/// Returns the subset of PRs that have terminated
pub fn terminated_pull_requests(
    pull_requests: &Vec<PullRequest>,
    terminal_state: PullRequestTerminatingState,
) -> Vec<PullRequest> {
    pull_requests
        .iter()
        .filter(|pr| pull_request_terminated(pr, terminal_state))
        .cloned()
        .collect()
}

pub const SECONDS_PER_DAY: u64 = 60 * 60 * 24;

/// Returns the mean number of days a PR takes to terminate.
///
/// Note that non-terminated PRs (i.e., PRs that are still open) are ignored as
/// part of this calculation.
pub fn mean_pr_duration(
    user: &str,
    pull_requests: &Vec<PullRequest>,
    terminal_state: PullRequestTerminatingState,
) -> Option<f64> {
    let users_prs: Vec<PullRequest> = pull_requests_by_author(user, &pull_requests);

    if users_prs.is_empty() {
        None
    } else {
        Some(
            users_prs
                .iter()
                .cloned()
                .map(|pr| pull_request_duration(pr, terminal_state))
                .map(|timedelta| timedelta.num_seconds())
                .map(|secs| secs as f64 / SECONDS_PER_DAY as f64)
                .sum::<f64>()
                / users_prs.len() as f64,
        )
    }
}

/// Returns the median number of days a PR takes to terminate.
///
/// Note that non-terminated PRs (i.e., PRs that are still open) are ignored as
/// part of this calculation.
pub fn median_pr_duration(
    user: &str,
    pull_requests: &Vec<PullRequest>,
    terminal_state: PullRequestTerminatingState,
) -> Option<f64> {
    let durations: Vec<f64> = pull_requests_by_author(user, &pull_requests)
        .iter()
        .cloned()
        .map(|pr| pull_request_duration(pr, terminal_state))
        .map(|timedelta| timedelta.num_seconds())
        .map(|secs| secs as f64 / SECONDS_PER_DAY as f64)
        .collect();
    let n: usize = durations.len();

    match n {
        0 => None,
        1 => Some(durations[0]),
        _ => Some(if n % 2 == 0 {
            (durations[n / 2] + durations[(n / 2) + 1]) / 2.0
        } else {
            durations[(n + 1) / 2]
        }),
    }
}

/// Returns the mean net change of the subset of `pull_requests` that are authored by `user`
#[allow(dead_code)]
pub fn mean_net_change(
    user: &str,
    pull_requests: &Vec<PullRequest>,
    terminal_state: PullRequestTerminatingState,
) -> Option<f64> {
    let users_prs: Vec<PullRequest> = pull_requests_by_author(user, &pull_requests);

    if users_prs.is_empty() {
        None
    } else {
        Some(
            users_prs
                .iter()
                .cloned()
                .filter(|pr| pull_request_terminated(pr, terminal_state))
                .filter_map(|pr| pull_request_net_change(&pr))
                .map(|net_change| net_change as f64)
                .sum::<f64>()
                / users_prs.len() as f64,
        )
    }
}

/// Returns the number of PRs in `pull_requests` authored by `user`
pub fn total_pull_requests(user: &str, pull_requests: &Vec<PullRequest>) -> usize {
    pull_requests_by_author(user, pull_requests).len()
}
