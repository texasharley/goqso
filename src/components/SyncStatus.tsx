import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RefreshCw, Cloud } from "lucide-react";

interface SyncStatus {
  pending_uploads: number;
  last_upload: string | null;
  last_download: string | null;
  is_syncing: boolean;
}

export function SyncStatus() {
  const [status, setStatus] = useState<SyncStatus>({
    pending_uploads: 0,
    last_upload: null,
    last_download: null,
    is_syncing: false,
  });

  useEffect(() => {
    // Poll sync status
    const fetchStatus = async () => {
      try {
        const result = await invoke<SyncStatus>("get_sync_status");
        setStatus(result);
      } catch (error) {
        console.error("Failed to get sync status:", error);
      }
    };

    fetchStatus();
    const interval = setInterval(fetchStatus, 10000);
    return () => clearInterval(interval);
  }, []);

  const handleSync = async () => {
    try {
      await invoke("sync_lotw_upload");
      await invoke("sync_lotw_download");
    } catch (error) {
      console.error("Sync failed:", error);
    }
  };

  if (status.pending_uploads === 0 && !status.is_syncing) {
    return null;
  }

  return (
    <button
      onClick={handleSync}
      disabled={status.is_syncing}
      className="flex items-center gap-2 px-3 py-1.5 text-sm bg-secondary hover:bg-accent rounded-lg transition-colors disabled:opacity-50"
    >
      {status.is_syncing ? (
        <>
          <RefreshCw className="h-4 w-4 animate-spin" />
          <span>Syncing...</span>
        </>
      ) : (
        <>
          <Cloud className="h-4 w-4" />
          <span>{status.pending_uploads} pending</span>
        </>
      )}
    </button>
  );
}
