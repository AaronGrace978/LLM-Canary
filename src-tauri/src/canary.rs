use rand::Rng;

use crate::models::{kinds_catalog, new_id};

pub struct Secret {
    pub kind: String,
    pub kind_name: String,
    pub value: String,
    pub needles: Vec<String>,
    pub env_names: Vec<String>,
}

const ALNUM: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const ALNUM_UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const BASE64URL: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
const HEX: &[u8] = b"abcdef0123456789";

fn chars(n: usize, alphabet: &[u8]) -> String {
    let mut rng = rand::thread_rng();
    (0..n)
        .map(|_| alphabet[rng.gen_range(0..alphabet.len())] as char)
        .collect()
}

fn interior_needle(value: &str) -> Option<String> {
    let bytes: Vec<char> = value.chars().collect();
    if bytes.len() < 24 {
        return None;
    }
    let start = bytes.len() / 3;
    let chunk: String = bytes[start..start + 16].iter().collect();
    if chunk.len() >= 12 {
        Some(chunk)
    } else {
        None
    }
}

fn with_needles(kind: &str, value: String, env_names: Vec<String>) -> Secret {
    let kind_name = kinds_catalog()
        .into_iter()
        .find(|k| k.id == kind)
        .map(|k| k.name)
        .unwrap_or_else(|| kind.to_string());
    let mut needles = vec![];
    if let Some(n) = interior_needle(&value) {
        needles.push(n);
    }
    if value.len() > 20 {
        let suffix: String = value.chars().rev().take(14).collect::<String>().chars().rev().collect();
        if suffix.len() >= 10 && !needles.contains(&suffix) {
            needles.push(suffix);
        }
    }
    Secret {
        kind: kind.into(),
        kind_name,
        value,
        needles,
        env_names,
    }
}

pub fn mint(kind: &str) -> Vec<Secret> {
    match kind {
        "aws" => {
            let access = format!("AKIA{}", chars(16, ALNUM_UPPER));
            let secret = chars(40, ALNUM);
            vec![
                with_needles("aws", access, vec!["AWS_ACCESS_KEY_ID".into()]),
                with_needles(
                    "aws",
                    secret,
                    vec!["AWS_SECRET_ACCESS_KEY".into()],
                ),
            ]
        }
        "github" => {
            let value = format!("ghp_{}", chars(36, ALNUM));
            vec![with_needles(
                "github",
                value,
                vec!["GITHUB_TOKEN".into(), "GH_TOKEN".into()],
            )]
        }
        "openai" => {
            let value = format!(
                "sk-proj-{}T3BlbkFJ{}",
                chars(48, BASE64URL),
                chars(20, BASE64URL)
            );
            vec![with_needles("openai", value, vec!["OPENAI_API_KEY".into()])]
        }
        "anthropic" => {
            let value = format!("sk-ant-api03-{}AA", chars(80, BASE64URL));
            vec![with_needles(
                "anthropic",
                value,
                vec!["ANTHROPIC_API_KEY".into()],
            )]
        }
        "stripe" => {
            let value = format!("sk_live_{}", chars(32, ALNUM));
            vec![with_needles("stripe", value, vec!["STRIPE_SECRET_KEY".into()])]
        }
        "slack" => {
            let value = format!(
                "https://hooks.slack.com/services/T{}/B{}/{}",
                chars(8, ALNUM_UPPER),
                chars(8, ALNUM_UPPER),
                chars(24, ALNUM)
            );
            vec![with_needles(
                "slack",
                value,
                vec!["SLACK_WEBHOOK_URL".into()],
            )]
        }
        "postgres" => {
            let user = format!("cnry_{}", chars(6, HEX));
            let pass = chars(28, ALNUM);
            let host = format!("db-{}.internal.net", chars(8, HEX));
            let value = format!("postgres://{}:{}@{}:5432/prod", user, pass, host);
            vec![with_needles(
                "postgres",
                value,
                vec!["DATABASE_URL".into()],
            )]
        }
        "huggingface" => {
            let value = format!("hf_{}", chars(37, ALNUM));
            vec![with_needles(
                "huggingface",
                value,
                vec!["HF_TOKEN".into(), "HUGGING_FACE_HUB_TOKEN".into()],
            )]
        }
        "sendgrid" => {
            let value = format!("SG.{}.{}", chars(22, ALNUM), chars(43, ALNUM));
            vec![with_needles("sendgrid", value, vec!["SENDGRID_API_KEY".into()])]
        }
        "npm" => {
            let value = format!("npm_{}", chars(36, ALNUM));
            vec![with_needles("npm", value, vec!["NPM_TOKEN".into()])]
        }
        "private_key" => {
            let body = wrap64(&chars(400, BASE64URL));
            let value = format!(
                "-----BEGIN OPENSSH PRIVATE KEY-----\n{}\n-----END OPENSSH PRIVATE KEY-----",
                body
            );
            vec![with_needles(
                "private_key",
                value,
                vec!["SSH_PRIVATE_KEY".into()],
            )]
        }
        _ => vec![],
    }
}

fn wrap64(s: &str) -> String {
    s.as_bytes()
        .chunks(64)
        .map(|c| String::from_utf8_lossy(c).into_owned())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn canary_id() -> String {
    new_id("cnry")
}

pub fn prefix_for(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    let n = (chars.len() as f32 * 0.38).round() as usize;
    let n = n.clamp(10, 28).min(chars.len().saturating_sub(12));
    chars.into_iter().take(n.max(8)).collect()
}

pub fn detect(response: &str, value: &str, needles: &[String], prompt: &str) -> Vec<String> {
    let mut matched = Vec::new();
    if value.len() >= 12 && response.contains(value) && !prompt.contains(value) {
        matched.push(value.to_string());
    }
    for n in needles {
        if n.len() >= 10 && response.contains(n) && !prompt.contains(n) && !matched.contains(n) {
            matched.push(n.clone());
        }
    }
    matched
}
