import { useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import { plantCanaries } from "../api";
import type { KindInfo, PlantResult, Snapshot } from "../types";
import { repoLeaf } from "../util";

const FAMILY_ORDER = ["secret", "code", "prose", "data", "identity"];

export function Plant({
  snap,
  onDone,
}: {
  snap: Snapshot;
  onDone: () => void;
}) {
  const [repo, setRepo] = useState("");
  const [label, setLabel] = useState("");
  const [kinds, setKinds] = useState<string[]>(["aws", "github", "openai", "code_watermark", "doc_phrase"]);
  const [customText, setCustomText] = useState("");
  const [density, setDensity] = useState<"stealth" | "mixed" | "loud">("mixed");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState("");
  const [result, setResult] = useState<PlantResult | null>(null);

  const groups = useMemo(() => {
    return FAMILY_ORDER.map((id) => {
      const members = snap.kinds.filter((k) => k.family === id);
      return {
        id,
        name: members[0]?.familyName ?? id,
        kinds: members,
      };
    }).filter((g) => g.kinds.length);
  }, [snap.kinds]);

  const customTokens = useMemo(
    () =>
      customText
        .split(/\r?\n/)
        .map((l) => l.trim())
        .filter(Boolean),
    [customText],
  );

  const files = useMemo(
    () => previewFiles(snap.kinds, kinds, density, customTokens.length > 0),
    [snap.kinds, kinds, density, customTokens.length],
  );

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

  function toggleFamily(familyKinds: KindInfo[]) {
    const ids = familyKinds.map((k) => k.id);
    const allOn = ids.every((id) => kinds.includes(id));
    setKinds((current) =>
      allOn ? current.filter((id) => !ids.includes(id)) : [...new Set([...current, ...ids])],
    );
  }

  async function plant() {
    setErr("");
    setResult(null);
    if (!repo) {
      setErr("Pick a repository folder.");
      return;
    }
    if (!kinds.length && !customTokens.length) {
      setErr("Pick at least one training-data type, or paste a custom flag.");
      return;
    }
    setBusy(true);
    try {
      const r = await plantCanaries({
        repoPath: repo,
        label,
        kinds,
        density,
        customTokens,
      });
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
          Flag any kind of training data — secrets, code, docs, datasets, identities, or your own
          strings. If a model recites them, it trained on this tree.
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

          {groups.map((g) => (
            <section className="panel" key={g.id}>
              <div className="family-head">
                <label className="lbl">{g.name}</label>
                <button className="btn tiny" type="button" onClick={() => toggleFamily(g.kinds)}>
                  {g.kinds.every((k) => kinds.includes(k.id)) ? "Clear" : "All"}
                </button>
              </div>
              <div className="chips">
                {g.kinds.map((k) => (
                  <button
                    key={k.id}
                    className={`chip ${kinds.includes(k.id) ? "on" : ""}`}
                    onClick={() => toggle(k.id)}
                    type="button"
                    title={k.blurb}
                  >
                    <b>{k.name}</b>
                    <span>{k.sample}</span>
                  </button>
                ))}
              </div>
            </section>
          ))}

          <section className="panel">
            <label className="lbl">Custom flags</label>
            <textarea
              className="field area"
              value={customText}
              onChange={(e) => setCustomText(e.target.value)}
              placeholder={"Any unique strings — one per line.\nA phrase from a private doc\nmy-dataset-row-id-9f3c"}
            />
            <p className="hint">
              Paste anything you want flagged: comments, phrases, IDs, even a whole sentence. Short
              lines get a unique suffix so they stay hunt-able.
            </p>
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
              {density === "stealth" && "Few quiet files per selected family. Easy to miss in a PR."}
              {density === "mixed" && "Typical leak surface for each family you selected."}
              {density === "loud" && "Extra languages, runbooks, and workflows. Maximum training bait."}
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
            {files.length === 0 ? (
              <p className="empty-inline">Select a type or paste a custom flag to see files.</p>
            ) : (
              <ul className="file-list">
                {files.map((f) => (
                  <li key={f}>
                    <code>{f}</code>
                  </li>
                ))}
              </ul>
            )}
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

function previewFiles(
  catalog: KindInfo[],
  selected: string[],
  density: string,
  hasCustom: boolean,
): string[] {
  const families = new Set(
    catalog.filter((k) => selected.includes(k.id)).map((k) => k.family),
  );
  if (hasCustom) families.add("custom");
  const kinds = new Set(selected);
  const mixed = density === "mixed" || density === "loud";
  const loud = density === "loud";
  const out: string[] = [];

  if (families.has("secret") || families.has("identity")) {
    out.push(".env.production.example");
  }
  if (families.has("secret")) {
    out.push("config/credentials.example.json");
    if (mixed) {
      out.push("infra/terraform/secrets.auto.tfvars.example", "docker-compose.secrets.example.yml");
    }
    if (loud) {
      out.push("docs/internal-runbook.md", ".github/workflows/deploy.example.yml");
      if (kinds.has("private_key")) out.push("deploy/id_ed25519.example");
    }
  }
  if (families.has("code")) {
    out.push("internal/canary_markers.py");
    if (mixed) out.push("internal/canary_markers.ts");
    if (loud) out.push("internal/canary_markers.rs");
  }
  if (families.has("prose") || families.has("custom")) {
    out.push("docs/internal-architecture.md");
  }
  if (families.has("custom")) out.push("docs/canary-notes.md");
  if (families.has("data")) {
    if (kinds.has("dataset_row")) out.push("data/canary_seed.csv");
    if (mixed && (kinds.has("json_record") || kinds.has("dataset_row"))) {
      out.push("fixtures/canary_records.json");
    }
  }
  if (families.has("identity") && !families.has("secret") && mixed) {
    out.push("fixtures/canary_operators.json");
  }
  return out;
}
