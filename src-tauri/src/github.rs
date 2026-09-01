use serde_json::Value;

use crate::corpus;
use crate::models::{new_id, now, IngestResult, LinkedRepo};

const MAX_FILES: usize = 18;
const MAX_FILE_BYTES: usize = 400_000;

pub fn parse_repo_url(input: &str) -> Result<(String, String), String> {
    let s = input.trim().trim_end_matches('/');
    let s = s.strip_suffix(".git").unwrap_or(s);
    let path = if let Some(rest) = s.strip_prefix("https://github.com/") {
        rest
    } else if let Some(rest) = s.strip_prefix("http://github.com/") {
        rest
    } else if let Some(rest) = s.strip_prefix("github.com/") {
        rest
    } else if let Some(rest) = s.strip_prefix("git@github.com:") {
        rest
    } else if s.contains('/') && !s.contains(' ') {
        s
    } else {
        return Err("Paste a GitHub URL like https://github.com/owner/repo".into());
    };
    let mut parts = path.split('/').filter(|p| !p.is_empty());
    let owner = parts
        .next()
        .ok_or_else(|| "Missing owner in GitHub URL.".to_string())?
        .to_string();
    let name = parts
        .next()
        .ok_or_else(|| "Missing repository name in GitHub URL.".to_string())?
        .to_string();
    if owner.is_empty() || name.is_empty() {
        return Err("Could not parse owner/repo from that URL.".into());
    }
    Ok((owner, name))
}

pub async fn link_and_ingest(
    http: &reqwest::Client,
    url: &str,
    token: &str,
    max_passages: Option<usize>,
) -> Result<(LinkedRepo, IngestResult), String> {
    let (owner, name) = parse_repo_url(url)?;
    let api = format!("https://api.github.com/repos/{owner}/{name}");
    let meta = github_get(http, &api, token).await?;
    let default_branch = meta
        .get("default_branch")
        .and_then(|v| v.as_str())
        .unwrap_or("main")
        .to_string();
    let description = meta
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let html_url = meta
        .get("html_url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("https://github.com/{owner}/{name}"));

    let tree_url = format!(
        "https://api.github.com/repos/{owner}/{name}/git/trees/{default_branch}?recursive=1"
    );
    let tree = github_get(http, &tree_url, token).await?;
    let paths = pick_paths(&tree);

    let title = format!("{owner}/{name}");
    let max = max_passages.unwrap_or(12).clamp(1, 24);
    let per_file = (max / paths.len().max(1)).max(1).min(max);
    let mut canaries = Vec::new();
    let mut files_used = Vec::new();
    let mut skipped = 0usize;

    for rel in &paths {
        let raw_url = format!(
            "https://raw.githubusercontent.com/{owner}/{name}/{default_branch}/{rel}"
        );
        let body = match fetch_text(http, &raw_url, token).await {
            Ok(t) => t,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };
        if body.trim().len() < 80 {
            skipped += 1;
            continue;
        }
        let made = corpus::canaries_from_text(
            &body,
            &title,
            &html_url,
            rel,
            "github",
            per_file,
        )?;
        if made.is_empty() {
            skipped += 1;
            continue;
        }
        files_used.push(rel.clone());
        canaries.extend(made);
        if canaries.len() >= max {
            break;
        }
    }

    if canaries.is_empty() {
        return Err(
            "Linked the repo but found no distinctive text passages. Try a different repo or add a token for private files."
                .into(),
        );
    }
    canaries.truncate(max);

    let linked = LinkedRepo {
        id: new_id("gh"),
        url: html_url,
        owner,
        name,
        default_branch,
        description,
        linked_at: now(),
        files: files_used,
    };

    Ok((
        linked,
        IngestResult {
            works: 1,
            skipped,
            canaries,
        },
    ))
}

fn pick_paths(tree: &Value) -> Vec<String> {
    let mut scored: Vec<(i32, String)> = Vec::new();
    let Some(arr) = tree.get("tree").and_then(|t| t.as_array()) else {
        return Vec::new();
    };
    for item in arr {
        if item.get("type").and_then(|t| t.as_str()) != Some("blob") {
            continue;
        }
        let Some(path) = item.get("path").and_then(|p| p.as_str()) else {
            continue;
        };
        if !keep_path(path) {
            continue;
        }
        scored.push((score_path(path), path.to_string()));
    }
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    scored
        .into_iter()
        .take(MAX_FILES)
        .map(|(_, p)| p)
        .collect()
}

fn keep_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    if lower.contains("/node_modules/")
        || lower.contains("/.git/")
        || lower.contains("/dist/")
        || lower.contains("/target/")
        || lower.contains("/vendor/")
        || lower.contains("/.next/")
        || lower.ends_with(".min.js")
        || lower.ends_with(".lock")
        || lower.ends_with("package-lock.json")
        || lower.ends_with("cargo.lock")
        || lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".gif")
        || lower.ends_with(".webp")
        || lower.ends_with(".ico")
        || lower.ends_with(".pdf")
        || lower.ends_with(".zip")
        || lower.ends_with(".wasm")
    {
        return false;
    }
    matches!(
        lower.rsplit('.').next().unwrap_or(""),
        "md" | "txt" | "rst"
            | "py" | "rs" | "ts" | "tsx" | "js" | "jsx" | "go" | "java" | "kt"
            | "c" | "h" | "cpp" | "hpp" | "cs" | "rb" | "php" | "swift"
            | "json" | "yml" | "yaml" | "toml" | "csv" | "sql" | "sh"
            | "html" | "css" | "scss"
    ) || lower.ends_with("readme")
        || lower.ends_with("license")
}

