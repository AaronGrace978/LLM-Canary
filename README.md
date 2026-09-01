# LLM Canary

![LLM Canary](docs/hero.png)

Plant unique canaries in your repos, or **probe a real corpus** — a book, wiki, dataset, or any text you import. If a model regurgitates a distinctive passage, Hits **cites the work** and which lab sang.

Desktop app (Tauri 2). If they trained on it, the bird sings.

This is membership evidence, not a dump of a model’s training set. Famous public-domain books are a calibration check (most models already know Ishmael). Unique or private files are the product.

**License:** proprietary. Copyright © 2026 Aaron Grace. See [LICENSE](LICENSE). Commercial licenses are available from the owner.

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

## Use

1. **Cages** — paste API keys for Ollama Cloud, OpenAI, Anthropic, OpenRouter, Gemini, Groq, DeepSeek, Mistral, xAI, or any OpenAI-compatible endpoint. Fetch models, pick one, Test.
2. **Plant** — drop unique bait into a repo (secrets, code watermarks, phrases, dataset rows, identities, custom flags). Commit and push, or nobody will train on them.
3. **Probe** — import a file/folder or paste text. We extract distinctive passages. Or load the **public-domain pack** (Moby-Dick, Pride and Prejudice, Alice, Frankenstein, Sherlock, A Christmas Carol, The Time Machine) as a sanity check. Copyrighted books are not shipped; import your own copy if you have the right to probe it.
4. **Hunt** — prefix / recall / needle against planted canaries and corpus passages. Copy web prompts into ChatGPT and scan the reply on **Hits**.
5. **Hits** — when a model sings, the report cites **source title + locator + provider**. Export markdown for legal / security.

Keys live in the app data directory. They are sent only to the provider you configured.
