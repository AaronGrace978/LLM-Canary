use serde_json::{json, Value};
use std::time::Duration;

use crate::models::{fallback_models, now, ChatMessage, Provider, TestResult};

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(90))
        .user_agent("LLM-Canary/0.4")
        .build()
        .expect("http client")
}

pub fn http() -> reqwest::Client {
    client()
}

/// Default sampling temperature for conversational use.
pub const CHAT_TEMPERATURE: f32 = 0.2;
/// Extraction probes run greedy so repeated trials measure the model, not the sampler.
pub const HUNT_TEMPERATURE: f32 = 0.0;

pub async fn chat_at(
    http: &reqwest::Client,
    p: &Provider,
    prompt: &str,
    temperature: f32,
) -> Result<String, String> {
    chat_messages_at(
        http,
        p,
        &[ChatMessage {
            role: "user".into(),
            content: prompt.to_string(),
        }],
        temperature,
    )
    .await
}

pub async fn chat_messages(
    http: &reqwest::Client,
    p: &Provider,
    messages: &[ChatMessage],
) -> Result<String, String> {
    chat_messages_at(http, p, messages, CHAT_TEMPERATURE).await
}

pub async fn chat_messages_at(
    http: &reqwest::Client,
    p: &Provider,
    messages: &[ChatMessage],
    temperature: f32,
) -> Result<String, String> {
    let temperature = if temperature.is_finite() { temperature.clamp(0.0, 2.0) } else { CHAT_TEMPERATURE };
    if p.model.trim().is_empty() {
        return Err("Pick a model first.".into());
    }
    if messages.is_empty() {
        return Err("Nothing to send.".into());
    }
    match p.kind.as_str() {
        "ollama" => ollama_chat(http, p, messages, temperature).await,
        "openai" => openai_chat(http, p, messages, false, temperature).await,
        "anthropic" => anthropic_chat(http, p, messages, temperature).await,
        "gemini" => gemini_chat(http, p, messages, temperature).await,
        _ => openai_compat_chat(http, p, messages, temperature).await,
    }
}

pub async fn list_models(http: &reqwest::Client, p: &Provider) -> Result<Vec<String>, String> {
    let fetched = match p.kind.as_str() {
        "ollama" => ollama_models(http, p).await,
        "openai" => openai_models(http, p).await,
        "anthropic" => anthropic_models(http, p).await,
        "gemini" => gemini_models(http, p).await,
        _ => openai_compat_models(http, p).await,
    };
    let mut models = fallback_models(&p.id);
    match fetched {
        Ok(list) => {
            for m in list {
                if !m.is_empty() && !models.contains(&m) {
                    models.push(m);
                }
            }
            Ok(models)
        }
        Err(e) => {
            if models.is_empty() {
                Err(e)
            } else {
                Ok(models)
            }
        }
    }
}

pub async fn test_provider(http: &reqwest::Client, p: &Provider) -> Result<TestResult, String> {
    let preview = chat_at(
        http,
        p,
        "Reply with the single word PONG and nothing else.",
        HUNT_TEMPERATURE,
    )
    .await?;
    Ok(TestResult {
        ok: true,
        model: p.model.clone(),
        preview: preview.chars().take(240).collect(),
    })
}

fn require_key(p: &Provider) -> Result<(), String> {
    if p.id == "custom" {
        if p.base_url.trim().is_empty() {
            return Err("Set a base URL.".into());
        }
        return Ok(());
    }
    if p.kind == "ollama" && p.base_url.contains("localhost") {
        return Ok(());
    }
    if p.api_key.trim().is_empty() {
        Err("Paste an API key first.".into())
    } else {
        Ok(())
    }
}

fn openai_style_messages(messages: &[ChatMessage]) -> Value {
    let arr: Vec<Value> = messages
        .iter()
        .filter(|m| {
            let role = m.role.trim();
            (role == "user" || role == "assistant" || role == "system") && !m.content.trim().is_empty()
        })
        .map(|m| {
            json!({
                "role": m.role.trim(),
                "content": m.content.clone(),
            })
        })
        .collect();
    Value::Array(arr)
}

