import { invoke } from "@tauri-apps/api/core";
import type { HuntSummary, IngestResult, PlantResult, ScanHit, Snapshot, WebPrompt } from "./types";

export type TestResult = { ok: boolean; model: string; preview: string };

export async function loadSnapshot(): Promise<Snapshot> {
  return invoke("load_snapshot");
}

export async function saveProvider(patch: {
  id: string;
  enabled?: boolean;
  apiKey?: string;
  baseUrl?: string;
  model?: string;
}): Promise<Snapshot> {
  return invoke("save_provider", { patch });
}

export async function plantCanaries(req: {
  repoPath: string;
  label: string;
  kinds: string[];
  density: string;
  customTokens?: string[];
}): Promise<PlantResult> {
  return invoke("plant_canaries", { req });
}

export async function ingestCorpus(req: {
  path?: string;
  title?: string;
  text?: string;
  maxPassages?: number;
}): Promise<IngestResult> {
  return invoke("ingest_corpus", { req });
}

export async function loadPublicDomainPack(): Promise<IngestResult> {
  return invoke("load_public_domain_pack");
}

export async function deleteCanary(id: string): Promise<Snapshot> {
  return invoke("delete_canary", { id });
}

export async function fetchModels(id: string): Promise<string[]> {
  return invoke("fetch_models", { id });
}

export async function testProvider(id: string): Promise<TestResult> {
  return invoke("test_provider", { id });
}

export async function runHunt(req: {
  canaryIds: string[];
  providerIds: string[];
  strategies: string[];
}): Promise<HuntSummary> {
  return invoke("run_hunt", { req });
}

export async function scanText(text: string, source: string): Promise<ScanHit[]> {
  return invoke("scan_text", { req: { text, source } });
}

export async function webPrompts(canaryIds: string[]): Promise<WebPrompt[]> {
  return invoke("web_prompts", { canaryIds });
}

export async function exportReport(path: string): Promise<void> {
  return invoke("export_report", { path });
}
