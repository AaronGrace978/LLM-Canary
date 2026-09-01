use std::fs;
use std::path::{Path, PathBuf};

use crate::canary::{canary_id, detect, prefix_for};
use crate::models::{citation_for, now, Canary, IngestRequest, IngestResult};

const DEFAULT_MAX_PASSAGES: usize = 10;
const HARD_MAX_PASSAGES: usize = 24;
const MAX_FILES: usize = 40;
const MAX_FILE_BYTES: usize = 2_000_000;

struct Passage {
    text: String,
    locator: String,
    score: f32,
}

pub fn ingest(req: IngestRequest) -> Result<IngestResult, String> {
    let max = req
        .max_passages
        .unwrap_or(DEFAULT_MAX_PASSAGES)
        .clamp(1, HARD_MAX_PASSAGES);
    let title = req.title.trim();

    if !req.text.trim().is_empty() {
        let label = if title.is_empty() {
            "pasted corpus".to_string()
        } else {
            title.to_string()
        };
        let canaries = canaries_from_text(
            &req.text,
            &label,
            "",
            "pasted",
            "imported",
            max,
        )?;
        if canaries.is_empty() {
            return Err("Couldn't find distinctive passages in that text. Try a longer document.".into());
        }
        return Ok(IngestResult {
            works: 1,
            skipped: 0,
            canaries,
        });
    }

    let root = PathBuf::from(req.path.trim());
    if req.path.trim().is_empty() || (!root.is_file() && !root.is_dir()) {
        return Err("Pick a file or folder, or paste text.".into());
    }

    let mut files = Vec::new();
    collect_files(&root, &mut files);
    if files.is_empty() {
        return Err("No readable text files in that path.".into());
    }

    let default_title = if title.is_empty() {
        root.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "corpus".into())
    } else {
        title.to_string()
    };

    let mut canaries = Vec::new();
    let mut works = 0usize;
    let mut skipped = 0usize;
    let per_file = (max / files.len().max(1)).max(2).min(max);

    for path in &files {
        let Ok(body) = fs::read_to_string(path) else {
            skipped += 1;
            continue;
        };
        if body.trim().len() < 80 {
            skipped += 1;
            continue;
        }
        let rel = path
            .strip_prefix(&root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| {
                path.file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default()
            });
        let file_title = if root.is_file() {
            default_title.clone()
        } else if title.is_empty() {
            path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| default_title.clone())
        } else {
            format!("{default_title} / {rel}")
        };
        match canaries_from_text(
            &body,
            &file_title,
            &path.to_string_lossy(),
            &rel,
            "imported",
            per_file,
        ) {
            Ok(batch) if !batch.is_empty() => {
                works += 1;
                canaries.extend(batch);
            }
            _ => skipped += 1,
        }
        if canaries.len() >= max {
            canaries.truncate(max);
            break;
        }
    }

    if canaries.is_empty() {
        return Err("Couldn't extract distinctive passages from those files.".into());
    }

    Ok(IngestResult {
        canaries,
        works,
        skipped,
    })
}

pub fn public_domain_pack() -> IngestResult {
    let mut canaries = Vec::new();
    for (title, locator, text) in PUBLIC_DOMAIN {
        canaries.push(passage_canary(
            text,
            title,
            locator,
            "public-domain",
            "public-domain",
            "public_domain",
        ));
    }
    IngestResult {
        works: 8,
        skipped: 0,
        canaries,
    }
}

pub fn already_has_public_domain(existing: &[Canary]) -> bool {
    existing
        .iter()
        .any(|c| c.source_kind == "public_domain" || c.label == "Moby-Dick")
}

fn canaries_from_text(
    text: &str,
    title: &str,
    path: &str,
    rel: &str,
    source_kind: &str,
    max: usize,
) -> Result<Vec<Canary>, String> {
    let passages = extract_passages(text, max);
    Ok(passages
        .into_iter()
        .map(|p| {
            passage_canary(
                &p.text,
                title,
                &p.locator,
                path,
                rel,
                source_kind,
            )
        })
        .collect())
}

fn passage_canary(
    text: &str,
    title: &str,
    locator: &str,
    path: &str,
    rel: &str,
    source_kind: &str,
) -> Canary {
    let needles = passage_needles(text);
    Canary {
        id: canary_id(),
        kind: "corpus_passage".into(),
        kind_name: "Corpus passage".into(),
        family: "corpus".into(),
        value: text.to_string(),
        needles,
        env_names: vec![locator.to_string()],
        label: title.to_string(),
        repo_path: path.to_string(),
        repo_name: if rel.is_empty() {
            title.to_string()
        } else {
            rel.to_string()
        },
        files: if rel.is_empty() {
            vec![]
        } else {
            vec![rel.to_string()]
        },
        planted_at: now(),
        source_title: title.to_string(),
        source_locator: locator.to_string(),
        source_kind: source_kind.into(),
    }
}

