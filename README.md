# LLM Canary

![LLM Canary](docs/hero.png)

**Get answers about where a model’s training came from** — for the sources you care about.

Models are trained on public and private data. Marketing noise won’t tell you which. LLM Canary cuts through that: plant unique canaries or probe real corpora, hunt the labs, and get a cited answer — **public vs private** — for each model.

Desktop app (Tauri 2). If they trained on it, the bird sings.

**License:** proprietary. Copyright © 2026 Aaron Grace. See [LICENSE](LICENSE). Commercial licenses are available from the owner.

## What you get

- **Answers by model** — which public sources and which private / unique sources each provider can reproduce
- **Chat interrogation** — question models (including GLM) about where knowledge came from; scan replies for regurgitated sources
- **GitHub linking** — watch a repo’s distinctive passages as membership bait
- **Citations** — work title, locator, and the lab that sang
- **Calibration vs smoking gun** — famous public-domain hits are expected; your private wiki, dataset, or planted canary is the evidence
- **Exportable provenance report** — markdown for legal / security

This is membership evidence against sources **you** load or plant. It does not invent a dump of a lab’s entire training set. It answers the question that matters: *did this model train on this?*

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

1. **Cages** — paste API keys for Ollama Cloud, OpenAI, Anthropic, OpenRouter, Gemini, Groq, DeepSeek, Mistral, xAI, or any OpenAI-compatible endpoint. Fetch models, pick one, Test.
2. **Probe** — import a file/folder, paste text, or **link a GitHub repo**. Or load the **public-domain pack** as a baseline.
3. **Chat** — question a model about its training. Fish prompts push for sources and verbatim recall; replies that regurgitate watched material become Answers.
4. **Plant** — optional unique bait in your repos (secrets, code watermarks, phrases, dataset rows, identities, custom flags).
5. **Hunt** — automated prefix / recall / needle probes against every watched source.
6. **Answers** — per-model provenance: private vs public sources, citations, raw evidence, exportable report.

Keys live in the app data directory. They are sent only to the provider you configured.
