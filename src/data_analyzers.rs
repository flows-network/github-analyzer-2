use crate::github_data_fetchers::*;
use crate::utils::*;
use chrono::{DateTime, Utc};
use github_flows::{
    get_octo,
    octocrab::models::{issues::Comment, issues::Issue},
    GithubLogin,
};
use http::header::{HeaderMap, HeaderValue, CONNECTION};
use log;
use serde::Deserialize;

pub async fn get_repo_info(about_repo: &str) -> Option<String> {
    #[derive(Deserialize)]
    struct CommunityProfile {
        health_percentage: u16,
        description: Option<String>,
        readme: Option<String>,
        updated_at: Option<DateTime<Utc>>,
    }

    let community_profile_url = format!("repos/{}/community/profile", about_repo);

    let mut description = String::new();
    let mut date = Utc::now().date_naive();
    let octocrab = get_octo(&GithubLogin::Default);

    match octocrab
        .get::<CommunityProfile, _, ()>(&community_profile_url, None::<&()>)
        .await
    {
        Ok(profile) => {
            description = profile
                .description
                .as_ref()
                .unwrap_or(&String::from(""))
                .to_string();
            date = profile
                .updated_at
                .as_ref()
                .unwrap_or(&Utc::now())
                .date_naive();
        }
        Err(e) => log::error!("Error parsing Community Profile: {:?}", e),
    }

    let mut payload = String::new();
    match get_readme_owner_repo(about_repo).await {
        Some(content) => {
            let content = content.chars().take(20000).collect::<String>();
            match analyze_readme(&content).await {
                Some(summary) => payload = summary,
                None => log::error!("Error parsing README.md: {}", about_repo),
            }
        }
        None => log::error!("Error fetching README.md: {}", about_repo),
    };
    if description.is_empty() && payload.is_empty() {
        return None;
    }

    if payload.is_empty() {
        return Some(description);
    } else {
        return Some(payload);
    }
}
pub async fn get_repo_overview_by_scraper(about_repo: &str) -> Option<String> {
    let repo_home_url = format!("https://github.com/{}", about_repo);

    let mut raw_text = String::new();
    match web_scraper_flows::get_page_text(&repo_home_url).await {
        Ok(page_text) => raw_text = page_text,
        Err(_e) => {
            log::error!("Error getting response from Github: {:?}", _e);

            return None;
        }
    }

    let raw_text = if raw_text.len() > 48_000 {
        squeeze_fit_post_texts(&raw_text, 12_000, 0.7)
    } else {
        raw_text.to_string()
    };

    let sys_prompt = "Your task is to examine the textual content from a GitHub repo page, emphasizing the Header, About, Release, Contributors, Languages, and README sections. This process should be carried out objectively, focusing on factual information extraction from each segment. Avoid making subjective judgments or inferences. The data should be presented systematically, corresponding to each section. Please note, the provided text will be in a flattened format.";

    let usr_prompt = &format!("I’ve obtained a flattened text from a GitHub repo page and require analysis of the following sections: 1) Header, with data on Fork, Star, Issues, Pull Request, etc.; 2) About, containing project description, keywords, number of stars, watchers, and forks; 3) Release, with details on the latest release and total releases; 4) Contributors, showing the number of contributors; 5) Languages, displaying the language composition in the project, and 6) README, which is usually a body of text describing the project, please summarize README when presenting result. Please extract and present data from these sections individually. Here is the text: {}", raw_text);

    match chat_inner(sys_prompt, usr_prompt, 700, "gpt-3.5-turbo-1106").await {
        Ok(r) => return Some(r),
        Err(_e) => {
            log::error!("Error summarizing meta data: {}", _e);
            return None;
        }
    }
}

