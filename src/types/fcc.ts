// FCC Database types

export interface FccSyncStatus {
  last_sync_at: string | null;
  record_count: number;
  file_date: string | null;
  sync_in_progress: boolean;
  error_message: string | null;
}

export interface FccLicenseInfo {
  call: string;
  name: string | null;
  state: string | null;
  city: string | null;
  grid: string | null;
}