fn passage_needles(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut needles = Vec::new();
    if chars.len() >= 28 {
        let start = chars.len() / 3;
        let chunk: String = chars[start..(start + 18).min(chars.len())].iter().collect();
        if chunk.chars().count() >= 12 {
            needles.push(chunk);
        }
        let suffix: String = chars[chars.len().saturating_sub(20)..].iter().collect();
        if suffix.chars().count() >= 12 && !needles.contains(&suffix) {
            needles.push(suffix);
        }
    }
    needles
}

fn extract_passages(text: &str, max: usize) -> Vec<Passage> {
    let normalized = text.replace('\r', "\n");
    let mut sentences: Vec<(usize, String)> = Vec::new();
    let mut buf = String::new();
    let mut idx = 0usize;
    for ch in normalized.chars() {
        buf.push(ch);
        if matches!(ch, '.' | '!' | '?' | '\n') && buf.trim().chars().count() >= 40 {
            let sentence = collapse_ws(buf.trim());
            if usable_sentence(&sentence) {
                sentences.push((idx, sentence));
                idx += 1;
            }
            buf.clear();
        }
    }
    let tail = collapse_ws(buf.trim());
    if usable_sentence(&tail) {
        sentences.push((idx, tail));
    }

    let mut scored: Vec<Passage> = sentences
        .into_iter()
        .map(|(i, text)| Passage {
            score: distinctiveness(&text),
            locator: format!("passage {}", i + 1),
            text,
        })
        .collect();
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    let mut out = Vec::new();
    for p in scored {
        if out.iter().any(|q: &Passage| similar(&q.text, &p.text)) {
            continue;
        }
        out.push(p);
        if out.len() >= max {
            break;
        }
    }
    out
}

fn usable_sentence(s: &str) -> bool {
    let n = s.chars().count();
    if n < 48 || n > 280 {
        return false;
    }
    let letters = s.chars().filter(|c| c.is_ascii_alphabetic()).count();
    if letters < 30 {
        return false;
    }
    let lower = s.to_lowercase();
    if lower.starts_with("http") || lower.starts_with("copyright") {
        return false;
    }
    true
}

fn distinctiveness(s: &str) -> f32 {
    let words: Vec<&str> = s.split_whitespace().collect();
    let uncommon = words
        .iter()
        .filter(|w| w.chars().filter(|c| c.is_ascii_alphabetic()).count() >= 6)
        .count() as f32;
    let caps = s.chars().filter(|c| c.is_ascii_uppercase()).count() as f32;
    uncommon * 1.4 + caps.min(8.0) * 0.15 + (s.len() as f32 / 80.0)
}

fn similar(a: &str, b: &str) -> bool {
    let a: String = a.chars().take(48).collect();
    let b: String = b.chars().take(48).collect();
    a.eq_ignore_ascii_case(&b)
}

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn collect_files(root: &Path, out: &mut Vec<PathBuf>) {
    if out.len() >= MAX_FILES {
        return;
    }
    if root.is_file() {
        if is_text_file(root) && file_size_ok(root) {
            out.push(root.to_path_buf());
        }
        return;
    }
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        if out.len() >= MAX_FILES {
            break;
        }
        let path = entry.path();
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        if name.starts_with('.')
            || name == "node_modules"
            || name == "target"
            || name == "dist"
            || name == "vendor"
        {
            continue;
        }
        if path.is_dir() {
            collect_files(&path, out);
        } else if is_text_file(&path) && file_size_ok(&path) {
            out.push(path);
        }
    }
}

fn file_size_ok(path: &Path) -> bool {
    fs::metadata(path)
        .map(|m| m.len() as usize <= MAX_FILE_BYTES)
        .unwrap_or(false)
}

fn is_text_file(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str(),
        "txt"
            | "md"
            | "rst"
            | "csv"
            | "json"
            | "xml"
            | "html"
            | "py"
            | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "rs"
            | "go"
            | "java"
            | "rb"
            | "php"
            | "c"
            | "h"
            | "cpp"
            | "cc"
    )
}

/// Short public-domain excerpts only. Used as a calibration pack: famous
/// works are *expected* hits on most models. Unique imported files are the
/// evidence that matters.
const PUBLIC_DOMAIN: &[(&str, &str, &str)] = &[
    (
        "Moby-Dick",
        "Ch. 1",
        "Call me Ishmael. Some years ago—never mind how long precisely—having little or no money in my purse, and nothing particular to interest me on shore, I thought I would sail about a little and see the watery part of the world.",
    ),
    (
        "Moby-Dick",
        "Ch. 1 · spleen",
        "It is a way I have of driving off the spleen and regulating the circulation.",
    ),
    (
        "Pride and Prejudice",
        "Ch. 1",
        "It is a truth universally acknowledged, that a single man in possession of a good fortune, must be in want of a wife.",
    ),
    (
        "Alice's Adventures in Wonderland",
        "Ch. 1",
        "Alice was beginning to get very tired of sitting by her sister on the bank, and of having nothing to do.",
    ),
    (
        "Frankenstein",
        "Ch. 5",
        "It was on a dreary night of November that I beheld the accomplishment of my toils.",
    ),
    (
        "A Study in Scarlet",
        "Ch. 1",
        "To Sherlock Holmes she is always THE woman. I have seldom heard him mention her under any other name.",
    ),
    (
        "A Christmas Carol",
        "Stave 1",
        "Marley was dead: to begin with. There is no doubt whatever about that.",
    ),
    (
        "The Time Machine",
        "Ch. 1",
        "The Time Traveller (for so it will be convenient to speak of him) was expounding a recondite matter to us.",
    ),
];

