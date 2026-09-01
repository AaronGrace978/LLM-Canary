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
  const singing = snap.hits > 0;
  const armed = snap.providers.filter((p) => p.apiKey.trim().length > 0).length;
  const last = snap.probes.length ? snap.probes[snap.probes.length - 1] : null;
  const recentHits = snap.probes.filter((p) => p.hit).slice(-4).reverse();

  return (
    <div className="view">
      <header className="hero">
        <div className={`hero-bird ${singing ? "pulse" : ""}`}>
          <CanaryMark singing={singing} size={148} />
        </div>
        <div className="hero-copy">
          <p className="eyebrow">{singing ? "Alert" : "The shaft is dark. The bird is watching."}</p>
          <h1 className="display">
            {singing ? "A canary is singing." : "The mine is quiet."}
          </h1>
          <p className="lede">
            Plant unique canaries in a repo — secrets, code, docs, datasets, or any string you
            flag. If any model ever regurgitates them, you know exactly who trained on your data.
          </p>
          <div className="hero-actions">
            <button className="btn primary" onClick={() => go("plant")}>
              Plant canaries
            </button>
            <button className="btn ghost" onClick={() => go("cages")}>
              Arm providers
            </button>
            <button className="btn ghost" onClick={() => go("hunt")} disabled={!snap.canaries.length}>
              Hunt
            </button>
          </div>
        </div>
      </header>

      <div className="stats">
        <Stat k="Planted" v={String(snap.canaries.length)} d="unique tokens in repos" />
        <Stat k="Armed cages" v={String(armed)} d="providers with a key" />
        <Stat k="Probes" v={String(snap.probes.length)} d="hunt questions asked" />
        <Stat k="Hits" v={String(snap.hits)} d="regurgitations caught" hit={snap.hits > 0} />
      </div>

      <div className="split">
        <section className="panel">
          <h2>How the trap works</h2>
          <ol className="steps">
            <li>
              <b>Plant</b> high-entropy bait that looks like real training data — secrets, unique
              code, internal phrases, dataset rows, or any string you paste. Only this machine
              knows the values.
            </li>
            <li>
              <b>Commit and push.</b> If the files never leave your laptop, no trainer will ever see them.
            </li>
            <li>
              <b>Hunt.</b> We ask each model to complete a prefix, recall a project, or reproduce a needle.
              If the unique remainder comes back, that lab trained on this tree.
            </li>
          </ol>
        </section>
        <section className="panel">
          <h2>Recent activity</h2>
          {recentHits.length ? (
            <ul className="feed">
              {recentHits.map((p) => (
                <li key={p.id} className="hit-row">
                  <span className="pill hit">HIT</span>
                  <div>
                    <strong>
                      {p.providerName} / {p.model}
                    </strong>
                    <em>
                      {p.canaryKind} · {timeAgo(p.at)}
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
            <p className="empty-inline">No hunts yet. Arm a cage, then send the flock.</p>
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
