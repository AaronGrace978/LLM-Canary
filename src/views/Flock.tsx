import { openPath } from "@tauri-apps/plugin-opener";
import { deleteCanary } from "../api";
import type { Snapshot } from "../types";
import { copyText, maskSecret, timeAgo, familyLabel, citationFor } from "../util";
import { useState } from "react";

export function Flock({
  snap,
  reveal,
  onDone,
}: {
  snap: Snapshot;
  reveal: boolean;
  onDone: () => void;
}) {
  const [copied, setCopied] = useState("");

  async function remove(id: string) {
    if (!confirm("Remove this canary from the ledger? Files in the repo are not deleted.")) return;
    await deleteCanary(id);
    onDone();
  }

  return (
    <div className="view">
      <header className="page-head">
        <p className="eyebrow">The ledger</p>
        <h1>Flock</h1>
        <p className="lede tight">
          Every unique value we planted or ingested — secrets, code, docs, datasets, identities,
          custom flags, and corpus passages. Keep this list off Git.
        </p>
      </header>

      {snap.canaries.length === 0 ? (
        <section className="panel">
          <p className="empty-inline">The cage is empty. Plant something first.</p>
        </section>
      ) : (
        <div className="table-wrap">
          <table className="grid">
            <thead>
              <tr>
                <th>Kind</th>
                <th>Family</th>
                <th>Source</th>
                <th>Value</th>
                <th>Label</th>
                <th>Repo</th>
                <th>Planted</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {snap.canaries.map((c) => (
                <tr key={c.id}>
                  <td>{c.kindName}</td>
                  <td>{familyLabel(c.family)}</td>
                  <td>{citationFor(c)}</td>
                  <td>
                    <code className="secret">{maskSecret(c.value, reveal)}</code>
                  </td>
                  <td>{c.label}</td>
                  <td>
                    {c.repoPath && c.repoPath !== "public-domain" && c.sourceKind !== "public_domain" ? (
                      <button className="linkish" onClick={() => openPath(c.repoPath)}>
                        {c.repoName}
                      </button>
                    ) : (
                      <span>{c.repoName}</span>
                    )}
                  </td>
                  <td>{timeAgo(c.plantedAt)}</td>
                  <td className="actions">
                    <button
                      className="btn tiny"
                      onClick={async () => {
                        await copyText(c.value);
                        setCopied(c.id);
                      }}
                    >
                      {copied === c.id ? "Copied" : "Copy"}
                    </button>
                    <button className="btn tiny danger" onClick={() => remove(c.id)}>
                      Drop
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
