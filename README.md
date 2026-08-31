# LLM Canary

Desktop app (Tauri 2) that plants unique fake secrets in a repository. If an AI service ever regurgitates them, you know which lab trained on your code.

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
2. **Plant** — choose a repo, secret types, density. Commit and push the files (they never train anyone if they stay on disk).
3. **Hunt** — prefix / recall / needle probes. Or copy web prompts into ChatGPT and scan the reply on **Hits**.

Keys live in the app data directory. They are sent only to the provider you configured.