pub async fn is_valid_owner_repo_integrated(owner: &str, repo: &str) -> anyhow::Result<GitMemory> {
    #[derive(Deserialize)]
    struct CommunityProfile {
        health_percentage: u16,
        description: Option<String>,
        files: FileDetails,
        updated_at: Option<DateTime<Utc>>,
    }
    #[derive(Debug, Deserialize)]
    pub struct FileDetails {
        readme: Option<Readme>,
    }
    #[derive(Debug, Deserialize)]
    pub struct Readme {
        url: Option<String>,
    }
    let community_profile_url = format!("repos/{}/{}/community/profile", owner, repo);

    let mut description = String::new();
    let mut date = Utc::now().date_naive();
    let mut has_readme = false;
    let octocrab = get_octo(&GithubLogin::Default);

    match octocrab
        .get::<CommunityProfile, _, ()>(&community_profile_url, None::<&()>)
        .await
    {
        Ok(profile) => {
            description = profile
                .description
                .as_ref()
                .unwrap_or(&String::from(""))
                .to_string();
            date = profile
                .updated_at
                .as_ref()
                .unwrap_or(&Utc::now())
                .date_naive();
            has_readme = profile
                .files
                .readme
                .as_ref()
                .unwrap_or(&Readme { url: None })
                .url
                .is_some();
        }
        Err(e) => {
            log::error!("Error parsing Community Profile: {:?}", e);
            return Err(anyhow::anyhow!(
                "no Community Profile, so invalid owner/repo: {:?}",
                e
            ));
        }
    }

    let mut payload = String::new();

    if has_readme {
        if let Some(content) = get_readme(owner, repo).await {
            let content = content.chars().take(20000).collect::<String>();
            match analyze_readme(&content).await {
                Some(summary) => payload = summary,
                None => log::error!("Error parsing README.md: {}/{}", owner, repo),
            }
        }
    }

    if description.is_empty() {
        description = payload.clone();
    } else if payload.is_empty() {
        payload = description.clone();
    }

    Ok(GitMemory {
        memory_type: MemoryType::Meta,
        name: format!("{}/{}", owner, repo),
        tag_line: description,
        source_url: community_profile_url,
        payload: payload,
        date: date,
    })
}

pub async fn process_issues(
    inp_vec: Vec<Issue>,
    target_person: Option<String>,
    _turbo: bool,
    is_sparce: bool,
    token: Option<String>,
) -> Option<(String, usize, Vec<GitMemory>)> {
    let mut issues_summaries = String::new();
    let mut git_memory_vec = vec![];

    for issue in &inp_vec {
        match analyze_issue_integrated(
            issue,
            target_person.clone(),
            _turbo,
            is_sparce,
            token.clone(),
        )
        .await
        {
            None => {
                log::error!("Error analyzing issue: {:?}", issue.url.to_string());
                continue;
            }
            Some((summary, gm)) => {
                issues_summaries.push_str(&format!("{} {}\n", gm.date, summary));
                // slack_flows::send_message_to_channel("ik8", "ch_iss", gm.source_url.to_string())
                //     .await;

                git_memory_vec.push(gm);
                if git_memory_vec.len() > 20 {
                    break;
                }
            }
        }
    }

    let count = git_memory_vec.len();
    if count == 0 {
        log::error!("No issues processed");
        return None;
    }
    Some((issues_summaries, count, git_memory_vec))
}
pub async fn analyze_readme(content: &str) -> Option<String> {
    let sys_prompt_1 = &format!(
        "Your task is to objectively analyze a GitHub profile and the README of their project. Focus on extracting factual information about the features of the project, and its stated objectives. Avoid making judgments or inferring subjective value."
    );

    let content = if content.len() > 48_000 {
        squeeze_fit_remove_quoted(&content, 9_000, 0.7)
    } else {
        content.to_string()
    };
    let usr_prompt_1 = &format!(
        "Based on the profile and README provided: {content}, extract a concise summary detailing this project's factual significance in its domain, their areas of expertise, and the main features and goals of the project. Ensure the insights are objective and under 110 tokens."
    );

    match chat_inner(sys_prompt_1, usr_prompt_1, 256, "gpt-3.5-turbo-1106").await {
        Ok(r) => return Some(r),
        Err(e) => {
            log::error!("Error summarizing meta data: {}", e);
            None
        }
    }
}

