use std::collections::HashMap;

use crate::data_analyzers::*;
use crate::github_data_fetchers::*;
use crate::utils::parse_summary_from_raw_json;
use chrono::{Duration, Utc};
use log;
use webhook_flows::send_response;

pub async fn weekly_report(
    owner: &str,
    repo: &str,
    user_name: Option<String>,
    token: Option<String>,
) -> String {
    let n_days = 7u16;
    let mut report = Vec::<String>::new();

    let mut _profile_data = String::new();

    match is_valid_owner_repo_integrated(owner, repo).await {
        Err(_e) => {
            send_response(
                400,
                vec![(String::from("content-type"), String::from("text/plain"))],
                "You've entered invalid owner/repo, or the target is private. Please try again."
                    .as_bytes()
                    .to_vec(),
            );
            std::process::exit(1);
        }
        Ok(gm) => {
            _profile_data = format!("About {}/{}: {}", owner, repo, gm.payload);
        }
    }

    let mut commits_count = 0;
    let mut issues_count = 0;

    let mut commits_map = HashMap::<String, String>::new();
    'commits_block: {
        match get_commits_in_range_search(owner, repo, user_name.clone(), n_days, token.clone())
            .await
        {
            Some((count, commits_vec)) => {
                let commits_str = commits_vec
                    .iter()
                    .map(|com| com.source_url.to_owned())
                    .collect::<Vec<String>>()
                    .join("\n");

                report.push(format!("found {count} commits:\n{commits_str}"));
                match count {
                    0 => break 'commits_block,
                    _ => {}
                };
                commits_count = count;
                let _ = process_commits(commits_vec, &mut commits_map, token.clone()).await;
            }
            None => log::error!("failed to get commits"),
        }
    }

    let mut issues_map = HashMap::<String, String>::new();

    'issues_block: {
        match get_issues_in_range(owner, repo, user_name.clone(), n_days, token.clone()).await {
            Some((count, issue_vec)) => {
                let issues_str = issue_vec
                    .iter()
                    .map(|issue| issue.html_url.to_string())
                    .collect::<Vec<String>>()
                    .join("\n");

                report.push(format!("found {count} issues:\n{issues_str}"));

                match count {
                    0 => break 'issues_block,
                    _ => {}
                };
                issues_count = count;
                let _ =
                    process_issues(issue_vec, user_name.clone(), &mut issues_map, token.clone())
                        .await;
            }
            None => log::error!("failed to get issues"),
        }
    }

    let now = Utc::now();
    let a_week_ago = now - Duration::days(n_days as i64 + 30);
    let n_days_ago_str = a_week_ago.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let discussion_query = match &user_name {
        Some(user_name) => {
            format!("repo:{owner}/{repo} involves:{user_name} updated:>{n_days_ago_str}")
        }
        None => format!("repo:{owner}/{repo} updated:>{n_days_ago_str}"),
    };

    let mut discussion_data = String::new();
    match search_discussions_integrated(&discussion_query, &user_name).await {
        Ok((summary, discussion_vec)) => {
            let count = discussion_vec.len();
            let discussions_str = discussion_vec
                .iter()
                .map(|discussion| discussion.source_url.to_owned())
                .collect::<Vec<String>>()
                .join("\n");

            report.push(format!(
                "{count} discussions were referenced in analysis:\n {discussions_str}"
            ));

            discussion_data = summary;
        }
        Err(_e) => log::error!("No discussions involving user found at {owner}/{repo}: {_e}"),
    }

    let total_input_entry_count = (commits_count + issues_count) as u16;

    if commits_map.len() == 0 && issues_map.len() == 0 && discussion_data.is_empty() {
        match &user_name {
            Some(target_person) => {
                report = vec![format!(
                    "No useful data found for {}, you may try alternative means to find out more about {}",
                    target_person, target_person
                )];
            }

            None => {
                report = vec!["No useful data found, nothing to report".to_string()];
            }
        }
    } else {
        if commits_map.len() == 0 && issues_map.len() > 0 {
            report = vec!["No useful data found, nothing to report".to_string()];

            todo!("implement issues-only report generation");
        }
        for (user_name, commits_summaries) in commits_map {
            let issues_summaries = match issues_map.get(&user_name) {
                Some(issues_summaries) => issues_summaries.to_owned(),
                None => "".to_string(),
            };

            match correlate_commits_issues_discussions(
                Some(&_profile_data),
                Some(&commits_summaries),
                Some(&issues_summaries),
                Some(&discussion_data),
                Some(user_name.as_str()),
                total_input_entry_count,
            )
            .await
            {
                None => {
                    // report = vec!["no report generated".to_string()];
                }
                Some(final_summary) => {
                    if let Ok(clean_summary) = parse_summary_from_raw_json(&final_summary) {
                        report.push(clean_summary);
                    }
                }
            }
        }
    }

    report.join("\n")
}
