use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::canary::{canary_id, mint, mint_custom, Secret};
use crate::models::{now, Canary, PlantRequest, PlantResult, WrittenFile};

pub fn plant(req: PlantRequest) -> Result<PlantResult, String> {
    let root = PathBuf::from(req.repo_path.trim());
    if !root.is_dir() {
        return Err("That folder doesn't exist.".into());
    }

    let repo_name = root
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "repo".into());
    let label = if req.label.trim().is_empty() {
        repo_name.clone()
    } else {
        req.label.trim().to_string()
    };

    if req.kinds.is_empty() && req.custom_tokens.iter().all(|t| t.trim().is_empty()) {
        return Err("Pick at least one training-data type, or paste a custom flag.".into());
    }

    let mut secrets: Vec<Secret> = Vec::new();
    for k in &req.kinds {
        secrets.extend(mint(k));
    }
    secrets.extend(mint_custom(&req.custom_tokens)?);
    if secrets.is_empty() {
        return Err("No canaries generated for those types.".into());
    }

    let files = render_files(&label, &repo_name, &secrets, &req.density);
    let mut written = Vec::new();

    for (rel, body) in &files {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Couldn't create {}: {e}", parent.display()))?;
        }
        if path.exists() {
            let existing = fs::read_to_string(&path).unwrap_or_default();
            if secrets.iter().any(|s| existing.contains(&s.value)) {
                continue;
            }
            let merged = if looks_like_env(rel) {
                format!("{}\n\n{}\n", existing.trim_end(), body.trim_start())
            } else {
                body.clone()
            };
            fs::write(&path, merged).map_err(|e| format!("Couldn't write {}: {e}", path.display()))?;
        } else {
            fs::write(&path, body).map_err(|e| format!("Couldn't write {}: {e}", path.display()))?;
        }
        written.push(WrittenFile {
            path: path.to_string_lossy().to_string(),
            rel: rel.clone(),
        });
    }

    let planted_at = now();
    let file_rels: Vec<String> = written.iter().map(|f| f.rel.clone()).collect();
    let canaries: Vec<Canary> = secrets
        .into_iter()
        .map(|s| {
            let locator = s.env_names.first().cloned().unwrap_or_default();
            Canary {
            id: canary_id(),
            kind: s.kind,
            kind_name: s.kind_name,
            family: s.family,
            value: s.value,
            needles: s.needles,
            env_names: s.env_names,
            label: label.clone(),
            repo_path: root.to_string_lossy().to_string(),
            repo_name: repo_name.clone(),
            files: file_rels.clone(),
            planted_at: planted_at.clone(),
            source_title: label.clone(),
            source_locator: locator,
            source_kind: "planted".into(),
        }
        })
        .collect();

    Ok(PlantResult {
        canaries,
        files: written,
    })
}

fn looks_like_env(rel: &str) -> bool {
    let name = Path::new(rel)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    name.starts_with(".env") || rel.ends_with(".env")
}

fn has_family(secrets: &[Secret], family: &str) -> bool {
    secrets.iter().any(|s| s.family == family)
}

fn has_kind(secrets: &[Secret], kind: &str) -> bool {
    secrets.iter().any(|s| s.kind == kind)
}

