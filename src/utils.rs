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
            // log::info!("{:?}", res);
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
            // log::info!("{:?}", res);
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
    use tokio::time::Instant;
    let start_time = Instant::now();
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
            // log::info!("{:?}", chat.choices[0].message.clone());
            // let elapsed = start_time.elapsed();
            // log::info!(
            //     "Time elapsed in chat_inner is: {} seconds",
            //     elapsed.as_secs(),
            // );
            Ok(res)
        }
        None => Err(anyhow::anyhow!("Failed to get reply from OpenAI")),
    }
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

pub async fn github_http_post_gql(query: &str) -> anyhow::Result<Vec<u8>> {
    use http_req::{request::Method, request::Request, uri::Uri};
    let token = std::env::var("GITHUB_TOKEN").expect("github_token is required");
    let base_url = "https://api.github.com/graphql";
    let base_url = Uri::try_from(base_url).unwrap();
    let mut writer = Vec::new();

    let query = serde_json::json!({"query": query});
    match Request::new(&base_url)
        .method(Method::POST)
        .header("User-Agent", "flows-network connector")
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Length", &query.to_string().len())
        .body(&query.to_string().into_bytes())
        .send(&mut writer)
    {
        Ok(res) => {
            if !res.status_code().is_success() {
                log::error!("Github http error {:?}", res.status_code());
                return Err(anyhow::anyhow!("Github http error {:?}", res.status_code()));
            };
            Ok(writer)
        }
        Err(_e) => {
            log::error!("Error getting response from Github: {:?}", _e);
            Err(anyhow::anyhow!(_e))
        }
    }
}

pub async fn github_http_get(url: &str) -> anyhow::Result<Vec<u8>> {
    use http_req::{request::Method, request::Request, uri::Uri};
    let token = std::env::var("GITHUB_TOKEN").expect("github_token is required");
    let mut writer = Vec::new();
    let url = Uri::try_from(url).unwrap();

    match Request::new(&url)
        .method(Method::GET)
        .header("User-Agent", "flows-network connector")
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .header("CONNECTION", "close")
        .send(&mut writer)
    {
        Ok(res) => {
            if !res.status_code().is_success() {
                log::error!("Github http error {:?}", res.status_code());
                return Err(anyhow::anyhow!("Github http error {:?}", res.status_code()));
            };
            Ok(writer)
        }
        Err(_e) => {
            log::error!("Error getting response from Github: {:?}", _e);
            Err(anyhow::anyhow!(_e))
        }
    }
}
