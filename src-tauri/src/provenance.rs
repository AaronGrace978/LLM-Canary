use std::collections::{BTreeMap, BTreeSet};

use crate::models::{citation_for, Canary, Probe, ProvenanceAnswer, StrategyStat};
use crate::score::{wilson, HIT_RUN_CHARS, HIT_RUN_SCORE, HIT_SCORE};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sensitivity {
    Public,
    Private,
}

impl Sensitivity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Sensitivity::Public => "public",
            Sensitivity::Private => "private",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Sensitivity::Public => "Public / expected",
            Sensitivity::Private => "Private / unique",
        }
    }
}

pub fn sensitivity_for(canary: &Canary) -> Sensitivity {
    match canary.source_kind.as_str() {
        "public_domain" => Sensitivity::Public,
        _ => Sensitivity::Private,
    }
}

#[derive(Debug, Clone)]
pub struct SourceAnswer {
    pub provider_name: String,
    pub model: String,
    pub source_title: String,
    pub citation: String,
    pub sensitivity: Sensitivity,
    pub hit_count: usize,
    pub canary_ids: Vec<String>,
}

pub fn build_source_answers(canaries: &[Canary], probes: &[Probe]) -> Vec<SourceAnswer> {
    let by_id: BTreeMap<&str, &Canary> = canaries.iter().map(|c| (c.id.as_str(), c)).collect();
    let mut map: BTreeMap<(String, String, String), SourceAnswer> = BTreeMap::new();

    for p in probes.iter().filter(|p| p.hit && !p.control) {
        let Some(c) = by_id.get(p.canary_id.as_str()) else {
            continue;
        };
        let title = if c.source_title.trim().is_empty() {
            c.label.clone()
        } else {
            c.source_title.clone()
        };
        let key = (
            p.provider_name.clone(),
            p.model.clone(),
            title.clone(),
        );
        let entry = map.entry(key).or_insert_with(|| SourceAnswer {
            provider_name: p.provider_name.clone(),
            model: p.model.clone(),
            source_title: title,
            citation: if p.citation.trim().is_empty() {
                citation_for(c)
            } else {
                p.citation.clone()
            },
            sensitivity: sensitivity_for(c),
            hit_count: 0,
            canary_ids: Vec::new(),
        });
        entry.hit_count += 1;
        if !entry.canary_ids.contains(&c.id) {
            entry.canary_ids.push(c.id.clone());
        }
    }

    let mut out: Vec<_> = map.into_values().collect();
    out.sort_by(|a, b| {
        b.private_first()
            .cmp(&a.private_first())
            .then(b.hit_count.cmp(&a.hit_count))
            .then(a.provider_name.cmp(&b.provider_name))
            .then(a.source_title.cmp(&b.source_title))
    });
    out
}

impl SourceAnswer {
    fn private_first(&self) -> u8 {
        match self.sensitivity {
            Sensitivity::Private => 1,
            Sensitivity::Public => 0,
        }
    }
}

#[derive(Default)]
struct Tally {
    probes: usize,
    hits: usize,
    score_sum: f32,
    abstained: usize,
    private_probes: usize,
    public_probes: usize,
    control_probes: usize,
    control_hits: usize,
    errors: usize,
    by_strategy: BTreeMap<String, (usize, usize, f32)>,
}

