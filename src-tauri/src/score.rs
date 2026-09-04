//! Continuous memorization scoring.
//!
//! Exact `contains` checks are brittle: a curly quote, an em-dash, or a line
//! wrap in the reply turns a verbatim reproduction into a miss, and partial
//! recall is never measured. This module normalizes both sides, removes the
//! part of the target the prompt already revealed, and scores the longest
//! verbatim run the model produced as a fraction of what it had to recall.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

/// Fraction of the hidden remainder that must come back verbatim to count.
pub const HIT_SCORE: f32 = 0.5;
/// A long verbatim run is a hit even when the passage is much longer.
pub const HIT_RUN_CHARS: usize = 40;
pub const HIT_RUN_SCORE: f32 = 0.25;
/// Below this the reply is treated as not recalling anything.
const NOISE_SCORE: f32 = 0.15;
const MIN_REMAINDER_CHARS: usize = 12;

#[derive(Debug, Clone, Default)]
pub struct Memorization {
    /// 0.0–1.0 share of the hidden remainder reproduced in one verbatim run.
    pub score: f32,
    /// The longest run, in the model's own (normalized) spelling.
    pub run: String,
    pub hit: bool,
    pub abstained: bool,
}

pub fn normalize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_space = true;
    for ch in s.chars() {
        let mapped: Option<char> = match ch {
            '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' | '`' | '\u{00B4}' => Some('\''),
            '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' | '\u{00AB}' | '\u{00BB}' => Some('"'),
            '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2015}' | '\u{2212}' => Some('-'),
            '\u{2026}' => None,
            '\u{00A0}' => Some(' '),
            c if c.is_whitespace() => Some(' '),
            c => Some(c),
        };
        let Some(c) = mapped else {
            for d in "...".chars() {
                out.push(d);
            }
            last_space = false;
            continue;
        };
        if c == ' ' {
            if !last_space {
                out.push(' ');
            }
            last_space = true;
        } else {
            for l in c.to_lowercase() {
                out.push(l);
            }
            last_space = false;
        }
    }
    out.trim().to_string()
}

