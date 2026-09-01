import { useMemo } from "react";
import { CanaryMark } from "../CanaryMark";
import type { Snapshot, Tab } from "../types";
import { timeAgo } from "../util";

export function Mine({
  snap,
  go,
}: {
  snap: Snapshot;
  go: (t: Tab) => void;
}) {
  const answers = snap.provenance?.answers ?? [];
  const privateHits = snap.provenance?.privateHits ?? 0;
  const publicHits = snap.provenance?.publicHits ?? 0;
  const singing = privateHits > 0 || snap.hits > 0;
  const armed = snap.providers.filter((p) => p.apiKey.trim().length > 0).length;
  const last = snap.probes.length ? snap.probes[snap.probes.length - 1] : null;
  const top = useMemo(() => answers.slice(0, 4), [answers]);

  return (
    <div className="view">
      <header className="hero">
        <div className={`hero-bird ${singing ? "pulse" : ""}`}>
          <CanaryMark singing={singing} size={148} />
        </div>
        <div className="hero-copy">
          <p className="eyebrow">
            {privateHits > 0
              ? "Private training evidence"
              : singing
                ? "Calibration hits only"
                : "Training provenance"}
          </p>
          <h1 className="display">
            {privateHits > 0
              ? "We know which private sources they ate."
              : singing
                ? "Public sources only — so far."
                : "Get answers about where training came from."}
          </h1>
          <p className="lede">
            Models are trained on public and private data. This product cuts through the noise:
            load the sources you care about, hunt the labs, and get a cited answer — public vs
            private — for each model.
          </p>
          <div className="hero-actions">
            <button className="btn primary" onClick={() => go("chat")}>
              Chat with a model
            </button>
            <button className="btn ghost" onClick={() => go("probe")}>
              Probe / link GitHub
            </button>
            <button className="btn ghost" onClick={() => go("hits")}>
              See answers
            </button>
          </div>
        </div>
      </header>

      <div className="stats">
        <Stat k="Sources watched" v={String(snap.canaries.length)} d="planted + imported passages" />
        <Stat k="Armed cages" v={String(armed)} d="providers with a key" />
        <Stat
          k="Private hits"
          v={String(privateHits)}
          d="unique / proprietary regurgitations"
          hit={privateHits > 0}
        />
        <Stat k="Public hits" v={String(publicHits)} d="expected calibration hits" />
      </div>

      <div className="split">
        <section className="panel">
          <h2>How you get answers</h2>
          <ol className="steps">
            <li>
              <b>Probe</b> sources — link a GitHub repo, import files, or load the public-domain pack.
            </li>
            <li>
              <b>Chat</b> with a cage (GLM, Claude, GPT…) and fish it for where its knowledge came
              from. Replies that regurgitate watched sources become Answers.
            </li>
            <li>
              <b>Hunt</b> for automated membership tests. <b>Plant</b> unique canaries when you want
              bait that never existed before you minted it.
            </li>
          </ol>
        </section>
        <section className="panel">
          <h2>Provenance so far</h2>
          {top.length ? (
            <ul className="feed">
              {top.map((a) => (
                <li key={`${a.providerName}-${a.model}`} className="hit-row">
                  <span className={`pill ${a.privateHits > 0 ? "hit" : "public"}`}>
                    {a.privateHits > 0 ? "PRIVATE" : "PUBLIC"}
                  </span>
                  <div>
                    <strong>
                      {a.providerName} / {a.model}
                    </strong>
                    <em>
                      {a.privateSources[0] || a.publicSources[0] || "sources"}
                      {a.privateHits + a.publicHits > 1
                        ? ` · +${a.privateHits + a.publicHits - 1} more`
                        : ""}
                    </em>
                  </div>
                </li>
              ))}
            </ul>
          ) : last ? (
            <p className="empty-inline">
              Last probe {timeAgo(last.at)} on {last.providerName}. No songs yet.
            </p>
          ) : (
            <p className="empty-inline">No hunts yet. Probe sources, arm a cage, then Hunt.</p>
          )}
        </section>
      </div>
    </div>
  );
}

function Stat({
  k,
  v,
  d,
  hit,
}: {
  k: string;
  v: string;
  d: string;
  hit?: boolean;
}) {
  return (
    <div className={`stat ${hit ? "is-hit" : ""}`}>
      <span>{k}</span>
      <strong>{v}</strong>
      <em>{d}</em>
    </div>
  );
}
