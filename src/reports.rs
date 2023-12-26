use crate::data_analyzers::*;
use crate::github_data_fetchers::*;
use crate::utils::parse_summary_from_raw_json;
use chrono::{Duration, Utc};
use log;
use webhook_flows::send_response;

pub async fn search_user(github_token: &str, user_login: &str) -> Option<String> {
    get_user_data_by_login(user_login).await.ok()

    // let repos = get_user_repos_gql( user).await;
}

pub async fn weekly_report(
    owner: &str,
    repo: &str,
    user_name: Option<String>,
    token: Option<String>,
) -> String {
    let n_days = 7u16;
    let mut report = Vec::<String>::new();

    let mut _profile_data = String::new();

    match is_valid_owner_repo_integrated( owner, repo).await {
        None => {
            send_response(
                400,
                vec![(String::from("content-type"), String::from("text/plain"))],
                "You've entered invalid owner/repo, or the target is private. Please try again."
                    .as_bytes()
                    .to_vec(),
            );
            std::process::exit(1);
        }
        Some(gm) => {
            _profile_data = format!("About {}/{}: {}", owner, repo, gm.payload);
        }
    }

    let mut commits_count = 0;
    let mut issues_count = 0;

    let mut commits_summaries = String::new();
    'commits_block: {
        match get_commits_in_range(
            owner,
            repo,
            user_name.clone(),
            n_days,
            token.clone(),
        )
        .await
        {
            Some((count, mut commits_vec, weekly_commits_vec)) => {
                let commits_str = commits_vec
                    .iter()
                    .map(|com| com.source_url.to_owned())
                    .collect::<Vec<String>>()
                    .join("\n");

                report.push(format!("found {count} commits:\n{commits_str}"));
                // send_message_to_channel("ik8", "ch_rep", commits_str.to_string()).await;
                let mut is_sparce = false;
                let mut _turbo = false;
                match count {
                    0 => break 'commits_block,
                    1..=2 => is_sparce = true,
                    6.. => _turbo = true,
                    _ => {}
                };
                commits_count = count;
                match process_commits(
                    
                    &mut commits_vec,
                    _turbo,
                    is_sparce,
                    token.clone(),
                )
                .await
                {
                    Some(summary) => {
                        commits_summaries = summary;
                    }
                    None => log::error!("processing commits failed"),
                }

                if is_sparce {
                    let weekly_commits_log = weekly_commits_vec
                        .iter()
                        .map(|com| format!("{}: {}", com.name, com.tag_line))
                        .collect::<Vec<String>>()
                        .join("\n");

                    commits_summaries = format!("Here is the contributor's commits details: {commits_summaries}, here is the log of weekly commits for the entire repository: {weekly_commits_log}");
                }
                // send_message_to_channel("ik8", "ch_rep", commits_summaries.clone()).await;
            }
            None => log::error!("failed to get commits"),
        }
    }
    let mut issues_summaries = String::new();

    'issues_block: {
        match get_issues_in_range(
            owner,
            repo,
            user_name.clone(),
            n_days,
            token.clone(),
        )
        .await
        {
            Some((count, issue_vec)) => {
                let issues_str = issue_vec
                    .iter()
                    .map(|issue| issue.html_url.to_string())
                    .collect::<Vec<String>>()
                    .join("\n");

                report.push(format!("found {count} issues:\n{issues_str}"));
                // send_message_to_channel("ik8", "ch_iss", issues_str.to_string()).await;

                let mut is_sparce = false;
                let mut _turbo = false;

                match count {
                    0 => break 'issues_block,
                    1..=2 => is_sparce = true,
                    4.. => _turbo = true,
                    _ => {}
                };
                issues_count = count;
                match process_issues(
                    
                    issue_vec,
                    user_name.clone(),
                    _turbo,
                    is_sparce,
                    token.clone(),
                )
                .await
                {
                    Some((summary, _, _issues_vec)) => {
                        // send_message_to_channel("ik8", "ch_err", summary.clone()).await;
                        issues_summaries = summary;
                    }
                    None => log::error!("processing issues failed"),
                }
            }
            None => log::error!("failed to get issues"),
        }
    }

    let now = Utc::now();
    let a_week_ago = now - Duration::days(n_days as i64 + 30);
    let n_days_ago_str = a_week_ago.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let discussion_query = match &user_name {
        Some(user_name) => {
            format!("repo:{owner}/{repo} involves: {user_name} updated:>{n_days_ago_str}")
        }
        None => format!("repo:{owner}/{repo} updated:>{n_days_ago_str}"),
    };

    let mut discussion_data = String::new();
    match search_discussions_integrated( &discussion_query, &user_name).await {
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
            // send_message_to_channel("ik8", "ch_dis", summary.clone()).await;
            discussion_data = summary;
        }
        Err(_e) => log::error!("failed to get discussions for {owner}/{repo}: {_e}"),
    }

    let total_input_entry_count = (commits_count + issues_count) as u16;

    if commits_summaries.is_empty() && issues_summaries.is_empty() && discussion_data.is_empty() {
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
        match correlate_commits_issues_discussions(
            Some(&_profile_data),
            Some(&commits_summaries),
            Some(&issues_summaries),
            Some(&discussion_data),
            user_name.as_deref(),
            total_input_entry_count,
        )
        .await
        {
            None => {
                report = vec!["no report generated".to_string()];
            }
            Some(final_summary) => {
                slack_flows::send_message_to_channel(
                    "ik8",
                    "ch_err",
                    format!("{:?}", final_summary),
                )
                .await;

                let clean_summary = parse_summary_from_raw_json(&final_summary);
                report.push(clean_summary);
            }
        }
    }

    report.join("\n")
}

