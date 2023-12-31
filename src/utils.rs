use async_openai::{
    types::{
        // ChatCompletionFunctionsArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs,
        // ChatCompletionTool, ChatCompletionToolArgs, ChatCompletionToolType,
        CreateChatCompletionRequestArgs,
        // FinishReason,
    },
    Client,
};
use log;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;
use store_flows::{get, set};
/*
use crypto::{symmetriccipher, buffer, aes, blockmodes};
use crypto::buffer::{ReadBuffer, WriteBuffer, BufferResult};
use rand::Rng;

use std::str;
use hex::{encode, decode};
use brotli::{BrotliCompress, BrotliDecompress};
use std::io::{Read, Write};

fn gen_key(url: &str, username: &str) -> String {
    let combined = format!("{}+{}", url, username);
    let encrypted_bytes = encrypt(combined.as_bytes(), KEY, IV).expect("Failed to encrypt");
    hex::encode(&encrypted_bytes)
}

fn get_vals(hex_key: &str) -> (String, String) {
    let encrypted_bytes = hex::decode(&hex_key).expect("Failed to decode hex");
    let decrypted_bytes = decrypt(&encrypted_bytes, KEY, IV).expect("Failed to decrypt");

    let decrypted_str = str::from_utf8(&decrypted_bytes).unwrap();
    let parts: Vec<&str> = decrypted_str.split('+').collect();
    (parts[0].to_string(), parts[1].to_string())
}



 */

pub fn squeeze_fit_remove_quoted(inp_str: &str, max_len: u16, split: f32) -> String {
    let mut body = String::new();
    let mut inside_quote = false;

    for line in inp_str.lines() {
        if line.contains("```") || line.contains("\"\"\"") {
            inside_quote = !inside_quote;
            continue;
        }

        if !inside_quote {
            let cleaned_line = line
                .split_whitespace()
                .filter(|word| word.len() < 150)
                .collect::<Vec<&str>>()
                .join(" ");
            body.push_str(&cleaned_line);
            body.push('\n');
        }
    }

    let body_words: Vec<&str> = body.split_whitespace().collect();
    let body_len = body_words.len();
    let n_take_from_beginning = (body_len as f32 * split) as usize;
    let n_keep_till_end = body_len - n_take_from_beginning;

    // Range check for drain operation
    let drain_start = if n_take_from_beginning < body_len {
        n_take_from_beginning
    } else {
        body_len
    };

    let drain_end = if n_keep_till_end <= body_len {
        body_len - n_keep_till_end
    } else {
        0
    };

    let final_text = if body_len > max_len as usize {
        let mut body_text_vec = body_words.to_vec();
        body_text_vec.drain(drain_start..drain_end);
        body_text_vec.join(" ")
    } else {
        body
    };

    final_text
}

pub fn squeeze_fit_post_texts(inp_str: &str, max_len: u16, split: f32) -> String {
    let bpe = tiktoken_rs::cl100k_base().unwrap();

    let input_token_vec = bpe.encode_ordinary(inp_str);
    let input_len = input_token_vec.len();
    if input_len < max_len as usize {
        return inp_str.to_string();
    }
    // // Filter out the tokens corresponding to lines with undesired patterns
    // let mut filtered_tokens = Vec::new();
    // for line in inp_str.lines() {
    //     let mut tokens_for_line = bpe.encode_ordinary(line);
    //     if !line.contains("{{") && !line.contains("}}") {
    //         filtered_tokens.extend(tokens_for_line.drain(..));
    //     }
    // }
    let n_take_from_beginning = (input_len as f32 * split).ceil() as usize;
    let n_take_from_end = max_len as usize - n_take_from_beginning;

    let mut concatenated_tokens = Vec::with_capacity(max_len as usize);
    concatenated_tokens.extend_from_slice(&input_token_vec[..n_take_from_beginning]);
    concatenated_tokens.extend_from_slice(&input_token_vec[input_len - n_take_from_end..]);

    bpe.decode(concatenated_tokens)
        .ok()
        .map_or("failed to decode tokens".to_string(), |s| s.to_string())
}

pub async fn chain_of_chat(
    sys_prompt_1: &str,
    usr_prompt_1: &str,
    chat_id: &str,
    gen_len_1: u16,
    usr_prompt_2: &str,
    gen_len_2: u16,
    error_tag: &str,
) -> anyhow::Result<String> {
    let client = Client::new();

    let mut messages = vec![
        ChatCompletionRequestSystemMessageArgs::default()
            .content(sys_prompt_1)
            .build()
            .expect("Failed to build system message")
            .into(),
        ChatCompletionRequestUserMessageArgs::default()
            .content(usr_prompt_1)
            .build()?
            .into(),
    ];
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(gen_len_1)
        .model("gpt-3.5-turbo-1106")
        .messages(messages.clone())
        .build()?;

    let chat = client.chat().create(request).await?;

    match chat.choices[0].message.clone().content {
        Some(res) => {
            log::info!("{:?}", res);
        }
        None => return Err(anyhow::anyhow!(error_tag.to_string())),
    }

    messages.push(
        ChatCompletionRequestUserMessageArgs::default()
            .content(usr_prompt_2)
            .build()?
            .into(),
    );

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(gen_len_2)
        .model("gpt-3.5-turbo-1106")
        .messages(messages)
        .build()?;

    let chat = client.chat().create(request).await?;

    match chat.choices[0].message.clone().content {
        Some(res) => {
            log::info!("{:?}", res);
            Ok(res)
        }
        None => return Err(anyhow::anyhow!(error_tag.to_string())),
    }
}

