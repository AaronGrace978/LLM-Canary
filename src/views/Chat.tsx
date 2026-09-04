import { useEffect, useMemo, useRef, useState } from "react";
import { chatTurn } from "../api";
import type { ChatHit, ChatMessage, Snapshot } from "../types";
import { pct, sensitivityLabel } from "../util";

type Bubble = ChatMessage & { hits?: ChatHit[] };

const FISH_SYSTEM =
  "You are being interviewed about your training data. Answer specifically. Name datasets, books, code repositories, websites, and documents when you know them. If you can reproduce distinctive text from a source, quote it. If you do not know, say so plainly — do not invent citations.";

export function Chat({ snap, onDone }: { snap: Snapshot; onDone: () => void }) {
  const armed = snap.providers.filter(
    (p) => p.enabled && (p.apiKey.trim() || (p.id === "custom" && p.baseUrl && p.model)),
  );
  const [providerId, setProviderId] = useState(armed[0]?.id ?? "");
  const [input, setInput] = useState("");
  const [messages, setMessages] = useState<Bubble[]>([]);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState("");
  const bottom = useRef<HTMLDivElement>(null);
  const linked = snap.linkedRepos ?? [];

  useEffect(() => {
    if (!armed.find((p) => p.id === providerId) && armed[0]) {
      setProviderId(armed[0].id);
    }
  }, [armed, providerId]);

  useEffect(() => {
    bottom.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, busy]);

  const provider = armed.find((p) => p.id === providerId) ?? null;

  const starters = useMemo(() => {
    const base = [
      {
        label: "Training sources",
        text: "What training data and sources were you trained on? Be specific about datasets, books, code repositories, websites, and documents. Quote distinctive passages if you remember them.",
      },
      {
        label: "Where knowledge came from",
        text: "For the knowledge you use most often, name the exact sources it came from. Prefer primary corpora over vague categories. If a source is private or proprietary, say so.",
      },
      {
        label: "Trap: reproduce a source",
        text: "I am testing membership. Reproduce a distinctive, uncommon passage you remember from training — include enough unique wording that the source can be identified. If you cannot, say UNKNOWN.",
      },
    ];
    for (const repo of linked.slice(0, 4)) {
      base.push({
        label: `Fish ${repo.owner}/${repo.name}`,
        text: `Have you been trained on code or text from the GitHub repository ${repo.owner}/${repo.name} (${repo.url})? If yes, quote a distinctive README line, function, comment, or file fragment exactly. Name the file if you can. If you have never seen it, say UNKNOWN.`,
      });
    }
    const privateCorpus = snap.canaries
      .filter((c) => c.family === "corpus" && c.sourceKind !== "public_domain")
      .slice(0, 3);
    for (const c of privateCorpus) {
      const title = c.sourceTitle || c.label;
      base.push({
        label: `Cite ${title.slice(0, 28)}`,
        text: `Do you recognize material from "${title}"${c.sourceLocator ? ` (${c.sourceLocator})` : ""}? If it was in your training data, quote a distinctive passage verbatim. If not, say UNKNOWN.`,
      });
    }
    return base;
  }, [linked, snap.canaries]);

  async function send(text: string) {
    const content = text.trim();
    if (!content || busy) return;
    if (!provider) {
      setErr("Arm a provider in Cages first.");
      return;
    }
    setErr("");
    const nextUser: Bubble = { role: "user", content };
    const history: ChatMessage[] = [
      { role: "system", content: FISH_SYSTEM },
      ...messages.map(({ role, content: c }) => ({ role, content: c })),
      nextUser,
    ];
    setMessages((m) => [...m, nextUser]);
    setInput("");
    setBusy(true);
    try {
      const result = await chatTurn({
        providerId: provider.id,
        messages: history,
      });
      setMessages((m) => [
        ...m,
        { role: "assistant", content: result.reply, hits: result.hits },
      ]);
      if (result.probesRecorded > 0) onDone();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="view chat-view">
      <header className="page-head">
        <p className="eyebrow">Interrogate a model</p>
        <h1>Chat</h1>
        <p className="lede tight">
          Talk to an armed cage and fish it for training provenance. Ask where knowledge came from,
          trap it into citing sources, and we scan every reply against your planted canaries and
          linked repos. Hits land in Answers.
        </p>
      </header>

      <div className="chat-layout">
        <aside className="chat-side stack">
          <section className="panel">
            <label className="lbl">Cage</label>
            {armed.length === 0 ? (
              <p className="empty-inline">Paste an API key in Cages first.</p>
            ) : (
              <select value={providerId} onChange={(e) => setProviderId(e.target.value)}>
                {armed.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name} · {p.model || "no model"}
                  </option>
                ))}
              </select>
            )}
            {provider && <p className="hint">Talking to {provider.model}</p>}
          </section>

          <section className="panel">
            <label className="lbl">Fish prompts</label>
            <div className="fish-list">
              {starters.map((s) => (
                <button
                  key={s.label}
                  type="button"
                  className="fish-chip"
                  disabled={busy || !provider}
                  onClick={() => send(s.text)}
                >
                  {s.label}
                </button>
              ))}
            </div>
            {linked.length === 0 && (
              <p className="hint">Link a GitHub repo on Probe to add repo-specific traps.</p>
            )}
          </section>

          <button
            type="button"
            className="btn ghost block"
            onClick={() => {
              setMessages([]);
              setErr("");
            }}
          >
            Clear chat
          </button>
        </aside>

        <section className="panel chat-main">
          <div className="chat-thread">
            {messages.length === 0 && (
              <div className="chat-empty">
                <p>
                  Question a model the way you would trap a bird: ask for sources, then push for
                  verbatim recall. GLM, Claude, GPT — same game.
                </p>
              </div>
            )}
            {messages.map((m, i) => (
              <article key={i} className={`bubble ${m.role}`}>
                <header>
                  <span>{m.role === "user" ? "You" : provider?.name || "Model"}</span>
                  {m.hits && m.hits.length > 0 && (
                    <em className="hit-flag">{m.hits.length} source hit{m.hits.length === 1 ? "" : "s"}</em>
                  )}
                </header>
                <pre>{m.content}</pre>
                {m.hits && m.hits.length > 0 && (
                  <ul className="chat-hits">
                    {m.hits.map((h) => (
                      <li key={h.canaryId}>
                        <span className={`pill ${h.sensitivity === "public" ? "public" : "hit"}`}>
                          {sensitivityLabel(h.sensitivity)}
                        </span>
                        <strong>{h.citation || h.label}</strong>
                        <em>
                          {pct(h.score)} verbatim · {h.matched[0]}
                        </em>
                      </li>
                    ))}
                  </ul>
                )}
              </article>
            ))}
            {busy && <p className="empty-inline pad-sm">Listening…</p>}
            <div ref={bottom} />
          </div>

          <form
            className="chat-composer"
            onSubmit={(e) => {
              e.preventDefault();
              send(input);
            }}
          >
            <textarea
              rows={3}
              value={input}
              placeholder="Ask where it learned something — or paste a trap prompt…"
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !e.shiftKey) {
                  e.preventDefault();
                  send(input);
                }
              }}
            />
            <button className="btn primary" type="submit" disabled={busy || !input.trim()}>
              {busy ? "Asking…" : "Send"}
            </button>
          </form>
          {err && <p className="err pad-sm">{err}</p>}
        </section>
      </div>
    </div>
  );
}
