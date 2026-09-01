import { useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { ingestCorpus, linkGithubRepo, loadPublicDomainPack, unlinkGithubRepo } from "../api";
import type { IngestResult, LinkGithubResult, Snapshot } from "../types";
import { familyLabel, repoLeaf, timeAgo } from "../util";

export function Probe({
  snap,
  onDone,
}: {
  snap: Snapshot;
  onDone: () => void;
}) {
  const [path, setPath] = useState("");
  const [title, setTitle] = useState("");
  const [text, setText] = useState("");
  const [ghUrl, setGhUrl] = useState("");
  const [ghToken, setGhToken] = useState("");
  const [saveToken, setSaveToken] = useState(true);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState("");
  const [result, setResult] = useState<IngestResult | null>(null);
  const [ghResult, setGhResult] = useState<LinkGithubResult | null>(null);

  const corpus = useMemo(
    () => snap.canaries.filter((c) => c.family === "corpus"),
    [snap.canaries],
  );
  const grouped = useMemo(() => {
    const map = new Map<string, typeof corpus>();
    for (const c of corpus) {
      const key = c.sourceTitle || c.label;
      const list = map.get(key) ?? [];
      list.push(c);
      map.set(key, list);
    }
    return [...map.entries()];
  }, [corpus]);
  const linked = snap.linkedRepos ?? [];

  async function pickFile() {
    const file = await open({
      multiple: false,
      title: "Choose a document to probe",
      filters: [{ name: "Text", extensions: ["txt", "md", "rst", "csv", "json", "py", "ts", "rs"] }],
    });
    if (typeof file === "string") {
      setPath(file);
      if (!title) setTitle(repoLeaf(file));
    }
  }

  async function pickFolder() {
    const dir = await open({ directory: true, title: "Choose a folder of documents" });
    if (typeof dir === "string") {
      setPath(dir);
      if (!title) setTitle(repoLeaf(dir));
    }
  }

  async function ingest() {
    setErr("");
    setResult(null);
    setGhResult(null);
    if (!text.trim() && !path) {
      setErr("Paste a document or pick a file / folder.");
      return;
    }
    setBusy(true);
    try {
      const r = await ingestCorpus({
        path,
        title,
        text: text.trim(),
        maxPassages: 12,
      });
      setResult(r);
      setText("");
      onDone();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function loadPack() {
    setErr("");
    setBusy(true);
    try {
      const r = await loadPublicDomainPack();
      setResult(r);
      onDone();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function linkRepo() {
    setErr("");
    setGhResult(null);
    setResult(null);
    if (!ghUrl.trim()) {
      setErr("Paste a GitHub repo URL.");
      return;
    }
    setBusy(true);
    try {
      const r = await linkGithubRepo({
        url: ghUrl.trim(),
        token: ghToken.trim() || undefined,
        maxPassages: 12,
        saveToken: saveToken && !!ghToken.trim(),
      });
      setGhResult(r);
      setGhUrl("");
      onDone();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function unlink(id: string) {
    setErr("");
    try {
      await unlinkGithubRepo(id);
      onDone();
    } catch (e) {
      setErr(String(e));
    }
  }

  return (
    <div className="view">
      <header className="page-head">
        <p className="eyebrow">Ask where the training came from</p>
        <h1>Probe sources</h1>
        <p className="lede tight">
          Load the works you care about — private wikis, datasets, manuscripts, GitHub repos, or
          public books. We extract distinctive passages, hunt or chat with models against them, and
          answer whether each lab can reproduce that source.
        </p>
      </header>

      <div className="split plant-grid">
        <div className="stack">
          <section className="panel">
            <label className="lbl">Link a GitHub repo</label>
            <input
              className="field"
              value={ghUrl}
              onChange={(e) => setGhUrl(e.target.value)}
              placeholder="https://github.com/owner/repo"
            />
            <label className="lbl">GitHub token (optional, for private repos)</label>
            <input
              className="field"
              type="password"
              value={ghToken}
              onChange={(e) => setGhToken(e.target.value)}
              placeholder={snap.hasGithubToken ? "Saved token on file — paste to replace" : "ghp_…"}
            />
            <label className="check tight-check">
              <input
                type="checkbox"
                checked={saveToken}
                onChange={(e) => setSaveToken(e.target.checked)}
              />
              <span>Remember token for later links</span>
            </label>
            <button className="btn primary block" disabled={busy} onClick={linkRepo}>
              {busy ? "Linking…" : "Link repo & extract passages"}
            </button>
            <p className="hint">
              Pulls README and distinctive source files, then watches them as private corpus. Use
              Chat to fish models about that exact repo.
            </p>
          </section>

          <section className="panel">
            <label className="lbl">Work title</label>
            <input
              className="field"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Moby-Dick · internal wiki · How to Train a Pet Dragon"
            />
            <label className="lbl">File or folder</label>
            <div className="row">
              <input
                className="field grow"
                value={path}
                onChange={(e) => setPath(e.target.value)}
                placeholder="Path to a document or directory"
              />
              <button className="btn ghost" type="button" onClick={pickFile}>
                File
              </button>
              <button className="btn ghost" type="button" onClick={pickFolder}>
                Folder
              </button>
            </div>
            <label className="lbl">Or paste text</label>
            <textarea
              className="field area"
              value={text}
              onChange={(e) => setText(e.target.value)}
              placeholder="Paste a chapter, README, dataset dump, or any passage you want cited…"
            />
            <button className="btn primary block" disabled={busy} onClick={ingest}>
              {busy ? "Extracting…" : "Extract passages"}
            </button>
            <button className="btn ghost block" disabled={busy} onClick={loadPack}>
              Load public-domain pack
            </button>
            <p className="hint">
              The pack is calibration: Moby-Dick, Pride and Prejudice, Alice, Frankenstein, Sherlock,
              A Christmas Carol, The Time Machine. Expected hits on most frontier models. We do not
              ship copyrighted books — import those yourself.
            </p>
            {err && <p className="err">{err}</p>}
          </section>
        </div>

        <div className="stack">
          {(result || ghResult) && (
            <section className="panel success">
              <h2>
                Loaded {(result ?? ghResult)!.canaries.length} passages
                {ghResult
                  ? ` from ${ghResult.linked.owner}/${ghResult.linked.name}`
                  : result?.works
                    ? ` from ${result.works} work${result.works === 1 ? "" : "s"}`
                    : ""}
              </h2>
              <p className="hint">Open Chat to question a model, or Hunt to run membership probes.</p>
            </section>
          )}

          <section className="panel">
            <h2>Linked GitHub repos</h2>
            {linked.length === 0 ? (
              <p className="empty-inline">No repos linked yet.</p>
            ) : (
              <ul className="file-list">
                {linked.map((r) => (
                  <li key={r.id} className="linked-row">
                    <div>
                      <b>
                        {r.owner}/{r.name}
                      </b>
                      <span className="hint">
                        {" "}
                        {r.files.length} file{r.files.length === 1 ? "" : "s"} · linked{" "}
                        {timeAgo(r.linkedAt)}
                      </span>
                    </div>
                    <button className="btn tiny" type="button" onClick={() => unlink(r.id)}>
                      Unlink
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </section>

          <section className="panel">
            <h2>What a hit means</h2>
            <ol className="steps">
              <li>
                <b>Not a dump of the training set.</b> Models do not hand you a bibliography. We
                test whether they can reproduce a distinctive passage.
              </li>
              <li>
                <b>Citation is the source you provided</b> — title, file, and passage locator — plus
                the model that sang.
              </li>
              <li>
                <b>Famous books are expected.</b> Ishmael is not a leak. Your private wiki, unique
                dataset row, or unpublished manuscript is.
              </li>
            </ol>
          </section>
          <section className="panel">
            <h2>In flock</h2>
            {grouped.length === 0 ? (
              <p className="empty-inline">No corpus passages yet.</p>
            ) : (
              <ul className="file-list">
                {grouped.map(([name, items]) => (
                  <li key={name}>
                    <b>{name}</b>
                    <span className="hint">
                      {" "}
                      {items.length} passage{items.length === 1 ? "" : "s"} ·{" "}
                      {items[0].sourceKind === "github"
                        ? "GitHub"
                        : familyLabel(
                            items[0].sourceKind === "public_domain" ? "corpus" : items[0].family,
                          )}
                      {items[0].sourceKind === "public_domain" ? " · public domain" : ""}
                      {items[0].sourceKind === "github" ? " · watched repo" : ""}
                    </span>
                  </li>
                ))}
              </ul>
            )}
          </section>
        </div>
      </div>
    </div>
  );
}
