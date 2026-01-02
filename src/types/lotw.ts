// TypeScript types for LoTW integration

export interface SyncStatus {
  pending_uploads: number;
  last_upload: string | null;
  last_download: string | null;
  is_syncing: boolean;
}

export interface SyncQueueEntry {
  id: number;
  qso_id: number;
  action: string;
  status: "pending" | "in_progress" | "completed" | "failed";
  attempts: number;
  last_error?: string;
  created_at: string;
  updated_at: string;
}

export interface LotwConfirmation {
  call: string;
  qso_date: string;
  time_on: string;
  band: string;
  mode: string;
  mode_group: string;
  dxcc?: number;
  state?: string;
  gridsquare?: string;
  credit_granted?: string;
}

export interface ImportResult {
  imported: number;
  duplicates: number;
  errors: number;
}