pub async fn chat_inner(
    system_prompt: &str,
    user_input: &str,
    max_token: u16,
    model: &str,
) -> anyhow::Result<String> {
    let client = Client::new();

    let messages = vec![
        ChatCompletionRequestSystemMessageArgs::default()
            .content(system_prompt)
            .build()
            .expect("Failed to build system message")
            .into(),
        ChatCompletionRequestUserMessageArgs::default()
            .content(user_input)
            .build()?
            .into(),
    ];
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(max_token)
        .model(model)
        .messages(messages)
        .build()?;

    let chat = client.chat().create(request).await?;

    // let check = chat.choices.get(0).clone().unwrap();
    // send_message_to_channel("ik8", "general", format!("{:?}", check)).await;

    match chat.choices[0].message.clone().content {
        Some(res) => {
            log::info!("{:?}", chat.choices[0].message.clone());
            Ok(res)
        }
        None => Err(anyhow::anyhow!("Failed to get reply from OpenAI")),
    }
}

pub async fn save_user(owner: &str, repo: &str, user_name: &str) -> bool {
    use std::hash::Hasher;
    use twox_hash::XxHash;
    let repo_string = format!("{owner}/{repo}");
    let mut hasher = XxHash::with_seed(0);
    hasher.write(repo_string.as_bytes());
    let hash = hasher.finish();
    let key = &format!("{:x}", hash);

    let mut existing_users: HashSet<String> = get(key)
        .and_then(|val| serde_json::from_value(val).ok())
        .unwrap_or_else(HashSet::new);

    // Check if the user_name already exists
    let already_exists = existing_users.contains(user_name);

    // If the user_name is not in the set, add it
    if !already_exists {
        existing_users.insert(user_name.to_string());
    }

    // Save updated records
    set(
        key,
        Value::String(serde_json::to_string(&existing_users).unwrap()),
        None,
    );

    // If the user_name was added, return true; otherwise, return false
    !already_exists
}

pub fn custom_json_parser(input: &str) -> Option<String> {
    #[derive(Debug, Deserialize)]
    struct GitHubIssueSummary {
        principal_arguments: Option<Vec<String>>,
        suggested_solutions: Option<Vec<String>>,
        areas_of_consensus: Option<Vec<String>>,
        areas_of_disagreement: Option<Vec<String>>,
        concise_summary: Option<String>,
    }

    let mut parsed_data: std::collections::HashMap<String, serde_json::Value> =
        std::collections::HashMap::new();

    let lines: Vec<&str> = input.lines().collect();
    for line in lines {
        if line.trim().starts_with("\"") {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 {
                let key = parts[0].trim_matches(|c| c == '"' || c == ' ');
                let value: String = parts[1..].join(":");

                if value.len() >= 15 {
                    // Ignore if data is less than 15 characters
                    if let Ok(json_value) = serde_json::from_str(&value) {
                        parsed_data.insert(key.to_string(), json_value);
                    }
                }
            }
        }
    }

    let mut summary = GitHubIssueSummary {
        principal_arguments: None,
        suggested_solutions: None,
        areas_of_consensus: None,
        areas_of_disagreement: None,
        concise_summary: None,
    };

    if let Some(val) = parsed_data.get("principal_arguments") {
        if let Ok(converted) = serde_json::from_value(val.clone()) {
            summary.principal_arguments = Some(converted);
        }
    }

    if let Some(val) = parsed_data.get("suggested_solutions") {
        if let Ok(converted) = serde_json::from_value(val.clone()) {
            summary.suggested_solutions = Some(converted);
        }
    }

    if let Some(val) = parsed_data.get("areas_of_consensus") {
        if let Ok(converted) = serde_json::from_value(val.clone()) {
            summary.areas_of_consensus = Some(converted);
        }
    }

    if let Some(val) = parsed_data.get("areas_of_disagreement") {
        if let Ok(converted) = serde_json::from_value(val.clone()) {
            summary.areas_of_disagreement = Some(converted);
        }
    }

    if let Some(val) = parsed_data.get("concise_summary") {
        if let Ok(converted) = serde_json::from_value(val.clone()) {
            summary.concise_summary = Some(converted);
        }
    }

    Some(summary.concise_summary.unwrap_or("".to_string()))
}

pub fn parse_summary_from_raw_json(input: &str) -> anyhow::Result<String> {
    let parsed: Value = serde_json::from_str(input)?;

    let mut output = String::new();

    let keys = [
        "impactful",
        "alignment",
        "patterns",
        "synergy",
        "significance",
    ];

    for key in keys.iter() {
        if let Some(value) = parsed.get(key) {
            if value.is_string() {
                if !output.is_empty() {
                    output.push_str(" ");
                }
                output.push_str(value.as_str().unwrap());
            }
        }
    }

    Ok(output)
}
