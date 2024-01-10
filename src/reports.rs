use std::collections::HashMap;
use crate::data_analyzers::*;
use crate::github_data_fetchers::*;
use crate::utils::parse_summary_from_raw_json;
use log;
// use store_flows::{del, get, set, Expire};
use webhook_flows::send_response;

pub async fn weekly_report(
    owner: &str,
    repo: &str,
    user_name: Option<String>,
    token: Option<String>
) -> String {
    let n_days = 7u16;

    let contributors_set;

    match is_valid_owner_repo(owner, repo).await {
        Err(_e) => {
            send_response(
                400,
                vec![(String::from("content-type"), String::from("text/plain"))],
                "You've entered invalid owner/repo, or the target is private. Please try again."
                    .as_bytes()
                    .to_vec()
            );
            std::process::exit(1);
        }
        Ok((_, _, inner_set)) => {
            contributors_set = inner_set;
        }
    }

    let mut commits_map = HashMap::<String, (String, String)>::new();
    'commits_block: {
        match
            get_commits_in_range_search(owner, repo, user_name.clone(), n_days, token.clone()).await
        {
            Some((count, commits_vec)) => {
                match count {
                    0 => {
                        break 'commits_block;
                    }
                    _ => {}
                }
                let _ = process_commits(commits_vec, &mut commits_map, token.clone()).await;
            }
            None => log::error!("failed to get commits"),
        }
    }

    let mut issues_map = HashMap::<String, (String, String)>::new();

    'issues_block: {
        match get_issues_in_range(owner, repo, user_name.clone(), n_days, token.clone()).await {
            Some((count, issue_vec)) => {
                match count {
                    0 => {
                        break 'issues_block;
                    }
                    _ => {}
                }
                issues_map = match
                    process_issues(
                        issue_vec,
                        user_name.clone(),
                        contributors_set,
                        token.clone()
                    ).await
                {
                    Ok(map) => map,
                    Err(_e) => HashMap::<String, (String, String)>::new(),
                };
            }
            None => log::error!("failed to get issues"),
        }
    }

    let mut report = Vec::<String>::new();

    if commits_map.len() == 0 && issues_map.len() == 0 {
        match &user_name {
            Some(target_person) => {
                report = vec![
                    format!(
                        "No useful data found for {}, you may try alternative means to find out more about {}",
                        target_person,
                        target_person
                    )
                ];
            }

            None => {
                report = vec!["No useful data found, nothing to report".to_string()];
            }
        }
    } else {
        for (user_name, (commits_str, commits_summaries)) in commits_map {
            let mut one_user_report = Vec::<String>::new();

            let commits_count = commits_str.lines().count();

            one_user_report.push(
                format!("{user_name} made {commits_count} commits:\n{commits_str}")
            );
            // log::info!("found {commits_count} commits:\n{commits_str}");

            let issues_summaries = match issues_map.get(&user_name) {
                Some(tup) => {
                    let issues_str = tup.0.to_owned();
                    let issues_count = issues_str.lines().count();
                    one_user_report.push(
                        format!("{user_name} participated in {issues_count} issues:\n{issues_str}")
                    );

                    tup.1.to_owned()
                }
                None => "".to_string(),
            };
            match
                correlate_commits_issues_sparse(
                    &commits_summaries,
                    &issues_summaries,
                    &user_name
                ).await
            {
                None => {
                    log::error!("Error generating report for user: {}", &user_name);
                    log::info!("commits_summaries: {commits_summaries:?}");
                    log::info!("issue_summaries: {:?}", &issues_summaries);
                }
                Some(final_summary) => {
                    match parse_summary_from_raw_json(&final_summary) {
                        Ok(clean_summary) => {
                            one_user_report.push(clean_summary);
                        }
                        Err(_e) => {
                            log::error!(
                                "Failed to parse summary for user: {}, summary: {:?}, {:?}",
                                &user_name,
                                &final_summary,
                                _e
                            );

                            continue;
                        }
                    }
                }
            }
            report.push(one_user_report.join("\n"));
        }
    }

    report.join("\n")
}