async fn ollama_chat(
    http: &reqwest::Client,
    p: &Provider,
    messages: &[ChatMessage],
    temperature: f32,
) -> Result<String, String> {
    require_key(p)?;
    let url = format!("{}/api/chat", p.base_url.trim_end_matches('/'));
    let mut req = http.post(&url).json(&json!({
        "model": p.model,
        "messages": openai_style_messages(messages),
        "stream": false,
        "options": {"temperature": temperature, "num_predict": 1600}
    }));
    if !p.api_key.trim().is_empty() {
        req = req.bearer_auth(p.api_key.trim());
    }
    let v = send(req).await?;
    text_at(&v, &["message", "content"])
        .or_else(|| text_at(&v, &["response"]))
        .ok_or_else(|| "Ollama returned no content.".into())
}

async fn ollama_models(http: &reqwest::Client, p: &Provider) -> Result<Vec<String>, String> {
    let url = format!("{}/api/tags", p.base_url.trim_end_matches('/'));
    let mut req = http.get(&url);
    if !p.api_key.trim().is_empty() {
        req = req.bearer_auth(p.api_key.trim());
    }
    let v = send(req).await?;
    let mut out = Vec::new();
    if let Some(arr) = v.get("models").and_then(|x| x.as_array()) {
        for m in arr {
            if let Some(name) = m.get("name").and_then(|x| x.as_str()) {
                out.push(name.to_string());
            } else if let Some(name) = m.get("model").and_then(|x| x.as_str()) {
                out.push(name.to_string());
            }
        }
    }
    Ok(out)
}

async fn openai_chat(
    http: &reqwest::Client,
    p: &Provider,
    messages: &[ChatMessage],
    completion_tokens: bool,
    temperature: f32,
) -> Result<String, String> {
    require_key(p)?;
    let url = format!("{}/chat/completions", p.base_url.trim_end_matches('/'));
    let use_completion = completion_tokens
        || p.model.starts_with("gpt-5")
        || p.model.starts_with("o1")
        || p.model.starts_with("o3");
    let mut body = json!({
        "model": p.model,
        "messages": openai_style_messages(messages),
    });
    if use_completion {
        body["max_completion_tokens"] = json!(1600);
    } else {
        body["max_tokens"] = json!(1600);
        body["temperature"] = json!(temperature);
    }
    let req = http
        .post(&url)
        .bearer_auth(p.api_key.trim())
        .json(&body);
    match send(req).await {
        Ok(v) => openai_content(&v),
        Err(e) => {
            if !use_completion
                && (e.contains("max_completion_tokens") || e.contains("unsupported_parameter"))
            {
                Box::pin(openai_chat(http, p, messages, true, temperature)).await
            } else {
                Err(e)
            }
        }
    }
}

fn openai_content(v: &Value) -> Result<String, String> {
    if let Some(arr) = v["choices"].as_array() {
        if let Some(c) = arr.first() {
            if let Some(t) = c.pointer("/message/content").and_then(|x| x.as_str()) {
                return Ok(t.to_string());
            }
            if let Some(t) = c.get("text").and_then(|x| x.as_str()) {
                return Ok(t.to_string());
            }
        }
    }
    Err("No completion in response.".into())
}

async fn openai_models(http: &reqwest::Client, p: &Provider) -> Result<Vec<String>, String> {
    require_key(p)?;
    let url = format!("{}/models", p.base_url.trim_end_matches('/'));
    let v = send(http.get(&url).bearer_auth(p.api_key.trim())).await?;
    let mut out = Vec::new();
    if let Some(arr) = v.get("data").and_then(|x| x.as_array()) {
        for m in arr {
            if let Some(id) = m.get("id").and_then(|x| x.as_str()) {
                let idl = id.to_lowercase();
                if idl.contains("embed")
                    || idl.contains("whisper")
                    || idl.contains("tts")
                    || idl.contains("dall-e")
                    || idl.contains("audio")
                    || idl.contains("image")
                    || idl.contains("transcribe")
                    || idl.contains("moderation")
                    || idl.contains("realtime")
                    || idl.contains("sora")
                {
                    continue;
                }
                out.push(id.to_string());
            }
        }
    }
    out.sort();
    out.reverse();
    Ok(out)
}