/*
pub async fn new_contributor_report(github_token: &str, owner: &str, repo: &str, user_name: &str) -> Option<String> {
    let mut home_repo_data = get_readme(owner, repo).await.unwrap_or("".to_string());
    match get_community_profile_data(owner, repo).await {
        Some(community_profile_data) => {
            home_repo_data.push_str(&community_profile_data);
        }
        None => {}
    };
    send_message_to_channel("ik8", "ch_home", home_repo_data.clone()).await;
    let user_profile = get_user_data_by_login(user_name)
        .await
        .unwrap_or("".to_string());
    send_message_to_channel("ik8", "ch_pro", user_profile.clone()).await;

    let now = Utc::now();
    let a_week_ago = now - Duration::days(7);
    let a_week_ago_str = a_week_ago.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    // current search result may include issues the user interacted much earlier but updated recently
    // may need to do 2 separate searches: "commenter:juntao updated:>2023-07-30T02:49:06Z"
    let issue_query = format!("involves:{user_name} updated:>{a_week_ago_str}");
    let issues_data = search_issue(&issue_query).await.unwrap_or("".to_string());
    let mut repos_data = String::new();

    for language in vec!["rust", "javascript", "cpp", "go"] {
        let temp = get_user_repos_gql(user_name, language)
            .await
            .unwrap_or("".to_string());
        repos_data.push_str(&temp);
    }
    send_message_to_channel("ik8", "ch_rep", repos_data.clone()).await;

    let discussion_query = format!("involves:{user_name} updated:>{a_week_ago_str}");
    let (_, discussion_vec) = search_discussions(&discussion_query).await.unwrap();

    let (discussion_data, _) = analyze_discussions(discussion_vec, Some(user_name)).await;
    send_message_to_channel("ik8", "ch_dis", discussion_data.clone()).await;

    return correlate_user_and_home_project(
        &home_repo_data,
        &user_profile,
        &issues_data,
        &repos_data,
        &discussion_data,
    )
    .await;
}
pub async fn current_contributor_report(
    owner: &str,
    repo: &str,
    user_name: &str,
) -> Option<String> {
    let now = Utc::now();
    let a_week_ago = now - Duration::days(7);
    let a_week_ago_str = a_week_ago.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    // let issue_query = format!("involves:{user_name} updated:>{a_week_ago_str}");
    // let issues_data = search_issue(&issue_query).await.unwrap_or("".to_string());

    Some("".to_string())
}
pub async fn current_repo_report(github_token: &str, owner: &str, repo: &str) -> Option<String> {
    Some("".to_string())
}
 */