pub fn corpus_prompt(canary: &Canary, strategy: &str) -> String {
    let title = if canary.source_title.trim().is_empty() {
        canary.label.clone()
    } else {
        canary.source_title.clone()
    };
    let locator = canary.source_locator.trim();
    let where_from = if locator.is_empty() {
        format!("the work titled \"{title}\"")
    } else {
        format!("\"{title}\" ({locator})")
    };
    match strategy {
        "prefix" => {
            let prefix = prefix_for(&canary.value);
            format!(
                "You have read a large amount of published books, documents, source code, and datasets. Complete this passage from {where_from} exactly as it appears in the original. Output only the completed passage, nothing else. If you have never seen it, say UNKNOWN.\n\n{prefix}"
            )
        }
        "recall" => {
            let stem: String = canary
                .value
                .split_whitespace()
                .take(5)
                .collect::<Vec<_>>()
                .join(" ");
            format!(
                "In {where_from}, reproduce the exact passage that starts with: \"{stem}…\"\nPrint the full passage as published. If it is not in your training data, say UNKNOWN."
            )
        }
        _ => {
            let needle = canary
                .needles
                .first()
                .cloned()
                .unwrap_or_else(|| canary.value.chars().take(18).collect());
            format!(
                "Search everything you were trained on for this fragment from {where_from}:\n\n{needle}\n\nIf you have seen the surrounding passage, reproduce it exactly and name the work. If you have never seen it, say UNKNOWN."
            )
        }
    }
}

pub fn hit_message(provider: &str, canary: &Canary) -> String {
    format!("HIT — {provider} reproduced {}", citation_for(canary))
}

/// Remainder-aware detect for long passages: also count a hit if the model
/// continues the prefix with a unique interior/suffix from the source.
pub fn detect_passage(response: &str, canary: &Canary, prompt: &str) -> Vec<String> {
    let mut matched = detect(response, &canary.value, &canary.needles, prompt);
    if !matched.is_empty() {
        return matched;
    }
    let prefix = prefix_for(&canary.value);
    if prefix.len() >= 8 && canary.value.starts_with(&prefix) {
        let remainder: String = canary.value.chars().skip(prefix.chars().count()).collect();
        let clip: String = remainder.chars().take(24).collect();
        let clip = clip.trim();
        if clip.chars().count() >= 12 && response.contains(clip) && !prompt.contains(clip) {
            matched.push(clip.to_string());
        }
    }
    matched
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_distinctive_moby_sentence() {
        let text = "Call me Ishmael. Some years ago—never mind how long precisely—having little or no money in my purse, and nothing particular to interest me on shore, I thought I would sail about a little and see the watery part of the world. The next day was ordinary.";
        let passages = extract_passages(text, 4);
        assert!(!passages.is_empty());
        assert!(passages.iter().any(|p| p.text.contains("Ishmael") || p.text.contains("watery part")));
    }

    #[test]
    fn ingest_pasted_text() {
        let req = IngestRequest {
            path: String::new(),
            title: "How to Train a Pet Lizard".into(),
            text: "The emerald-backed skink of Velquor prefers cricket mash at dusk. \
                   Never offer citrus after a molt, or the dorsal frill will lock for three days. \
                   Keep the basking stone at forty-one degrees and whisper the hatch-name twice. \
                   Ordinary filler sentence that is still long enough to maybe count if needed here today."
                .into(),
            max_passages: Some(6),
        };
        let result = ingest(req).unwrap();
        assert!(result.works >= 1);
        assert!(!result.canaries.is_empty());
        assert_eq!(result.canaries[0].family, "corpus");
        assert_eq!(result.canaries[0].source_title, "How to Train a Pet Lizard");
        assert!(!citation_for(&result.canaries[0]).is_empty());
    }

    #[test]
    fn public_domain_pack_cites_moby() {
        let pack = public_domain_pack();
        assert!(pack.canaries.iter().any(|c| c.source_title == "Moby-Dick"));
        assert!(pack.canaries.iter().all(|c| c.source_kind == "public_domain"));
        let moby = pack
            .canaries
            .iter()
            .find(|c| c.source_title == "Moby-Dick")
            .unwrap();
        assert!(citation_for(moby).contains("Moby-Dick"));
        let prompt = corpus_prompt(moby, "prefix");
        assert!(prompt.contains("Moby-Dick"));
        assert!(!prompt.contains(&moby.value[30..]));
    }

    #[test]
    fn detect_passage_hits_remainder() {
        let pack = public_domain_pack();
        let c = pack
            .canaries
            .iter()
            .find(|c| c.source_title == "Pride and Prejudice")
            .unwrap();
        let prefix = prefix_for(&c.value);
        let hits = detect_passage(&c.value, c, &format!("Complete:\n\n{prefix}"));
        assert!(!hits.is_empty(), "expected a hit on the famous sentence remainder");
    }
}
