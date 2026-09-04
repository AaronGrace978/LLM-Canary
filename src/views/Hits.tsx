import { useMemo, useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { exportReport, scanText } from "../api";
import type { ScanHit, Snapshot } from "../types";
import { timeAgo, citationFor, sensitivityLabel, pct } from "../util";

export function Hits({ snap, onDone }: { snap: Snapshot; onDone: () => void }) {
  const hits = snap.probes.filter((p) => p.hit && !p.control).slice().reverse();
  const answers = snap.provenance?.answers ?? [];
  const scoredProbes = answers.reduce((n, a) => n + a.probes, 0);
  const controlProbes = answers.reduce((n, a) => n + a.controlProbes, 0);
  const controlHits = answers.reduce((n, a) => n + a.controlHits, 0);
  const privateHits = snap.provenance?.privateHits ?? 0;
  const publicHits = snap.provenance?.publicHits ?? 0;
  const [paste, setPaste] = useState("");
  const [found, setFound] = useState<ScanHit[] | null>(null);
  const [err, setErr] = useState("");

  const watched = useMemo(() => {
    let priv = 0;
    let pub = 0;
    for (const c of snap.canaries) {
      if (c.sourceKind === "public_domain") pub += 1;
      else priv += 1;
    }
    return { priv, pub };
  }, [snap.canaries]);

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
      defaultPath: "llm-canary-provenance.md",
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
        <p className="eyebrow">Training provenance</p>
        <h1>Answers</h1>
        <p className="lede tight">
          Cut through the noise. For every source you planted or imported, we answer whether a
          model can reproduce it — and cite exactly which work it came from. Public-domain hits are
          expected calibration. Private / unique hits are the smoking gun.
        </p>
      </header>

      <div className="stats">
        <div className={`stat ${privateHits > 0 ? "is-hit" : ""}`}>
          <span>Private hits</span>
          <strong>{privateHits}</strong>
          <em>unique / proprietary sources</em>
        </div>
        <div className="stat">
          <span>Public hits</span>
          <strong>{publicHits}</strong>
          <em>famous / expected sources</em>
        </div>
        <div className="stat">
          <span>Under watch</span>
          <strong>{watched.priv + watched.pub}</strong>
          <em>
            {watched.priv} private · {watched.pub} public
          </em>
        </div>
        <div className="stat">
          <span>Scored probes</span>
          <strong>{scoredProbes}</strong>
          <em>
            {answers.length} model{answers.length === 1 ? "" : "s"} · {hits.length} hit
            {hits.length === 1 ? "" : "s"}
          </em>
        </div>
        <div className={`stat ${controlHits > 0 ? "is-hit" : ""}`}>
          <span>Control false positives</span>
          <strong>{controlProbes ? `${controlHits}/${controlProbes}` : "–"}</strong>
          <em>{controlProbes ? "decoy targets scored as hits" : "run Hunt with controls on"}</em>
        </div>
      </div>

      <section className="panel">
        <h2>Benchmark by model</h2>
        <p className="hint">
          Hit rate is hits over scored probes with a 95% Wilson interval. Mean score is the average
          share of each hidden remainder reproduced verbatim. Abstain is how often the model said it
          did not know. Controls are decoy targets: any hit there is the detector’s fault, not the
          model’s memory.
        </p>
        {answers.length === 0 ? (
          <p className="empty-inline">
            No probes yet. Load sources on Probe (or Plant bait), arm Cages, then Hunt.
          </p>
        ) : (
          <div className="table-wrap">
            <table className="grid bench">
              <thead>
                <tr>
                  <th>Model</th>
                  <th>Probes</th>
                  <th>Hit rate</th>
                  <th>95% CI</th>
                  <th>Mean score</th>
                  <th>Abstain</th>
                  <th>Private</th>
                  <th>Public</th>
                  <th>Controls</th>
                  <th>Errors</th>
                </tr>
              </thead>
              <tbody>
                {answers.map((a) => (
                  <tr key={`${a.providerName}-${a.model}`} className={a.privateHits > 0 ? "is-private" : ""}>
                    <td>
                      <b>{a.providerName}</b> <span className="hint">{a.model}</span>
                      <div className="hint">
                        {a.byStrategy.map((s) => `${s.strategy} ${s.hits}/${s.probes}`).join(" · ")}
                      </div>
                    </td>
                    <td>{a.probes}</td>
                    <td>
                      <b>{pct(a.hitRate)}</b>{" "}
                      <span className="hint">
                        {a.hits}/{a.probes}
                      </span>
                    </td>
                    <td>
                      {pct(a.ciLow)}–{pct(a.ciHigh)}
                    </td>
                    <td>{pct(a.meanScore)}</td>
                    <td>{pct(a.abstainRate)}</td>
                    <td>
                      {a.privateHits}/{a.privateProbes}
                    </td>
                    <td>
                      {a.publicHits}/{a.publicProbes}
                    </td>
                    <td className={a.controlHits > 0 ? "err" : ""}>
                      {a.controlProbes ? `${a.controlHits}/${a.controlProbes}` : "–"}
                    </td>
                    <td>{a.errors}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>

      <section className="panel">
        <h2>Where each model’s training shows up</h2>
        {answers.length === 0 ? (
          <p className="empty-inline">
            No answers yet. Load sources on Probe (or Plant bait), arm Cages, then Hunt.
          </p>
        ) : (
          <div className="answer-grid">
            {answers.map((a) => (
              <article
                key={`${a.providerName}-${a.model}`}
                className={`answer-card ${a.privateHits > 0 ? "is-private" : ""}`}
              >
                <header>
                  <strong>
                    {a.providerName} · {a.model}
                  </strong>
                  <em>
                    {a.privateHits} private · {a.publicHits} public · {pct(a.hitRate)} of {a.probes}
                  </em>
                </header>
                <div className="answer-cols">
                  <div>
                    <span className="pill private">Private / unique</span>
                    {a.privateSources.length ? (
                      <ul>
                        {a.privateSources.map((s) => (
                          <li key={s}>{s}</li>
                        ))}
                      </ul>
                    ) : (
                      <p className="empty-inline">None detected</p>
                    )}
                  </div>
                  <div>
                    <span className="pill public">Public / expected</span>
                    {a.publicSources.length ? (
                      <ul>
                        {a.publicSources.map((s) => (
                          <li key={s}>{s}</li>
                        ))}
                      </ul>
                    ) : (
                      <p className="empty-inline">None detected</p>
                    )}
                  </div>
                </div>
              </article>
            ))}
          </div>
        )}
      </section>

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
            Match to sources
          </button>
          {err && <p className="err">{err}</p>}
          {found && (
            <div className="scan-result">
              {found.length === 0 ? (
                <p className="empty-inline">No watched sources in that text.</p>
              ) : (
                found.map((h) => (
                  <p key={h.canaryId} className="hit-banner">
                    {h.sensitivity === "public" ? "PUBLIC" : "PRIVATE"} —{" "}
                    {h.citation || `${h.kind} / ${h.label}`} · {pct(h.score)} verbatim
                  </p>
                ))
              )}
            </div>
          )}
        </section>
        <section className="panel">
          <h2>Export provenance report</h2>
          <p className="hint">
            Markdown answers by model: private vs public sources, then raw prompt/response evidence.
          </p>
          <button className="btn ghost" onClick={exportMd} disabled={!hits.length}>
            Save provenance report
          </button>
        </section>
      </div>

      {hits.length === 0 ? (
        <section className="panel">
          <p className="empty-inline">No regurgitations yet. That’s quiet — or you haven’t hunted.</p>
        </section>
      ) : (
        <div className="stack">
          <h2 className="section-title">Raw evidence</h2>
          {hits.map((p) => (
            <article key={p.id} className="evidence">
              <header>
                <span className={`pill ${p.sensitivity === "public" ? "public" : "hit"}`}>
                  {p.sensitivity === "public" ? "PUBLIC" : "PRIVATE"}
                </span>
                <strong>
                  {p.providerName} · {p.model}
                </strong>
                <em>
                  {p.canaryKind} · {p.strategy}
                  {p.trial && p.trial > 1 ? ` · trial ${p.trial}` : ""} · {timeAgo(p.at)}
                </em>
              </header>
              <p className="citation">
                Source: {p.citation || citationFor({ label: p.canaryLabel })} ·{" "}
                {sensitivityLabel(p.sensitivity)}
                {p.score !== undefined ? ` · ${pct(p.score)} of hidden remainder verbatim` : ""}
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
