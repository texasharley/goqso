import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { 
  X, Cloud, Download, Upload, RefreshCw, CheckCircle2, AlertCircle, 
  Loader2, Key, Shield, FolderSearch, HelpCircle, ChevronUp,
  Link, PartyPopper, Clock
} from "lucide-react";

interface LotwSyncProps {
  onClose: () => void;
  onSyncComplete: (matched: number) => void;
}

interface UnmatchedQso {
  call: string;
  qso_date: string;
  time_on: string;
  band: string;
  mode: string;
}

interface LotwDownloadResult {
  total_records: number;
  matched: number;
  unmatched: number;
  unmatched_qsos: UnmatchedQso[];
  errors: string[];
  last_qsl: string | null;
}

interface LotwUploadResult {
  qsos_exported: number;
  success: boolean;
  message: string;
}

interface TqslInfo {
  path: string | null;
  stationCallsigns: string[];
  lastUploadDate: string | null;
}

// Helper component for tooltips
function Tooltip({ text, children }: { text: string; children: React.ReactNode }) {
  return (
    <span className="relative group">
      {children}
      <span className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 text-xs bg-zinc-700 text-zinc-200 rounded shadow-lg whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-10">
        {text}
      </span>
    </span>
  );
}

export function LotwSync({ onClose, onSyncComplete }: LotwSyncProps) {
  // Credentials
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [saveCredentials, setSaveCredentials] = useState(true);
  
  // TQSL info
  const [tqslInfo, setTqslInfo] = useState<TqslInfo>({
    path: null,
    stationCallsigns: [],
    lastUploadDate: null,
  });
  
  // Sync state
  const [isSyncing, setIsSyncing] = useState(false);
  const [syncResult, setSyncResult] = useState<LotwDownloadResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  
  // Upload state
  const [isUploading, setIsUploading] = useState(false);
  const [uploadResult, setUploadResult] = useState<LotwUploadResult | null>(null);
  const [pendingUploads, setPendingUploads] = useState<number>(0);
  const [totalQsos, setTotalQsos] = useState<number>(0);
  const [qslsReceived, setQslsReceived] = useState<number>(0);
  
  // Options
  const [sinceDate, setSinceDate] = useState<string>(""); // Full datetime string for API
  const [lastSyncDisplay, setLastSyncDisplay] = useState<string>(""); // Formatted for display
  const [syncMode, setSyncMode] = useState<"all" | "new">("all"); // Default to all for first sync
  const [isFirstSync, setIsFirstSync] = useState(true);
  
  // Help/onboarding
  const [showHelp, setShowHelp] = useState(false);

  // Load saved credentials and detect TQSL on mount
  useEffect(() => {
    const loadSettings = async () => {
      try {
        // Try to load saved credentials
        const savedUsername = await invoke<string | null>("get_setting", { key: "lotw_username" });
        const savedPassword = await invoke<string | null>("get_setting", { key: "lotw_password" });
        const lastSync = await invoke<string | null>("get_setting", { key: "lotw_last_download" });
        
        if (savedUsername) setUsername(savedUsername);
        if (savedPassword) setPassword(savedPassword);
        if (lastSync) {
          // Keep the FULL datetime for API - LoTW accepts "YYYY-MM-DD HH:MM:SS" format
          // This ensures we only get NEW confirmations since the exact last sync time
          setSinceDate(lastSync);
          // Format nicely for display
          try {
            const dt = new Date(lastSync.replace(' ', 'T'));
            setLastSyncDisplay(dt.toLocaleString());
          } catch {
            setLastSyncDisplay(lastSync);
          }
          setSyncMode("new"); // Switch to incremental mode if we have previous sync
          setIsFirstSync(false);
        }
        
        // Detect TQSL
        const tqslPath = await invoke<string | null>("detect_tqsl_path");
        if (tqslPath) {
          setTqslInfo(prev => ({ ...prev, path: tqslPath }));
          
          // Try to read station callsigns from TQSL
          try {
            const callsigns = await invoke<string[]>("get_tqsl_callsigns");
            setTqslInfo(prev => ({ ...prev, stationCallsigns: callsigns }));
          } catch {
            // get_tqsl_callsigns may not exist yet
          }
        }
        
        // Get sync status with counts
        try {
          const syncStatus = await invoke<{ pending_uploads: number; total_qsos: number; qsls_received: number }>("get_sync_status");
          setPendingUploads(syncStatus.pending_uploads);
          setTotalQsos(syncStatus.total_qsos);
          setQslsReceived(syncStatus.qsls_received);
        } catch {
          // Ignore sync status errors
        }
      } catch (err) {
        console.error("Failed to load settings:", err);
      }
    };
    
    loadSettings();
  }, []);

  const handleSync = useCallback(async () => {
    if (!username || !password) {
      setError("Please enter your LoTW username and password");
      return;
    }
    
    setIsSyncing(true);
    setError(null);
    setSyncResult(null);
    
    try {
      // Save credentials if requested
      if (saveCredentials) {
        await invoke("set_setting", { key: "lotw_username", value: username });
        await invoke("set_setting", { key: "lotw_password", value: password });
      }
      
      // Determine since date
      const effectiveSinceDate = syncMode === "new" && sinceDate ? sinceDate : null;
      
      // Call the sync command
      const result = await invoke<LotwDownloadResult>("sync_lotw_download", {
        username,
        password,
        sinceDate: effectiveSinceDate,
      });
      
      setSyncResult(result);
      
      // Save the last sync date and update state for next sync
      // IMPORTANT: Add 1 second to the timestamp because LoTW's qso_qslsince is INCLUSIVE
      // (returns records "on or after" the date). Without this, the same record keeps returning.
      if (result.last_qsl) {
        // Parse the datetime, add 1 second, format back to LoTW format
        let nextSinceDate = result.last_qsl;
        try {
          const dt = new Date(result.last_qsl.replace(' ', 'T'));
          dt.setSeconds(dt.getSeconds() + 1);
          // Format back to "YYYY-MM-DD HH:MM:SS" format that LoTW expects
          const year = dt.getFullYear();
          const month = String(dt.getMonth() + 1).padStart(2, '0');
          const day = String(dt.getDate()).padStart(2, '0');
          const hours = String(dt.getHours()).padStart(2, '0');
          const minutes = String(dt.getMinutes()).padStart(2, '0');
          const seconds = String(dt.getSeconds()).padStart(2, '0');
          nextSinceDate = `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
          setLastSyncDisplay(dt.toLocaleString());
        } catch {
          setLastSyncDisplay(result.last_qsl);
        }
        
        await invoke("set_setting", { 
          key: "lotw_last_download", 
          value: nextSinceDate 
        });
        // Update local state so next "Sync Again" uses new date
        setSinceDate(nextSinceDate);
        setIsFirstSync(false);
      }
      
      if (result.matched > 0) {
        onSyncComplete(result.matched);
      }
    } catch (err) {
      const errorStr = String(err);
      if (errorStr.includes("password") || errorStr.includes("login") || errorStr.includes("Auth")) {
        setError("Invalid username or password. Check your LoTW credentials.");
      } else if (errorStr.includes("Network") || errorStr.includes("timeout")) {
        setError("Network error. Please check your internet connection.");
      } else {
        setError(`Sync failed: ${err}`);
      }
    } finally {
      setIsSyncing(false);
    }
  }, [username, password, saveCredentials, syncMode, sinceDate, onSyncComplete]);

  const handleUpload = useCallback(async () => {
    if (!tqslInfo.path) {
      setError("TQSL is required to upload to LoTW. Please install TQSL first.");
      return;
    }
    
    setIsUploading(true);
    setError(null);
    setUploadResult(null);
    
    try {
      const result = await invoke<LotwUploadResult>("upload_to_lotw", {
        tqslPath: tqslInfo.path,
      });
      
      setUploadResult(result);
      
      // Refresh pending count after upload
      try {
        const syncStatus = await invoke<{ pending_uploads: number; total_qsos: number; qsls_received: number }>("get_sync_status");
        setPendingUploads(syncStatus.pending_uploads);
        setTotalQsos(syncStatus.total_qsos);
        setQslsReceived(syncStatus.qsls_received);
      } catch {
        // Ignore
      }
      
      if (!result.success) {
        setError(result.message);
      }
    } catch (err) {
      setError(`Upload failed: ${err}`);
    } finally {
      setIsUploading(false);
    }
  }, [tqslInfo.path]);

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-zinc-900 rounded-lg border border-zinc-700 w-full max-w-md max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-zinc-700">
          <div className="flex items-center gap-2">
            <Cloud className="h-5 w-5 text-sky-500" />
            <h2 className="text-lg font-semibold">LoTW Confirmations</h2>
            <Tooltip text="What is LoTW?">
              <button
                onClick={() => setShowHelp(!showHelp)}
                className="p-1 hover:bg-zinc-800 rounded transition-colors"
              >
                <HelpCircle className="h-4 w-4 text-zinc-500 hover:text-zinc-300" />
              </button>
            </Tooltip>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-zinc-800 rounded-lg transition-colors"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        {/* Help Section - Collapsible */}
        {showHelp && (
          <div className="mx-4 mt-4 p-3 bg-sky-900/30 border border-sky-800/50 rounded-lg text-sm space-y-2">
            <div className="flex items-start gap-2">
              <Link className="h-4 w-4 text-sky-400 mt-0.5 flex-shrink-0" />
              <div>
                <span className="font-medium text-sky-300">Logbook of The World (LoTW)</span> is ARRL's 
                free online service that confirms contacts. When both stations upload their logs, the QSO is 
                <span className="text-green-400"> automatically verified</span> — no paper QSL cards needed!
              </div>
            </div>
            <div className="text-zinc-400 text-xs pl-6">
              These confirmations count toward DXCC, WAS, and other awards. 
              <a 
                href="https://lotw.arrl.org/" 
                target="_blank" 
                rel="noopener noreferrer"
                className="text-sky-400 hover:underline ml-1"
              >
                Create a free account →
              </a>
            </div>
            <button
              onClick={() => setShowHelp(false)}
              className="text-xs text-zinc-500 hover:text-zinc-400 flex items-center gap-1 ml-6"
            >
              <ChevronUp className="h-3 w-3" /> Hide
            </button>
          </div>
        )}

        {/* Content */}
        <div className="p-4 space-y-4">
          {/* QSO/QSL Summary */}
          <div className="bg-zinc-800 rounded-lg p-4">
            <div className="grid grid-cols-3 gap-4 text-center">
              <div>
                <div className="text-2xl font-bold text-zinc-100">{totalQsos}</div>
                <div className="text-xs text-zinc-400">Total QSOs</div>
              </div>
              <div>
                <div className="text-2xl font-bold text-green-400">{qslsReceived}</div>
                <div className="text-xs text-zinc-400">QSLs Received</div>
              </div>
              <div>
                <div className={`text-2xl font-bold ${pendingUploads > 0 ? 'text-amber-400' : 'text-zinc-500'}`}>
                  {pendingUploads}
                </div>
                <div className="text-xs text-zinc-400">Pending Upload</div>
              </div>
            </div>
          </div>

          {/* TQSL Status + Upload */}
          <div className="bg-zinc-800 rounded-lg p-3 space-y-3">
            <div className="flex items-center gap-3">
              <Shield className={`h-5 w-5 ${tqslInfo.path ? "text-green-500" : "text-zinc-500"}`} />
              <div className="flex-1">
                <div className="text-sm font-medium flex items-center gap-2">
                  {tqslInfo.path ? "TQSL Detected" : "TQSL Not Found"}
                  <Tooltip text="TQSL is the software used to sign and upload your log to LoTW. You don't need it to download confirmations.">
                    <HelpCircle className="h-3 w-3 text-zinc-500" />
                  </Tooltip>
                </div>
                {tqslInfo.path && (
                  <div className="text-xs text-zinc-500 truncate">
                    {tqslInfo.path}
                  </div>
                )}
              </div>
              {!tqslInfo.path && (
                <button className="text-xs text-sky-400 hover:text-sky-300">
                  <FolderSearch className="h-4 w-4" />
                </button>
              )}
            </div>
            
            {/* Upload Button */}
            {tqslInfo.path && (
              <button
                onClick={handleUpload}
                disabled={isUploading || isSyncing}
                className="w-full px-4 py-2.5 text-sm bg-emerald-600 hover:bg-emerald-500 disabled:bg-zinc-700 disabled:text-zinc-500 rounded-lg transition-colors flex items-center justify-center gap-2"
              >
                {isUploading ? (
                  <>
                    <Loader2 className="h-4 w-4 animate-spin" />
                    Uploading to LoTW...
                  </>
                ) : (
                  <>
                    <Upload className="h-4 w-4" />
                    Upload Pending QSOs to LoTW
                  </>
                )}
              </button>
            )}
            
            {/* Upload Result */}
            {uploadResult && (
              <div className={`p-3 rounded-lg border ${
                uploadResult.success 
                  ? "bg-green-900/30 border-green-700" 
                  : "bg-red-900/30 border-red-700"
              }`}>
                <div className="flex items-center gap-2 text-sm">
                  {uploadResult.success ? (
                    <CheckCircle2 className="h-4 w-4 text-green-500" />
                  ) : (
                    <AlertCircle className="h-4 w-4 text-red-500" />
                  )}
                  <span className={uploadResult.success ? "text-green-300" : "text-red-300"}>
                    {uploadResult.message}
                  </span>
                </div>
                {uploadResult.qsos_exported > 0 && (
                  <div className="text-xs text-zinc-400 mt-1 ml-6">
                    {uploadResult.qsos_exported} QSO(s) processed
                  </div>
                )}
              </div>
            )}
          </div>

          {/* Credentials */}
          <div className="space-y-3">
            <div className="flex items-center gap-2 text-sm text-zinc-400">
              <Key className="h-4 w-4" />
              <span>LoTW Credentials</span>
            </div>
            <div>
              <input
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value.toUpperCase())}
                placeholder="Callsign (username)"
                className="w-full px-3 py-2 bg-zinc-800 rounded border border-zinc-700 focus:border-sky-500 focus:outline-none"
                disabled={isSyncing}
              />
            </div>
            <div>
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Password"
                className="w-full px-3 py-2 bg-zinc-800 rounded border border-zinc-700 focus:border-sky-500 focus:outline-none"
                disabled={isSyncing}
              />
            </div>
            <label className="flex items-center gap-2 text-sm text-zinc-400">
              <input
                type="checkbox"
                checked={saveCredentials}
                onChange={(e) => setSaveCredentials(e.target.checked)}
                className="rounded"
                disabled={isSyncing}
              />
              Save credentials
            </label>
          </div>

          {/* Sync Options */}
          <div className="space-y-3">
            <div className="flex items-center gap-2 text-sm text-zinc-400">
              <Clock className="h-4 w-4" />
              <span>Time Range</span>
              <Tooltip text="'New Only' downloads confirmations since your last sync. 'All Time' downloads everything.">
                <HelpCircle className="h-3 w-3" />
              </Tooltip>
            </div>
            <div className="flex gap-2">
              <button
                onClick={() => setSyncMode("new")}
                disabled={isSyncing || isFirstSync}
                className={`flex-1 py-2 text-sm rounded-lg border transition-colors ${
                  syncMode === "new"
                    ? "border-sky-500 bg-sky-500/20 text-sky-400"
                    : "border-zinc-700 hover:bg-zinc-800"
                } ${isFirstSync ? "opacity-50 cursor-not-allowed" : ""}`}
              >
                <div className="font-medium">New Only</div>
                <div className="text-xs opacity-60">Since last sync</div>
              </button>
              <button
                onClick={() => setSyncMode("all")}
                disabled={isSyncing}
                className={`flex-1 py-2 text-sm rounded-lg border transition-colors ${
                  syncMode === "all"
                    ? "border-sky-500 bg-sky-500/20 text-sky-400"
                    : "border-zinc-700 hover:bg-zinc-800"
                }`}
              >
                <div className="font-medium">All Time</div>
                <div className="text-xs opacity-60">Every confirmation</div>
              </button>
            </div>
            {isFirstSync && (
              <div className="text-xs text-sky-400/80 flex items-center gap-1">
                <CheckCircle2 className="h-3 w-3" />
                First sync — downloading all your confirmations
              </div>
            )}
            {!isFirstSync && syncMode === "new" && sinceDate && (
              <div className="text-xs text-zinc-500 flex items-center gap-1">
                <Clock className="h-3 w-3" />
                Looking for confirmations since {lastSyncDisplay || sinceDate}
              </div>
            )}
            {!isFirstSync && syncMode === "all" && (
              <div className="text-xs text-zinc-500 flex items-center gap-1">
                <RefreshCw className="h-3 w-3" />
                Re-downloading all confirmations (slower, but complete)
              </div>
            )}
          </div>

          {/* Error */}
          {error && (
            <div className="bg-red-900/30 border border-red-700 rounded-lg p-3 flex items-start gap-2">
              <AlertCircle className="h-5 w-5 text-red-500 flex-shrink-0 mt-0.5" />
              <p className="text-sm text-red-300">{error}</p>
            </div>
          )}

          {/* Result */}
          {syncResult && (
            <div className="bg-zinc-800 rounded-lg p-4 space-y-3">
              {/* Celebratory header for matches */}
              {syncResult.matched > 0 ? (
                <div className="flex items-center gap-2 text-green-500">
                  <PartyPopper className="h-5 w-5" />
                  <span className="font-medium">
                    {syncResult.matched} Contact{syncResult.matched !== 1 ? 's' : ''} Confirmed!
                  </span>
                </div>
              ) : (
                <div className="flex items-center gap-2 text-zinc-400">
                  <CheckCircle2 className="h-5 w-5" />
                  <span className="font-medium">Sync Complete</span>
                </div>
              )}
              
              {/* Stats with explanations */}
              <div className="space-y-2 text-sm">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2 text-zinc-400">
                    <Download className="h-4 w-4" />
                    <span>Downloaded from LoTW</span>
                    <Tooltip text="Total QSL confirmations LoTW sent us based on your time range selection">
                      <HelpCircle className="h-3 w-3 text-zinc-500" />
                    </Tooltip>
                  </div>
                  <span>{syncResult.total_records}</span>
                </div>
                
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2 text-green-400">
                    <CheckCircle2 className="h-4 w-4" />
                    <span>Found in your log</span>
                    <Tooltip text="These QSOs are now marked as confirmed in your log">
                      <HelpCircle className="h-3 w-3 text-zinc-500" />
                    </Tooltip>
                  </div>
                  <span className="text-green-400 font-medium">{syncResult.matched}</span>
                </div>
                
                {syncResult.unmatched > 0 && (
                  <div className="space-y-1">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2 text-yellow-500/80">
                        <AlertCircle className="h-4 w-4" />
                        <span>Not in your log</span>
                        <Tooltip text="The other station confirmed these QSOs, but they're not in GoQSO. They may be from a different logging program.">
                          <HelpCircle className="h-3 w-3 text-zinc-500" />
                        </Tooltip>
                      </div>
                      <span className="text-yellow-500/80">{syncResult.unmatched}</span>
                    </div>
                    {/* Show unmatched QSO details */}
                    {syncResult.unmatched_qsos && syncResult.unmatched_qsos.length > 0 && (
                      <div className="ml-6 text-xs text-yellow-600/70 space-y-0.5 max-h-20 overflow-y-auto">
                        {syncResult.unmatched_qsos.map((qso, i) => (
                          <div key={i} className="font-mono">
                            {qso.call} • {qso.qso_date} {qso.time_on} • {qso.band} {qso.mode}
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                )}
              </div>
              
              {/* Diagnostic info when numbers don't add up */}
              {syncResult.total_records > 0 && syncResult.matched === 0 && (
                <div className="mt-2 p-2 bg-yellow-900/20 border border-yellow-800/50 rounded text-xs text-yellow-400">
                  <strong>No matches found.</strong> This usually means your QSOs haven't been imported into GoQSO yet.
                  Try importing your ADIF file first, then sync again.
                </div>
              )}
              
              {syncResult.errors.length > 0 && (
                <div className="mt-2 text-xs text-red-400 max-h-16 overflow-y-auto">
                  {syncResult.errors.slice(0, 3).map((msg, i) => (
                    <div key={i}>• {msg}</div>
                  ))}
                </div>
              )}
              
              {/* Next steps hint */}
              {syncResult.matched > 0 && (
                <div className="text-xs text-zinc-500 pt-2 border-t border-zinc-700">
                  ✨ Your confirmed contacts now show a green "L" badge in the log
                </div>
              )}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="p-4 border-t border-zinc-700 flex justify-end gap-2">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm hover:bg-zinc-800 rounded-lg transition-colors"
          >
            {syncResult ? "Close" : "Cancel"}
          </button>
          {!syncResult && (
            <button
              onClick={handleSync}
              disabled={isSyncing || !username || !password}
              className="px-4 py-2 text-sm bg-sky-600 hover:bg-sky-500 disabled:bg-zinc-700 disabled:text-zinc-500 rounded-lg transition-colors flex items-center gap-2"
            >
              {isSyncing ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Downloading...
                </>
              ) : (
                <>
                  <Download className="h-4 w-4" />
                  Download Confirmations
                </>
              )}
            </button>
          )}
          {syncResult && (
            <button
              onClick={() => {
                setSyncResult(null);
                setError(null);
              }}
              className="px-4 py-2 text-sm bg-sky-600 hover:bg-sky-500 rounded-lg transition-colors flex items-center gap-2"
            >
              <RefreshCw className="h-4 w-4" />
              Sync Again
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
