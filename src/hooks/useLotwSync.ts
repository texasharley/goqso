import { useState, useCallback } from "react";
import * as api from "@/lib/tauri";
import type { SyncStatus } from "@/types/lotw";

export function useLotwSync() {
  const [status, setStatus] = useState<SyncStatus>({
    pending_uploads: 0,
    last_upload: null,
    last_download: null,
    is_syncing: false,
  });
  const [error, setError] = useState<string | null>(null);

  const fetchStatus = useCallback(async () => {
    try {
      const result = await api.getSyncStatus();
      setStatus(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to get sync status");
    }
  }, []);

  const upload = useCallback(async () => {
    setStatus((s) => ({ ...s, is_syncing: true }));
    setError(null);
    
    try {
      const count = await api.syncLotwUpload();
      await fetchStatus();
      return count;
    } catch (err) {
      setError(err instanceof Error ? err.message : "Upload failed");
      throw err;
    } finally {
      setStatus((s) => ({ ...s, is_syncing: false }));
    }
  }, [fetchStatus]);

  const download = useCallback(async () => {
    setStatus((s) => ({ ...s, is_syncing: true }));
    setError(null);
    
    try {
      const count = await api.syncLotwDownload();
      await fetchStatus();
      return count;
    } catch (err) {
      setError(err instanceof Error ? err.message : "Download failed");
      throw err;
    } finally {
      setStatus((s) => ({ ...s, is_syncing: false }));
    }
  }, [fetchStatus]);

  const sync = useCallback(async () => {
    await upload();
    await download();
  }, [upload, download]);

  return {
    status,
    error,
    fetchStatus,
    upload,
    download,
    sync,
  };
}