fn score_path(path: &str) -> i32 {
    let lower = path.to_lowercase();
    let mut score = 0;
    if lower == "readme.md" || lower.starts_with("readme.") {
        score += 100;
    }
    if lower.contains("readme") {
        score += 40;
    }
    if lower.starts_with("docs/") || lower.starts_with("doc/") {
        score += 20;
    }
    if lower.starts_with("src/") || lower.starts_with("lib/") || lower.starts_with("app/") {
        score += 15;
    }
    if lower.ends_with(".md") {
        score += 10;
    }
    // Prefer shorter paths (root / top-level)
    score += (40i32 - (path.matches('/').count() as i32 * 8)).max(0);
    score
}

async fn github_get(http: &reqwest::Client, url: &str, token: &str) -> Result<Value, String> {
    let mut req = http
        .get(url)
        .header("User-Agent", "LLM-Canary")
        .header("Accept", "application/vnd.github+json");
    if !token.trim().is_empty() {
        req = req.bearer_auth(token.trim());
    }
    let res = req.send().await.map_err(|e| format!("GitHub network: {e}"))?;
    let status = res.status();
    let body = res.text().await.map_err(|e| format!("GitHub read: {e}"))?;
    if !status.is_success() {
        let snippet: String = body.chars().take(280).collect();
        if status.as_u16() == 404 {
            return Err("Repo not found (or private — paste a GitHub token).".into());
        }
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(format!("GitHub auth/rate limit: {snippet}"));
        }
        return Err(format!("GitHub HTTP {status}: {snippet}"));
    }
    serde_json::from_str(&body).map_err(|e| format!("GitHub bad JSON: {e}"))
}

async fn fetch_text(http: &reqwest::Client, url: &str, token: &str) -> Result<String, String> {
    let mut req = http.get(url).header("User-Agent", "LLM-Canary");
    if !token.trim().is_empty() {
        req = req.bearer_auth(token.trim());
    }
    let res = req.send().await.map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("HTTP {}", res.status()));
    }
    let bytes = res.bytes().await.map_err(|e| e.to_string())?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err("file too large".into());
    }
    String::from_utf8(bytes.to_vec()).map_err(|_| "not utf-8".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_urls() {
        assert_eq!(
            parse_repo_url("https://github.com/AaronGrace978/LLM-Canary").unwrap(),
            ("AaronGrace978".into(), "LLM-Canary".into())
        );
        assert_eq!(
            parse_repo_url("AaronGrace978/LLM-Canary").unwrap(),
            ("AaronGrace978".into(), "LLM-Canary".into())
        );
    }
}
