use crate::github_data_fetchers::*;
use crate::utils::*;
use chrono::{ DateTime, Utc };
use github_flows::{ get_octo, octocrab::models::{ issues::Comment, issues::Issue }, GithubLogin };
use log;
use serde::Deserialize;
use std::collections::{ HashMap, HashSet };
use openai_flows::chat::ChatModel;

pub async fn get_repo_info(about_repo: &str) -> Option<String> {
    #[derive(Deserialize)]
    struct CommunityProfile {
        description: Option<String>,
        readme: Option<String>,
        updated_at: Option<DateTime<Utc>>,
    }

    let community_profile_url = format!("repos/{}/community/profile", about_repo);

    let mut description = String::new();
    let octocrab = get_octo(&GithubLogin::Default);

    match octocrab.get::<CommunityProfile, _, ()>(&community_profile_url, None::<&()>).await {
        Ok(profile) => {
            description = profile.description.as_ref().unwrap_or(&String::from("")).to_string();
        }
        Err(e) => log::error!("Error parsing Community Profile: {:?}", e),
    }

    let mut payload = String::new();
    match get_readme_owner_repo(about_repo).await {
        Some(content) => {
            let content = content.chars().take(20000).collect::<String>();
            match analyze_readme(&content).await {
                Some(summary) => {
                    payload = summary;
                }
                None => log::error!("Error parsing README.md: {}", about_repo),
            }
        }
        None => log::error!("Error fetching README.md: {}", about_repo),
    }
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

    let raw_text;
    match web_scraper_flows::get_page_text(&repo_home_url).await {
        Ok(page_text) => {
            raw_text = page_text;
        }
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

    let sys_prompt =
        "Your task is to examine the textual content from a GitHub repo page, emphasizing the Header, About, Release, Contributors, Languages, and README sections. This process should be carried out objectively, focusing on factual information extraction from each segment. Avoid making subjective judgments or inferences. The data should be presented systematically, corresponding to each section. Please note, the provided text will be in a flattened format.";

    let usr_prompt =
        &format!("I’ve obtained a flattened text from a GitHub repo page and require analysis of the following sections: 1) Header, with data on Fork, Star, Issues, Pull Request, etc.; 2) About, containing project description, keywords, number of stars, watchers, and forks; 3) Release, with details on the latest release and total releases; 4) Contributors, showing the number of contributors; 5) Languages, displaying the language composition in the project, and 6) README, which is usually a body of text describing the project, please summarize README when presenting result. Please extract and present data from these sections individually. Here is the text: {}", raw_text);

    match chat_inner(sys_prompt, usr_prompt, 700, ChatModel::GPT35Turbo16K).await {
        Ok(r) => {
            return Some(r);
        }
        Err(_e) => {
            log::error!("Error summarizing meta data: {}", _e);
            return None;
        }
    }
}

pub async fn is_valid_owner_repo(
    owner_repo: &str
) -> anyhow::Result<(String, String, HashSet<String>)> {
    #[derive(Deserialize)]
    struct CommunityProfile {
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
    let community_profile_url = format!("repos/{}/community/profile", owner_repo);

    let description;
    let mut has_readme = false;
    let octocrab = get_octo(&GithubLogin::Default);

    match octocrab.get::<CommunityProfile, _, ()>(&community_profile_url, None::<&()>).await {
        Ok(profile) => {
            description = profile.description.as_ref().unwrap_or(&String::from("")).to_string();

            has_readme = profile.files.readme
                .as_ref()
                .unwrap_or(&(Readme { url: None }))
                .url.is_some();
        }
        Err(e) => {
            log::error!("Error parsing Community Profile: {:?}", e);
            return Err(anyhow::anyhow!("no Community Profile, so invalid owner/repo: {:?}", e));
        }
    }

    let mut payload = String::new();

    if has_readme {
        if let Some(content) = get_readme(owner_repo).await {
            let content = content.chars().take(20000).collect::<String>();
            match analyze_readme(&content).await {
                Some(summary) => {
                    payload = summary;
                }
                None => log::error!("Error parsing README.md: {}", owner_repo),
            }
        }
    }

    if payload.is_empty() {
        payload = description.clone();
    }

    let contributors_set = match get_contributors(owner_repo).await {
        Ok(contributors) => contributors.into_iter().collect::<HashSet<String>>(),
        Err(_e) => HashSet::<String>::new(),
    };

    Ok((owner_repo.to_string(), payload, contributors_set))
}

pub async fn process_issues(
    inp_vec: Vec<Issue>,
    target_person: Option<String>,
    contributors_set: HashSet<String>,
    token: Option<String>
) -> anyhow::Result<HashMap<String, (String, String)>> {
    use futures::future::join_all;

    let issue_futures: Vec<_> = inp_vec
        .into_iter()
        .map(|issue| {
            let target_person = target_person.clone();
            let token = token.clone();
            let contributors_set = contributors_set.clone();
            async move {
                let ve = analyze_issue_integrated(
                    &issue,
                    target_person,
                    contributors_set,
                    token
                ).await.ok()?;
                Some(ve)
            }
        })
        .collect();

    let results = join_all(issue_futures).await;
    let mut issues_map = HashMap::<String, (String, String)>::new();

    for result in results.into_iter().flatten() {
        for item in result {
            let (user_name, url, summary) = item;
            // log::info!(
            //     "User: {:?}, Url: {:?}, Summary: {:?}",
            //     user_name.clone(),
            //     url.clone(),
            //     summary.clone()
            // );
            issues_map
                .entry(user_name.clone())
                .and_modify(|tup| {
                    tup.0.push_str("\n");
                    tup.0.push_str(&url);
                    tup.1.push_str("\n");
                    tup.1.push_str(&summary);
                })
                .or_insert((url.to_string(), summary.to_string()));
        }
    }

    if issues_map.len() == 0 {
        anyhow::bail!("No issues processed");
    }

    Ok(issues_map)
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

    match chat_inner(sys_prompt_1, usr_prompt_1, 256, ChatModel::GPT35Turbo16K).await {
        Ok(r) => {
            return Some(r);
        }
        Err(e) => {
            log::error!("Error summarizing meta data: {}", e);
            None
        }
    }
}

pub async fn analyze_issue_integrated(
    issue: &Issue,
    target_person: Option<String>,
    contributors_set: HashSet<String>,
    token: Option<String>
) -> anyhow::Result<Vec<(String, String, String)>> {
    let issue_creator_name = &issue.user.login;
    let issue_title = issue.title.to_string();
    let issue_number = issue.number;
    let mut issue_commenters_to_watch = Vec::new();
    let issue_body = match &issue.body {
        Some(body) => squeeze_fit_remove_quoted(body, 400, 0.7),
        None => "".to_string(),
    };
    let issue_url = issue.url.to_string();
    let source_url = issue.html_url.to_string();

    let labels = issue.labels
        .iter()
        .map(|lab| lab.name.clone())
        .collect::<Vec<String>>()
        .join(", ");

    let mut all_text_from_issue = format!(
        "User '{}', opened an issue titled '{}', labeled '{}', with the following post: '{}'.",
        issue_creator_name,
        issue_title,
        labels,
        issue_body
    );

    let token_str = match token {
        None => String::new(),
        Some(t) => format!("&token={}", t.as_str()),
    };

    // let comments_url = format!(
    //     "{}/comments?sort=updated&order=desc&per_page=100{}",
    //     issue_url.replace("https://api.github.com/", ""),
    //     token_str
    // );
    let comments_url = format!(
        "{}/comments?sort=updated&order=desc&per_page=100{}",
        issue_url,
        token_str
    );
    // let octocrab = get_octo(&GithubLogin::Default);

    let response = github_http_get(&comments_url).await?;
    let comments_obj = serde_json::from_slice::<Vec<Comment>>(&response)?;

    for comment in &comments_obj {
        let comment_body = match &comment.body {
            Some(body) => squeeze_fit_remove_quoted(body, 200, 1.0),
            None => String::new(),
        };
        let commenter = &comment.user.login;
        if contributors_set.contains(commenter) {
            issue_commenters_to_watch.push(commenter.to_string());
        }

        let commenter_input = format!("{} commented: {}", commenter, comment_body);
        all_text_from_issue.push_str(&commenter_input);
    }

    all_text_from_issue = all_text_from_issue.chars().take(32_000).collect();

    let target_str = target_person
        .clone()
        .map_or("key participants".to_string(), |t| t.to_string());

    let sys_prompt_1 = &format!(
        "Given the information that user '{issue_creator_name}' opened an issue titled '{issue_title}', your task is to deeply analyze the content of the issue posts. Distill the crux of the issue, the potential solutions suggested, and evaluate the significant contributions of the participants in resolving or progressing the discussion."
    );

    let commenters_to_watch_str = if !target_str.is_empty() || issue_commenters_to_watch.len() == 0 {
        target_str
    } else {
        issue_commenters_to_watch.join(", ")
    };

    // let usr_prompt_1 = &format!(
    //     "Analyze the GitHub issue content: {}. Provide a concise analysis touching upon: The central problem discussed in the issue. The main solutions proposed or agreed upon. Highlight the role and significance of '{}' in contributing towards the resolution or progression of the discussion. Format the analysis into a flat JSON structure with one level of depth where each key maps directly to a single string value. Use the following template, replacing 'contributor_name' with the actual contributor's name, and 'summary' with your analysis of their contributions:
    //     {{
    //     \"contributor_name_1\": \"summary\",
    //     \"contributor_name_2\": \"summary\"
    //     }}",
    //     all_text_from_issue,
    //     commenters_to_watch_str
    // );
    let usr_prompt_1 = &format!(
        "Analyze the GitHub issue content: {}. Provide a concise analysis touching upon: The central problem discussed in the issue. The main solutions proposed or agreed upon. Highlight the role and significance of '{}' in contributing towards the resolution or progression of the discussion. If the target person's contribution is negligible or non-existent, leave the corresponding summary blank. Format the analysis into a flat JSON structure with one level of depth where each key maps directly to a single string value. Use the following template, replacing 'contributor_name' with the actual contributor's name, and 'summary' with your analysis of their contributions or an empty string if their contribution is negligible: 
        {{ 
        \"contributor_name_1\": \"summary\",
        \"contributor_name_2\": \"summary\"
        }}",
        all_text_from_issue,
        commenters_to_watch_str
    );

    match chat_inner(sys_prompt_1, usr_prompt_1, 128, ChatModel::GPT35Turbo16K).await {
        Ok(r) => {
            let parsed = parse_issue_summary_from_json(&r)
                .ok()
                .unwrap_or_else(|| vec![]);

            let out = parsed
                .into_iter()
                .map(|(user_name, summary)| { (user_name, source_url.clone(), summary) })
                .collect::<Vec<(String, String, String)>>();

            Ok(out)

            // let out = format!("{} {}", issue_url, r);
            // let name = target_person.map_or(issue_creator_name.to_string(), |t| t.to_string());
            // let gm = GitMemory {
            //     memory_type: MemoryType::Issue,
            //     name: name,
            //     tag_line: issue_title,
            //     source_url: source_url,
            //     payload: r,
            // };

            // Ok((out, gm))
        }
        Err(_e) => {
            log::error!("Error generating issue summary #{}: {}", issue_number, _e);
            Err(anyhow::anyhow!("Error generating issue summary #{}: {}", issue_number, _e))
        }
    }
}
pub async fn process_commits(
    inp_vec: Vec<GitMemory>,
    commits_map: &mut HashMap<String, (String, String)>,
    token: Option<String>
) -> anyhow::Result<()> {
    use futures::future::join_all;
    let token_query = match token {
        None => String::new(),
        Some(t) => format!("?token={}", t),
    };

    let commit_futures: Vec<_> = inp_vec
        .into_iter()
        .map(|commit_obj| {
            let url = format!("{}.patch{}", commit_obj.source_url, token_query);
            async move {
                let response = github_http_get(&url).await.ok()?;
                let text = String::from_utf8(response).ok()?;

                let stripped_texts = text.chars().take(24_000).collect::<String>();
                // let stripped_texts = String::from_utf8(response).ok()?.chars().take(24_000).collect::<String>();
                let user_name = commit_obj.name.clone();
                let sys_prompt_1 = format!(
                    "Given a commit patch from user {user_name}, analyze its content. Focus on changes that substantively alter code or functionality. A good analysis prioritizes the commit message for clues on intent and refrains from overstating the impact of minor changes. Aim to provide a balanced, fact-based representation that distinguishes between major and minor contributions to the project. Keep your analysis concise."
                );
                let tag_line = commit_obj.tag_line;
                let usr_prompt_1 = format!(
                    "Analyze the commit patch: {stripped_texts}, and its description: {tag_line}. Summarize the main changes, but only emphasize modifications that directly affect core functionality. A good summary is fact-based, derived primarily from the commit message, and avoids over-interpretation. It recognizes the difference between minor textual changes and substantial code adjustments. Conclude by evaluating the realistic impact of {user_name}'s contributions in this commit on the project. Limit the response to 110 tokens."
                );
                let summary = chat_inner(
                    &sys_prompt_1,
                    &usr_prompt_1,
                    128,
                    ChatModel::GPT35Turbo16K
                ).await.ok()?;
                // log::info!("Summary: {:?}", summary.clone());
                Some((commit_obj.name, commit_obj.source_url, summary))
            }
        })
        .collect();

    let results = join_all(commit_futures).await;
    for result in results.into_iter().flatten() {
        let (user_name, url, summary): (String, String, String) = result;
        // log::info!(
        //     "User: {:?}, Url: {:?}, Summary: {:?}",
        //     user_name.clone(),
        //     url.clone(),
        //     summary.clone()
        // );
        commits_map
            .entry(user_name.clone())
            .and_modify(|tup| {
                tup.0.push_str("\n");
                tup.0.push_str(&url);
                tup.1.push_str("\n");
                tup.1.push_str(&summary);
            })
            .or_insert((url, summary.to_string()));
    }

    Ok(())
}

pub async fn correlate_commits_issues_sparse(
    _commits_summary: &str,
    _issues_summary: &str,
    target_person: &str
) -> Option<String> {
    let system_prompt =
        "You're a GitHub data analysis bot. You're tasked to analyze a GitHub contributor's activity data over the week to detect both key impactful contributions and connections between commits and issues. Highlight specific code changes, resolutions, and improvements.";

    let user_input = &format!(
        r#"From {_commits_summary}, {_issues_summary}. Analyze the key technical contributions made by {target_person} this week and summarize the information into a flat JSON structure with just one level of depth. Each key in the JSON should map directly to a single string value describing the contribution or observation in a full sentence or a short paragraph without using nested objects or arrays. If no information is available for a point, provide an empty string as the value. 
Please ensure that the JSON output does not include any Markdown formatting, such as code block syntax ("```") or escaped characters (like "\\n" for new lines). The output should be plain JSON that can be parsed directly without any preprocessing.

Your JSON response should use the following keys with appropriate string values:
{{
"impactful": "Provide a single string value summarizing impactful contributions and their interconnections.",
"alignment": "Provide a single string value explaining how the contributions align with the project's goals.",
"patterns": "Provide a single string value identifying any recurring patterns or trends in the contributions.",
"synergy": "Provide a single string value discussing the synergy between individual and collective advancement.",
"significance": "Provide a single string value commenting on the significance of the contributions."
}}
Ensure that the JSON is properly formatted, with correct escaping of special characters, and is ready to be parsed by a JSON parser that expects RFC8259-compliant JSON. Avoid adding any non-JSON content or formatting."#
    );

    chat_inner_async(system_prompt, user_input, 500, "gpt-3.5-turbo-1106").await.ok()
}

/* pub async fn github_http_fetch(token: &str, url: &str) -> Option<Vec<u8>> {
    let url = http_req::uri::Uri::try_from(url).unwrap();
    let mut writer = Vec::new();

    match http_req::request::Request::new(&url)
        .method(http_req::request::Method::GET)
        .header("User-Agent", "flows-network connector")
        .header("Content-Type", "application/vnd.github.v3+json")
        .header("Authorization", &format!("Bearer {token}"))
        .send(&mut writer)
    {
        Ok(res) => {
            if !res.status_code().is_success() {
                log::error!("Github http error {:?}", res.status_code());
                return None;
            };

            Some(writer)
        }
        Err(_e) => {
            log::error!("Error getting response from Github: {:?}", _e);
            None
        }
    }
} */