pub async fn analyze_issue_integrated(
    issue: &Issue,
    target_person: Option<String>,
    _turbo: bool,
    is_sparce: bool,
    token: Option<String>,
) -> Option<(String, GitMemory)> {
    let bpe = tiktoken_rs::cl100k_base().unwrap();

    let issue_creator_name = &issue.user.login;
    let issue_title = issue.title.to_string();
    let issue_number = issue.number;
    let issue_date = issue.created_at.date_naive();

    let issue_body = match &issue.body {
        Some(body) => {
            squeeze_fit_remove_quoted(body, 400, 0.7)

            // if is_sparce {
            //     //   let temp =      squeeze_fit_remove_quoted(body, 500, 0.6);
            //     squeeze_fit_remove_quoted(body, 500, 0.6)
            // } else {
            //     squeeze_fit_remove_quoted(body, 400, 0.7)
            // }
        }
        None => "".to_string(),
    };
    let issue_url = issue.url.to_string();
    let source_url = issue.html_url.to_string();

    let labels = issue
        .labels
        .iter()
        .map(|lab| lab.name.clone())
        .collect::<Vec<String>>()
        .join(", ");

    let all_text_from_issue = format!(
        "User '{}', opened an issue titled '{}', labeled '{}', with the following post: '{}'.",
        issue_creator_name, issue_title, labels, issue_body
    );
    let mut all_text_tokens = bpe.encode_ordinary(&all_text_from_issue);
    let token_str = match token {
        None => String::new(),
        Some(t) => format!("&token={}", t.as_str()),
    };
    let url_str = format!(
        "{}/comments?&sort=updated&order=desc&per_page=100{}",
        issue_url, token_str
    );

    let octocrab = get_octo(&GithubLogin::Default);

    match octocrab
        .get::<Vec<Comment>, _, ()>(&url_str, None::<&()>)
        .await
    {
        Err(_e) => {
            log::error!("Error parsing Vec<Comment> : {:?}", _e);
        }
        Ok(comments_obj) => {
            for comment in &comments_obj {
                let comment_body = match &comment.body {
                    Some(body) => {
                        squeeze_fit_remove_quoted(body, 200, 1.0)

                        // if is_sparce {
                        //     squeeze_fit_remove_quoted(body, 300, 1.0)
                        // } else {
                        //     squeeze_fit_remove_quoted(body, 200, 1.0)
                        // }
                    }
                    None => String::new(),
                };
                let commenter = &comment.user.login;
                let commenter_input = format!("{} commented: {}", commenter, comment_body);
                let mut commenter_token = bpe.encode_ordinary(&commenter_input);
                all_text_tokens.append(&mut commenter_token);
                if is_sparce {
                    if all_text_tokens.len() > 12_000 {
                        break;
                    }
                } else {
                    if all_text_tokens.len() > 3_000 {
                        break;
                    }
                }
            }
        }
    }

    let all_text_from_issue = bpe.decode(all_text_tokens).ok().unwrap_or(String::new());

    let target_str = target_person
        .clone()
        .map_or("key participants".to_string(), |t| t.to_string());

    let sys_prompt_1 = &format!(
        "Given the information that user '{issue_creator_name}' opened an issue titled '{issue_title}', your task is to deeply analyze the content of the issue posts. Distill the crux of the issue, the potential solutions suggested, and evaluate the significant contributions of the participants in resolving or progressing the discussion."
    );

    let usr_prompt_1 = &format!(
        "Analyze the GitHub issue content: {all_text_from_issue}. Provide a concise analysis touching upon: The central problem discussed in the issue. The main solutions proposed or agreed upon. Emphasize the role and significance of '{target_str}' in contributing towards the resolution or progression of the discussion. Aim for a succinct, analytical summary that stays under 110 tokens."
    );

    match chat_inner(sys_prompt_1, usr_prompt_1, 128, "gpt-3.5-turbo-1106").await {
        Ok(r) => {
            let out = format!("{} {}", issue_url, r);
            let name = target_person.map_or(issue_creator_name.to_string(), |t| t.to_string());
            let gm = GitMemory {
                memory_type: MemoryType::Issue,
                name: name,
                tag_line: issue_title,
                source_url: source_url,
                payload: r,
                date: issue_date,
            };

            Some((out, gm))
        }
        Err(_e) => {
            log::error!("Error generating issue summary #{}: {}", issue_number, _e);
            None
        }
    }
}

