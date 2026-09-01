# Changelog

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
