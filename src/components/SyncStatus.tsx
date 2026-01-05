import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RefreshCw, Cloud } from "lucide-react";

interface SyncStatus {
  pending_uploads: number;
  last_upload: string | null;
  last_download: string | null;
  is_syncing: boolean;
  lotw_configured: boolean;
}

export function SyncStatus() {
  const [status, setStatus] = useState<SyncStatus>({
    pending_uploads: 0,
    last_upload: null,
    last_download: null,
    is_syncing: false,
    lotw_configured: false,
  });
  const [_isReady, setIsReady] = useState(false);

  useEffect(() => {
    // Only fetch once DB is ready, then poll less frequently
    const fetchStatus = async () => {
      try {
        const result = await invoke<SyncStatus>("get_sync_status");
        setStatus(result);
        setIsReady(true);
      } catch {
        // DB not ready yet, will retry
      }
    };

    // Initial fetch after short delay (let DB initialize)
    const initialTimeout = setTimeout(fetchStatus, 1000);
    
    // Poll every 60s (not 10s) - sync status doesn't change often
    const interval = setInterval(fetchStatus, 60000);
    
    return () => {
      clearTimeout(initialTimeout);
      clearInterval(interval);
    };
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
