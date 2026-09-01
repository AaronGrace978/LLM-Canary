mod canary;
mod corpus;
mod github;
mod models;
mod plant;
mod provenance;
mod providers;
mod secrets;
mod store;

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use models::*;
use tauri::{AppHandle, Emitter, Manager, State};

pub struct AppState {
    pub http: reqwest::Client,
    pub db: Mutex<Db>,
    pub data_dir: PathBuf,
}

fn persist(state: &AppState) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::save(&state.data_dir, &db)
}

fn snapshot_from(db: &Db) -> Snapshot {
    let hits = db.probes.iter().filter(|p| p.hit).count();
    let answers = provenance::build_provider_answers(&db.canaries, &db.probes);
    let private_hits = answers.iter().map(|a| a.private_hits).sum();
    let public_hits = answers.iter().map(|a| a.public_hits).sum();
    let provenance = ProvenanceSummary {
        answers: answers
            .into_iter()
            .map(|a| ProvenanceAnswer {
                provider_name: a.provider_name,
                model: a.model,
                public_sources: a.public_sources,
                private_sources: a.private_sources,
                public_hits: a.public_hits,
                private_hits: a.private_hits,
            })
            .collect(),
        private_hits,
        public_hits,
    };
    Snapshot {
        canaries: db.canaries.clone(),
        providers: db.providers.clone(),
        probes: db.probes.clone(),
        kinds: kinds_catalog(),
        hits,
        provenance,
        linked_repos: db.linked_repos.clone(),
        has_github_token: !db.github_token.trim().is_empty(),
    }
}

#[tauri::command]
fn load_snapshot(state: State<AppState>) -> Result<Snapshot, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    Ok(snapshot_from(&db))
}

#[tauri::command]
fn save_provider(state: State<AppState>, patch: ProviderPatch) -> Result<Snapshot, String> {
    {
        let mut db = state.db.lock().map_err(|e| e.to_string())?;
        if let Some(p) = db.providers.iter_mut().find(|p| p.id == patch.id) {
            if let Some(v) = patch.enabled {
                p.enabled = v;
            }
            if let Some(v) = patch.api_key {
                p.api_key = v;
                if !p.api_key.trim().is_empty() {
                    p.enabled = true;
                }
            }
            if let Some(v) = patch.base_url {
                p.base_url = v;
            }
            if let Some(v) = patch.model {
                p.model = v;
            }
        } else {
            return Err("Unknown provider.".into());
        }
    }
    persist(&state)?;
    load_snapshot(state)
}

#[tauri::command]
fn plant_canaries(state: State<AppState>, req: PlantRequest) -> Result<PlantResult, String> {
    let result = plant::plant(req)?;
    {
        let mut db = state.db.lock().map_err(|e| e.to_string())?;
        db.canaries.extend(result.canaries.clone());
    }
    persist(&state)?;
    Ok(result)
}

#[tauri::command]
fn ingest_corpus(state: State<AppState>, req: IngestRequest) -> Result<IngestResult, String> {
    let result = corpus::ingest(req)?;
    {
        let mut db = state.db.lock().map_err(|e| e.to_string())?;
        db.canaries.extend(result.canaries.clone());
    }
    persist(&state)?;
    Ok(result)
}

#[tauri::command]
fn load_public_domain_pack(state: State<AppState>) -> Result<IngestResult, String> {
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        if corpus::already_has_public_domain(&db.canaries) {
            return Err("Public-domain pack is already in the flock.".into());
        }
    }
    let result = corpus::public_domain_pack();
    {
        let mut db = state.db.lock().map_err(|e| e.to_string())?;
        db.canaries.extend(result.canaries.clone());
    }
    persist(&state)?;
    Ok(result)
}

#[tauri::command]
fn delete_canary(state: State<AppState>, id: String) -> Result<Snapshot, String> {
    {
        let mut db = state.db.lock().map_err(|e| e.to_string())?;
        db.canaries.retain(|c| c.id != id);
        db.probes.retain(|p| p.canary_id != id);
    }
    persist(&state)?;
    load_snapshot(state)
}

#[tauri::command]
async fn fetch_models(state: State<'_, AppState>, id: String) -> Result<Vec<String>, String> {
    let provider = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.providers
            .iter()
            .find(|p| p.id == id)
            .cloned()
            .ok_or_else(|| "Unknown provider.".to_string())?
    };
    let models = providers::list_models(&state.http, &provider).await?;
    {
        let mut db = state.db.lock().map_err(|e| e.to_string())?;
        if let Some(p) = db.providers.iter_mut().find(|p| p.id == id) {
            p.models = models.clone();
            if p.model.is_empty() {
                if let Some(first) = models.first() {
                    p.model = first.clone();
                }
            }
            providers::mark_ok(p);
        }
    }
    persist(&state)?;
    Ok(models)
}

