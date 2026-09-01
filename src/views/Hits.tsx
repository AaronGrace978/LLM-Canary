import { useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { exportReport, scanText } from "../api";
import type { ScanHit, Snapshot } from "../types";
import { timeAgo, citationFor } from "../util";

export function Hits({ snap, onDone }: { snap: Snapshot; onDone: () => void }) {
  const hits = snap.probes.filter((p) => p.hit).slice().reverse();
  const [paste, setPaste] = useState("");
  const [found, setFound] = useState<ScanHit[] | null>(null);
  const [err, setErr] = useState("");

  async function scan() {
    setErr("");
    try {
      const r = await scanText(paste, "clipboard");
      setFound(r);
    } catch (e) {
      setErr(String(e));
    }
  }

  async function exportMd() {
    const path = await save({
      defaultPath: "llm-canary-report.md",
      filters: [{ name: "Markdown", extensions: ["md"] }],
    });
    if (typeof path === "string") {
      await exportReport(path);
      onDone();
    }
  }

  return (
    <div className="view">
      <header className="page-head">
        <p className="eyebrow">Evidence locker</p>
        <h1>Hits</h1>
        <p className="lede tight">
          Full prompt + response when a model sings. Corpus hits cite the work and locator. Export
          a report for legal / security.
        </p>
      </header>

      <div className="split plant-grid">
        <section className="panel">
          <label className="lbl">Scan a web reply</label>
          <textarea
            className="field area"
            value={paste}
            onChange={(e) => setPaste(e.target.value)}
            placeholder="Paste a ChatGPT / Claude / Gemini answer here…"
          />
          <button className="btn primary" onClick={scan} disabled={!paste.trim()}>
            Scan for canaries
          </button>
          {err && <p className="err">{err}</p>}
          {found && (
            <div className="scan-result">
              {found.length === 0 ? (
                <p className="empty-inline">No planted canaries in that text.</p>
              ) : (
                found.map((h) => (
                  <p key={h.canaryId} className="hit-banner">
                    HIT — {h.citation || `${h.kind} / ${h.label}`}
                  </p>
                ))
              )}
            </div>
          )}
        </section>
        <section className="panel">
          <h2>Export</h2>
          <p className="hint">Markdown with every hit: provider, model, prompt, raw response.</p>
          <button className="btn ghost" onClick={exportMd} disabled={!hits.length}>
            Save evidence report
          </button>
        </section>
      </div>

      {hits.length === 0 ? (
        <section className="panel">
          <p className="empty-inline">No regurgitations yet. That’s good — or you haven’t hunted.</p>
        </section>
      ) : (
        <div className="stack">
          {hits.map((p) => (
            <article key={p.id} className="evidence">
              <header>
                <span className="pill hit">HIT</span>
                <strong>
                  {p.providerName} · {p.model}
                </strong>
                <em>
                  {p.canaryKind} · {p.strategy} · {timeAgo(p.at)}
                </em>
              </header>
              <p className="citation">
                Source: {p.citation || citationFor({ label: p.canaryLabel })}
              </p>
              <p className="matched">Matched {p.matched.map((m) => maskBit(m)).join(" · ")}</p>
              <details>
                <summary>Prompt</summary>
                <pre>{p.prompt}</pre>
              </details>
              <details open>
                <summary>Response</summary>
                <pre>{p.response}</pre>
              </details>
            </article>
          ))}
        </div>
      )}
    </div>
  );
}

function maskBit(s: string) {
  if (s.length < 16) return s;
  return `${s.slice(0, 8)}…${s.slice(-4)}`;
}
