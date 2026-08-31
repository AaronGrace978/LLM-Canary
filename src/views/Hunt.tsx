import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { runHunt, webPrompts } from "../api";
import type { HuntProgress, Snapshot, WebPrompt } from "../types";
import { copyText } from "../util";

export function Hunt({ snap, onDone }: { snap: Snapshot; onDone: () => void }) {
  const armed = snap.providers.filter(
    (p) => p.enabled && (p.apiKey.trim() || (p.id === "custom" && p.baseUrl && p.model)),
  );
  const [canaryIds, setCanaryIds] = useState<string[]>([]);
  const [providerIds, setProviderIds] = useState<string[]>([]);
  const [strategies, setStrategies] = useState<string[]>(["prefix", "recall", "needle"]);
  const [log, setLog] = useState<{ phase: string; message: string }[]>([]);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState("");
  const [pack, setPack] = useState<WebPrompt[] | null>(null);
  const [copied, setCopied] = useState("");

  useEffect(() => {
    setCanaryIds(snap.canaries.map((c) => c.id));
    setProviderIds(armed.map((p) => p.id));
  }, [snap.canaries.length, armed.length]);

  useEffect(() => {
    let un: (() => void) | undefined;
    listen<HuntProgress>("hunt-progress", (e) => {
      setLog((l) => [...l, { phase: e.payload.phase, message: e.payload.message }]);
    }).then((fn) => {
      un = fn;
    });
    return () => {
      un?.();
    };
  }, []);

  function tog(list: string[], id: string, set: (v: string[]) => void) {
    set(list.includes(id) ? list.filter((x) => x !== id) : [...list, id]);
  }

  async function hunt() {
    setErr("");
    setPack(null);
    setLog([{ phase: "start", message: "Opening the cages…" }]);
    setBusy(true);
    try {
      const r = await runHunt({ canaryIds, providerIds, strategies });
      setLog((l) => [
        ...l,
        {
          phase: r.hits ? "hit" : "done",
          message: `Done. ${r.hits} hit${r.hits === 1 ? "" : "s"}, ${r.errors} error${r.errors === 1 ? "" : "s"}.`,
        },
      ]);
      onDone();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function copyPack() {
    const prompts = await webPrompts(canaryIds);
    setPack(prompts);
  }

  return (
    <div className="view">
      <header className="page-head">
        <p className="eyebrow">Membership test</p>
        <h1>Hunt</h1>
        <p className="lede tight">
          We never send the full secret. Prefix, recall, and needle probes. If the unique remainder
          comes back, that provider trained on your repo.
        </p>
      </header>

      <div className="split plant-grid">
        <div className="stack">
          <section className="panel">
            <label className="lbl">Canaries</label>
            {snap.canaries.length === 0 ? (
              <p className="empty-inline">Nothing planted yet.</p>
            ) : (
              <div className="check-list">
                {snap.canaries.map((c) => (
                  <label key={c.id} className="check">
                    <input
                      type="checkbox"
                      checked={canaryIds.includes(c.id)}
                      onChange={() => tog(canaryIds, c.id, setCanaryIds)}
                    />
                    <span>
                      {c.kindName}
                      <em>
                        {c.label} · {c.repoName}
                      </em>
                    </span>
                  </label>
                ))}
              </div>
            )}
          </section>

          <section className="panel">
            <label className="lbl">Cages</label>
            <div className="check-list">
              {snap.providers.map((p) => {
                const ready = p.apiKey.trim() || (p.id === "custom" && p.baseUrl && p.model);
                return (
                  <label key={p.id} className={`check ${ready ? "" : "dim"}`}>
                    <input
                      type="checkbox"
                      disabled={!ready}
                      checked={providerIds.includes(p.id)}
                      onChange={() => tog(providerIds, p.id, setProviderIds)}
                    />
                    <span>
                      {p.name}
                      <em>{ready ? p.model || "no model" : "paste a key in Cages"}</em>
                    </span>
                  </label>
                );
              })}
            </div>
          </section>

          <section className="panel">
            <label className="lbl">Strategies</label>
            <div className="chips compact">
              {["prefix", "recall", "needle"].map((s) => (
                <button
                  key={s}
                  type="button"
                  className={`chip ${strategies.includes(s) ? "on" : ""}`}
                  onClick={() => tog(strategies, s, setStrategies)}
                >
                  <b>{s}</b>
                </button>
              ))}
            </div>
            <button className="btn primary block" disabled={busy} onClick={hunt}>
              {busy ? "Hunting…" : "Send the canaries"}
            </button>
            <button className="btn ghost block" onClick={copyPack} disabled={!snap.canaries.length}>
              Build web prompts
            </button>
            {err && <p className="err">{err}</p>}
          </section>
        </div>

        <div className="stack">
          <section className="panel log-panel">
            <h2>Wire</h2>
            <div className="log">
              {log.length === 0 && <p className="empty-inline">Waiting. The mine is silent.</p>}
              {log.map((l, i) => (
                <div key={i} className={`log-line ${l.phase}`}>
                  <span className="dot" />
                  {l.message}
                </div>
              ))}
            </div>
          </section>

          {pack && (
            <section className="panel">
              <h2>Paste these into ChatGPT / Claude / Gemini</h2>
              <p className="hint">No API key required. Scan the reply on Hits.</p>
              {pack.map((p) => (
                <article key={p.title + p.canaryId} className="prompt-card">
                  <header>
                    <span>{p.title}</span>
                    <button
                      className="btn tiny"
                      onClick={async () => {
                        await copyText(p.prompt);
                        setCopied(p.title);
                      }}
                    >
                      {copied === p.title ? "Copied" : "Copy"}
                    </button>
                  </header>
                  <pre>{p.prompt}</pre>
                </article>
              ))}
            </section>
          )}
        </div>
      </div>
    </div>
  );
}
