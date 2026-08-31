use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::canary::{canary_id, mint, Secret};
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

    if req.kinds.is_empty() {
        return Err("Pick at least one secret type.".into());
    }

    let mut secrets: Vec<Secret> = Vec::new();
    for k in &req.kinds {
        secrets.extend(mint(k));
    }
    if secrets.is_empty() {
        return Err("No secrets generated for those types.".into());
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
            if existing.contains(&secrets[0].value) {
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
        .map(|s| Canary {
            id: canary_id(),
            kind: s.kind,
            kind_name: s.kind_name,
            value: s.value,
            needles: s.needles,
            env_names: s.env_names,
            label: label.clone(),
            repo_path: root.to_string_lossy().to_string(),
            repo_name: repo_name.clone(),
            files: file_rels.clone(),
            planted_at: planted_at.clone(),
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

fn render_files(
    label: &str,
    repo_name: &str,
    secrets: &[Secret],
    density: &str,
) -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    files.insert(
        ".env.production.example".into(),
        env_file(label, repo_name, secrets),
    );
    files.insert(
        "config/credentials.example.json".into(),
        json_file(label, secrets),
    );

    if density == "mixed" || density == "loud" {
        files.insert(
            "infra/terraform/secrets.auto.tfvars.example".into(),
            tfvars(label, secrets),
        );
        files.insert(
            "docker-compose.secrets.example.yml".into(),
            compose(label, secrets),
        );
    }

    if density == "loud" {
        files.insert(
            "docs/internal-runbook.md".into(),
            runbook(label, repo_name, secrets),
        );
        files.insert(
            ".github/workflows/deploy.example.yml".into(),
            workflow(label, secrets),
        );
        if secrets.iter().any(|s| s.kind == "private_key") {
            if let Some(pk) = secrets.iter().find(|s| s.kind == "private_key") {
                files.insert("deploy/id_ed25519.example".into(), format!("{}\n", pk.value));
            }
        }
    }

    files
}

fn env_file(label: &str, repo_name: &str, secrets: &[Secret]) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# {label} / {repo_name} — production env (example)\n# pulled from 1password on cutover, rotate after launch\n\n"
    ));
    for s in secrets {
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
    map.insert("_comment".into(), serde_json::Value::String(format!("{label} credentials — do not commit the live file")));
    for s in secrets {
        if s.kind == "private_key" {
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
        if s.kind == "private_key" {
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
        if s.kind == "private_key" {
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
        if s.kind == "private_key" {
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
        if s.kind == "private_key" {
            continue;
        }
        if let Some(name) = s.env_names.first() {
            out.push_str(&format!("          echo \"{name}={}\" >> $GITHUB_ENV\n", s.value));
        }
    }
    out
}