pub async fn analyze_commit_integrated(
    user_name: &str,
    tag_line: &str,
    url: &str,
    _turbo: bool,
    is_sparce: bool,
    token: Option<String>,
) -> anyhow::Result<String> {
    let token_str = match token {
        None => String::new(),
        Some(t) => format!("&token={}", t.as_str()),
    };

    let commit_patch_str = format!("{url}.patch{token_str}");
    // let url = Url::try_from(commit_patch_str.as_str())
    //     .expect(&format!("Error generating URI from {:?}", commit_patch_str));

    let octocrab = get_octo(&GithubLogin::Default);
    let mut headers = HeaderMap::new();
    headers.insert(CONNECTION, HeaderValue::from_static("close"));

    // let route = format!("http://10.0.0.174/headers");
    let response = octocrab
        ._get_with_headers(commit_patch_str, None::<&()>, Some(headers))
        .await?;

    // let response = octocrab._get(&commit_patch_str, None::<&()>).await?;
    let text = response.text().await?;
    // let mut stripped_texts = String::with_capacity(text.len());

    // 'commit_text_block: {
    //     let lines_count = text.lines().count();
    //     if lines_count > 150 {
    //         stripped_texts = text
    //             .splitn(2, "diff --git")
    //             .nth(0)
    //             .unwrap_or("")
    //             .to_string();
    //         break 'commit_text_block;
    //     }

    //     let mut inside_diff_block = false;

    //     match is_sparce {
    //         false => {
    //             for line in text.lines() {
    //                 if line.starts_with("diff --git") {
    //                     inside_diff_block = true;
    //                     stripped_texts.push_str(line);
    //                     stripped_texts.push('\n');
    //                     continue;
    //                 }

    //                 if inside_diff_block {
    //                     if line
    //                         .chars()
    //                         .any(|ch| ch == '[' || ch == ']' || ch == '{' || ch == '}')
    //                     {
    //                         continue;
    //                     }
    //                 }

    //                 stripped_texts.push_str(line);
    //                 stripped_texts.push('\n');

    //                 if line.is_empty() {
    //                     inside_diff_block = false;
    //                 }
    //             }
    //         }
    //         true => stripped_texts = text.to_string(),
    //     }
    // }
    // slack_flows::send_message_to_channel("ik8", "ch_rep", stripped_texts.clone()).await;

    let sys_prompt_1 = &format!(
                "Given a commit patch from user {user_name}, analyze its content. Focus on changes that substantively alter code or functionality. A good analysis prioritizes the commit message for clues on intent and refrains from overstating the impact of minor changes. Aim to provide a balanced, fact-based representation that distinguishes between major and minor contributions to the project. Keep your analysis concise."
            );

    let stripped_texts = if !is_sparce {
        let stripped_texts = text
            .splitn(2, "diff --git")
            .nth(0)
            .unwrap_or("")
            .to_string();

        let stripped_texts = squeeze_fit_remove_quoted(&stripped_texts, 5_000, 1.0);
        squeeze_fit_post_texts(&stripped_texts, 3_000, 0.6)
    } else {
        text.chars().take(24_000).collect::<String>()
    };

    // let stripped_texts = if turbo {
    //     squeeze_fit_post_texts(&stripped_texts, 3_000, 0.6)
    // } else {
    //     if stripped_texts.len() > 12_000 {
    //     }
    //     squeeze_fit_post_texts(&stripped_texts, 12_000, 0.6)
    // };

    let usr_prompt_1 = &format!(
                "Analyze the commit patch: {stripped_texts}, and its description: {tag_line}. Summarize the main changes, but only emphasize modifications that directly affect core functionality. A good summary is fact-based, derived primarily from the commit message, and avoids over-interpretation. It recognizes the difference between minor textual changes and substantial code adjustments. Conclude by evaluating the realistic impact of {user_name}'s contributions in this commit on the project. Limit the response to 110 tokens."
            );

    let sha_serial = match url.rsplitn(2, "/").nth(0) {
        Some(s) => s.chars().take(5).collect::<String>(),
        None => "0000".to_string(),
    };
    match chat_inner(sys_prompt_1, usr_prompt_1, 128, "gpt-3.5-turbo-1106").await {
        Ok(r) => {
            let out = format!("{} {}", url, r);
            Ok(out)
        }
        Err(_e) => {
            log::error!("Error generating issue summary #{}: {}", sha_serial, _e);
            Err(anyhow::anyhow!(
                "Error generating issue summary #{}: {}",
                sha_serial,
                _e
            ))
        }
    }
}

