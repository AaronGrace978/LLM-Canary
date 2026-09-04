export type KindInfo = {
  id: string;
  name: string;
  blurb: string;
  sample: string;
  family: string;
  familyName: string;
};

export type Canary = {
  id: string;
  kind: string;
  kindName: string;
  family?: string;
  value: string;
  needles: string[];
  envNames: string[];
  label: string;
  repoPath: string;
  repoName: string;
  files: string[];
  plantedAt: string;
  sourceTitle?: string;
  sourceLocator?: string;
  sourceKind?: string;
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
  citation?: string;
  sensitivity?: string;
  /** 0–1 share of the hidden remainder reproduced verbatim. */
  score?: number;
  trial?: number;
  control?: boolean;
  abstained?: boolean;
  temperature?: number;
};

export type StrategyStat = {
  strategy: string;
  probes: number;
  hits: number;
  hitRate: number;
  meanScore: number;
};

export type ProvenanceAnswer = {
  providerName: string;
  model: string;
  publicSources: string[];
  privateSources: string[];
  publicHits: number;
  privateHits: number;
  probes: number;
  hits: number;
  hitRate: number;
  ciLow: number;
  ciHigh: number;
  meanScore: number;
  abstainRate: number;
  privateProbes: number;
  publicProbes: number;
  controlProbes: number;
  controlHits: number;
  errors: number;
  byStrategy: StrategyStat[];
};

export type ProvenanceSummary = {
  answers: ProvenanceAnswer[];
  privateHits: number;
  publicHits: number;
};

export type LinkedRepo = {
  id: string;
  url: string;
  owner: string;
  name: string;
  defaultBranch: string;
  description: string;
  linkedAt: string;
  files: string[];
};

export type Snapshot = {
  canaries: Canary[];
  providers: Provider[];
  probes: Probe[];
  kinds: KindInfo[];
  hits: number;
  provenance?: ProvenanceSummary;
  linkedRepos?: LinkedRepo[];
  hasGithubToken?: boolean;
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
  score: number;
  control: boolean;
  done: number;
  total: number;
};

export type HuntSummary = {
  probes: Probe[];
  hits: number;
  errors: number;
  controlProbes: number;
  controlHits: number;
  trials: number;
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
  citation?: string;
  sensitivity?: string;
  score?: number;
};

export type IngestResult = {
  canaries: Canary[];
  works: number;
  skipped: number;
};

export type ChatMessage = {
  role: "user" | "assistant" | "system";
  content: string;
};

export type ChatHit = {
  canaryId: string;
  kind: string;
  label: string;
  matched: string[];
  citation: string;
  sensitivity: string;
  score: number;
};

export type ChatTurnResult = {
  reply: string;
  hits: ChatHit[];
  probesRecorded: number;
};

export type LinkGithubResult = {
  linked: LinkedRepo;
  canaries: Canary[];
  works: number;
  skipped: number;
};

export type Tab = "mine" | "plant" | "probe" | "chat" | "hunt" | "flock" | "hits" | "cages";

export const TABS: { id: Tab; label: string; hint: string }[] = [
  { id: "mine", label: "Mine", hint: "Overview" },
  { id: "plant", label: "Plant", hint: "Drop canaries" },
  { id: "probe", label: "Probe", hint: "Cite a corpus" },
  { id: "chat", label: "Chat", hint: "Question models" },
  { id: "hunt", label: "Hunt", hint: "Ask models" },
  { id: "flock", label: "Flock", hint: "Planted tokens" },
  { id: "hits", label: "Answers", hint: "Provenance" },
  { id: "cages", label: "Cages", hint: "Providers" },
];
