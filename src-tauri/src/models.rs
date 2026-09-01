use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KindInfo {
    pub id: String,
    pub name: String,
    pub blurb: String,
    pub sample: String,
    pub family: String,
    pub family_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Canary {
    pub id: String,
    pub kind: String,
    pub kind_name: String,
    pub value: String,
    pub needles: Vec<String>,
    pub env_names: Vec<String>,
    pub label: String,
    pub repo_path: String,
    pub repo_name: String,
    pub files: Vec<String>,
    pub planted_at: String,
    #[serde(default = "default_secret_family")]
    pub family: String,
    #[serde(default)]
    pub source_title: String,
    #[serde(default)]
    pub source_locator: String,
    #[serde(default = "default_planted_source")]
    pub source_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub blurb: String,
    pub docs_url: String,
    pub enabled: bool,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub models: Vec<String>,
    pub last_error: Option<String>,
    pub last_ok_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Probe {
    pub id: String,
    pub at: String,
    pub provider_id: String,
    pub provider_name: String,
    pub model: String,
    pub canary_id: String,
    pub canary_kind: String,
    pub canary_label: String,
    pub strategy: String,
    pub prompt: String,
    pub response: String,
    pub hit: bool,
    pub matched: Vec<String>,
    pub error: Option<String>,
    #[serde(default)]
    pub citation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Db {
    pub canaries: Vec<Canary>,
    pub providers: Vec<Provider>,
    pub probes: Vec<Probe>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub canaries: Vec<Canary>,
    pub providers: Vec<Provider>,
    pub probes: Vec<Probe>,
    pub kinds: Vec<KindInfo>,
    pub hits: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlantRequest {
    pub repo_path: String,
    pub label: String,
    pub kinds: Vec<String>,
    pub density: String,
    #[serde(default)]
    pub custom_tokens: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestRequest {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub max_passages: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestResult {
    pub canaries: Vec<Canary>,
    pub works: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WrittenFile {
    pub path: String,
    pub rel: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlantResult {
    pub canaries: Vec<Canary>,
    pub files: Vec<WrittenFile>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HuntRequest {
    pub canary_ids: Vec<String>,
    pub provider_ids: Vec<String>,
    pub strategies: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HuntSummary {
    pub probes: Vec<Probe>,
    pub hits: usize,
    pub errors: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HuntProgress {
    pub phase: String,
    pub provider_id: String,
    pub provider_name: String,
    pub model: String,
    pub canary_id: String,
    pub strategy: String,
    pub message: String,
    pub hit: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanRequest {
    pub text: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanHit {
    pub canary_id: String,
    pub kind: String,
    pub label: String,
    pub matched: Vec<String>,
    #[serde(default)]
    pub citation: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebPrompt {
    pub canary_id: String,
    pub title: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderPatch {
    pub id: String,
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestResult {
    pub ok: bool,
    pub model: String,
    pub preview: String,
}

fn default_secret_family() -> String {
    "secret".into()
}

fn default_planted_source() -> String {
    "planted".into()
}

pub fn family_name(family: &str) -> &'static str {
    match family {
        "secret" => "Secrets",
        "code" => "Code",
        "prose" => "Documents",
        "data" => "Datasets",
        "identity" => "Identity",
        "custom" => "Custom",
        "corpus" => "Corpus",
        _ => "Other",
    }
}

pub fn citation_for(canary: &Canary) -> String {
    let title = if canary.source_title.trim().is_empty() {
        canary.label.as_str()
    } else {
        canary.source_title.as_str()
    };
    let loc = canary.source_locator.trim();
    if loc.is_empty() {
        title.to_string()
    } else {
        format!("{title} · {loc}")
    }
}

fn kind(id: &str, name: &str, blurb: &str, sample: &str, family: &str) -> KindInfo {
    KindInfo {
        id: id.into(),
        name: name.into(),
        blurb: blurb.into(),
        sample: sample.into(),
        family: family.into(),
        family_name: family_name(family).into(),
    }
}

pub fn kinds_catalog() -> Vec<KindInfo> {
    vec![
        kind(
            "aws",
            "AWS keys",
            "AKIA access key + 40-char secret",
            "AKIA…  /  wJalr…",
            "secret",
        ),
        kind(
            "github",
            "GitHub PAT",
            "Classic ghp_ personal access token",
            "ghp_…",
            "secret",
        ),
        kind(
            "openai",
            "OpenAI key",
            "sk-proj project key",
            "sk-proj-…",
            "secret",
        ),
        kind(
            "anthropic",
            "Anthropic key",
            "sk-ant-api03 Claude key",
            "sk-ant-…",
            "secret",
        ),
        kind(
            "stripe",
            "Stripe live",
            "sk_live restricted-looking key",
            "sk_live_…",
            "secret",
        ),
        kind(
            "slack",
            "Slack webhook",
            "Incoming webhook URL",
            "hooks.slack.com/…",
            "secret",
        ),
        kind(
            "postgres",
            "Postgres URL",
            "postgres:// user, password, host",
            "postgres://…",
            "secret",
        ),
        kind(
            "huggingface",
            "Hugging Face",
            "hf_ user access token",
            "hf_…",
            "secret",
        ),
        kind(
            "sendgrid",
            "SendGrid",
            "SG. API key",
            "SG.…",
            "secret",
        ),
        kind(
            "npm",
            "npm token",
            "npm_ automation token",
            "npm_…",
            "secret",
        ),
        kind(
            "private_key",
            "SSH private key",
            "OPENSSH private key block",
            "BEGIN OPENSSH…",
            "secret",
        ),
        kind(
            "code_watermark",
            "Code watermark",
            "Unique function + constant that only this tree should know",
            "velquor_wmk_…",
            "code",
        ),
        kind(
            "code_comment",
            "Comment tag",
            "High-entropy freeze-frame marker in source comments",
            "CNRY-TAG-…",
            "code",
        ),
        kind(
            "doc_phrase",
            "Doc phrase",
            "Unique sentence in internal architecture notes",
            "CNRY… lattice handshake",
            "prose",
        ),
        kind(
            "codename",
            "Project codename",
            "Fake internal proper noun trainers would memorize",
            "Velquor-…",
            "prose",
        ),
        kind(
            "dataset_row",
            "Dataset row",
            "Unique CSV record id that looks like seed data",
            "cnry_row_…",
            "data",
        ),
        kind(
            "json_record",
            "JSON record",
            "Unique fixture object id in a JSON dump",
            "cnry_json_…",
            "data",
        ),
        kind(
            "canary_email",
            "Operator email",
            "Unique @canary.invalid address",
            "nora.velquor…@canary.invalid",
            "identity",
        ),
        kind(
            "employee_id",
            "Employee id",
            "Fake staff / operator identifier",
            "CNV-…",
            "identity",
        ),
    ]
}

pub fn default_providers() -> Vec<Provider> {
    vec![
        Provider {
            id: "ollama-cloud".into(),
            name: "Ollama Cloud".into(),
            kind: "ollama".into(),
            blurb: "Cloud models on ollama.com — paste a key, pick a model.".into(),
            docs_url: "https://ollama.com/settings/keys".into(),
            enabled: true,
            api_key: String::new(),
            base_url: "https://ollama.com".into(),
            model: "gpt-oss:120b".into(),
            models: fallback_models("ollama-cloud"),
            last_error: None,
            last_ok_at: None,
        },
        Provider {
            id: "openai".into(),
            name: "OpenAI".into(),
            kind: "openai".into(),
            blurb: "GPT-5.6 Sol / Terra / Luna and the rest of the API catalog.".into(),
            docs_url: "https://platform.openai.com/api-keys".into(),
            enabled: true,
            api_key: String::new(),
            base_url: "https://api.openai.com/v1".into(),
            model: "gpt-5.6".into(),
            models: fallback_models("openai"),
            last_error: None,
            last_ok_at: None,
        },
        Provider {
            id: "anthropic".into(),
            name: "Anthropic".into(),
            kind: "anthropic".into(),
            blurb: "Claude Fable 5, Opus 5, Sonnet 5 — Messages API.".into(),
            docs_url: "https://console.anthropic.com/settings/keys".into(),
            enabled: true,
            api_key: String::new(),
            base_url: "https://api.anthropic.com".into(),
            model: "claude-sonnet-5".into(),
            models: fallback_models("anthropic"),
            last_error: None,
            last_ok_at: None,
        },
        Provider {
            id: "openrouter".into(),
            name: "OpenRouter".into(),
            kind: "openai-compat".into(),
            blurb: "One key, hundreds of models. Live catalog from /api/v1/models.".into(),
            docs_url: "https://openrouter.ai/keys".into(),
            enabled: true,
            api_key: String::new(),
            base_url: "https://openrouter.ai/api/v1".into(),
            model: "anthropic/claude-sonnet-5".into(),
            models: fallback_models("openrouter"),
            last_error: None,
            last_ok_at: None,
        },
        Provider {
            id: "gemini".into(),
            name: "Google Gemini".into(),
            kind: "gemini".into(),
            blurb: "Gemini 3.x via Google AI Studio.".into(),
            docs_url: "https://aistudio.google.com/apikey".into(),
            enabled: false,
            api_key: String::new(),
            base_url: "https://generativelanguage.googleapis.com/v1beta".into(),
            model: "gemini-3.7-flash".into(),
            models: fallback_models("gemini"),
            last_error: None,
            last_ok_at: None,
        },
        Provider {
            id: "groq".into(),
            name: "Groq".into(),
            kind: "openai-compat".into(),
            blurb: "OpenAI-compatible, very fast inference.".into(),
            docs_url: "https://console.groq.com/keys".into(),
            enabled: false,
            api_key: String::new(),
            base_url: "https://api.groq.com/openai/v1".into(),
            model: "llama-3.3-70b-versatile".into(),
            models: fallback_models("groq"),
            last_error: None,
            last_ok_at: None,
        },
        Provider {
            id: "deepseek".into(),
            name: "DeepSeek".into(),
            kind: "openai-compat".into(),
            blurb: "DeepSeek-V4 and reasoner via the official API.".into(),
            docs_url: "https://platform.deepseek.com/api_keys".into(),
            enabled: false,
            api_key: String::new(),
            base_url: "https://api.deepseek.com".into(),
            model: "deepseek-chat".into(),
            models: fallback_models("deepseek"),
            last_error: None,
            last_ok_at: None,
        },
        Provider {
            id: "mistral".into(),
            name: "Mistral".into(),
            kind: "openai-compat".into(),
            blurb: "Mistral Large / Codestral, OpenAI-compatible.".into(),
            docs_url: "https://console.mistral.ai/api-keys".into(),
            enabled: false,
            api_key: String::new(),
            base_url: "https://api.mistral.ai/v1".into(),
            model: "mistral-large-latest".into(),
            models: fallback_models("mistral"),
            last_error: None,
            last_ok_at: None,
        },
        Provider {
            id: "xai".into(),
            name: "xAI".into(),
            kind: "openai-compat".into(),
            blurb: "Grok via api.x.ai.".into(),
            docs_url: "https://console.x.ai/".into(),
            enabled: false,
            api_key: String::new(),
            base_url: "https://api.x.ai/v1".into(),
            model: "grok-4.6".into(),
            models: fallback_models("xai"),
            last_error: None,
            last_ok_at: None,
        },
        Provider {
            id: "custom".into(),
            name: "OpenAI-compatible".into(),
            kind: "openai-compat".into(),
            blurb: "Any /v1/chat/completions endpoint. Paste base URL, key, model.".into(),
            docs_url: String::new(),
            enabled: false,
            api_key: String::new(),
            base_url: "http://localhost:1234/v1".into(),
            model: String::new(),
            models: vec![],
            last_error: None,
            last_ok_at: None,
        },
    ]
}

pub fn fallback_models(id: &str) -> Vec<String> {
    let list: &[&str] = match id {
        "openai" => &[
            "gpt-5.6",
            "gpt-5.6-sol",
            "gpt-5.6-terra",
            "gpt-5.6-luna",
            "gpt-5.5",
            "gpt-5.5-pro",
            "gpt-5.4",
            "gpt-4.1",
            "gpt-4o",
        ],
        "anthropic" => &[
            "claude-fable-5",
            "claude-opus-5",
            "claude-sonnet-5",
            "claude-haiku-4-5",
            "claude-opus-4-5",
            "claude-sonnet-4-5",
        ],
        "ollama-cloud" => &[
            "gpt-oss:120b",
            "gemma4:31b",
            "qwen3.5:397b",
            "glm-5.2",
            "kimi-k2.6",
            "minimax-m3",
            "deepseek-v4-flash",
            "mistral-large-3:675b",
        ],
        "openrouter" => &[
            "openai/gpt-5.6",
            "anthropic/claude-sonnet-5",
            "anthropic/claude-fable-5",
            "google/gemini-3.7-flash",
            "x-ai/grok-4.6",
            "deepseek/deepseek-chat",
        ],
        "gemini" => &[
            "gemini-3.7-flash",
            "gemini-3.1-pro-preview",
            "gemini-2.5-pro",
            "gemini-2.5-flash",
        ],
        "groq" => &[
            "llama-3.3-70b-versatile",
            "llama-3.1-8b-instant",
            "mixtral-8x7b-32768",
            "openai/gpt-oss-120b",
        ],
        "deepseek" => &["deepseek-chat", "deepseek-reasoner", "deepseek-v4-pro"],
        "mistral" => &[
            "mistral-large-latest",
            "mistral-medium-latest",
            "mistral-small-latest",
            "codestral-latest",
        ],
        "xai" => &["grok-4.6", "grok-4.5", "grok-4", "grok-3"],
        _ => &[],
    };
    list.iter().map(|s| (*s).to_string()).collect()
}

pub fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub fn new_id(prefix: &str) -> String {
    use rand::RngCore;
    let mut b = [0u8; 6];
    rand::thread_rng().fill_bytes(&mut b);
    format!("{}_{}", prefix, hex_encode(&b))
}

pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn merge_providers(existing: &mut Vec<Provider>) {
    let defaults = default_providers();
    for d in defaults {
        if !existing.iter().any(|p| p.id == d.id) {
            existing.push(d);
        }
    }
}
