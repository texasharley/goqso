import { invoke } from "@tauri-apps/api/core";
import type { Qso } from "@/stores/qsoStore";

// QSO Operations
export async function getQsos(limit: number, offset: number): Promise<Qso[]> {
  return invoke("get_qsos", { limit, offset });
}

export async function addQso(qso: Omit<Qso, "id" | "uuid" | "created_at">): Promise<Qso> {
  return invoke("add_qso", { qso });
}

export async function deleteQso(id: number): Promise<void> {
  return invoke("delete_qso", { id });
}

// ADIF Import/Export
export interface ImportResult {
  imported: number;
  duplicates: number;
  errors: number;
}

export async function importAdif(path: string): Promise<ImportResult> {
  return invoke("import_adif", { path });
}

export async function exportAdif(path: string, qsoIds?: number[]): Promise<number> {
  return invoke("export_adif", { path, qsoIds });
}

// LoTW Sync
export interface SyncStatus {
  pending_uploads: number;
  last_upload: string | null;
  last_download: string | null;
  is_syncing: boolean;
  lotw_configured: boolean;
}

export interface LotwDownloadResult {
  total_records: number;
  matched: number;
  unmatched: number;
  errors: string[];
  last_qsl: string | null;
}

export async function syncLotwUpload(): Promise<number> {
  return invoke("sync_lotw_upload");
}

export async function syncLotwDownload(): Promise<number> {
  return invoke("sync_lotw_download");
}

export async function getSyncStatus(): Promise<SyncStatus> {
  return invoke("get_sync_status");
}

export async function detectTqslPath(): Promise<string | null> {
  return invoke("detect_tqsl_path");
}

// Awards Progress
export interface DxccProgress {
  worked: number;
  confirmed: number;
  total: number;
}

export interface WasProgress {
  worked: number;
  confirmed: number;
  total: number;
}

export interface VuccProgress {
  worked: number;
  confirmed: number;
  target: number;
  band?: string;
}

export async function getDxccProgress(): Promise<DxccProgress> {
  return invoke("get_dxcc_progress");
}

export async function getWasProgress(): Promise<WasProgress> {
  return invoke("get_was_progress");
}

export async function getVuccProgress(band?: string): Promise<VuccProgress> {
  return invoke("get_vucc_progress", { band });
}

// Callsign Lookup
export interface CallsignInfo {
  call: string;
  dxcc?: number;
  entity_name?: string;
  cq_zone?: number;
  itu_zone?: number;
  continent?: string;
  latitude?: number;
  longitude?: number;
}

export async function lookupCallsign(call: string): Promise<CallsignInfo> {
  return invoke("lookup_callsign", { call });
}

// UDP Listener
export async function startUdpListener(port: number): Promise<void> {
  return invoke("start_udp_listener", { port });
}

export async function stopUdpListener(): Promise<void> {
  return invoke("stop_udp_listener");
}

export async function getUdpStatus(): Promise<boolean> {
  return invoke("get_udp_status");
}

// Settings
export async function getSetting(key: string): Promise<string | null> {
  return invoke("get_setting", { key });
}

export async function setSetting(key: string, value: string): Promise<void> {
  return invoke("set_setting", { key, value });
}