async fn openai_compat_chat(
    http: &reqwest::Client,
    p: &Provider,
    messages: &[ChatMessage],
    temperature: f32,
) -> Result<String, String> {
    require_key(p)?;
    let base = p.base_url.trim_end_matches('/');
    let url = if base.ends_with("/v1") || base.contains("/api/v1") {
        format!("{base}/chat/completions")
    } else {
        format!("{base}/v1/chat/completions")
    };
    let body = json!({
        "model": p.model,
        "messages": openai_style_messages(messages),
        "max_tokens": 1600,
        "temperature": temperature
    });
    let mut req = http.post(&url).json(&body);
    if !p.api_key.trim().is_empty() {
        req = req.bearer_auth(p.api_key.trim());
    }
    if p.id == "openrouter" {
        req = req
            .header("HTTP-Referer", "https://llmcanary.app")
            .header("X-Title", "LLM Canary");
    }
    let v = send(req).await?;
    openai_content(&v)
}

async fn openai_compat_models(http: &reqwest::Client, p: &Provider) -> Result<Vec<String>, String> {
    if p.api_key.trim().is_empty() && p.id != "custom" {
        return Err("Paste an API key first.".into());
    }
    let base = p.base_url.trim_end_matches('/');
    let url = if base.ends_with("/v1") || base.contains("/api/v1") {
        format!("{base}/models")
    } else {
        format!("{base}/v1/models")
    };
    let mut req = http.get(&url);
    if !p.api_key.trim().is_empty() {
        req = req.bearer_auth(p.api_key.trim());
    }
    if p.id == "openrouter" {
        req = req
            .header("HTTP-Referer", "https://llmcanary.app")
            .header("X-Title", "LLM Canary");
    }
    let v = send(req).await?;
    let mut out = Vec::new();
    let arr = v
        .get("data")
        .and_then(|x| x.as_array())
        .or_else(|| v.get("models").and_then(|x| x.as_array()));
    if let Some(arr) = arr {
        for m in arr {
            if let Some(id) = m.get("id").and_then(|x| x.as_str()) {
                out.push(id.to_string());
            } else if let Some(id) = m.get("name").and_then(|x| x.as_str()) {
                out.push(id.to_string());
            }
        }
    }
    Ok(out)
}

async fn anthropic_chat(
    http: &reqwest::Client,
    p: &Provider,
    messages: &[ChatMessage],
    temperature: f32,
) -> Result<String, String> {
    require_key(p)?;
    let url = format!("{}/v1/messages", p.base_url.trim_end_matches('/'));
    let mut system = String::new();
    let mut api_messages: Vec<Value> = Vec::new();
    for m in messages {
        let role = m.role.trim();
        if role == "system" {
            if !system.is_empty() {
                system.push('\n');
            }
            system.push_str(&m.content);
            continue;
        }
        let mapped = if role == "assistant" { "assistant" } else { "user" };
        if let Some(last) = api_messages.last_mut() {
            let last_role = last["role"].as_str().unwrap_or("");
            if last_role == mapped {
                let existing = last["content"].as_str().unwrap_or("").to_string();
                last["content"] = json!(format!("{existing}\n\n{}", m.content));
                continue;
            }
        }
        api_messages.push(json!({
            "role": mapped,
            "content": m.content.clone(),
        }));
    }
    if api_messages.is_empty() {
        return Err("Nothing to send.".into());
    }
    let mut body = json!({
        "model": p.model,
        "max_tokens": 1600,
        "temperature": temperature.min(1.0),
        "messages": api_messages
    });
    if !system.trim().is_empty() {
        body["system"] = json!(system);
    }
    let req = http
        .post(&url)
        .header("x-api-key", p.api_key.trim())
        .header("anthropic-version", "2023-06-01")
        .json(&body);
    let v = send(req).await?;
    if let Some(arr) = v.get("content").and_then(|x| x.as_array()) {
        let mut text = String::new();
        for block in arr {
            if block.get("type").and_then(|x| x.as_str()) == Some("text") {
                if let Some(t) = block.get("text").and_then(|x| x.as_str()) {
                    text.push_str(t);
                }
            }
        }
        if !text.is_empty() {
            return Ok(text);
        }
    }
    Err("Anthropic returned no text.".into())
}

