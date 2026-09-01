export function maskSecret(value: string, reveal: boolean): string {
  if (reveal) return value;
  if (value.length <= 12) return "••••••••••••";
  return `${value.slice(0, 7)}${"•".repeat(10)}${value.slice(-4)}`;
}

export function timeAgo(iso: string): string {
  const t = new Date(iso).getTime();
  if (Number.isNaN(t)) return iso;
  const s = Math.round((Date.now() - t) / 1000);
  if (s < 45) return "just now";
  if (s < 3600) return `${Math.floor(s / 60)}m ago`;
  if (s < 86400) return `${Math.floor(s / 3600)}h ago`;
  if (s < 604800) return `${Math.floor(s / 86400)}d ago`;
  return new Date(iso).toLocaleDateString();
}

export function initials(name: string): string {
  const parts = name.split(/\s+/).filter(Boolean);
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
  return (parts[0][0] + parts[1][0]).toUpperCase();
}

export async function copyText(text: string) {
  await navigator.clipboard.writeText(text);
}

export function repoLeaf(path: string): string {
  const parts = path.split(/[/\\]/).filter(Boolean);
  return parts[parts.length - 1] || path;
}

export function familyLabel(family?: string): string {
  switch (family) {
    case "code":
      return "Code";
    case "prose":
      return "Documents";
    case "data":
      return "Datasets";
    case "identity":
      return "Identity";
    case "custom":
      return "Custom";
    case "corpus":
      return "Corpus";
    case "secret":
    default:
      return "Secrets";
  }
}

export function citationFor(c: {
  sourceTitle?: string;
  sourceLocator?: string;
  label: string;
}): string {
  const title = (c.sourceTitle || c.label).trim() || c.label;
  const loc = (c.sourceLocator || "").trim();
  return loc ? `${title} · ${loc}` : title;
}
