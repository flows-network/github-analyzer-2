use crate::utils::*;
use chrono::{DateTime, Duration, Utc};
use derivative::Derivative;
use github_flows::octocrab::models::{issues::Comment, issues::Issue, Repository, User};
use github_flows::{get_octo, octocrab, GithubLogin};
use serde::{Deserialize, Serialize};

#[derive(Derivative, Serialize, Deserialize, Debug, Clone)]
pub struct GitMemory {
    pub memory_type: MemoryType,
    #[derivative(Default(value = "String::from(\"\")"))]
    pub name: String,
    #[derivative(Default(value = "String::from(\"\")"))]
    pub tag_line: String,
    #[derivative(Default(value = "String::from(\"\")"))]
    pub source_url: String,
    #[derivative(Default(value = "String::from(\"\")"))]
    pub payload: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MemoryType {
    Commit,
    Issue,
    Discussion,
    Meta,
}

pub async fn get_user_profile(user: &str) -> Option<User> {
    let user_profile_url = format!("users/{user}");

    let octocrab = get_octo(&GithubLogin::Default);

    octocrab
        .get::<User, _, ()>(&user_profile_url, None::<&()>)
        .await
        .ok()
}

pub async fn get_user_data_by_login(login: &str) -> anyhow::Result<String> {
    #[derive(Debug, Deserialize)]
    struct User {
        name: Option<String>,
        login: Option<String>,
        url: Option<String>,
        #[serde(rename = "twitterUsername")]
        twitter_username: Option<String>,
        bio: Option<String>,
        company: Option<String>,
        location: Option<String>,
        #[serde(rename = "createdAt")]
        created_at: Option<DateTime<Utc>>,
        email: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    struct RepositoryOwner {
        #[serde(rename = "repositoryOwner")]
        repository_owner: Option<User>,
    }

    #[derive(Debug, Deserialize)]
    struct UserRoot {
        data: Option<RepositoryOwner>,
    }

    let mut out = String::from("USER_profile: \n");

    let query = format!(
        r#"
        query {{
            repositoryOwner(login: "{login}") {{
                ... on User {{
                    name
                    login
                    url
                    twitterUsername
                    bio
                    company
                    location
                    createdAt
                    email
                }}
            }}
        }}
        "#
    );

    let octocrab = get_octo(&GithubLogin::Default);

    let res: UserRoot = octocrab.graphql::<UserRoot>(&query).await?;
    if let Some(repository_owner) = &res.data {
        if let Some(user) = &repository_owner.repository_owner {
            let login_str = match &user.login {
                Some(login) => format!("Login: {},", login),
                None => String::new(),
            };

            let name_str = match &user.name {
                Some(name) => format!("Name: {},", name),
                None => String::new(),
            };

            let url_str = match &user.url {
                Some(url) => format!("Url: {},", url),
                None => String::new(),
            };

            let twitter_str = match &user.twitter_username {
                Some(twitter) => format!("Twitter: {},", twitter),
                None => String::new(),
            };

            let bio_str = match &user.bio {
                Some(bio) if bio.is_empty() => String::new(),
                Some(bio) => format!("Bio: {},", bio),
                None => String::new(),
            };

            let company_str = match &user.company {
                Some(company) => format!("Company: {},", company),
                None => String::new(),
            };

            let location_str = match &user.location {
                Some(location) => format!("Location: {},", location),
                None => String::new(),
            };

            let date_str = match &user.created_at {
                Some(date) => {
                    format!("Created At: {},", date.date_naive().to_string())
                }
                None => String::new(),
            };

            let email_str = match &user.email {
                Some(email) => format!("Email: {}", email),
                None => String::new(),
            };

            out.push_str(
                &format!(
                    "{name_str} {login_str} {url_str} {twitter_str} {bio_str} {company_str} {location_str} {date_str} {email_str}\n"
                )
            );
        }
    }

    Ok(out)
}

pub async fn get_community_profile_data(owner: &str, repo: &str) -> Option<String> {
    #[derive(Deserialize, Debug)]
    struct CommunityProfile {
        description: String,
        // documentation: Option<String>,
    }

    let community_profile_url = format!("repos/{owner}/{repo}/community/profile");

    let octocrab = get_octo(&GithubLogin::Default);

    match octocrab
        .get::<CommunityProfile, _, ()>(&community_profile_url, None::<&()>)
        .await
    {
        Ok(profile) => {
            return Some(format!("Description: {}", profile.description));
        }
        Err(e) => log::error!("Error parsing Community Profile: {:?}", e),
    }
    None
}

pub async fn get_contributors(owner: &str, repo: &str) -> Result<Vec<String>, octocrab::Error> {
    #[derive(Debug, Deserialize)]
    struct GithubUser {
        login: String,
    }
    let mut contributors = Vec::new();
    let octocrab = get_octo(&GithubLogin::Default);
    'outer: for n in 1..50 {
        log::info!("contributors loop {}", n);

        let contributors_route =
            format!("repos/{owner}/{repo}/contributors?per_page=100&page={n}",);

        match octocrab
            .get::<Vec<GithubUser>, _, ()>(&contributors_route, None::<&()>)
            .await
        {
            Ok(user_vec) => {
                if user_vec.is_empty() {
                    break 'outer;
                }
                for user in &user_vec {
                    contributors.push(user.login.clone());
                    // log::info!("user: {}", user.login);
                    // upload_airtable(&user.login, "email", "twitter_username", false).await;
                }
            }

            Err(_e) => {
                log::error!("looping stopped: {:?}", _e);
                break 'outer;
            }
        }
    }

    Ok(contributors)
}

pub async fn get_readme(owner: &str, repo: &str) -> Option<String> {
    #[derive(Deserialize, Debug)]
    struct GithubReadme {
        content: Option<String>,
    }

    let readme_url = format!("repos/{owner}/{repo}/readme");

    let octocrab = get_octo(&GithubLogin::Default);

    match octocrab
        .get::<GithubReadme, _, ()>(&readme_url, None::<&()>)
        .await
    {
        Ok(readme) => {
            if let Some(c) = readme.content {
                let cleaned_content = c.replace("\n", "");
                match base64::decode(&cleaned_content) {
                    Ok(decoded_content) => match String::from_utf8(decoded_content) {
                        Ok(out) => {
                            return Some(format!("Readme: {}", out));
                        }
                        Err(e) => {
                            log::error!("Failed to convert cleaned readme to String: {:?}", e);
                            return None;
                        }
                    },
                    Err(e) => {
                        log::error!("Error decoding base64 content: {:?}", e);
                        None
                    }
                }
            } else {
                log::error!("Content field in readme is null.");
                None
            }
        }
        Err(e) => {
            log::error!("Error parsing Readme: {:?}", e);
            None
        }
    }
}
pub async fn get_readme_owner_repo(about_repo: &str) -> Option<String> {
    #[derive(Deserialize, Debug)]
    struct GithubReadme {
        content: Option<String>,
    }

    let readme_url = format!("repos/{about_repo}/readme");

    let octocrab = get_octo(&GithubLogin::Default);

    match octocrab
        .get::<GithubReadme, _, ()>(&readme_url, None::<&()>)
        .await
    {
        Ok(readme) => {
            if let Some(c) = readme.content {
                let cleaned_content = c.replace("\n", "");
                match base64::decode(&cleaned_content) {
                    Ok(decoded_content) => match String::from_utf8(decoded_content) {
                        Ok(out) => {
                            return Some(format!("Readme: {}", out));
                        }
                        Err(e) => {
                            log::error!("Failed to convert cleaned readme to String: {:?}", e);
                            return None;
                        }
                    },
                    Err(e) => {
                        log::error!("Error decoding base64 content: {:?}", e);
                        None
                    }
                }
            } else {
                log::error!("Content field in readme is null.");
                None
            }
        }
        Err(e) => {
            log::error!("Error parsing Readme: {:?}", e);
            None
        }
    }
}
pub async fn get_issues_in_range(
    owner: &str,
    repo: &str,
    user_name: Option<String>,
    range: u16,
    token: Option<String>,
) -> Option<(usize, Vec<Issue>)> {
    #[derive(Debug, Deserialize)]
    struct Page<T> {
        pub items: Vec<T>,
        pub total_count: Option<u64>,
    }

    let n_days_ago = (Utc::now() - Duration::days(range as i64))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    let user_str = user_name.map_or(String::new(), |u| format!("involves:{}", u));

    let query = format!("repo:{owner}/{repo} is:issue {user_str} updated:>{n_days_ago}");
    let encoded_query = urlencoding::encode(&query);
    let token_str = match token {
        None => String::new(),
        Some(t) => format!("&token={}", t.as_str()),
    };
    let url_str = format!(
        "search/issues?q={}&sort=updated&order=desc&per_page=100{token_str}",
        encoded_query
    );
    // let url_str = format!(
    //     "https://api.github.com/search/issues?q={}&sort=updated&order=desc&per_page=100{token_str}",
    //     encoded_query
    // );

    let octocrab = get_octo(&GithubLogin::Default);

    match octocrab
        .get::<Page<Issue>, _, ()>(&url_str, None::<&()>)
        .await
    {
        Err(e) => {
            log::error!("error: {:?}", e);
            None
        }
        Ok(issue_page) => {
            // let count = issue_page.total_count.unwrap_or(0);
            let count = issue_page.items.len();
            Some((count, issue_page.items))
        }
    }
}

pub async fn get_issue_texts(issue: &Issue) -> Option<String> {
    let issue_creator_name = &issue.user.login;
    let issue_title = &issue.title;
    let issue_body = match &issue.body {
        Some(body) => squeeze_fit_remove_quoted(body, 500, 0.6),
        None => "".to_string(),
    };
    let issue_url = &issue.url.to_string();

    let labels = issue
        .labels
        .iter()
        .map(|lab| lab.name.clone())
        .collect::<Vec<String>>()
        .join(", ");

    let mut all_text_from_issue = format!(
        "User '{}', opened an issue titled '{}', labeled '{}', with the following post: '{}'.",
        issue_creator_name, issue_title, labels, issue_body
    );

    let mut current_page = 1;
    loop {
        let url_str = format!("{}/comments?&page={}", issue_url, current_page);

        let octocrab = get_octo(&GithubLogin::Default);

        match octocrab
            .get::<Vec<Comment>, _, ()>(&url_str, None::<&()>)
            .await
        {
            Err(_e) => {
                log::error!(
                    "Error parsing Vec<Comment> at page {}: {:?}",
                    current_page,
                    _e
                );
                break;
            }
            Ok(comments_obj) => {
                if comments_obj.is_empty() {
                    break;
                }
                for comment in &comments_obj {
                    let comment_body = match &comment.body {
                        Some(body) => squeeze_fit_remove_quoted(body, 300, 0.6),
                        None => "".to_string(),
                    };
                    let commenter = &comment.user.login;
                    let commenter_input = format!("{} commented: {}", commenter, comment_body);
                    if all_text_from_issue.len() > 45_000 {
                        break;
                    }
                    all_text_from_issue.push_str(&commenter_input);
                }
            }
        }

        current_page += 1;
    }

    Some(all_text_from_issue)
}
pub async fn get_commits_in_range_search(
    owner: &str,
    repo: &str,
    user_name: Option<String>,
    range: u16,
    token: Option<String>,
) -> Option<(usize, Vec<GitMemory>)> {
    #[derive(Debug, Deserialize)]
    struct Page<T> {
        pub items: Vec<T>,
        pub total_count: Option<u64>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    struct User {
        login: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct GithubCommit {
        sha: String,
        html_url: String,
        author: Option<User>, // made nullable
        commit: CommitDetails,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct CommitDetails {
        message: String,
        // committer: CommitUserDetails,
    }
    let token_str = match &token {
        None => String::from(""),
        Some(t) => format!("&token={}", t.as_str()),
    };
    let author_str = match &user_name {
        None => String::from(""),
        Some(t) => format!("author:{}", t.as_str()),
    };
    let now = Utc::now();
    let n_days_ago = (now - Duration::days(range as i64)).date_naive();

    let query = format!("repo:{owner}/{repo} {author_str} updated:>{n_days_ago}");
    let encoded_query = urlencoding::encode(&query);

    let url_str = format!(
        "search/commits?q={}&sort=author-date&order=desc&per_page=100{token_str}",
        encoded_query
    );
    // let url_str = format!(
    //     "https://api.github.com/search/commits?q={}&sort=author-date&order=desc&per_page=100{token_str}",
    //     encoded_query
    // );

    let mut git_memory_vec = vec![];
    let octocrab = get_octo(&GithubLogin::Default);

    match octocrab
        .get::<Page<GithubCommit>, _, ()>(&url_str, None::<&()>)
        .await
    {
        Err(e) => {
            log::error!("Error parsing commits: {:?}", e);
        }
        Ok(commits_page) => {
            for commit in commits_page.items {
                if let Some(author) = &commit.author {
                    git_memory_vec.push(GitMemory {
                        memory_type: MemoryType::Commit,
                        name: author.login.clone(),
                        tag_line: commit.commit.message.clone(),
                        source_url: commit.html_url.clone(),
                        payload: String::from(""),
                    });
                }
            }
        }
    }

    let count = git_memory_vec.len();

    Some((count, git_memory_vec))
}

/* pub async fn get_commits_in_range(
    owner: &str,
    repo: &str,
    user_name: Option<String>,
    range: u16,
    token: Option<String>,
) -> Option<(usize, Vec<GitMemory>, Vec<GitMemory>)> {
    #[derive(Debug, Deserialize, Serialize, Clone)]
    struct User {
        login: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct GithubCommit {
        sha: String,
        html_url: String,
        author: Option<User>,    // made nullable
        committer: Option<User>, // made nullable
        commit: CommitDetails,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct CommitDetails {
        author: CommitUserDetails,
        message: String,
        // committer: CommitUserDetails,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct CommitUserDetails {
        date: Option<DateTime<Utc>>,
    }
    let token_str = match &token {
        None => String::from(""),
        Some(t) => format!("&token={}", t.as_str()),
    };
    let author_str = match &user_name {
        None => String::from(""),
        Some(t) => format!("&author={}", t.as_str()),
    };
    let base_commit_url =
        format!("repos/{owner}/{repo}/commits?{author_str}&sort=desc&per_page=100{token_str}");
    // let base_commit_url =
    //     format!("https://api.github.com/repos/{owner}/{repo}/commits?&author={author}&sort=desc&per_page=100{token_str}");

    // let url_str = format!(
    //     "search/commits?q={}&sort=updated&order=desc&per_page=100{token_str}",
    //     encoded_query
    // );
    // let url_str = format!(
    //     "https://api.github.com/search/issues?q={}&sort=updated&order=desc&per_page=100{token_str}",
    //     encoded_query
    // );

    let mut git_memory_vec = vec![];
    let now = Utc::now();
    let n_days_ago = (now - Duration::days(range as i64)).date_naive();
    let octocrab = get_octo(&GithubLogin::Default);

    // match octocrab
    // .get::<Page<Issue>, _, ()>(&url_str, None::<&()>)
    // .await

    match octocrab
        .get::<Vec<GithubCommit>, _, ()>(&base_commit_url, None::<&()>)
        .await
    {
        Err(e) => {
            log::error!("Error parsing commits: {:?}", e);
        }
        Ok(commits) => {
            for commit in commits {
                if let Some(commit_date) = &commit.commit.author.date {
                    if commit_date.date_naive() <= n_days_ago {
                        continue;
                    }

                    if let Some(user_name) = &user_name {
                        if let Some(author) = &commit.author {
                            if author.login.as_str() == user_name {
                                git_memory_vec.push(GitMemory {
                                    memory_type: MemoryType::Commit,
                                    name: author.login.clone(),
                                    tag_line: commit.commit.message.clone(),
                                    source_url: commit.html_url.clone(),
                                    payload: String::from(""),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    let count = git_memory_vec.len();

    Some((count, git_memory_vec))
} */

pub async fn get_user_repos_in_language(user: &str, language: &str) -> Option<Vec<Repository>> {
    #[derive(Debug, Deserialize)]
    struct Page<T> {
        pub items: Vec<T>,
        pub total_count: Option<u64>,
    }

    let query = format!("user:{} language:{} sort:stars", user, language);
    let encoded_query = urlencoding::encode(&query);

    let mut out: Vec<Repository> = vec![];
    let mut total_pages = None;
    let mut current_page = 1;

    loop {
        let url_str = format!(
            "search/repositories?q={}&page={}",
            encoded_query, current_page
        );

        let octocrab = get_octo(&GithubLogin::Default);

        match octocrab
            .get::<Page<Repository>, _, ()>(&url_str, None::<&()>)
            .await
        {
            Err(_e) => {
                log::error!("Error parsing Page<Repository>: {:?}", _e);
                break;
            }
            Ok(repo_page) => {
                if total_pages.is_none() {
                    if let Some(count) = repo_page.total_count {
                        total_pages = Some(((count as f64) / 30.0).ceil() as usize);
                    }
                }

                for repo in repo_page.items {
                    out.push(repo);
                }

                current_page += 1;
                if current_page > total_pages.unwrap_or(usize::MAX) {
                    break;
                }
            }
        }
    }

    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

pub async fn get_user_repos_gql(user_name: &str, language: &str) -> Option<String> {
    #[derive(Debug, Deserialize)]
    struct Root {
        data: Data,
    }

    #[derive(Debug, Deserialize)]
    struct Data {
        search: Search,
    }

    #[derive(Debug, Deserialize)]
    struct Search {
        nodes: Vec<Node>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Node {
        pub name: String,
        #[serde(rename = "defaultBranchRef")]
        default_branch_ref: BranchRef,
        stargazers: Stargazers,
        pub description: Option<String>,
    }
    #[derive(Debug, Deserialize)]
    struct BranchRef {
        target: Target,
    }

    #[derive(Debug, Deserialize)]
    struct Target {
        history: History,
    }

    #[derive(Debug, Deserialize)]
    struct History {
        #[serde(rename = "totalCount")]
        total_count: i32,
    }

    #[derive(Debug, Deserialize)]
    struct Stargazers {
        #[serde(rename = "totalCount")]
        total_count: i32,
    }

    let query = format!(
        r#"
    query {{
        search(query: "user:{} language:{}", type: REPOSITORY, first: 100) {{
            nodes {{
                ... on Repository {{
                    name
                    defaultBranchRef {{
                        target {{
                            ... on Commit {{
                                history(first: 0) {{
                                    totalCount
                                }}
                            }}
                        }}
                    }}
                    description
                    stargazers {{
                        totalCount
                    }}
                }}
            }}
        }}
    }}
    "#,
        user_name, language
    );

    let octocrab = get_octo(&GithubLogin::Default);
    let mut out = format!("Repos in {language}:\n");

    match octocrab.graphql::<Root>(&query).await {
        Err(e) => log::error!("Failed to parse the response: {}", e),
        Ok(repos) => {
            let mut repos_sorted: Vec<&Node> = repos.data.search.nodes.iter().collect();
            repos_sorted.sort_by(|a, b| b.stargazers.total_count.cmp(&a.stargazers.total_count));

            for repo in repos_sorted {
                let name_str = format!("Repo: {}", repo.name);

                let description_str = match &repo.description {
                    Some(description) => format!("Description: {},", description),
                    None => String::new(),
                };

                let stars_str = match repo.stargazers.total_count {
                    0 => String::new(),
                    count => format!("Stars: {count}"),
                };

                let commits_str = format!(
                    "Commits: {}",
                    repo.default_branch_ref.target.history.total_count
                );

                let temp = format!("{name_str} {description_str} {stars_str} {commits_str}\n");

                out.push_str(&temp);
            }

            log::info!("Found {} repositories", repos.data.search.nodes.len());
        }
    }
    Some(out)
}

pub async fn search_issue(search_query: &str) -> anyhow::Result<String> {
    #[derive(Debug, Deserialize, Clone)]
    pub struct User {
        login: Option<String>,
    }

    #[derive(Debug, Deserialize, Clone)]
    struct AssigneeNode {
        node: Option<User>,
    }

    #[derive(Debug, Deserialize, Clone)]
    struct AssigneeEdge {
        edges: Option<Vec<Option<AssigneeNode>>>,
    }

    #[derive(Debug, Deserialize, Clone)]
    struct Issue {
        url: Option<String>,
        number: Option<u64>,
        state: Option<String>,
        title: Option<String>,
        body: Option<String>,
        author: Option<User>,
        assignees: Option<AssigneeEdge>,
        #[serde(rename = "authorAssociation")]
        author_association: Option<String>,
        #[serde(rename = "createdAt")]
        created_at: Option<DateTime<Utc>>,
        #[serde(rename = "updatedAt")]
        updated_at: Option<DateTime<Utc>>,
    }

    #[derive(Debug, Deserialize)]
    struct IssueNode {
        node: Option<Issue>,
    }

    #[derive(Debug, Deserialize, Clone)]
    struct PageInfo {
        #[serde(rename = "endCursor")]
        end_cursor: Option<String>,
        #[serde(rename = "hasNextPage")]
        has_next_page: Option<bool>,
    }

    #[derive(Debug, Deserialize)]
    struct SearchResult {
        edges: Option<Vec<Option<IssueNode>>>,
        #[serde(rename = "pageInfo")]
        page_info: Option<PageInfo>,
    }

    #[derive(Debug, Deserialize)]
    struct IssueSearch {
        search: Option<SearchResult>,
    }

    #[derive(Debug, Deserialize)]
    struct IssueRoot {
        data: Option<IssueSearch>,
    }

    let mut out = String::from("ISSUES \n");

    let mut cursor: Option<String> = None;
    let mut has_next_page = true;

    loop {
        let query = format!(
            r#"
            query {{
                search(query: "{}", type: ISSUE, first: 100{}) {{
                    edges {{
                        node {{
                            ... on Issue {{
                                url
                                number
                                state
                                title
                                body
                                author {{
                                    login
                                }}
                                assignees(first: 100) {{
                                    edges {{
                                        node {{
                                            login
                                        }}
                                    }}
                                }}
                                authorAssociation
                                createdAt
                                updatedAt
                            }}
                        }}
                    }}
                    pageInfo {{
                        endCursor
                        hasNextPage
                      }}
                }}
            }}
            "#,
            search_query,
            cursor
                .as_ref()
                .map_or(String::new(), |c| format!(r#", after: "{}""#, c))
        );

        let octocrab = get_octo(&GithubLogin::Default);
        let response: IssueRoot = octocrab.graphql(&query).await?;

        if let Some(search) = response.data.as_ref().and_then(|d| d.search.as_ref()) {
            if let Some(edges) = &search.edges {
                for edge in edges.iter().filter_map(|e| e.as_ref()) {
                    if let Some(issue) = &edge.node {
                        let date = match issue.created_at {
                            Some(date) => date.date_naive().to_string(),
                            None => {
                                continue;
                            }
                        };
                        let title_str = match &issue.title {
                            Some(title) => format!("Title: {},", title),
                            None => String::new(),
                        };
                        let url_str = match &issue.url {
                            Some(u) => format!("Url: {}", u),
                            None => String::new(),
                        };

                        let author_str = match issue.clone().author.and_then(|a| a.login) {
                            Some(auth) => format!("Author: {},", auth),
                            None => String::new(),
                        };

                        let assignees_str = {
                            let assignee_names = issue
                                .assignees
                                .as_ref()
                                .and_then(|e| e.edges.as_ref())
                                .map_or(Vec::new(), |assignee_edges| {
                                    assignee_edges
                                        .iter()
                                        .filter_map(|edge| {
                                            edge.as_ref().and_then(|actual_edge| {
                                                actual_edge.node.as_ref().and_then(|user| {
                                                    user.login
                                                        .as_ref()
                                                        .map(|login_str| login_str.as_str())
                                                })
                                            })
                                        })
                                        .collect::<Vec<&str>>()
                                });

                            if !assignee_names.is_empty() {
                                format!("Assignees: {},", assignee_names.join(", "))
                            } else {
                                String::new()
                            }
                        };

                        let state_str = match &issue.state {
                            Some(s) => format!("State: {},", s),
                            None => String::new(),
                        };

                        let body_str = match &issue.body {
                            Some(body_text) if body_text.len() > 180 => {
                                let truncated_body = body_text
                                    .chars()
                                    .take(100)
                                    .chain(body_text.chars().skip(body_text.chars().count() - 80))
                                    .collect::<String>();

                                format!("Body: {}", truncated_body)
                            }
                            Some(body_text) => format!("Body: {},", body_text),
                            None => String::new(),
                        };

                        let assoc_str = match &issue.author_association {
                            Some(association) => {
                                format!("Author Association: {}", association)
                            }
                            None => String::new(),
                        };

                        let temp = format!(
                                "{title_str} {url_str} Created At: {date} {author_str} {assignees_str}  {state_str} {body_str} {assoc_str}"
                            );

                        out.push_str(&temp);
                        out.push_str("\n");
                    } else {
                        continue;
                    }
                }
            }

            if let Some(page_info) = &search.page_info {
                if let Some(has_next_page) = page_info.has_next_page {
                    if has_next_page {
                        match &page_info.end_cursor {
                            Some(end_cursor) => {
                                cursor = Some(end_cursor.clone());
                                log::info!(
                                    "Fetched a page, moving to next page with cursor: {}",
                                    end_cursor
                                );
                                continue;
                            }
                            None => {
                                log::error!(
                                        "Warning: hasNextPage is true, but endCursor is None. This might result in missing data."
                                    );
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(out)
}

pub async fn search_repository(search_query: &str) -> anyhow::Result<String> {
    #[derive(Debug, Deserialize)]
    struct Payload {
        data: Option<Data>,
    }

    #[derive(Debug, Deserialize)]
    struct Data {
        search: Option<Search>,
    }

    #[derive(Debug, Deserialize)]
    struct Search {
        edges: Option<Vec<Option<Edge>>>,
        #[serde(rename = "pageInfo")]
        page_info: Option<PageInfo>,
    }

    #[derive(Debug, Deserialize)]
    struct Edge {
        node: Option<Node>,
    }

    #[derive(Debug, Deserialize)]
    struct Node {
        name: Option<String>,
        description: Option<String>,
        url: Option<String>,
        #[serde(rename = "createdAt")]
        created_at: Option<DateTime<Utc>>,
        stargazers: Option<Stargazers>,
        #[serde(rename = "forkCount")]
        fork_count: Option<u32>,
    }

    #[derive(Debug, Deserialize)]
    struct Stargazers {
        #[serde(rename = "totalCount")]
        total_count: Option<u32>,
    }

    #[derive(Debug, Deserialize)]
    struct PageInfo {
        #[serde(rename = "endCursor")]
        end_cursor: Option<String>,
        #[serde(rename = "hasNextPage")]
        has_next_page: Option<bool>,
    }

    let octocrab = get_octo(&GithubLogin::Default);
    let mut out = String::from("REPOSITORY \n");

    let mut cursor: Option<String> = None;
    let mut has_next_page = true;

    while has_next_page {
        let query = format!(
            r#"
            query {{
                search(query: "{search_query}", type: REPOSITORY, first: 100{after}) {{
                    edges {{
                        node {{
                            ... on Repository {{
                                name
                                description
                                url
                                createdAt
                                stargazers {{
                                    totalCount
                                }}
                                forkCount
                            }}
                        }}
                    }}
                    pageInfo {{
                        endCursor
                        hasNextPage
                    }}
                }}
            }}
            "#,
            search_query = search_query,
            after = cursor
                .as_ref()
                .map_or(String::new(), |c| format!(r#", after: "{}""#, c))
        );

        let response: Payload = octocrab.graphql(&query).await?;

        if let Some(data) = &response.data {
            if let Some(search) = &data.search {
                if let Some(edges) = &search.edges {
                    for edge_option in edges {
                        if let Some(edge) = edge_option {
                            if let Some(repo) = &edge.node {
                                let date_str = match &repo.created_at {
                                    Some(date) => date.date_naive().to_string(),
                                    None => {
                                        continue;
                                    }
                                };

                                let name_str = match &repo.name {
                                    Some(name) => format!("Name: {name},"),
                                    None => String::new(),
                                };

                                let desc_str = match &repo.description {
                                    Some(desc) if desc.len() > 300 => {
                                        let truncated_desc = desc
                                            .chars()
                                            .take(180)
                                            .chain(desc.chars().skip(desc.chars().count() - 120))
                                            .collect::<String>();

                                        format!("Description: {truncated_desc}")
                                    }
                                    Some(desc) => format!("Description: {desc},"),
                                    None => String::new(),
                                };

                                let url_str = match &repo.url {
                                    Some(url) => format!("Url: {url}"),
                                    None => String::new(),
                                };

                                let stars_str = match &repo.stargazers {
                                    Some(sg) => format!("Stars: {},", sg.total_count.unwrap_or(0)),
                                    None => String::new(),
                                };

                                let forks_str = match &repo.fork_count {
                                    Some(fork_count) => format!("Forks: {fork_count}"),
                                    None => String::new(),
                                };

                                out.push_str(
                                        &format!(
                                            "{name_str} {desc_str} {url_str} Created At: {date_str} {stars_str} {forks_str}\n"
                                        )
                                    );
                            }
                        }
                    }
                }
                if let Some(page_info) = &search.page_info {
                    if page_info.has_next_page.unwrap_or(false) {
                        cursor = page_info.end_cursor.clone();
                    } else {
                        break;
                    }
                }
            }
        };
    }

    Ok(out)
}

pub async fn search_discussions_integrated(
    search_query: &str,
    target_person: &Option<String>,
) -> anyhow::Result<(String, Vec<GitMemory>)> {
    #[derive(Debug, Deserialize)]
    struct DiscussionRoot {
        data: Option<Data>,
    }

    #[derive(Debug, Deserialize)]
    struct Data {
        search: Option<Search>,
    }

    #[derive(Debug, Deserialize)]
    struct Search {
        edges: Option<Vec<Option<Edge>>>,
    }

    #[derive(Debug, Deserialize)]
    struct Edge {
        node: Option<Discussion>,
    }

    #[derive(Debug, Deserialize)]
    struct Discussion {
        title: Option<String>,
        url: Option<String>,
        html_url: Option<String>,
        author: Option<Author>,
        body: Option<String>,
        comments: Option<Comments>,
        #[serde(rename = "createdAt")]
        created_at: DateTime<Utc>,
        #[serde(rename = "upvoteCount")]
        upvote_count: Option<u32>,
    }

    #[derive(Debug, Deserialize)]
    struct Comments {
        edges: Option<Vec<Option<CommentEdge>>>,
    }

    #[derive(Debug, Deserialize)]
    struct CommentEdge {
        node: Option<CommentNode>,
    }

    #[derive(Debug, Deserialize)]
    struct CommentNode {
        author: Option<Author>,
        body: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    struct Author {
        login: Option<String>,
    }

    let query = format!(
        r#"
        query {{
            search(query: "{search_query}", type: DISCUSSION, first: 100) {{
                edges {{
                    node {{
                        ... on Discussion {{
                            title
                            url
                            html_url
                            body
                            author {{
                                login
                            }}
                            createdAt
                            upvoteCount
                            comments (first: 100) {{
                                edges {{
                                    node {{
                                        author {{
                                            login
                                        }}
                                        body
                                    }}
                                }}
                            }}
                        }}
                    }}
                }}
            }}
        }}
        "#,
        search_query = search_query
    );
    let mut git_mem_vec = Vec::with_capacity(100);
    let mut text_out = String::from("DISCUSSIONS \n");

    let octocrab = get_octo(&GithubLogin::Default);
    let response: DiscussionRoot = octocrab.graphql(&query).await?;
    let empty_str = "".to_string();

    if let Some(search) = response
        .data
        .ok_or_else(|| anyhow::Error::msg("Missing data in the response"))?
        .search
    {
        for edge_option in search
            .edges
            .ok_or_else(|| anyhow::Error::msg("Missing edges in the response"))?
            .iter()
            .filter_map(|e| e.as_ref())
        {
            if let Some(discussion) = &edge_option.node {
                let date = discussion.created_at.date_naive();
                let title = discussion.title.as_ref().unwrap_or(&empty_str).to_string();
                let url = discussion.url.as_ref().unwrap_or(&empty_str).to_string();
                let source_url = discussion
                    .html_url
                    .as_ref()
                    .unwrap_or(&empty_str)
                    .to_string();
                let author_login = discussion
                    .author
                    .as_ref()
                    .and_then(|a| a.login.as_ref())
                    .unwrap_or(&empty_str)
                    .to_string();

                let upvotes_str = match discussion.upvote_count {
                    Some(c) if c > 0 => format!("Upvotes: {}", c),
                    _ => "".to_string(),
                };
                let body_text = match discussion.body.as_ref() {
                    Some(text) => squeeze_fit_remove_quoted(&text, 500, 0.6),
                    None => "".to_string(),
                };
                let mut disuccsion_texts = format!(
                    "Title: '{}' Url: '{}' Body: '{}' Created At: {} {} Author: {}\n",
                    title, url, body_text, date, upvotes_str, author_login
                );

                if let Some(comments) = &discussion.comments {
                    if let Some(ref edges) = comments.edges {
                        for comment_edge_option in edges.iter().filter_map(|e| e.as_ref()) {
                            if let Some(comment) = &comment_edge_option.node {
                                let stripped_comment_text = squeeze_fit_remove_quoted(
                                    &comment.body.as_ref().unwrap_or(&empty_str),
                                    300,
                                    0.6,
                                );
                                let comment_author = comment
                                    .author
                                    .as_ref()
                                    .and_then(|a| a.login.as_ref())
                                    .unwrap_or(&empty_str);
                                disuccsion_texts.push_str(
                                    &(format!(
                                        "{comment_author} comments: '{stripped_comment_text}'\n"
                                    )),
                                );
                            }
                        }
                    }
                }
                let disuccsion_texts = squeeze_fit_post_texts(&disuccsion_texts, 12_000, 0.4);

                let target_str = match &target_person {
                    Some(person) => format!("{}'s", person),
                    None => "key participants'".to_string(),
                };

                let sys_prompt_1 = &format!(
                            "Analyze the provided GitHub discussion. Identify the main topic, actions by participants, crucial viewpoints, solutions or consensus reached, and particularly highlight the contributions of specific individuals, especially '{target_str}'. Summarize without being verbose."
                        );

                let usr_prompt_1 = &format!(
                            "Analyze the content: {disuccsion_texts}. Briefly summarize the central topic, participants' actions, primary viewpoints, and outcomes. Emphasize the role of '{target_str}' in driving the discussion or reaching a resolution. Aim for a succinct summary that is rich in analysis and under 192 tokens."
                        );

                match chat_inner(sys_prompt_1, usr_prompt_1, 256, "gpt-3.5-turbo-1106").await {
                    Ok(r) => {
                        text_out.push_str(&(format!("{} {}", url, r)));
                        git_mem_vec.push(GitMemory {
                            memory_type: MemoryType::Discussion,
                            name: author_login,
                            tag_line: title,
                            source_url: source_url,
                            payload: r,
                        });
                    }

                    Err(_e) => log::error!("Error generating discussion summary #{}: {}", url, _e),
                }
            }
        }
    }

    if git_mem_vec.is_empty() {
        Err(anyhow::anyhow!("No results found.").into())
    } else {
        Ok((text_out, git_mem_vec))
    }
}

pub async fn search_users(search_query: &str) -> anyhow::Result<String> {
    #[derive(Debug, Deserialize)]
    struct User {
        name: Option<String>,
        login: Option<String>,
        url: Option<String>,
        #[serde(rename = "twitterUsername")]
        twitter_username: Option<String>,
        bio: Option<String>,
        company: Option<String>,
        location: Option<String>,
        #[serde(rename = "createdAt")]
        created_at: Option<DateTime<Utc>>,
        email: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    struct UserNode {
        node: Option<User>,
    }

    #[derive(Debug, Deserialize)]
    struct UserEdge {
        edges: Option<Vec<Option<UserNode>>>,
    }

    #[derive(Debug, Deserialize)]
    struct UserSearch {
        search: Option<UserEdge>,
    }

    #[derive(Debug, Deserialize)]
    struct UserRoot {
        data: Option<UserSearch>,
    }

    let mut out = String::from("USERS: \n");
    let octocrab = get_octo(&GithubLogin::Default);

    let query = format!(
        r#"
        query {{
            search(query: "{search_query}", type: USER, first: 100) {{
                edges {{
                    node {{
                        ... on User {{
                            name
                            login
                            url
                            twitterUsername
                            bio
                            company
                            location
                            createdAt
                            email
                        }}
                    }}
                }}
            }}
        }}
        "#,
        search_query = search_query
    );

    let response: UserRoot = octocrab.graphql(&query).await?;

    if let Some(search) = &response.data {
        if let Some(edges) = &search.search {
            for edge_option in edges.edges.as_ref().unwrap_or(&vec![]) {
                if let Some(edge) = edge_option {
                    if let Some(user) = &edge.node {
                        let login_str = match &user.login {
                            Some(login) => format!("Login: {},", login),
                            None => {
                                continue;
                            }
                        };
                        let name_str = match &user.name {
                            Some(name) => format!("Name: {},", name),
                            None => String::new(),
                        };

                        let url_str = match &user.url {
                            Some(url) => format!("Url: {},", url),
                            None => String::new(),
                        };

                        let twitter_str = match &user.twitter_username {
                            Some(twitter) => format!("Twitter: {},", twitter),
                            None => String::new(),
                        };

                        let bio_str = match &user.bio {
                            Some(bio) => format!("Bio: {},", bio),
                            None => String::new(),
                        };

                        let company_str = match &user.company {
                            Some(company) => format!("Company: {},", company),
                            None => String::new(),
                        };

                        let location_str = match &user.location {
                            Some(location) => format!("Location: {},", location),
                            None => String::new(),
                        };

                        let date_str = match &user.created_at {
                            Some(date) => {
                                format!("Created At: {},", date.date_naive().to_string())
                            }
                            None => String::new(),
                        };

                        let email_str = match &user.email {
                            Some(email) => format!("Email: {}", email),
                            None => String::new(),
                        };

                        out.push_str(
                                &format!(
                                    "{name_str} {login_str} {url_str} {twitter_str} {bio_str} {company_str} {location_str} {date_str} {email_str}\n"
                                )
                            );
                    }
                }
            }
        }
    }

    Ok(out)
}
