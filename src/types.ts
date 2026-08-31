export type KindInfo = {
  id: string;
  name: string;
  blurb: string;
  sample: string;
};

export type Canary = {
  id: string;
  kind: string;
  kindName: string;
  value: string;
  needles: string[];
  envNames: string[];
  label: string;
  repoPath: string;
  repoName: string;
  files: string[];
  plantedAt: string;
};

export type Provider = {
  id: string;
  name: string;
  kind: string;
  blurb: string;
  docsUrl: string;
  enabled: boolean;
  apiKey: string;
  baseUrl: string;
  model: string;
  models: string[];
  lastError: string | null;
  lastOkAt: string | null;
};

export type Probe = {
  id: string;
  at: string;
  providerId: string;
  providerName: string;
  model: string;
  canaryId: string;
  canaryKind: string;
  canaryLabel: string;
  strategy: string;
  prompt: string;
  response: string;
  hit: boolean;
  matched: string[];
  error: string | null;
};

export type Snapshot = {
  canaries: Canary[];
  providers: Provider[];
  probes: Probe[];
  kinds: KindInfo[];
  hits: number;
};

export type PlantResult = {
  canaries: Canary[];
  files: { path: string; rel: string }[];
};

export type HuntProgress = {
  phase: string;
  providerId: string;
  providerName: string;
  model: string;
  canaryId: string;
  strategy: string;
  message: string;
  hit: boolean | null;
};

export type HuntSummary = {
  probes: Probe[];
  hits: number;
  errors: number;
};

export type WebPrompt = {
  canaryId: string;
  title: string;
  prompt: string;
};

export type ScanHit = {
  canaryId: string;
  kind: string;
  label: string;
  matched: string[];
};

export type Tab = "mine" | "plant" | "hunt" | "flock" | "hits" | "cages";

export const TABS: { id: Tab; label: string; hint: string }[] = [
  { id: "mine", label: "Mine", hint: "Overview" },
  { id: "plant", label: "Plant", hint: "Drop canaries" },
  { id: "hunt", label: "Hunt", hint: "Probe models" },
  { id: "flock", label: "Flock", hint: "Planted tokens" },
  { id: "hits", label: "Hits", hint: "Evidence" },
  { id: "cages", label: "Cages", hint: "Providers" },
];