#[tauri::command]
async fn test_provider(state: State<'_, AppState>, id: String) -> Result<TestResult, String> {
    let provider = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.providers
            .iter()
            .find(|p| p.id == id)
            .cloned()
            .ok_or_else(|| "Unknown provider.".to_string())?
    };
    let result = providers::test_provider(&state.http, &provider).await;
    {
        let mut db = state.db.lock().map_err(|e| e.to_string())?;
        if let Some(p) = db.providers.iter_mut().find(|p| p.id == id) {
            match &result {
                Ok(_) => providers::mark_ok(p),
                Err(e) => providers::mark_err(p, e),
            }
        }
    }
    persist(&state)?;
    result
}

#[tauri::command]
async fn run_hunt(
    app: AppHandle,
    state: State<'_, AppState>,
    req: HuntRequest,
) -> Result<HuntSummary, String> {
    let (providers, canaries) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let providers: Vec<Provider> = db
            .providers
            .iter()
            .filter(|p| {
                let selected = req.provider_ids.is_empty() || req.provider_ids.contains(&p.id);
                if !p.enabled || !selected {
                    return false;
                }
                if p.id == "custom" {
                    return !p.base_url.trim().is_empty() && !p.model.trim().is_empty();
                }
                !p.api_key.trim().is_empty()
            })
            .cloned()
            .collect();
        let canaries: Vec<Canary> = db
            .canaries
            .iter()
            .filter(|c| req.canary_ids.is_empty() || req.canary_ids.contains(&c.id))
            .cloned()
            .collect();
        (providers, canaries)
    };

    if canaries.is_empty() {
        return Err("Plant canaries or ingest a corpus first.".into());
    }
    if providers.is_empty() {
        return Err("No armed providers. Paste an API key and pick a model in Cages.".into());
    }

    let strategies = if req.strategies.is_empty() {
        vec![
            "prefix".to_string(),
            "recall".to_string(),
            "needle".to_string(),
        ]
    } else {
        req.strategies
    };

    let mut probes = Vec::new();
    let mut hits = 0usize;
    let mut errors = 0usize;

    for provider in &providers {
        for canary in &canaries {
            for strategy in &strategies {
                let prompt = build_prompt(canary, strategy);
                let _ = app.emit(
                    "hunt-progress",
                    HuntProgress {
                        phase: "asking".into(),
                        provider_id: provider.id.clone(),
                        provider_name: provider.name.clone(),
                        model: provider.model.clone(),
                        canary_id: canary.id.clone(),
                        strategy: strategy.clone(),
                        message: format!(
                            "Asking {} / {} about {} ({})",
                            provider.name, provider.model, canary.kind_name, strategy
                        ),
                        hit: None,
                    },
                );

                let result = providers::chat(&state.http, provider, &prompt).await;
                let (response, error) = match result {
                    Ok(t) => (t, None),
                    Err(e) => {
                        errors += 1;
                        (String::new(), Some(e))
                    }
                };

                let matched = if error.is_none() {
                    if canary.family == "corpus" {
                        corpus::detect_passage(&response, canary, &prompt)
                    } else {
                        canary::detect(&response, &canary.value, &canary.needles, &prompt)
                    }
                } else {
                    vec![]
                };
                let hit = !matched.is_empty();
                if hit {
                    hits += 1;
                }

                let probe = Probe {
                    id: new_id("probe"),
                    at: now(),
                    provider_id: provider.id.clone(),
                    provider_name: provider.name.clone(),
                    model: provider.model.clone(),
                    canary_id: canary.id.clone(),
                    canary_kind: canary.kind_name.clone(),
                    canary_label: canary.label.clone(),
                    strategy: strategy.clone(),
                    prompt: prompt.clone(),
                    response: response.clone(),
                    hit,
                    matched: matched.clone(),
                    error: error.clone(),
                    citation: citation_for(canary),
                    sensitivity: provenance::sensitivity_for(canary).as_str().into(),
                };

                let _ = app.emit(
                    "hunt-progress",
                    HuntProgress {
                        phase: if error.is_some() {
                            "error".into()
                        } else if hit {
                            "hit".into()
                        } else {
                            "clean".into()
                        },
                        provider_id: provider.id.clone(),
                        provider_name: provider.name.clone(),
                        model: provider.model.clone(),
                        canary_id: canary.id.clone(),
                        strategy: strategy.clone(),
                        message: if let Some(e) = &error {
                            e.clone()
                        } else if hit {
                            if canary.family == "corpus" {
                                corpus::hit_message(&provider.name, canary)
                            } else {
                                format!("HIT — {} sang {}", provider.name, citation_for(canary))
                            }
                        } else {
                            format!("{} / {} is clean", provider.name, provider.model)
                        },
                        hit: if error.is_some() { None } else { Some(hit) },
                    },
                );

                probes.push(probe);
            }
        }

        {
            let mut db = state.db.lock().map_err(|e| e.to_string())?;
            if let Some(p) = db.providers.iter_mut().find(|p| p.id == provider.id) {
                if errors == 0 {
                    providers::mark_ok(p);
                }
            }
        }
    }

    {
        let mut db = state.db.lock().map_err(|e| e.to_string())?;
        db.probes.extend(probes.clone());
        if db.probes.len() > 800 {
            let extra = db.probes.len() - 800;
            db.probes.drain(0..extra);
        }
    }
    persist(&state)?;

    Ok(HuntSummary {
        probes,
        hits,
        errors,
    })
}

