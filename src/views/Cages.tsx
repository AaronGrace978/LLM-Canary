import { useEffect, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { fetchModels, saveProvider, testProvider } from "../api";
import type { Provider, Snapshot } from "../types";
import { initials } from "../util";

export function Cages({ snap, onDone }: { snap: Snapshot; onDone: () => void }) {
  return (
    <div className="view">
      <header className="page-head">
        <p className="eyebrow">Keys never leave this machine except to the provider you paste</p>
        <h1>Cages</h1>
        <p className="lede tight">
          Paste an API key, fetch the live model list, pick one. Ollama Cloud, OpenAI, Anthropic,
          OpenRouter, Gemini, Groq, DeepSeek, Mistral, xAI, or any OpenAI-compatible endpoint.
        </p>
      </header>
      <div className="cages">
        {snap.providers.map((p) => (
          <CageCard key={p.id} provider={p} onDone={onDone} />
        ))}
      </div>
    </div>
  );
}

function CageCard({ provider: p, onDone }: { provider: Provider; onDone: () => void }) {
  const [show, setShow] = useState(false);
  const [key, setKey] = useState(p.apiKey);
  const [model, setModel] = useState(p.model);
  const [base, setBase] = useState(p.baseUrl);
  const [q, setQ] = useState("");
  const [busy, setBusy] = useState<"save" | "models" | "test" | null>(null);
  const [msg, setMsg] = useState("");
  const [err, setErr] = useState("");

  useEffect(() => {
    setKey(p.apiKey);
    setModel(p.model);
    setBase(p.baseUrl);
  }, [p.apiKey, p.model, p.baseUrl]);

  const models = p.models.filter((m) => m.toLowerCase().includes(q.toLowerCase()));

  async function persist(extra: Partial<{ apiKey: string; model: string; baseUrl: string; enabled: boolean }>) {
    setBusy("save");
    setErr("");
    try {
      await saveProvider({
        id: p.id,
        apiKey: extra.apiKey ?? key,
        model: extra.model ?? model,
        baseUrl: extra.baseUrl ?? base,
        enabled: extra.enabled ?? p.enabled,
      });
      onDone();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(null);
    }
  }

  async function refresh() {
    setBusy("models");
    setErr("");
    setMsg("");
    try {
      await persist({});
      const list = await fetchModels(p.id);
      setMsg(`${list.length} models`);
      if (list.length && !list.includes(model)) {
        setModel(list[0]);
        await saveProvider({ id: p.id, model: list[0] });
      }
      onDone();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(null);
    }
  }

  async function test() {
    setBusy("test");
    setErr("");
    setMsg("");
    try {
      await persist({});
      const r = await testProvider(p.id);
      setMsg(`OK · ${r.model} · ${r.preview}`);
      onDone();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(null);
    }
  }

  return (
    <article className={`cage ${p.apiKey ? "armed" : ""} ${p.enabled ? "" : "off"}`}>
      <header className="cage-head">
        <span className="mono-mark">{initials(p.name)}</span>
        <div>
          <h3>{p.name}</h3>
          <p>{p.blurb}</p>
        </div>
        <label className="switch">
          <input
            type="checkbox"
            checked={p.enabled}
            onChange={(e) => persist({ enabled: e.target.checked })}
          />
          <span />
        </label>
      </header>

      <label className="lbl">API key</label>
      <div className="row">
        <input
          className="field grow"
          type={show ? "text" : "password"}
          value={key}
          onChange={(e) => setKey(e.target.value)}
          onBlur={() => persist({ apiKey: key })}
          placeholder="Paste key"
          autoComplete="off"
          spellCheck={false}
        />
        <button className="btn ghost" type="button" onClick={() => setShow((s) => !s)}>
          {show ? "Hide" : "Show"}
        </button>
      </div>
      {p.docsUrl && (
        <button className="linkish docs" onClick={() => openUrl(p.docsUrl)}>
          Get a key
        </button>
      )}

      {(p.id === "custom" || p.kind === "ollama") && (
        <>
          <label className="lbl">Base URL</label>
          <input
            className="field"
            value={base}
            onChange={(e) => setBase(e.target.value)}
            onBlur={() => persist({ baseUrl: base })}
            spellCheck={false}
          />
        </>
      )}

      <label className="lbl">Model</label>
      <input
        className="field"
        value={model}
        onChange={(e) => setModel(e.target.value)}
        onBlur={() => persist({ model })}
        placeholder="Model id"
        spellCheck={false}
      />
      <input
        className="field search"
        value={q}
        onChange={(e) => setQ(e.target.value)}
        placeholder="Filter models…"
      />
      <div className="model-list">
        {models.length === 0 && <p className="empty-inline">No models. Fetch after pasting a key.</p>}
        {models.map((m) => (
          <button
            key={m}
            type="button"
            className={`model ${m === model ? "on" : ""}`}
            onClick={() => {
              setModel(m);
              persist({ model: m });
            }}
          >
            {m}
          </button>
        ))}
      </div>

      <div className="row gap">
        <button className="btn ghost grow" disabled={!!busy} onClick={refresh}>
          {busy === "models" ? "Fetching…" : "Fetch models"}
        </button>
        <button className="btn ghost grow" disabled={!!busy} onClick={test}>
          {busy === "test" ? "Testing…" : "Test"}
        </button>
      </div>
      {p.lastOkAt && !err && <p className="ok-line">Last good call recorded.</p>}
      {msg && <p className="ok-line">{msg}</p>}
      {(err || p.lastError) && <p className="err">{err || p.lastError}</p>}
    </article>
  );
}
