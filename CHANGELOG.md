# Changelog

## 0.5.0 — 2026-09-04

### Benchmarking rigor
- **Continuous memorization score.** Every probe is scored 0–100%: the longest verbatim run the model produced divided by the length of the target the prompt did *not* reveal. Reply and target are normalized first (case, whitespace, curly quotes, dashes, ellipses), so typography and line wrapping no longer turn a reproduction into a miss. Exact matches of planted random tokens remain a fast-path hit.
- **Greedy decoding for extraction.** Hunt probes run at temperature 0 so repeated runs measure the model, not the sampler. Chat keeps its conversational temperature.
- **Repeated trials.** Hunt repeats each prompt 1, 3, or 5 times; every probe records its trial index and temperature.
- **Negative controls.** Optional decoy targets (same prefix, scrambled remainder) are scored against the very same replies. Any control hit is a detector false positive and is reported separately; controls never count as evidence.
- **Rates, not anecdotes.** Answers now shows a benchmark table per model: scored probes, hit rate with a 95% Wilson interval, mean score, abstain rate, private vs public hits over probes, control false positives, errors, and a per-strategy breakdown. Models with zero hits get a row instead of disappearing.
- **Report.** The provenance markdown gains a Methodology section and the benchmark table; raw evidence lists score, trial, and temperature.
- Paste-scan and Chat hits use the same scorer and show the verbatim share.

### Fixes
- **Keys were not being saved.** `keyring` 3 ships no credential store unless a platform backend is enabled; the OS backends (Windows Credential Manager, macOS Keychain, Linux Secret Service) are now compiled in. Previously every save failed with a `credential vault` error.
- Release workflow default tag bumped.

## 0.4.2 — 2026-09-01

### License
- New **AI-training clause** (§3): the Software, its source, documentation, output, and derived content may not be used to train, fine-tune, distill, or improve any ML/AI model, system, or dataset — by any person, service, or automated process. A separate written license is required for such use, and the prohibition survives termination.
- Sections renumbered (Commercial licenses → §4, No warranty → §5, Third-party components → §6).


## 0.4.1 — 2026-09-01

### Secret storage
- Provider API keys and the optional GitHub token are no longer written into `db.json`.
- Secrets live in the OS credential store: Windows Credential Manager, macOS Keychain, Linux Secret Service.
- Existing plaintext keys in `db.json` are migrated into the vault on next launch, then wiped from disk.

## 0.4.0 — 2026-09-01

### Chat — question models about training
- New **Chat** tab: talk to any armed cage (GLM, Claude, GPT, etc.) and fish it for training provenance.
- Fish prompts ask where knowledge came from and push for verbatim source recall.
- Every reply is scanned against planted canaries and watched corpora; hits land in **Answers**.

### Link GitHub repos
- On **Probe**, paste a GitHub URL to pull README + distinctive source files and watch them as private corpus.
- Optional GitHub token for private repos.
- Chat adds repo-specific trap prompts for each linked repository.

## 0.3.1 — 2026-09-01

### Installers
- GitHub Actions Release workflow builds and uploads Windows (`.msi`/`.exe`), Linux (`.AppImage`/`.deb`, Steam Deck friendly), and macOS (Apple Silicon + Intel) installers to the GitHub Release.


## 0.3.0 — 2026-09-01

### Training provenance answers
- **Answers** view (formerly Hits) shows per-model provenance: which **private / unique** sources vs **public / expected** sources each lab can reproduce.
- Export is a **training provenance report** — answers by model, source ledger, then raw evidence.
- New **Probe** tab: import a file, folder, or pasted document; extract distinctive passages; hunt models against them with citations.
- Built-in **public-domain pack** for calibration: Moby-Dick, Pride and Prejudice, Alice, Frankenstein, Sherlock, A Christmas Carol, The Time Machine.
- Product framing: cut through public-vs-private training noise with cited membership evidence for the sources you load or plant.

## 0.2.0 — 2026-09-01

### Training-data flags
- Plant canaries for any family of training data, not just secrets: code watermarks, comment tags, document phrases, project codenames, dataset rows, JSON fixtures, operator emails, and employee ids.
- Paste **custom flags** (any unique strings, one per line) and have them written into internal notes.
- Hunt prompts now describe the canary family (secret, code, prose, data, identity, custom) instead of always asking for a credential.
- Plant UI groups types by family, with select-all and a live file preview that matches the selected mix.

### License
- Relicensed as proprietary software owned by Aaron Grace. See `LICENSE`. The owner may sell commercial licenses; others may not copy, modify, distribute, or sell the software without written permission.

## 0.1.0

- Initial LLM Canary desktop app: plant fake secrets and hunt models that regurgitate them.