pub async fn process_commits(
    inp_vec: &mut Vec<GitMemory>,
    _turbo: bool,
    is_sparce: bool,
    token: Option<String>,
) -> Option<String> {
    let mut commits_summaries = String::new();
    let mut processed_count = 0; // Number of processed entries

    for commit_obj in inp_vec.iter_mut() {
        match analyze_commit_integrated(
            &commit_obj.name,
            &commit_obj.tag_line,
            &commit_obj.source_url,
            _turbo,
            is_sparce,
            token.clone(),
        )
        .await
        {
            Ok(summary) => {
                let len = commits_summaries.split_whitespace().count();
                if len > 3000 {
                    break;
                }
                commits_summaries.push_str(&format!("{} {}\n", commit_obj.date, summary));

                processed_count += 1;
            }
            Err(_e) => {
                log::error!(
                    "Error analyzing commit {:?} for user {}: {:?}",
                    commit_obj.source_url,
                    commit_obj.name,
                    _e
                );
            }
        }
    }

    if processed_count == 0 {
        log::error!("No commits processed");
        return None;
    }

    Some(commits_summaries)
}

pub async fn correlate_commits_issues_discussions(
    _profile_data: Option<&str>,
    _commits_summary: Option<&str>,
    _issues_summary: Option<&str>,
    _discussions_summary: Option<&str>,
    target_person: Option<&str>,
    total_input_entry_count: u16,
) -> Option<String> {
    let total_space = 16000; // 16k tokens

    let _total_ratio = 11.0; // 1 + 4 + 4 + 2
    let profile_ratio = 1.0;
    let commit_ratio = 4.0;
    let issue_ratio = 4.0;
    let discussion_ratio = 2.0;

    let available_ratios = [
        _profile_data.map(|_| profile_ratio),
        _commits_summary.map(|_| commit_ratio),
        _issues_summary.map(|_| issue_ratio),
        _discussions_summary.map(|_| discussion_ratio),
    ];

    let total_available_ratio: f32 = available_ratios.iter().filter_map(|&x| x).sum();

    let compute_space =
        |ratio: f32| -> usize { ((total_space as f32) * (ratio / total_available_ratio)) as usize };

    let profile_space = _profile_data.map_or(0, |_| compute_space(profile_ratio));
    let commit_space = _commits_summary.map_or(0, |_| compute_space(commit_ratio));
    let issue_space = _issues_summary.map_or(0, |_| compute_space(issue_ratio));
    let discussion_space = _discussions_summary.map_or(0, |_| compute_space(discussion_ratio));

    let trim_to_allocated_space =
        |source: &str, space: usize| -> String { source.chars().take(space * 3).collect() };

    let profile_str = _profile_data.map_or("".to_string(), |x| {
        format!(
            "profile data: {}",
            trim_to_allocated_space(x, profile_space)
        )
    });
    let commits_str = _commits_summary.map_or("".to_string(), |x| {
        format!("commit logs: {}", trim_to_allocated_space(x, commit_space))
    });
    let issues_str = _issues_summary.map_or("".to_string(), |x| {
        format!("issue post: {}", trim_to_allocated_space(x, issue_space))
    });
    let discussions_str = _discussions_summary.map_or("".to_string(), |x| {
        format!(
            "discussion posts: {}",
            trim_to_allocated_space(x, discussion_space)
        )
    });

    let target_str = target_person.map_or("key participants'".to_string(), |t| format!("{t}'s"));

    let sys_prompt_1 =
        "Analyze the GitHub activity data and profile data over the week to detect both key impactful contributions and connections between commits, issues, and discussions. Highlight specific code changes, resolutions, and improvements. Furthermore, trace evidence of commits addressing specific issues, discussions leading to commits, or issues spurred by discussions. The aim is to map out both the impactful technical advancements and the developmental narrative of the project.";

    let usr_prompt_1 = &format!(
        "From {profile_str}, {commits_str}, {issues_str}, and {discussions_str}, detail {target_str} significant technical contributions. Enumerate individual tasks, code enhancements, and bug resolutions, emphasizing impactful contributions. Concurrently, identify connections: commits that appear to resolve specific issues, discussions that may have catalyzed certain commits, or issues influenced by preceding discussions. Extract tangible instances showcasing both impact and interconnections within the week."
    );

    let (gen_1_size, gen_2_size, gen_2_reminder) = match total_input_entry_count {
        0..=3 => (384, 96, 250),
        4..=14 => (512, 350, 350),
        15.. => (1024, 500, 500),
    };

    // let usr_prompt_2 = &format!(
    //     "Merge the identified impactful technical contributions and their interconnections into a coherent summary for {target_str} over the week. Describe how these contributions align with the project's technical objectives. Pinpoint recurring technical patterns or trends and shed light on the synergy between individual efforts and their collective progression. Detail both the weight of each contribution and their interconnectedness in shaping the project, please use bullet-points format in your reply. Limit to less than {gen_2_reminder} tokens."
    // );
    // let usr_prompt_2 = &format!(
    //     "If applicable, summarize the key technical contributions made by {target_str} this week in bullet-point format. Address the following points within a token limit of {gen_2_reminder}. If no information is available for a point, leave it blank:
    //     - Highlight impactful contributions and their interconnections.
    //     - Explain alignment with the project's goals.
    //     - Identify recurring patterns or trends.
    //     - Discuss synergy between individual and collective advancement.
    //     - Comment objectively on the significance of each contribution."
    // );
    let usr_prompt_2 = &format!(
        r#"Analyze the key technical contributions made by {target_str} this week and summarize the information into a flat JSON structure with just one level of depth. Each key in the JSON should map directly to a single string value describing the contribution or observation in a full sentence or a short paragraph. Do not include nested objects or arrays. If no information is available for a point, provide an empty string as the value. 

Please ensure that the JSON output is compliant with RFC8259 and can be iterated as simple key-value pairs where the values are strings. Your response should follow this template:
{{
"impactful": "Provide a single string value summarizing impactful contributions and their interconnections.",
"alignment": "Provide a single string value explaining how the contributions align with the project's goals.",
"patterns": "Provide a single string value identifying any recurring patterns or trends in the contributions.",
"synergy": "Provide a single string value discussing the synergy between individual and collective advancement.",
"significance": "Provide a single string value commenting on the significance of the contributions."
}}
"#,
    );
    chain_of_chat(
        sys_prompt_1,
        usr_prompt_1,
        "correlate-99",
        gen_1_size,
        usr_prompt_2,
        gen_2_size,
        "correlate_commits_issues_discussions",
    )
    .await
    .ok()
}

