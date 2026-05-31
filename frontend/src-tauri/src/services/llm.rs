use futures_util::StreamExt;
use reqwest::Client;

use super::config::AiConfig;
use super::notes::AppError;
use super::types::ChatMessage;

// ─── LLM Client State ─────────────────────────────────────────────────────

pub struct LlmClientState {
    pub client: Client,
}

impl LlmClientState {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap_or_default(),
        }
    }
}

impl Default for LlmClientState {
    fn default() -> Self {
        Self::new()
    }
}

// ─── LLM API Calls ────────────────────────────────────────────────────────

/// Non-streaming LLM call. Returns the assistant message content.
pub async fn call_llm(
    client: &Client,
    config: &AiConfig,
    messages: &[ChatMessage],
    model: Option<&str>,
    temperature: f64,
    max_tokens: u32,
) -> Result<String, AppError> {
    if config.llm_api_key.is_empty() {
        return Err(AppError::new("llmConfig", "LLM API key is not configured"));
    }

    let model = model.unwrap_or(&config.llm_model);
    let url = format!(
        "{}/chat/completions",
        config.llm_base_url.trim_end_matches('/')
    );

    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "temperature": temperature,
        "max_tokens": max_tokens,
    });

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.llm_api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::new("llm", format!("LLM API request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::new(
            "llm",
            format!("LLM API error {status}: {text}"),
        ));
    }

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::new("llmParse", format!("Failed to parse LLM response: {e}")))?;

    let content = data["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(content)
}

/// Cheap model call (for emotion extraction, diary generation, etc.)
pub async fn call_cheap_llm(
    client: &Client,
    config: &AiConfig,
    messages: &[ChatMessage],
    temperature: f64,
    max_tokens: u32,
) -> Result<String, AppError> {
    call_llm(
        client,
        config,
        messages,
        Some(&config.llm_cheap_model),
        temperature,
        max_tokens,
    )
    .await
}

/// Streaming LLM call. Returns a stream of token strings.
pub fn stream_llm(
    client: Client,
    config: AiConfig,
    messages: Vec<ChatMessage>,
    model: Option<String>,
    temperature: f64,
    max_tokens: u32,
) -> impl futures_util::Stream<Item = Result<String, AppError>> {
    let model = model.unwrap_or(config.llm_model.clone());
    let url = format!(
        "{}/chat/completions",
        config.llm_base_url.trim_end_matches('/')
    );
    let api_key = config.llm_api_key.clone();

    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "temperature": temperature,
        "max_tokens": max_tokens,
        "stream": true,
    });

    async_stream::stream! {
        if api_key.is_empty() {
            yield Err(AppError::new("llmConfig", "LLM API key is not configured"));
            return;
        }

        let resp = match client
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                yield Err(AppError::new("llm", format!("LLM stream request failed: {e}")));
                return;
            }
        };

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            yield Err(AppError::new("llm", format!("LLM stream error {status}: {text}")));
            return;
        }

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = match chunk_result {
                Ok(c) => c,
                Err(e) => {
                    yield Err(AppError::new("llm", format!("LLM stream read error: {e}")));
                    return;
                }
            };

            buffer.push_str(&String::from_utf8_lossy(&chunk));

            // Process complete SSE lines
            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].trim().to_string();
                buffer = buffer[pos + 1..].to_string();

                if !line.starts_with("data: ") {
                    continue;
                }

                let data = &line[6..];
                if data == "[DONE]" {
                    return;
                }

                match serde_json::from_str::<serde_json::Value>(data) {
                    Ok(chunk_json) => {
                        if let Some(content) = chunk_json["choices"][0]["delta"]["content"].as_str()
                        {
                            if !content.is_empty() {
                                yield Ok(content.to_string());
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_client_state_creation() {
        let state = LlmClientState::new();
        // Client was created successfully
        let _ = &state.client;
    }

    #[test]
    fn test_missing_api_key_error() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let client = Client::new();
            let config = AiConfig {
                llm_api_key: String::new(),
                ..AiConfig::default()
            };
            let messages = vec![ChatMessage {
                role: "user".to_string(),
                content: "test".to_string(),
            }];
            let result = call_llm(&client, &config, &messages, None, 0.7, 100).await;
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.code, "llmConfig");
        });
    }

    #[test]
    fn test_sse_parsing() {
        // Verify the SSE line parsing logic handles typical chunks
        let line =
            r#"data: {"id":"chatcmpl-123","choices":[{"delta":{"content":"Hello"},"index":0}]}"#;
        assert!(line.starts_with("data: "));
        let data = &line[6..];
        assert_ne!(data, "[DONE]");
        let parsed: serde_json::Value = serde_json::from_str(data).unwrap();
        let content = parsed["choices"][0]["delta"]["content"].as_str().unwrap();
        assert_eq!(content, "Hello");
    }

    #[test]
    fn test_sse_done_detection() {
        let line = "data: [DONE]";
        let data = &line[6..];
        assert_eq!(data, "[DONE]");
    }
}