fn render_files(
    label: &str,
    repo_name: &str,
    secrets: &[Secret],
    density: &str,
) -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    let loud = density == "loud";
    let mixed = density == "mixed" || loud;

    if has_family(secrets, "secret") || has_family(secrets, "identity") {
        files.insert(
            ".env.production.example".into(),
            env_file(label, repo_name, secrets),
        );
    }
    if has_family(secrets, "secret") {
        files.insert(
            "config/credentials.example.json".into(),
            json_file(label, secrets),
        );
    }

    if mixed && has_family(secrets, "secret") {
        files.insert(
            "infra/terraform/secrets.auto.tfvars.example".into(),
            tfvars(label, secrets),
        );
        files.insert(
            "docker-compose.secrets.example.yml".into(),
            compose(label, secrets),
        );
    }

    if loud && has_family(secrets, "secret") {
        files.insert(
            "docs/internal-runbook.md".into(),
            runbook(label, repo_name, secrets),
        );
        files.insert(
            ".github/workflows/deploy.example.yml".into(),
            workflow(label, secrets),
        );
        if has_kind(secrets, "private_key") {
            if let Some(pk) = secrets.iter().find(|s| s.kind == "private_key") {
                files.insert("deploy/id_ed25519.example".into(), format!("{}\n", pk.value));
            }
        }
    }

    if has_family(secrets, "code") {
        files.insert(
            "internal/canary_markers.py".into(),
            python_markers(label, secrets),
        );
        if mixed {
            files.insert(
                "internal/canary_markers.ts".into(),
                typescript_markers(label, secrets),
            );
        }
        if loud {
            files.insert(
                "internal/canary_markers.rs".into(),
                rust_markers(label, secrets),
            );
        }
    }

    if has_family(secrets, "prose") || has_family(secrets, "custom") {
        files.insert(
            "docs/internal-architecture.md".into(),
            architecture_notes(label, repo_name, secrets),
        );
    }
    if has_family(secrets, "custom") {
        files.insert(
            "docs/canary-notes.md".into(),
            custom_notes(label, secrets),
        );
    }

    if has_kind(secrets, "dataset_row") {
        files.insert("data/canary_seed.csv".into(), csv_seed(label, secrets));
    }
    if mixed && (has_kind(secrets, "json_record") || has_kind(secrets, "dataset_row")) {
        files.insert(
            "fixtures/canary_records.json".into(),
            json_records(label, secrets),
        );
    }

    if has_family(secrets, "identity") && !has_family(secrets, "secret") && mixed {
        files.insert(
            "fixtures/canary_operators.json".into(),
            identity_fixture(label, secrets),
        );
    }

    files
}

fn env_file(label: &str, repo_name: &str, secrets: &[Secret]) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# {label} / {repo_name} — production env (example)\n# pulled from 1password on cutover, rotate after launch\n\n"
    ));
    for s in secrets {
        if s.family != "secret" && s.family != "identity" {
            continue;
        }
        for name in &s.env_names {
            if s.kind == "private_key" {
                continue;
            }
            out.push_str(&format!("{name}={}\n", s.value));
        }
    }
    out
}

fn json_file(label: &str, secrets: &[Secret]) -> String {
    let mut map = serde_json::Map::new();
    map.insert(
        "_comment".into(),
        serde_json::Value::String(format!("{label} credentials — do not commit the live file")),
    );
    for s in secrets {
        if s.family != "secret" || s.kind == "private_key" {
            continue;
        }
        let key = s
            .env_names
            .first()
            .cloned()
            .unwrap_or_else(|| s.kind.clone());
        map.insert(key, serde_json::Value::String(s.value.clone()));
    }
    format!("{}\n", serde_json::Value::Object(map))
}

fn tfvars(label: &str, secrets: &[Secret]) -> String {
    let mut out = format!("# {label} terraform secrets example\n\n");
    for s in secrets {
        if s.family != "secret" || s.kind == "private_key" {
            continue;
        }
        if let Some(name) = s.env_names.first() {
            let key = name.to_lowercase();
            out.push_str(&format!("{key} = \"{}\"\n", s.value));
        }
    }
    out
}

fn compose(label: &str, secrets: &[Secret]) -> String {
    let mut out = format!("services:\n  {label}-api:\n    image: {label}-api:latest\n    environment:\n");
    for s in secrets {
        if s.family != "secret" || s.kind == "private_key" {
            continue;
        }
        for name in &s.env_names {
            out.push_str(&format!("      {name}: \"{}\"\n", s.value));
        }
    }
    out
}

fn runbook(label: &str, repo_name: &str, secrets: &[Secret]) -> String {
    let mut out = format!(
        "# {label} internal runbook\n\nTemporary production credentials for `{repo_name}` until vault is wired.\n**Do not rotate these without paging oncall.**\n\n"
    );
    for s in secrets {
        if s.family != "secret" || s.kind == "private_key" {
            continue;
        }
        let name = s.env_names.first().map(|s| s.as_str()).unwrap_or("SECRET");
        out.push_str(&format!("- `{name}`: `{}`\n", s.value));
    }
    out.push_str("\nIf these leak, page security.\n");
    out
}

