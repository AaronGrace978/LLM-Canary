use std::collections::{BTreeMap, BTreeSet};

use crate::models::{citation_for, Canary, Probe};

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

#[derive(Debug, Clone)]
pub struct ProviderAnswer {
    pub provider_name: String,
    pub model: String,
    pub public_sources: Vec<String>,
    pub private_sources: Vec<String>,
    pub public_hits: usize,
    pub private_hits: usize,
}

pub fn build_source_answers(canaries: &[Canary], probes: &[Probe]) -> Vec<SourceAnswer> {
    let by_id: BTreeMap<&str, &Canary> = canaries.iter().map(|c| (c.id.as_str(), c)).collect();
    let mut map: BTreeMap<(String, String, String), SourceAnswer> = BTreeMap::new();

    for p in probes.iter().filter(|p| p.hit) {
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

pub fn build_provider_answers(canaries: &[Canary], probes: &[Probe]) -> Vec<ProviderAnswer> {
    let sources = build_source_answers(canaries, probes);
    let mut map: BTreeMap<(String, String), ProviderAnswer> = BTreeMap::new();

    for s in sources {
        let key = (s.provider_name.clone(), s.model.clone());
        let entry = map.entry(key).or_insert_with(|| ProviderAnswer {
            provider_name: s.provider_name.clone(),
            model: s.model.clone(),
            public_sources: Vec::new(),
            private_sources: Vec::new(),
            public_hits: 0,
            private_hits: 0,
        });
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

    let mut out: Vec<_> = map.into_values().collect();
    out.sort_by(|a, b| {
        b.private_hits
            .cmp(&a.private_hits)
            .then(b.public_hits.cmp(&a.public_hits))
            .then(a.provider_name.cmp(&b.provider_name))
    });
    out
}

pub fn render_provenance_markdown(canaries: &[Canary], probes: &[Probe], generated_at: &str) -> String {
    let answers = build_provider_answers(canaries, probes);
    let sources = build_source_answers(canaries, probes);
    let hits: Vec<&Probe> = probes.iter().filter(|p| p.hit).collect();
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
        "- Sources under watch: {} private · {} public\n- Probes: {}\n- Hits: {}\n\n",
        private_sources.len(),
        public_sources.len(),
        probes.len(),
        hits.len()
    ));

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
            "### HIT — {} / {}\n\n- When: {}\n- Source: {}\n- Canary: {} ({})\n- Strategy: {}\n- Matched: {}\n\n#### Prompt\n\n```\n{}\n```\n\n#### Response\n\n```\n{}\n```\n\n",
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
        }
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