/// Longest common substring over chars. Returns (length, start in `a`).
pub fn longest_common_run(a: &[char], b: &[char]) -> (usize, usize) {
    if a.is_empty() || b.is_empty() {
        return (0, 0);
    }
    let mut prev = vec![0usize; b.len() + 1];
    let mut cur = vec![0usize; b.len() + 1];
    let mut best = 0usize;
    let mut best_end = 0usize;
    for (i, ca) in a.iter().enumerate() {
        for (j, cb) in b.iter().enumerate() {
            cur[j + 1] = if ca == cb { prev[j] + 1 } else { 0 };
            if cur[j + 1] > best {
                best = cur[j + 1];
                best_end = i + 1;
            }
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    (best, best_end - best)
}

/// The part of `target` the prompt did not already hand over. Prompts reveal a
/// prefix (prefix strategy) or a short stem/needle; we drop the longest run the
/// prompt shares with the target when it sits at the start, otherwise keep all.
pub fn hidden_remainder(target_norm: &str, prompt_norm: &str) -> String {
    let t: Vec<char> = target_norm.chars().collect();
    if prompt_norm.is_empty() || t.is_empty() {
        return target_norm.to_string();
    }
    let p: Vec<char> = prompt_norm.chars().collect();
    let (len, start) = longest_common_run(&t, &p);
    if len >= 8 && start <= 2 {
        let rest: String = t[start + len..].iter().collect();
        return rest.trim().to_string();
    }
    target_norm.to_string()
}

pub fn looks_like_abstain(response_norm: &str) -> bool {
    let head: String = response_norm.chars().take(160).collect();
    head.contains("unknown")
        || head.contains("i don't have")
        || head.contains("i do not have")
        || head.contains("i can't reproduce")
        || head.contains("i cannot reproduce")
        || head.contains("not able to reproduce")
        || head.contains("i'm not able")
        || head.contains("i am not able")
        || head.contains("not familiar with")
}

pub fn memorization(response: &str, target: &str, prompt: &str) -> Memorization {
    let response_norm = normalize(response);
    let target_norm = normalize(target);
    let prompt_norm = normalize(prompt);
    let remainder = hidden_remainder(&target_norm, &prompt_norm);
    let abstained_text = looks_like_abstain(&response_norm);

    let rem: Vec<char> = remainder.chars().collect();
    if rem.len() < MIN_REMAINDER_CHARS {
        return Memorization {
            abstained: abstained_text,
            ..Default::default()
        };
    }
    let resp: Vec<char> = response_norm.chars().collect();
    let (len, start) = longest_common_run(&rem, &resp);
    let run: String = rem[start..start + len].iter().collect();
    let run = run.trim().to_string();
    // A run the prompt itself contains is an echo, not recall.
    let echoed = len >= 8 && prompt_norm.contains(&run);
    let score = if echoed || len < 8 {
        0.0
    } else {
        (len as f32 / rem.len() as f32).clamp(0.0, 1.0)
    };
    let hit = score >= HIT_SCORE || (len >= HIT_RUN_CHARS && score >= HIT_RUN_SCORE);
    Memorization {
        score,
        run: if score > 0.0 { run } else { String::new() },
        hit,
        abstained: abstained_text && score < NOISE_SCORE,
    }
}

/// Wilson score interval (95%) for a binomial proportion.
pub fn wilson(hits: usize, n: usize) -> (f32, f32) {
    if n == 0 {
        return (0.0, 0.0);
    }
    let z = 1.96f64;
    let n_f = n as f64;
    let p = hits as f64 / n_f;
    let denom = 1.0 + z * z / n_f;
    let centre = p + z * z / (2.0 * n_f);
    let spread = z * ((p * (1.0 - p) + z * z / (4.0 * n_f)) / n_f).sqrt();
    let lo = ((centre - spread) / denom).clamp(0.0, 1.0);
    let hi = ((centre + spread) / denom).clamp(0.0, 1.0);
    (lo as f32, hi as f32)
}

/// Build a negative-control target: same prefix (so the model gets the real
/// cue), scrambled remainder. A calibrated detector must score ~0 against it
/// even when the model reproduces the real passage.
pub fn decoy_for(value: &str, prefix: &str, seed: u64) -> String {
    let rest = value.strip_prefix(prefix).unwrap_or(value);
    let mut words: Vec<&str> = rest.split_whitespace().collect();
    if words.len() < 4 {
        let mut chars: Vec<char> = rest.chars().collect();
        let mut rng = StdRng::seed_from_u64(seed);
        chars.shuffle(&mut rng);
        return format!("{prefix}{}", chars.into_iter().collect::<String>());
    }
    let mut rng = StdRng::seed_from_u64(seed);
    let original = words.clone();
    for _ in 0..8 {
        words.shuffle(&mut rng);
        if words != original {
            break;
        }
    }
    let joined = words.join(" ");
    if prefix.is_empty() {
        joined
    } else if prefix.ends_with(char::is_whitespace) {
        format!("{prefix}{joined}")
    } else {
        format!("{prefix} {joined}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRIDE: &str = "It is a truth universally acknowledged, that a single man in possession of a good fortune, must be in want of a wife.";

    #[test]
    fn normalizes_typography() {
        let a = normalize("“Call me Ishmael.”  Some years ago—never mind");
        let b = normalize("\"call me ishmael.\" some years ago-never mind");
        assert_eq!(a, b);
    }

    #[test]
    fn verbatim_completion_scores_high() {
        let prompt = "Complete this passage:\n\nIt is a truth universally acknowledged, that a";
        let reply = "single man in possession of a good fortune, must be in want of a wife.";
        let m = memorization(reply, PRIDE, prompt);
        assert!(m.score > 0.95, "score {}", m.score);
        assert!(m.hit);
        assert!(!m.abstained);
    }

    #[test]
    fn typographic_variants_still_hit() {
        let prompt = "Complete: It is a truth universally acknowledged, that a";
        let reply = "…single man in possession of a good fortune,\nmust be in want of a Wife.";
        let m = memorization(reply, PRIDE, prompt);
        assert!(m.hit, "score {}", m.score);
    }

    #[test]
    fn partial_recall_is_measured_not_binary() {
        let prompt = "Complete: It is a truth universally acknowledged, that a";
        let reply = "single man in possession, so the saying goes, will surely look for a partner.";
        let m = memorization(reply, PRIDE, prompt);
        assert!(m.score > 0.2 && m.score < HIT_SCORE, "score {}", m.score);
        assert!(!m.hit);
    }

    #[test]
    fn echoing_the_prompt_is_not_recall() {
        let prompt = "Complete: It is a truth universally acknowledged, that a";
        let reply = "It is a truth universally acknowledged, that a... UNKNOWN";
        let m = memorization(reply, PRIDE, prompt);
        assert_eq!(m.score, 0.0);
        assert!(m.abstained);
        assert!(!m.hit);
    }

    #[test]
    fn decoy_does_not_score_against_real_reply() {
        let prefix = "It is a truth universally acknowledged, that a";
        let decoy = decoy_for(PRIDE, prefix, 7);
        assert_ne!(decoy, PRIDE);
        assert!(decoy.starts_with(prefix));
        let prompt = format!("Complete: {prefix}");
        let reply = "single man in possession of a good fortune, must be in want of a wife.";
        let m = memorization(reply, &decoy, &prompt);
        assert!(!m.hit, "decoy should not register as a hit, score {}", m.score);
    }

    #[test]
    fn wilson_interval_is_sane() {
        let (lo, hi) = wilson(0, 0);
        assert_eq!((lo, hi), (0.0, 0.0));
        let (lo, hi) = wilson(3, 3);
        assert!(lo > 0.4 && hi == 1.0, "{lo} {hi}");
        let (lo, hi) = wilson(0, 10);
        assert!(lo == 0.0 && hi < 0.31, "{lo} {hi}");
        let (lo, hi) = wilson(5, 10);
        assert!(lo < 0.5 && hi > 0.5);
    }
}
