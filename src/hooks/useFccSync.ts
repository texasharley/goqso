import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { FccSyncStatus, FccLicenseInfo } from "@/types/fcc";

// Re-export types for convenience
export type { FccSyncStatus, FccLicenseInfo } from "@/types/fcc";

export function useFccSync() {
  const [status, setStatus] = useState<FccSyncStatus>({
    last_sync_at: null,
    record_count: 0,
    file_date: null,
    sync_in_progress: false,
    error_message: null,
  });
  const [progress, setProgress] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const fetchStatus = useCallback(async () => {
    try {
      const result = await invoke<FccSyncStatus>("get_fcc_sync_status");
      setStatus(result);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, []);

  // Listen for progress events
  useEffect(() => {
    const unlisten = listen<string>("fcc-sync-progress", (event) => {
      setProgress(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const sync = useCallback(async () => {
    setError(null);
    setProgress("Starting FCC database sync...");

    try {
      const result = await invoke<FccSyncStatus>("sync_fcc_database");
      setStatus(result);
      setProgress(null);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      setProgress(null);
      throw err;
    }
  }, []);

  return {
    status,
    progress,
    error,
    fetchStatus,
    sync,
    isReady: status.record_count > 0,
    isSyncing: status.sync_in_progress,
  };
}

/**
 * Lookup a single callsign in the FCC database
 */
export async function lookupCallsign(callsign: string): Promise<FccLicenseInfo | null> {
  return invoke<FccLicenseInfo | null>("lookup_fcc_callsign", { callsign });
}

/**
 * Lookup multiple callsigns in the FCC database (batch)
 */
export async function lookupCallsigns(callsigns: string[]): Promise<FccLicenseInfo[]> {
  return invoke<FccLicenseInfo[]>("lookup_fcc_callsigns", { callsigns });
}
