import { useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import { plantCanaries } from "../api";
import type { PlantResult, Snapshot } from "../types";
import { repoLeaf } from "../util";

export function Plant({
  snap,
  onDone,
}: {
  snap: Snapshot;
  onDone: () => void;
}) {
  const [repo, setRepo] = useState("");
  const [label, setLabel] = useState("");
  const [kinds, setKinds] = useState<string[]>(["aws", "github", "openai", "postgres"]);
  const [density, setDensity] = useState<"stealth" | "mixed" | "loud">("mixed");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState("");
  const [result, setResult] = useState<PlantResult | null>(null);

  const files = useMemo(() => previewFiles(density), [density]);

  async function pick() {
    const dir = await open({ directory: true, title: "Choose a repository folder" });
    if (typeof dir === "string") {
      setRepo(dir);
      if (!label) setLabel(repoLeaf(dir));
    }
  }

  function toggle(id: string) {
    setKinds((k) => (k.includes(id) ? k.filter((x) => x !== id) : [...k, id]));
  }

  async function plant() {
    setErr("");
    setResult(null);
    if (!repo) {
      setErr("Pick a repository folder.");
      return;
    }
    if (!kinds.length) {
      setErr("Pick at least one secret type.");
      return;
    }
    setBusy(true);
    try {
      const r = await plantCanaries({ repoPath: repo, label, kinds, density });
      setResult(r);
      onDone();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="view">
      <header className="page-head">
        <p className="eyebrow">Drop unique bait</p>
        <h1>Plant canaries</h1>
        <p className="lede tight">
          Files look like leftover production secrets. The values exist only here — if a model
          recites them, it trained on this tree.
        </p>
      </header>

      <div className="split plant-grid">
        <div className="stack">
          <section className="panel">
            <label className="lbl">Repository</label>
            <div className="row">
              <input
                className="field grow"
                value={repo}
                onChange={(e) => setRepo(e.target.value)}
                placeholder="C:\src\your-repo"
              />
              <button className="btn ghost" onClick={pick}>
                Browse
              </button>
            </div>
            <label className="lbl">Label</label>
            <input
              className="field"
              value={label}
              onChange={(e) => setLabel(e.target.value)}
              placeholder="payments-api"
            />
            <p className="hint">Used in hunt prompts (“the project named payments-api”).</p>
          </section>

          <section className="panel">
            <label className="lbl">Secret types</label>
            <div className="chips">
              {snap.kinds.map((k) => (
                <button
                  key={k.id}
                  className={`chip ${kinds.includes(k.id) ? "on" : ""}`}
                  onClick={() => toggle(k.id)}
                  type="button"
                >
                  <b>{k.name}</b>
                  <span>{k.sample}</span>
                </button>
              ))}
            </div>
          </section>

          <section className="panel">
            <label className="lbl">Density</label>
            <div className="seg">
              {(["stealth", "mixed", "loud"] as const).map((d) => (
                <button
                  key={d}
                  className={density === d ? "on" : ""}
                  onClick={() => setDensity(d)}
                  type="button"
                >
                  {d}
                </button>
              ))}
            </div>
            <p className="hint">
              {density === "stealth" && "Two quiet example files. Easy to miss in a PR."}
              {density === "mixed" && "Env, JSON, terraform, compose — typical leak surface."}
              {density === "loud" && "Also a runbook and a GitHub Actions example. Maximum training bait."}
            </p>
            <button className="btn primary block" disabled={busy} onClick={plant}>
              {busy ? "Planting…" : "Plant into repo"}
            </button>
            {err && <p className="err">{err}</p>}
          </section>
        </div>

        <div className="stack">
          <section className="panel">
            <h2>Will write</h2>
            <ul className="file-list">
              {files.map((f) => (
                <li key={f}>
                  <code>{f}</code>
                </li>
              ))}
            </ul>
            <p className="hint">
              Existing env files are appended. Commit and push, or nobody will ever train on them.
            </p>
          </section>

          {result && (
            <section className="panel success">
              <h2>Planted {result.canaries.length} canaries</h2>
              <ul className="file-list">
                {result.files.map((f) => (
                  <li key={f.path}>
                    <button className="linkish" onClick={() => openPath(f.path)}>
                      {f.rel}
                    </button>
                  </li>
                ))}
              </ul>
              <p className="hint">Open Flock to copy values. Then Hunt when you have keys in Cages.</p>
            </section>
          )}
        </div>
      </div>
    </div>
  );
}

function previewFiles(density: string): string[] {
  const base = [".env.production.example", "config/credentials.example.json"];
  if (density === "stealth") return base;
  const mid = [
    ...base,
    "infra/terraform/secrets.auto.tfvars.example",
    "docker-compose.secrets.example.yml",
  ];
  if (density === "mixed") return mid;
  return [
    ...mid,
    "docs/internal-runbook.md",
    ".github/workflows/deploy.example.yml",
    "deploy/id_ed25519.example",
  ];
}
