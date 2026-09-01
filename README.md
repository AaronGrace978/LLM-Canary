# LLM Canary

![LLM Canary](docs/hero.png)

Plant unique canaries in your repos — fake secrets, code watermarks, internal phrases, dataset rows, identities, or any string you flag. If an AI service ever regurgitates them, you know exactly who trained on your data.

Desktop app (Tauri 2). If they trained on it, the bird sings.

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
2. **Plant** — choose a repo and any mix of training-data types:
   - **Secrets** — AWS, GitHub, OpenAI, Anthropic, Stripe, Slack, Postgres, Hugging Face, SendGrid, npm, SSH keys
   - **Code** — unique functions, constants, and comment tags
   - **Documents** — internal architecture phrases and project codenames
   - **Datasets** — unique CSV / JSON fixture ids
   - **Identity** — operator emails and employee ids
   - **Custom flags** — paste any unique strings, one per line
   Pick density, then commit and push the files (they never train anyone if they stay on disk).
3. **Hunt** — prefix / recall / needle probes, adapted to each family. Or copy web prompts into ChatGPT and scan the reply on **Hits**.

Keys live in the app data directory. They are sent only to the provider you configured.
