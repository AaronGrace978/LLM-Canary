import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { CanaryMark } from "./CanaryMark";
import { loadSnapshot } from "./api";
import type { Snapshot, Tab } from "./types";
import { TABS } from "./types";
import { Mine } from "./views/Mine";
import { Plant } from "./views/Plant";
import { Hunt } from "./views/Hunt";
import { Flock } from "./views/Flock";
import { Hits } from "./views/Hits";
import { Cages } from "./views/Cages";
import "./App.css";

export default function App() {
  const [snap, setSnap] = useState<Snapshot | null>(null);
  const [tab, setTab] = useState<Tab>("mine");
  const [reveal, setReveal] = useState(false);
  const [bootErr, setBootErr] = useState("");
  const [maximized, setMaximized] = useState(false);

  async function reload() {
    try {
      setSnap(await loadSnapshot());
    } catch (e) {
      setBootErr(String(e));
    }
  }

  useEffect(() => {
    reload();
    const w = getCurrentWindow();
    w.isMaximized().then(setMaximized).catch(() => undefined);
    let unlisten: (() => void) | undefined;
    w.onResized(async () => {
      setMaximized(await w.isMaximized());
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const el = e.target as HTMLElement | null;
      if (el && (el.tagName === "INPUT" || el.tagName === "TEXTAREA" || el.isContentEditable)) return;
      const map: Record<string, Tab> = {
        "1": "mine",
        "2": "plant",
        "3": "hunt",
        "4": "flock",
        "5": "hits",
        "6": "cages",
      };
      if (map[e.key]) setTab(map[e.key]);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const hits = snap?.hits ?? 0;

  return (
    <div className="shell">
      <div className="grain" />
      <header className="titlebar">
        <div className="brand" data-tauri-drag-region>
          <CanaryMark singing={hits > 0} size={28} />
          <div>
            <strong>LLM Canary</strong>
            <em>if they trained on it, the bird sings</em>
          </div>
        </div>
        <div className="drag" data-tauri-drag-region />
        <div className="title-actions">
          <button
            className={`reveal ${reveal ? "on" : ""}`}
            onClick={() => setReveal((v) => !v)}
            type="button"
          >
            {reveal ? "Values shown" : "Values hidden"}
          </button>
          <div className="win">
            <button type="button" onClick={() => getCurrentWindow().minimize()} aria-label="Minimize">
              ─
            </button>
            <button
              type="button"
              onClick={() => getCurrentWindow().toggleMaximize()}
              aria-label="Maximize"
            >
              {maximized ? "❐" : "☐"}
            </button>
            <button
              className="close"
              type="button"
              onClick={() => getCurrentWindow().close()}
              aria-label="Close"
            >
              ✕
            </button>
          </div>
        </div>
      </header>

      <div className="body">
        <nav className="rail">
          {TABS.map((t) => (
            <button
              key={t.id}
              className={tab === t.id ? "on" : ""}
              onClick={() => setTab(t.id)}
              type="button"
            >
              <span>{t.label}</span>
              <em>{t.hint}</em>
              {t.id === "hits" && hits > 0 && <b>{hits}</b>}
            </button>
          ))}
        </nav>
        <main>
          {bootErr && <p className="err pad">{bootErr}</p>}
          {!snap && !bootErr && <p className="empty-inline pad">Lighting the lamp…</p>}
          {snap && tab === "mine" && <Mine snap={snap} go={setTab} />}
          {snap && tab === "plant" && <Plant snap={snap} onDone={reload} />}
          {snap && tab === "hunt" && <Hunt snap={snap} onDone={reload} />}
          {snap && tab === "flock" && <Flock snap={snap} reveal={reveal} onDone={reload} />}
          {snap && tab === "hits" && <Hits snap={snap} onDone={reload} />}
          {snap && tab === "cages" && <Cages snap={snap} onDone={reload} />}
        </main>
      </div>
    </div>
  );
}
