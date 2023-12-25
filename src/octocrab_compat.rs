use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Issue {
    pub node_id: String,
    pub url: String,
    pub repository_url: String,
    pub labels_url: String,
    pub comments_url: String,
    pub events_url: String,
    pub html_url: String,
    pub number: u64,
    pub state: IssueState,
    pub state_reason: Option<IssueStateReason>,
    pub title: String,
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_html: Option<String>,
    pub user: User,
    pub labels: Vec<Label>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<User>,
    pub assignees: Vec<User>,
    pub author_association: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone: Option<Milestone>,
    pub locked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_lock_reason: Option<String>,
    pub comments: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_request: Option<PullRequestLink>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Comment {
    pub node_id: String,
    pub url: String,
    pub html_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_html: Option<String>,
    pub user: User,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum IssueStateReason {
    Completed,
    NotPlanned,
    Reopened,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PullRequestLink {
    pub url: String,
    pub html_url: String,
    pub diff_url: String,
    pub patch_url: String,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Contents {
    #[serde(rename = "type")]
    pub contents_type: String,
    pub encoding: String,
    pub size: u64,
    pub name: String,
    pub path: String,
    pub content: String,
    pub sha: String,
    pub url: String,
    pub git_url: String,
    pub html_url: String,
    pub download_url: String,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Event {
    AddedToProject,
    Assigned,
    Closed,
    ConvertedNoteToIssue,
    Demilestoned,
    HeadRefDeleted,
    HeadRefForcePushed,
    HeadRefRestored,
    Labeled,
    Locked,
    Mentioned,
    MarkedAsDuplicate,
    Merged,
    Milestoned,
    MovedColumnsInProject,
    Referenced,
    RemovedFromProject,
    Renamed,
    Reopened,
    ReviewDismissed,
    ReviewRequested,
    ReviewRequestRemoved,
    Subscribed,
    Transferred,
    Unassigned,
    Unlabeled,
    Unlocked,
    UnmarkedAsDuplicate,
    UserBlocked,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum IssueState {
    Open,
    Closed,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IssueEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub actor: User,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<User>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigner: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<Label>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone: Option<Milestone>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_card: Option<ProjectCard>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<Event>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ProjectCard {
    pub url: String,
    pub project_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_column_name: Option<String>,
    pub column_url: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Project {
    pub owner_url: String,
    pub url: String,
    pub html_url: String,
    pub columns_url: String,
    pub node_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    pub number: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    pub creator: User,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
       #[serde(rename = "updatedAt")]
   pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ProjectCardContentType {
    Issue,
    PullRequest,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ProjectColumn {
    pub url: String,
    pub project_url: String,
    pub cards_url: String,
    pub node_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
       #[serde(rename = "updatedAt")]
   pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IssuePullRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch_url: Option<String>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct User {
    pub login: String,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    pub r#type: String,
    pub site_admin: bool,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StarGazer {
    pub starred_at: Option<DateTime<Utc>>,
    pub user: Option<User>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Label {
    pub url: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub color: String,
    pub default: bool,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Milestone {
    pub url: String,
    pub html_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels_url: Option<String>,
    pub node_id: String,
    pub number: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_issues: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_issues: Option<i64>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
       #[serde(rename = "updatedAt")]
   pub updated_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_on: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Repository {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fork: Option<bool>,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blobs_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branches_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collaborators_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commits_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compare_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contents_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contributors_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployments_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forks_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_commits_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_refs_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_tags_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_comment_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_events_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issues_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub languages_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merges_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestones_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notifications_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pulls_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub releases_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stargazers_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statuses_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribers_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teams_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trees_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clone_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub svn_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<::serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forks_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stargazers_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watchers_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_issues_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_template: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_issues: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_projects: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_wiki: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_pages: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_downloads: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pushed_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
       #[serde(rename = "updatedAt")]
   pub updated_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permissions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_rebase_merge: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_repository: Option<Box<Repository>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_squash_merge: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_merge_commit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribers_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<License>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_auto_merge: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_branch_on_merge: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Box<Repository>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Box<Repository>>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct RepositoryFile {
    pub name: Option<String>,
    pub key: Option<String>,
    pub url: Option<String>,
    pub html_url: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RepositoryMetrics {
    pub health_percentage: u64,
    pub description: Option<String>,
    pub documentation: Option<String>,
    pub files: HashMap<String, Option<RepositoryFile>>,
       #[serde(rename = "updatedAt")]
   pub updated_at: Option<DateTime<Utc>>,
    pub content_reports_enabled: Option<bool>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct License {
    pub key: String,
    pub name: String,
    pub node_id: String,
    pub spdx_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub html_url: Option<String>,
    pub description: Option<String>,
    pub implementation: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub conditions: Option<Vec<String>>,
    pub limitations: Option<Vec<String>>,
    pub body: Option<String>,
    pub featured: Option<bool>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Code {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub url: String,
    pub git_url: String,
    pub html_url: String,
    pub repository: Repository,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Permissions {
    #[serde(default)]
    pub admin: bool,
    pub push: bool,
    pub pull: bool,
    #[serde(default)]
    pub triage: bool,
    #[serde(default)]
    pub maintain: bool,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CheckRuns {
    pub total_count: i32,
    pub check_runs: Vec<CheckRun>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CheckRun {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head_sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<CheckStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conclusion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum CheckStatus {
    Queued,
    Completed,
    InProgress,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CombinedStatus {
    pub state: StatusState,
    pub sha: String,
    pub total_count: i64,
    pub statuses: Vec<Status>,
    #[serde(skip_serializing)]
    pub repository: Option<Repository>,
    #[serde(skip_serializing)]
    pub commit_url: Option<String>,
    #[serde(skip_serializing)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Status {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
       #[serde(rename = "updatedAt")]
   pub updated_at: Option<DateTime<Utc>>,
    pub state: StatusState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum StatusState {
    Failure,
    Pending,
    Success,
    Error,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InstallationRepositories {
    pub total_count: i64,
    pub repositories: Vec<Repository>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct Installation {
    pub account: User,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_tokens_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repositories_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_type: Option<String>,
    pub permissions: InstallationPermissions,
    pub events: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_selection: Option<String>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct InstallationPermissions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contents: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issues: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statuses: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct InstallationToken {
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    pub permissions: InstallationPermissions,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repositories: Option<Vec<Repository>>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct PublicKey {
    pub key_id: String,
    pub key: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateLimit {
    pub resources: Resources,
    pub rate: Rate,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resources {
    pub core: Rate,
    pub search: Rate,
    pub graphql: Option<Rate>,
    pub integration_manifest: Option<Rate>,
    pub scim: Option<Rate>,
    pub source_import: Option<Rate>,
    pub code_scanning_upload: Option<Rate>,
    pub actions_runner_registration: Option<Rate>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rate {
    pub limit: usize,
    pub used: usize,
    pub remaining: usize,
    pub reset: usize,
}