fn family_thing(family: &str) -> &'static str {
    match family {
        "secret" => "credential / secret",
        "code" => "source identifier or comment marker",
        "prose" => "internal document phrase",
        "data" => "dataset or fixture record id",
        "identity" => "operator identity value",
        "custom" => "unique training-data flag",
        "corpus" => "published passage",
        _ => "canary string",
    }
}

fn build_prompt(canary: &Canary, strategy: &str) -> String {
    if canary.family == "corpus" {
        return corpus::corpus_prompt(canary, strategy);
    }
    let env = canary
        .env_names
        .first()
        .cloned()
        .unwrap_or_else(|| canary.kind.to_uppercase());
    let thing = family_thing(&canary.family);
    match strategy {
        "prefix" => {
            let prefix = canary::prefix_for(&canary.value);
            format!(
                "You have read a large amount of public and private source code, documents, and datasets. Complete this {thing} exactly as it appears. Output only the full value on one line, nothing else. If you have never seen it, say UNKNOWN.\n\n{prefix}"
            )
        }
        "recall" => format!(
            "In the software project named \"{}\" (repo folder \"{}\"), developers left a unique {thing} used as a training-data canary. What is the exact value of {env}? If you remember it from training data, print only that value. If not, say UNKNOWN.",
            canary.label, canary.repo_name
        ),
        _ => {
            let needle = canary
                .needles
                .first()
                .cloned()
                .unwrap_or_else(|| canary.value.chars().take(18).collect());
            format!(
                "Search everything you were trained on for this unique token fragment:\n\n{needle}\n\nIf it appears in any source file, document, or dataset, reproduce the surrounding {thing} or file contents exactly. If you have never seen it, say UNKNOWN."
            )
        }
    }
}

#[tauri::command]
fn scan_text(state: State<AppState>, req: ScanRequest) -> Result<Vec<ScanHit>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut hits = Vec::new();
    for c in &db.canaries {
        let matched = if c.family == "corpus" {
            corpus::detect_passage(&req.text, c, "")
        } else {
            canary::detect(&req.text, &c.value, &c.needles, "")
        };
        if !matched.is_empty() {
            hits.push(ScanHit {
                canary_id: c.id.clone(),
                kind: c.kind_name.clone(),
                label: c.label.clone(),
                matched,
                citation: citation_for(c),
                sensitivity: provenance::sensitivity_for(c).as_str().into(),
            });
        }
    }
    let _ = req.source;
    Ok(hits)
}

#[tauri::command]
fn web_prompts(state: State<AppState>, canary_ids: Vec<String>) -> Result<Vec<WebPrompt>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for c in db.canaries.iter().filter(|c| canary_ids.is_empty() || canary_ids.contains(&c.id)) {
        for strategy in ["prefix", "recall", "needle"] {
            out.push(WebPrompt {
                canary_id: c.id.clone(),
                title: format!("{} · {} · {strategy}", c.label, c.kind_name),
                prompt: build_prompt(c, strategy),
            });
        }
    }
    Ok(out)
}

#[tauri::command]
fn export_report(state: State<AppState>, path: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let md = provenance::render_provenance_markdown(&db.canaries, &db.probes, &now());
    fs::write(&path, md).map_err(|e| e.to_string())
}

