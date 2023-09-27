//! User-facing command implementations
use std::sync::Arc;

use octocrab::{models::pulls::PullRequest, Octocrab};

use crate::{
    api::fetch_all_pull_requests,
    cli::{Opts, SoloOpts},
    error::GhProdError,
    metrics::{
        mean_net_change, mean_pr_duration, median_pr_duration, pull_requests_by_author,
        terminated_pull_requests, total_pull_requests, Metric, PullRequestTerminatingState,
    },
};

pub fn user_summary(
    owner: &str,
    repo: &str,
    user: &str,
    pull_requests: &Vec<PullRequest>,
    terminal_state: PullRequestTerminatingState,
) -> String {
    let mut report: String = String::new();

    report += format!("=== {}'s contributions to {}/{} ===\n", user, owner, repo).as_str();

    let num_prs: usize = total_pull_requests(user, pull_requests);
    let done_prs: usize = terminated_pull_requests(
        &pull_requests_by_author(user, pull_requests),
        terminal_state,
    )
    .len();

    if num_prs > 0 {
        if num_prs == done_prs {
            report += format!(
                "{} has {} PRs in total (all of which are completed)\n",
                user, num_prs
            )
            .as_str();
        } else {
            report += format!(
                "{} has {} PRs in total ({} of these are completed)\n",
                &user, num_prs, done_prs,
            )
            .as_str();
        }

        if let Some(mean) = mean_pr_duration(user, pull_requests, terminal_state) {
            report +=
                format!("{}'s PRs take {} days to complete on average\n", user, mean).as_str();
        }

        if let Some(net_change) = mean_net_change(user, pull_requests, terminal_state) {
            if net_change.is_sign_negative() {
                report += format!(
                    "{} reduces the size of the codebase by {} lines on average",
                    user, net_change
                )
                .as_str()
            } else if net_change.is_sign_positive() {
                report += format!(
                    "{} increases the size of the codebase by {} lines on average",
                    user, net_change
                )
                .as_str()
            } else {
                report += format!(
                    "{} doesn't change the size of the codebase on average!",
                    user
                )
                .as_str()
            };
        }
    } else {
        report += "There's not much here...\n";
    }

    report
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
            Metric::MeanPrDuration => {
                match mean_pr_duration(user.as_str(), &prs, pr_terminal_state) {
                    Some(mean_duration) => println!("{}", mean_duration),
                    None => println!("(null)"),
                }
            }
            Metric::MedianPrDuration => {
                match median_pr_duration(user.as_str(), &prs, pr_terminal_state) {
                    Some(median_duration) => println!("{}", median_duration),
                    None => println!("(null)"),
                }
            }
            Metric::MeanNetChange => unimplemented!(),
            Metric::TotalPullRequests => println!("{}", total_pull_requests(user.as_str(), &prs)),
        }
    } else {
        println!(
            "{}",
            user_summary(
                owner.as_str(),
                repo.as_str(),
                user.as_str(),
                &prs,
                pr_terminal_state
            )
        );
    }

    Ok(())
}