pub async fn correlate_user_and_home_project(
    home_repo_data: &str,
    user_profile: &str,
    issues_data: &str,
    repos_data: &str,
    discussion_data: &str,
) -> Option<String> {
    let home_repo_data = home_repo_data.chars().take(6000).collect::<String>();
    let user_profile = user_profile.chars().take(4000).collect::<String>();
    let issues_data = issues_data.chars().take(9000).collect::<String>();
    let repos_data = repos_data.chars().take(6000).collect::<String>();
    let discussion_data = discussion_data.chars().take(4000).collect::<String>();

    let sys_prompt_1 = &format!(
        "First, let's analyze and understand the provided Github data in a step-by-step manner. Begin by evaluating the user's activity based on their most active repositories, languages used, issues they're involved in, and discussions they've participated in. Concurrently, grasp the characteristics and requirements of the home project. Your aim is to identify overlaps or connections between the user's skills or activities and the home project's needs."
    );

    let usr_prompt_1 = &format!(
        "Using a structured approach, analyze the given data: User Profile: {} Active Repositories: {} Issues Involved: {} Discussions Participated: {} Home project's characteristics: {} Identify patterns in the user's activity and spot potential synergies with the home project. Pay special attention to the programming languages they use, especially if they align with the home project's requirements. Derive insights from their interactions and the data provided.",
        user_profile,
        repos_data,
        issues_data,
        discussion_data,
        home_repo_data
    );

    let usr_prompt_2 = &format!(
        "Now, using the insights from your step-by-step analysis, craft a concise bullet-point summary that underscores: - The user's main areas of expertise and interest. - The relevance of their preferred languages or technologies to the home project. - Their potential contributions to the home project, based on their skills and interactions. Ensure the summary is clear, insightful, and remains under 256 tokens. Emphasize any evident alignments between the user's skills and the project's needs."
    );
    chain_of_chat(
        sys_prompt_1,
        usr_prompt_1,
        "correlate-user-home",
        512,
        usr_prompt_2,
        256,
        "correlate-user-home-summary",
    )
    .await
    .ok()
}