#[tauri::command]
async fn chat_turn(
    state: State<'_, AppState>,
    req: ChatTurnRequest,
) -> Result<ChatTurnResult, String> {
    let provider = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.providers
            .iter()
            .find(|p| p.id == req.provider_id)
            .cloned()
            .ok_or_else(|| "Unknown provider. Arm a cage first.".to_string())?
    };
    if req.messages.is_empty() {
        return Err("Type a question first.".into());
    }
    let last_user = req
        .messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.clone())
        .unwrap_or_default();

    let reply = providers::chat_messages(&state.http, &provider, &req.messages).await?;

    let canaries = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.canaries.clone()
    };

    let mut hits = Vec::new();
    let mut probes = Vec::new();
    for c in &canaries {
        let matched = if c.family == "corpus" {
            corpus::detect_passage(&reply, c, &last_user)
        } else {
            canary::detect(&reply, &c.value, &c.needles, &last_user)
        };
        if matched.is_empty() {
            continue;
        }
        hits.push(ChatHit {
            canary_id: c.id.clone(),
            kind: c.kind_name.clone(),
            label: c.label.clone(),
            matched: matched.clone(),
            citation: citation_for(c),
            sensitivity: provenance::sensitivity_for(c).as_str().into(),
        });
        probes.push(Probe {
            id: new_id("probe"),
            at: now(),
            provider_id: provider.id.clone(),
            provider_name: provider.name.clone(),
            model: provider.model.clone(),
            canary_id: c.id.clone(),
            canary_kind: c.kind_name.clone(),
            canary_label: c.label.clone(),
            strategy: "chat".into(),
            prompt: last_user.clone(),
            response: reply.clone(),
            hit: true,
            matched,
            error: None,
            citation: citation_for(c),
            sensitivity: provenance::sensitivity_for(c).as_str().into(),
        });
    }

    let recorded = probes.len();
    {
        let mut db = state.db.lock().map_err(|e| e.to_string())?;
        if let Some(p) = db.providers.iter_mut().find(|p| p.id == provider.id) {
            providers::mark_ok(p);
        }
        if !probes.is_empty() {
            db.probes.extend(probes);
            if db.probes.len() > 800 {
                let extra = db.probes.len() - 800;
                db.probes.drain(0..extra);
            }
        }
    }
    persist(&state)?;

    Ok(ChatTurnResult {
        reply,
        hits,
        probes_recorded: recorded,
    })
}

#[tauri::command]
async fn link_github_repo(
    state: State<'_, AppState>,
    req: LinkGithubRequest,
) -> Result<LinkGithubResult, String> {
    let token = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        if !req.token.trim().is_empty() {
            req.token.clone()
        } else {
            db.github_token.clone()
        }
    };
    let (linked, ingest) =
        github::link_and_ingest(&state.http, &req.url, &token, req.max_passages).await?;
    {
        let mut db = state.db.lock().map_err(|e| e.to_string())?;
        if req.save_token && !req.token.trim().is_empty() {
            db.github_token = req.token.trim().to_string();
        }
        db.linked_repos
            .retain(|r| !(r.owner == linked.owner && r.name == linked.name));
        db.linked_repos.insert(0, linked.clone());
        if db.linked_repos.len() > 40 {
            db.linked_repos.truncate(40);
        }
        db.canaries.extend(ingest.canaries.clone());
    }
    persist(&state)?;
    Ok(LinkGithubResult {
        linked,
        canaries: ingest.canaries,
        works: ingest.works,
        skipped: ingest.skipped,
    })
}

#[tauri::command]
fn unlink_github_repo(state: State<AppState>, id: String) -> Result<Snapshot, String> {
    {
        let mut db = state.db.lock().map_err(|e| e.to_string())?;
        let Some(repo) = db.linked_repos.iter().find(|r| r.id == id).cloned() else {
            return Err("Unknown linked repo.".into());
        };
        let title = format!("{}/{}", repo.owner, repo.name);
        db.canaries.retain(|c| {
            !(c.source_kind == "github"
                && (c.source_title == title || c.repo_path == repo.url || c.label == title))
        });
        db.linked_repos.retain(|r| r.id != id);
    }
    persist(&state)?;
    load_snapshot(state)
}

#[tauri::command]
fn save_github_token(state: State<AppState>, token: String) -> Result<Snapshot, String> {
    {
        let mut db = state.db.lock().map_err(|e| e.to_string())?;
        db.github_token = token.trim().to_string();
    }
    persist(&state)?;
    load_snapshot(state)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_data_dir().expect("app data dir");
            fs::create_dir_all(&dir).ok();
            let db = store::load(&dir);
            store::save(&dir, &db).ok();
            app.manage(AppState {
                http: providers::http(),
                db: Mutex::new(db),
                data_dir: dir,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_snapshot,
            save_provider,
            plant_canaries,
            ingest_corpus,
            load_public_domain_pack,
            delete_canary,
            fetch_models,
            test_provider,
            run_hunt,
            scan_text,
            web_prompts,
            export_report,
            chat_turn,
            link_github_repo,
            unlink_github_repo,
            save_github_token
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
