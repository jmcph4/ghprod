use std::str::FromStr;

use chrono::{DateTime, Duration, Utc};
use octocrab::models::pulls::PullRequest;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Metric {
    MeanPrDuration,
    MedianPrDuration,
    MeanNetChange,
}

impl FromStr for Metric {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mean_pr_duration" => Ok(Self::MeanPrDuration),
            "median_pr_duration" => Ok(Self::MedianPrDuration),
            "mean_net_change" => Ok(Self::MeanNetChange),
            _ => Err("Unknown metric"),
        }
    }
}

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

pub fn pull_request_terminated(
    pull_request: &PullRequest,
    terminal_state: PullRequestTerminatingState,
) -> bool {
    match terminal_state {
        PullRequestTerminatingState::Closed => pull_request.closed_at.is_some(),
        PullRequestTerminatingState::Merged => pull_request.merged_at.is_some(),
    }
}

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

pub fn pull_requests_by_author(
    author: String,
    pull_requests: Vec<PullRequest>,
) -> Vec<PullRequest> {
    pull_requests
        .iter()
        .filter(|pr| pr.user.as_ref().is_some_and(|user| user.login == author))
        .cloned()
        .collect()
}

pub const SECONDS_PER_DAY: u64 = 60 * 60 * 24;

pub fn mean_pr_duration(
    user: String,
    pull_requests: Vec<PullRequest>,
    terminal_state: PullRequestTerminatingState,
) -> Option<f64> {
    let users_prs: Vec<PullRequest> = pull_requests_by_author(user, pull_requests);

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

pub fn median_pr_duration(
    user: String,
    pull_requests: Vec<PullRequest>,
    terminal_state: PullRequestTerminatingState,
) -> Option<f64> {
    let durations: Vec<f64> = pull_requests_by_author(user, pull_requests)
        .iter()
        .cloned()
        .map(|pr| pull_request_duration(pr, terminal_state))
        .map(|timedelta| timedelta.num_seconds())
        .map(|secs| secs as f64 / SECONDS_PER_DAY as f64)
        .collect();
    let n: usize = durations.len();

    if n == 0 {
        None
    } else {
        Some(if n % 2 == 0 {
            (durations[n / 2] + durations[(n / 2) + 1]) / 2.0
        } else {
            durations[(n + 1) / 2]
        })
    }
}

#[allow(dead_code)]
pub fn pull_request_net_change(pull_request: &PullRequest) -> Option<i64> {
    match (pull_request.additions, pull_request.deletions) {
        (Some(a), Some(d)) => Some(a as i64 - d as i64),
        (Some(a), None) => Some(a as i64),
        (None, Some(d)) => Some(0 - d as i64),
        _ => None,
    }
}

#[allow(dead_code)]
pub fn mean_net_change(
    user: String,
    pull_requests: Vec<PullRequest>,
    terminal_state: PullRequestTerminatingState,
) -> Option<f64> {
    let users_prs: Vec<PullRequest> = pull_requests_by_author(user, pull_requests);
    dbg!(&users_prs);

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
