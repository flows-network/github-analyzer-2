pub mod data_analyzers;
pub mod github_data_fetchers;
pub mod reports;
pub mod utils;
use data_analyzers::{get_repo_info, get_repo_overview_by_scraper};
use dotenv::dotenv;
use flowsnet_platform_sdk::logger;
use github_data_fetchers::get_user_data_by_login;
use reports::*;
use serde_json::Value;
use slack_flows::send_message_to_channel;
use std::collections::HashMap;
use std::env;
use webhook_flows::{create_endpoint, request_handler, send_response};

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    create_endpoint().await;
}

#[request_handler]
async fn handler(
    _headers: Vec<(String, String)>,
    _subpath: String,
    _qry: HashMap<String, Value>,
    _body: Vec<u8>,
) {
    dotenv().ok();
    logger::init();

    let about_repo = _qry
        .get("about_repo")
        .unwrap_or(&Value::Null)
        .as_str()
        .map(|n| n.to_string());

    if let Some(about_repo) = about_repo {
        match get_repo_overview_by_scraper(&about_repo).await {
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
            Some(summary) => {
                let _profile_data = format!("About {}: {}", about_repo, summary);
                send_response(
                    200,
                    vec![(String::from("content-type"), String::from("text/plain"))],
                    _profile_data.as_bytes().to_vec(),
                )
            }
        }
        return;
    }

    let (owner, repo) = match (
        _qry.get("owner").unwrap_or(&Value::Null).as_str(),
        _qry.get("repo").unwrap_or(&Value::Null).as_str(),
    ) {
        (Some(o), Some(r)) => (o.to_string(), r.to_string()),
        (_, _) => {
            send_response(
                400,
                vec![(String::from("content-type"), String::from("text/plain"))],
                "You must provide an owner and repo name."
                    .as_bytes()
                    .to_vec(),
            );
            return;
        }
    };

    let user_name = _qry
        .get("username")
        .unwrap_or(&Value::Null)
        .as_str()
        .map(|n| n.to_string());
    let token = _qry
        .get("token")
        .unwrap_or(&Value::Null)
        .as_str()
        .map(|n| n.to_string());

    let output = weekly_report(&owner, &repo, user_name, token.clone()).await;

    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/plain"))],
        output.as_bytes().to_vec(),
    );
    // send_message_to_channel("ik8", "ch_err", output.clone()).await;
}
