# LLM Canary

![LLM Canary](docs/hero.png)

**Get answers about where a model’s training came from** — for the sources you care about.

Models are trained on public and private data. Marketing noise won’t tell you which. LLM Canary cuts through that: plant unique canaries or probe real corpora, hunt the labs, and get a cited answer — **public vs private** — for each model.

Desktop app (Tauri 2). If they trained on it, the bird sings.

**License:** proprietary. Copyright © 2026 Aaron Grace. See [LICENSE](LICENSE). Commercial licenses are available from the owner. The Software may not be used to train, fine-tune, or distill any AI/ML model or dataset.

## What you get

- **Benchmark by model** — hit rate with a 95% Wilson interval, mean memorization score, abstain rate, private vs public hits, and a per-strategy breakdown — including models with zero hits
- **Continuous memorization score** — every reply is scored 0–100% by how much of the hidden remainder came back verbatim after normalizing quotes, dashes, case, and whitespace
- **Repeated trials + greedy decoding** — Hunt runs at temperature 0 and repeats each prompt 1/3/5 times so results measure the model, not the sampler
- **Decoy controls** — optional scrambled targets bound the detector’s false-positive rate; control hits never count as evidence
- **Chat interrogation** — question models (including GLM) about where knowledge came from; scan replies for regurgitated sources
- **GitHub linking** — watch a repo’s distinctive passages as membership bait
- **Citations** — work title, locator, and the lab that sang
- **Calibration vs smoking gun** — famous public-domain hits are expected; your private wiki, dataset, or planted canary is the evidence
- **Exportable provenance report** — methodology, benchmark table, and raw evidence (score / trial / temperature) as markdown

This is membership evidence against sources **you** load or plant. It does not invent a dump of a lab’s entire training set. It answers the question that matters: *did this model train on this?*

## How the benchmark works

LLM Canary is built to be comparable to published membership / memorization evals (prefix extraction, needle probes, negative controls) rather than a single-shot anecdote:

- **Prompts never contain the full target.** Prefix-completion, title-recall, and needle prompts reveal only a prefix, a five-word stem, or a short interior fragment.
- **Greedy decoding.** Extraction probes run at temperature 0 and are repeated (1/3/5 trials) so results replicate. Chat keeps a conversational temperature.
- **Continuous score.** Reply and target are normalized (case, whitespace, quotes, dashes, ellipses). The part of the target the prompt already revealed is removed, and the score is the longest verbatim run divided by the length of that hidden remainder. A hit is score ≥ 50%, or a verbatim run of ≥ 40 characters at ≥ 25%. Exact matches of planted random tokens (≥ 10 chars) also count. Prompt echoes score 0.
- **Negative controls.** Optionally, each reply is also scored against a scrambled decoy of its target (same prefix, shuffled remainder). Control hits are detector false positives and are reported separately.
- **Rates with uncertainty.** Answers reports hit rate with a 95% Wilson interval, mean score, abstain rate, private/public hits over probes, control false positives, errors, and a per-strategy breakdown for every model probed — including models with zero hits.

## What’s new in 0.5.0

- Continuous memorization scoring shared by Hunt, Chat, and paste-scan
- Hunt: trials selector, decoy-controls toggle, live `done/total` progress, temperature 0
- Answers: benchmark table (rates, CIs, mean score, abstain, controls) plus scores on raw evidence
- Provenance markdown: Methodology section and benchmark table
- Fix: OS credential-store backends enabled for `keyring` so API keys and the GitHub token actually persist

See [CHANGELOG.md](CHANGELOG.md) for the full notes.

## Run

Node.js 22+ and Rust are required. WebView2 ships with Windows 11.

If Node isn't on PATH (this machine has a portable install), from PowerShell:

```powershell
.\run.ps1
```

Or:

```bash
npm install
npm run tauri dev
```

```bash
npm run tauri build
```

## Installers

GitHub Actions builds desktop installers on each `v*` tag (and via the Release workflow):

| Platform | Artifact |
| --- | --- |
| Windows | `.msi` / NSIS `.exe` |
| Linux (incl. Steam Deck) | `.AppImage` / `.deb` |
| macOS Apple Silicon | `aarch64` `.dmg` |
| macOS Intel | `x86_64` `.dmg` |

Download them from the [Releases](https://github.com/AaronGrace978/LLM-Canary/releases) page. Steam store packaging is separate from these desktop builds.

## Use

1. **Cages** — paste API keys for Ollama Cloud, OpenAI, Anthropic, OpenRouter, Gemini, Groq, DeepSeek, Mistral, xAI, or any OpenAI-compatible endpoint. Fetch models, pick one, Test. Keys live in the OS credential store.
2. **Probe** — import a file/folder, paste text, or **link a GitHub repo**. Or load the **public-domain pack** as a calibration baseline.
3. **Chat** — question a model about its training. Fish prompts push for sources and verbatim recall; replies that regurgitate watched material become Answers (with a verbatim score).
4. **Plant** — optional unique bait in your repos (secrets, code watermarks, phrases, dataset rows, identities, custom flags).
5. **Hunt** — automated prefix / recall / needle probes at temperature 0. Pick 1/3/5 trials and optionally turn on decoy controls. Progress shows `done/total`.
6. **Answers** — benchmark table (hit rate + CI, mean score, abstain, private vs public, control false positives), per-model source lists, raw evidence, exportable report.

Keys are sent only to the provider you configured.