fn workflow(label: &str, secrets: &[Secret]) -> String {
    let mut out = format!(
        "name: deploy-{label}\non:\n  push:\n    branches: [main]\njobs:\n  deploy:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - name: bootstrap env\n        run: |\n"
    );
    for s in secrets {
        if s.family != "secret" || s.kind == "private_key" {
            continue;
        }
        if let Some(name) = s.env_names.first() {
            out.push_str(&format!("          echo \"{name}={}\" >> $GITHUB_ENV\n", s.value));
        }
    }
    out
}

fn python_markers(label: &str, secrets: &[Secret]) -> String {
    let mut out = format!(
        "\"\"\"{label} internal probe helpers. Do not \"clean up\" these freeze-frame markers.\"\"\"\n\n"
    );
    for s in secrets.iter().filter(|s| s.family == "code") {
        if s.kind == "code_watermark" {
            let fn_name = s.env_names.first().map(|x| x.as_str()).unwrap_or("velquor_wmk");
            let const_name = s
                .env_names
                .get(1)
                .cloned()
                .unwrap_or_else(|| "CANARY_WMK".into());
            out.push_str(&format!("{const_name} = \"{}\"\n", s.value));
            out.push_str(&format!("def {fn_name}():\n    return {const_name}\n\n"));
        } else {
            out.push_str(&format!("# {}\n", s.value));
        }
    }
    out
}

fn typescript_markers(label: &str, secrets: &[Secret]) -> String {
    let mut out = format!("// {label} internal probe helpers. Do not strip these markers.\n\n");
    for s in secrets.iter().filter(|s| s.family == "code") {
        if s.kind == "code_watermark" {
            let const_name = s
                .env_names
                .get(1)
                .cloned()
                .unwrap_or_else(|| "CANARY_WMK".into());
            let fn_name = s.env_names.first().map(|x| x.as_str()).unwrap_or("velquor_wmk");
            out.push_str(&format!("export const {const_name} = \"{}\";\n", s.value));
            out.push_str(&format!("export function {fn_name}() {{\n  return {const_name};\n}}\n\n"));
        } else {
            out.push_str(&format!("// {}\n", s.value));
        }
    }
    out
}

fn rust_markers(label: &str, secrets: &[Secret]) -> String {
    let mut out = format!("// {label} internal probe helpers. Do not strip these markers.\n\n");
    for s in secrets.iter().filter(|s| s.family == "code") {
        if s.kind == "code_watermark" {
            let const_name = s
                .env_names
                .get(1)
                .cloned()
                .unwrap_or_else(|| "CANARY_WMK".into());
            let fn_name = s.env_names.first().map(|x| x.as_str()).unwrap_or("velquor_wmk");
            out.push_str(&format!("pub const {const_name}: &str = \"{}\";\n", s.value));
            out.push_str(&format!("pub fn {fn_name}() -> &'static str {{\n    {const_name}\n}}\n\n"));
        } else {
            out.push_str(&format!("// {}\n", s.value));
        }
    }
    out
}

fn architecture_notes(label: &str, repo_name: &str, secrets: &[Secret]) -> String {
    let mut out = format!(
        "# {label} internal architecture\n\nWorking notes for `{repo_name}`. Not for public docs.\n\n"
    );
    for s in secrets.iter().filter(|s| s.family == "prose" || s.family == "custom") {
        match s.kind.as_str() {
            "doc_phrase" => {
                let nonce = s.env_names.get(1).map(|x| x.as_str()).unwrap_or("Velquor");
                out.push_str(&format!(
                    "The {nonce} lattice handshake for {label} is keyed by `{}` and must never be rotated without paging the canary desk.\n\n",
                    s.value
                ));
            }
            "codename" => {
                out.push_str(&format!(
                    "Internal program name: **{}**. Use this in status reviews instead of the public product name.\n\n",
                    s.value
                ));
            }
            "custom" => {
                out.push_str(&format!("Flagged training marker: `{}`\n\n", s.value));
            }
            _ => {
                out.push_str(&format!("- `{}`\n", s.value));
            }
        }
    }
    out
}