async fn anthropic_models(http: &reqwest::Client, p: &Provider) -> Result<Vec<String>, String> {
    require_key(p)?;
    let url = format!("{}/v1/models", p.base_url.trim_end_matches('/'));
    let v = send(
        http.get(&url)
            .header("x-api-key", p.api_key.trim())
            .header("anthropic-version", "2023-06-01"),
    )
    .await?;
    let mut out = Vec::new();
    if let Some(arr) = v.get("data").and_then(|x| x.as_array()) {
        for m in arr {
            if let Some(id) = m.get("id").and_then(|x| x.as_str()) {
                out.push(id.to_string());
            }
        }
    }
    Ok(out)
}

async fn gemini_chat(
    http: &reqwest::Client,
    p: &Provider,
    messages: &[ChatMessage],
    temperature: f32,
) -> Result<String, String> {
    require_key(p)?;
    let model = p.model.trim().trim_start_matches("models/");
    let url = format!(
        "{}/models/{}:generateContent?key={}",
        p.base_url.trim_end_matches('/'),
        model,
        urlencoding_lite(p.api_key.trim())
    );
    let mut contents = Vec::new();
    let mut system = String::new();
    for m in messages {
        let role = m.role.trim();
        if role == "system" {
            if !system.is_empty() {
                system.push('\n');
            }
            system.push_str(&m.content);
            continue;
        }
        let mapped = if role == "assistant" { "model" } else { "user" };
        contents.push(json!({
            "role": mapped,
            "parts": [{"text": m.content.clone()}]
        }));
    }
    if contents.is_empty() {
        return Err("Nothing to send.".into());
    }
    let mut body = json!({
        "contents": contents,
        "generationConfig": {"maxOutputTokens": 1600, "temperature": temperature}
    });
    if !system.trim().is_empty() {
        body["systemInstruction"] = json!({
            "parts": [{"text": system}]
        });
    }
    let req = http.post(&url).json(&body);
    let v = send(req).await?;
    if let Some(cands) = v.get("candidates").and_then(|x| x.as_array()) {
        if let Some(parts) = cands
            .first()
            .and_then(|c| c.pointer("/content/parts"))
            .and_then(|x| x.as_array())
        {
            let mut text = String::new();
            for part in parts {
                if let Some(t) = part.get("text").and_then(|x| x.as_str()) {
                    text.push_str(t);
                }
            }
            if !text.is_empty() {
                return Ok(text);
            }
        }
    }
    Err("Gemini returned no text.".into())
}

async fn gemini_models(http: &reqwest::Client, p: &Provider) -> Result<Vec<String>, String> {
    require_key(p)?;
    let url = format!(
        "{}/models?key={}",
        p.base_url.trim_end_matches('/'),
        urlencoding_lite(p.api_key.trim())
    );
    let v = send(http.get(&url)).await?;
    let mut out = Vec::new();
    if let Some(arr) = v.get("models").and_then(|x| x.as_array()) {
        for m in arr {
            let name = m.get("name").and_then(|x| x.as_str()).unwrap_or("");
            let methods = m
                .get("supportedGenerationMethods")
                .and_then(|x| x.as_array())
                .cloned()
                .unwrap_or_default();
            let can_gen = methods
                .iter()
                .any(|x| x.as_str() == Some("generateContent"));
            if can_gen {
                out.push(name.trim_start_matches("models/").to_string());
            }
        }
    }
    Ok(out)
}

fn urlencoding_lite(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

async fn send(req: reqwest::RequestBuilder) -> Result<Value, String> {
    let res = req.send().await.map_err(|e| format!("Network: {e}"))?;
    let status = res.status();
    let body = res.text().await.map_err(|e| format!("Read: {e}"))?;
    if !status.is_success() {
        let snippet: String = body.chars().take(400).collect();
        return Err(format!("HTTP {status}: {snippet}"));
    }
    serde_json::from_str(&body).map_err(|e| format!("Bad JSON: {e} — {}", body.chars().take(180).collect::<String>()))
}

fn text_at(v: &Value, path: &[&str]) -> Option<String> {
    let mut cur = v;
    for k in path {
        cur = cur.get(*k)?;
    }
    cur.as_str().map(|s| s.to_string())
}

pub fn mark_ok(p: &mut Provider) {
    p.last_ok_at = Some(now());
    p.last_error = None;
}

pub fn mark_err(p: &mut Provider, e: &str) {
    p.last_error = Some(e.chars().take(280).collect());
}