/// Every model that answered at least one probe gets a row, including models
/// with zero hits — a benchmark without denominators is just a list of anecdotes.
pub fn build_provider_answers(canaries: &[Canary], probes: &[Probe]) -> Vec<ProvenanceAnswer> {
    let by_id: BTreeMap<&str, &Canary> = canaries.iter().map(|c| (c.id.as_str(), c)).collect();
    let mut tallies: BTreeMap<(String, String), Tally> = BTreeMap::new();

    for p in probes {
        let t = tallies
            .entry((p.provider_name.clone(), p.model.clone()))
            .or_default();
        if p.error.is_some() {
            t.errors += 1;
            continue;
        }
        if p.control {
            t.control_probes += 1;
            if p.hit {
                t.control_hits += 1;
            }
            continue;
        }
        t.probes += 1;
        t.score_sum += p.score;
        if p.hit {
            t.hits += 1;
        }
        if p.abstained {
            t.abstained += 1;
        }
        let public = match by_id.get(p.canary_id.as_str()) {
            Some(c) => sensitivity_for(c) == Sensitivity::Public,
            None => p.sensitivity == "public",
        };
        if public {
            t.public_probes += 1;
        } else {
            t.private_probes += 1;
        }
        let s = t.by_strategy.entry(p.strategy.clone()).or_default();
        s.0 += 1;
        if p.hit {
            s.1 += 1;
        }
        s.2 += p.score;
    }

    let sources = build_source_answers(canaries, probes);
    let mut out: Vec<ProvenanceAnswer> = tallies
        .into_iter()
        .map(|((provider_name, model), t)| {
            let (ci_low, ci_high) = wilson(t.hits, t.probes);
            let rate = |h: usize, n: usize| if n == 0 { 0.0 } else { h as f32 / n as f32 };
            ProvenanceAnswer {
                provider_name,
                model,
                public_sources: Vec::new(),
                private_sources: Vec::new(),
                public_hits: 0,
                private_hits: 0,
                probes: t.probes,
                hits: t.hits,
                hit_rate: rate(t.hits, t.probes),
                ci_low,
                ci_high,
                mean_score: if t.probes == 0 { 0.0 } else { t.score_sum / t.probes as f32 },
                abstain_rate: rate(t.abstained, t.probes),
                private_probes: t.private_probes,
                public_probes: t.public_probes,
                control_probes: t.control_probes,
                control_hits: t.control_hits,
                errors: t.errors,
                by_strategy: t
                    .by_strategy
                    .into_iter()
                    .map(|(strategy, (n, h, sum))| StrategyStat {
                        strategy,
                        probes: n,
                        hits: h,
                        hit_rate: rate(h, n),
                        mean_score: if n == 0 { 0.0 } else { sum / n as f32 },
                    })
                    .collect(),
            }
        })
        .collect();

    for s in sources {
        let Some(entry) = out
            .iter_mut()
            .find(|a| a.provider_name == s.provider_name && a.model == s.model)
        else {
            continue;
        };
        match s.sensitivity {
            Sensitivity::Public => {
                entry.public_hits += s.hit_count;
                if !entry.public_sources.contains(&s.source_title) {
                    entry.public_sources.push(s.source_title);
                }
            }
            Sensitivity::Private => {
                entry.private_hits += s.hit_count;
                if !entry.private_sources.contains(&s.source_title) {
                    entry.private_sources.push(s.source_title);
                }
            }
        }
    }

    out.sort_by(|a, b| {
        b.private_hits
            .cmp(&a.private_hits)
            .then(
                b.hit_rate
                    .partial_cmp(&a.hit_rate)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
            .then(b.public_hits.cmp(&a.public_hits))
            .then(a.provider_name.cmp(&b.provider_name))
    });
    out
}

fn pct(x: f32) -> String {
    format!("{:.0}%", x * 100.0)
}

pub fn render_provenance_markdown(canaries: &[Canary], probes: &[Probe], generated_at: &str) -> String {
    let answers = build_provider_answers(canaries, probes);
    let sources = build_source_answers(canaries, probes);
    let hits: Vec<&Probe> = probes.iter().filter(|p| p.hit && !p.control).collect();
    let scored = probes.iter().filter(|p| p.error.is_none() && !p.control).count();
    let controls = probes.iter().filter(|p| p.error.is_none() && p.control).count();
    let control_hits = probes.iter().filter(|p| p.control && p.hit).count();
    let private_sources: BTreeSet<_> = canaries
        .iter()
        .filter(|c| sensitivity_for(c) == Sensitivity::Private)
        .map(|c| {
            if c.source_title.trim().is_empty() {
                c.label.clone()
            } else {
                c.source_title.clone()
            }
        })
        .collect();
    let public_sources: BTreeSet<_> = canaries
        .iter()
        .filter(|c| sensitivity_for(c) == Sensitivity::Public)
        .map(|c| c.source_title.clone())
        .collect();

    let mut md = String::from("# LLM Canary — training provenance report\n\n");
    md.push_str(&format!("Generated: {generated_at}\n\n"));
    md.push_str(
        "This report answers a narrow, evidence-based question: **for the sources you loaded or planted, which models reproduced them?**\n\n",
    );
    md.push_str(
        "It does **not** dump a lab's full training set. It cuts through marketing noise by separating **public / expected** hits (famous public-domain works) from **private / unique** hits (your files, planted canaries, unpublished text).\n\n",
    );
    md.push_str(&format!(
        "- Sources under watch: {} private · {} public\n- Scored probes: {} ({} hits)\n- Negative controls: {} ({} false positive{})\n\n",
        private_sources.len(),
        public_sources.len(),
        scored,
        hits.len(),
        controls,
        control_hits,
        if control_hits == 1 { "" } else { "s" }
    ));

    md.push_str("## Methodology\n\n");
    md.push_str(&format!(
        "- **Prompting.** Each source gets prefix-completion, title-recall, and needle prompts. The full target is never sent; only a prefix, a five-word stem, or a short interior fragment.\n\
         - **Decoding.** Extraction probes run at temperature 0 (greedy) so repeated trials measure the model, not the sampler. Trials per prompt are recorded on every probe.\n\
         - **Scoring.** Reply and target are normalized (case, whitespace, quotes, dashes). The part of the target the prompt already revealed is removed; the score is the longest verbatim run the model produced divided by the length of that hidden remainder.\n\
         - **Hit threshold.** score ≥ {} , or a verbatim run of ≥ {} characters with score ≥ {}. Exact matches of a planted random token (≥ 10 chars) also count.\n\
         - **Negative controls.** When enabled, every prompt is also scored against a scrambled decoy of its target. A control hit is a detector false positive; a clean control run bounds the false-positive rate.\n\
         - **Uncertainty.** Hit rates carry a 95% Wilson interval. Wide intervals mean run more trials.\n\n",
        HIT_SCORE, HIT_RUN_CHARS, HIT_RUN_SCORE
    ));

    md.push_str("## Benchmark by model\n\n");
    if answers.is_empty() {
        md.push_str("No probes recorded yet. Hunt after loading sources.\n\n");
    } else {
        md.push_str("| Model | Probes | Hits | Hit rate (95% CI) | Mean score | Abstain | Private | Public | Controls (FP) |\n");
        md.push_str("| --- | ---: | ---: | --- | ---: | ---: | ---: | ---: | --- |\n");
        for a in &answers {
            md.push_str(&format!(
                "| {} / {} | {} | {} | {} ({}–{}) | {} | {} | {}/{} | {}/{} | {} ({}) |\n",
                a.provider_name,
                a.model,
                a.probes,
                a.hits,
                pct(a.hit_rate),
                pct(a.ci_low),
                pct(a.ci_high),
                pct(a.mean_score),
                pct(a.abstain_rate),
                a.private_hits,
                a.private_probes,
                a.public_hits,
                a.public_probes,
                a.control_probes,
                a.control_hits
            ));
        }
        md.push('\n');
        for a in &answers {
            if a.by_strategy.is_empty() {
                continue;
            }
            md.push_str(&format!(
                "- {} / {} by strategy: {}\n",
                a.provider_name,
                a.model,
                a.by_strategy
                    .iter()
                    .map(|s| format!(
                        "{} {}/{} ({}, mean {})",
                        s.strategy,
                        s.hits,
                        s.probes,
                        pct(s.hit_rate),
                        pct(s.mean_score)
                    ))
                    .collect::<Vec<_>>()
                    .join("; ")
            ));
        }
        md.push('\n');
    }

    md.push_str("## Answers by model\n\n");
    if answers.is_empty() {
        md.push_str("No regurgitations recorded yet. Hunt after loading sources.\n\n");
    } else {
        for a in &answers {
            md.push_str(&format!("### {} / {}\n\n", a.provider_name, a.model));
            if a.private_sources.is_empty() {
                md.push_str("- **Private / unique sources:** none detected\n");
            } else {
                md.push_str(&format!(
                    "- **Private / unique sources (smoking gun):** {}\n",
                    a.private_sources.join("; ")
                ));
            }
            if a.public_sources.is_empty() {
                md.push_str("- **Public / expected sources:** none detected\n");
            } else {
                md.push_str(&format!(
                    "- **Public / expected sources (calibration):** {}\n",
                    a.public_sources.join("; ")
                ));
            }
            md.push_str(&format!(
                "- Hit counts: {} private · {} public\n\n",
                a.private_hits, a.public_hits
            ));
        }
    }

    md.push_str("## Source ledger\n\n");
    if sources.is_empty() {
        md.push_str("No source-level hits yet.\n\n");
    } else {
        for s in &sources {
            md.push_str(&format!(
                "- **{}** — {} / {} — {} ({} hit{})\n",
                s.citation,
                s.provider_name,
                s.model,
                s.sensitivity.label(),
                s.hit_count,
                if s.hit_count == 1 { "" } else { "s" }
            ));
        }
        md.push('\n');
    }

    md.push_str("## Raw evidence\n\n");
    if hits.is_empty() {
        md.push_str("No raw hits.\n");
    }
    for p in hits {
        md.push_str(&format!(
            "### HIT — {} / {}\n\n- When: {}\n- Source: {}\n- Canary: {} ({})\n- Strategy: {} · trial {} · temperature {}\n- Score: {} of hidden remainder verbatim\n- Matched: {}\n\n#### Prompt\n\n```\n{}\n```\n\n#### Response\n\n```\n{}\n```\n\n",
            p.provider_name,
            p.model,
            p.at,
            if p.citation.is_empty() {
                p.canary_label.clone()
            } else {
                p.citation.clone()
            },
            p.canary_label,
            p.canary_kind,
            p.strategy,
            p.trial,
            p.temperature,
            pct(p.score),
            p.matched.join(", "),
            p.prompt,
            p.response
        ));
    }
    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::now;

    fn canary(id: &str, title: &str, kind: &str) -> Canary {
        Canary {
            id: id.into(),
            kind: "corpus_passage".into(),
            kind_name: "Corpus passage".into(),
            value: format!("unique passage for {title}"),
            needles: vec![],
            env_names: vec![],
            label: title.into(),
            repo_path: String::new(),
            repo_name: title.into(),
            files: vec![],
            planted_at: now(),
            family: "corpus".into(),
            source_title: title.into(),
            source_locator: "Ch. 1".into(),
            source_kind: kind.into(),
        }
    }

    fn hit(id: &str, canary_id: &str, provider: &str, citation: &str) -> Probe {
        Probe {
            id: id.into(),
            at: now(),
            provider_id: provider.to_lowercase(),
            provider_name: provider.into(),
            model: "test-model".into(),
            canary_id: canary_id.into(),
            canary_kind: "Corpus passage".into(),
            canary_label: "x".into(),
            strategy: "prefix".into(),
            prompt: "p".into(),
            response: "r".into(),
            hit: true,
            matched: vec!["x".into()],
            error: None,
            citation: citation.into(),
            sensitivity: if citation.starts_with("Moby") {
                "public".into()
            } else {
                "private".into()
            },
            score: 1.0,
            trial: 1,
            control: false,
            abstained: false,
            temperature: 0.0,
        }
    }

    fn miss(id: &str, canary_id: &str, provider: &str, control: bool) -> Probe {
        let mut p = hit(id, canary_id, provider, "Internal Wiki · Ch. 1");
        p.hit = false;
        p.matched = vec![];
        p.score = 0.1;
        p.control = control;
        p.abstained = !control;
        p
    }

    #[test]
    fn zero_hit_models_still_get_a_row_with_rates() {
        let canaries = vec![canary("c2", "Internal Wiki", "imported")];
        let probes = vec![
            hit("p1", "c2", "OpenAI", "Internal Wiki · Ch. 1"),
            miss("p2", "c2", "OpenAI", false),
            miss("p3", "c2", "Anthropic", false),
            miss("p4", "c2", "Anthropic", false),
            miss("p5", "c2", "Anthropic", true),
        ];
        let answers = build_provider_answers(&canaries, &probes);
        assert_eq!(answers.len(), 2);
        let openai = &answers[0];
        assert_eq!(openai.provider_name, "OpenAI");
        assert_eq!((openai.probes, openai.hits), (2, 1));
        assert!((openai.hit_rate - 0.5).abs() < 1e-6);
        assert!(openai.ci_low < 0.5 && openai.ci_high > 0.5);
        assert_eq!(openai.by_strategy.len(), 1);
        assert_eq!(openai.by_strategy[0].strategy, "prefix");

        let anthropic = &answers[1];
        assert_eq!((anthropic.probes, anthropic.hits), (2, 0));
        assert_eq!(anthropic.hit_rate, 0.0);
        assert_eq!((anthropic.control_probes, anthropic.control_hits), (1, 0));
        assert!((anthropic.abstain_rate - 1.0).abs() < 1e-6);
        assert!(anthropic.private_sources.is_empty());

        let md = render_provenance_markdown(&canaries, &probes, "now");
        assert!(md.contains("## Methodology"));
        assert!(md.contains("## Benchmark by model"));
        assert!(md.contains("| Anthropic / test-model | 2 | 0 | 0%"));
    }

    #[test]
    fn separates_public_and_private_answers() {
        let canaries = vec![
            canary("c1", "Moby-Dick", "public_domain"),
            canary("c2", "Internal Wiki", "imported"),
        ];
        let probes = vec![
            hit("p1", "c1", "OpenAI", "Moby-Dick · Ch. 1"),
            hit("p2", "c2", "OpenAI", "Internal Wiki · Ch. 1"),
            hit("p3", "c1", "Anthropic", "Moby-Dick · Ch. 1"),
        ];
        let answers = build_provider_answers(&canaries, &probes);
        assert_eq!(answers.len(), 2);
        let openai = answers.iter().find(|a| a.provider_name == "OpenAI").unwrap();
        assert_eq!(openai.private_sources, vec!["Internal Wiki".to_string()]);
        assert_eq!(openai.public_sources, vec!["Moby-Dick".to_string()]);
        let anthropic = answers.iter().find(|a| a.provider_name == "Anthropic").unwrap();
        assert!(anthropic.private_sources.is_empty());
        assert_eq!(anthropic.public_sources, vec!["Moby-Dick".to_string()]);

        let md = render_provenance_markdown(&canaries, &probes, "now");
        assert!(md.contains("training provenance report"));
        assert!(md.contains("smoking gun"));
        assert!(md.contains("Internal Wiki"));
    }
}