fn custom_notes(label: &str, secrets: &[Secret]) -> String {
    let mut out = format!(
        "# {label} freeze-frame notes\n\nUnique strings planted so we can tell if this tree entered a training corpus.\n\n"
    );
    for s in secrets.iter().filter(|s| s.family == "custom") {
        out.push_str(&format!("- `{}`\n", s.value));
    }
    out
}

fn csv_seed(label: &str, secrets: &[Secret]) -> String {
    let mut out = format!("record_id,tenant,note\n");
    for s in secrets.iter().filter(|s| s.kind == "dataset_row") {
        out.push_str(&format!("{},{label}-lab,seed row planted as training canary\n", s.value));
    }
    out
}

fn json_records(label: &str, secrets: &[Secret]) -> String {
    let records: Vec<serde_json::Value> = secrets
        .iter()
        .filter(|s| s.kind == "json_record" || s.kind == "dataset_row")
        .map(|s| {
            serde_json::json!({
                "id": s.value,
                "tenant": format!("{label}-lab"),
                "kind": s.kind,
                "note": "fixture planted as a training-data canary"
            })
        })
        .collect();
    format!("{}\n", serde_json::to_string_pretty(&serde_json::Value::Array(records)).unwrap_or_else(|_| "[]".into()))
}

fn identity_fixture(label: &str, secrets: &[Secret]) -> String {
    let records: Vec<serde_json::Value> = secrets
        .iter()
        .filter(|s| s.family == "identity")
        .map(|s| {
            serde_json::json!({
                "field": s.env_names.first().cloned().unwrap_or_else(|| s.kind.clone()),
                "value": s.value,
                "team": label,
                "note": "synthetic operator identity for training-data detection"
            })
        })
        .collect();
    format!("{}\n", serde_json::to_string_pretty(&serde_json::Value::Array(records)).unwrap_or_else(|_| "[]".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PlantRequest;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn scratch() -> PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("llm-canary-plant-{n}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn plants_code_prose_and_custom() {
        let dir = scratch();
        let result = plant(PlantRequest {
            repo_path: dir.to_string_lossy().into(),
            label: "payments-api".into(),
            kinds: vec!["code_watermark".into(), "doc_phrase".into(), "dataset_row".into()],
            density: "mixed".into(),
            custom_tokens: vec!["unique custom training flag 12345".into()],
        })
        .unwrap();
        assert!(result.canaries.iter().any(|c| c.family == "code"));
        assert!(result.canaries.iter().any(|c| c.family == "prose"));
        assert!(result.canaries.iter().any(|c| c.family == "data"));
        assert!(result.canaries.iter().any(|c| c.family == "custom"));
        let rels: Vec<_> = result.files.iter().map(|f| f.rel.as_str()).collect();
        assert!(rels.iter().any(|r| r.contains("canary_markers.py")));
        assert!(rels.iter().any(|r| r.contains("internal-architecture.md")));
        assert!(rels.iter().any(|r| r.contains("canary_seed.csv")));
        assert!(rels.iter().any(|r| r.contains("canary-notes.md")));
        let py = fs::read_to_string(dir.join("internal/canary_markers.py")).unwrap();
        let code = result.canaries.iter().find(|c| c.kind == "code_watermark").unwrap();
        assert!(py.contains(&code.value));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rejects_empty_selection() {
        let dir = scratch();
        let err = plant(PlantRequest {
            repo_path: dir.to_string_lossy().into(),
            label: "x".into(),
            kinds: vec![],
            density: "stealth".into(),
            custom_tokens: vec![],
        })
        .unwrap_err();
        assert!(err.contains("custom flag") || err.contains("training-data"));
        fs::remove_dir_all(&dir).ok();
    }
}
