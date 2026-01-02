import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { X, Folder } from "lucide-react";

interface SettingsPanelProps {
  onClose: () => void;
}

export function SettingsPanel({ onClose }: SettingsPanelProps) {
  const [udpPort, setUdpPort] = useState("2237");
  const [udpStatus, _setUdpStatus] = useState<"listening" | "stopped">("stopped");
  const [lotwUsername, setLotwUsername] = useState("");
  const [lotwPassword, setLotwPassword] = useState("");
  const [tqslPath, setTqslPath] = useState("");
  const [tqslAutoDetected, setTqslAutoDetected] = useState(false);
  const [myCallsign, setMyCallsign] = useState("");
  const [myGrid, setMyGrid] = useState("");
  const [autoSync, setAutoSync] = useState(true);

  useEffect(() => {
    // Auto-detect TQSL path
    invoke<string | null>("detect_tqsl_path").then((path) => {
      if (path) {
        setTqslPath(path);
        setTqslAutoDetected(true);
      }
    });
  }, []);

  const handleSave = async () => {
    // TODO: Save settings to database
    console.log("Saving settings...");
    onClose();
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-card rounded-lg border border-border w-full max-w-lg max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-border">
          <h2 className="text-lg font-semibold">Settings</h2>
          <button
            onClick={onClose}
            className="p-2 hover:bg-accent rounded-lg transition-colors"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        {/* Content */}
        <div className="p-4 space-y-6">
          {/* WSJT-X Integration */}
          <section>
            <h3 className="font-medium mb-3">WSJT-X Integration</h3>
            <div className="bg-secondary rounded-lg p-4 space-y-3">
              <div>
                <label className="text-sm text-muted-foreground">UDP Port</label>
                <input
                  type="text"
                  value={udpPort}
                  onChange={(e) => setUdpPort(e.target.value)}
                  className="w-full mt-1 px-3 py-2 bg-background rounded border border-border"
                />
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm">
                  Status:{" "}
                  <span className={udpStatus === "listening" ? "text-green-500" : "text-muted-foreground"}>
                    {udpStatus === "listening" ? "● Listening" : "○ Stopped"}
                  </span>
                </span>
                <button className="px-3 py-1 text-sm bg-accent hover:bg-accent/80 rounded">
                  {udpStatus === "listening" ? "Stop" : "Start"}
                </button>
              </div>
            </div>
          </section>

          {/* LoTW Sync */}
          <section>
            <h3 className="font-medium mb-3">LoTW Sync</h3>
            <div className="bg-secondary rounded-lg p-4 space-y-3">
              <div>
                <label className="text-sm text-muted-foreground">Username</label>
                <input
                  type="text"
                  value={lotwUsername}
                  onChange={(e) => setLotwUsername(e.target.value)}
                  className="w-full mt-1 px-3 py-2 bg-background rounded border border-border"
                  placeholder="Your callsign"
                />
              </div>
              <div>
                <label className="text-sm text-muted-foreground">Password</label>
                <input
                  type="password"
                  value={lotwPassword}
                  onChange={(e) => setLotwPassword(e.target.value)}
                  className="w-full mt-1 px-3 py-2 bg-background rounded border border-border"
                />
              </div>
              <div>
                <label className="text-sm text-muted-foreground">TQSL Path</label>
                <div className="flex gap-2 mt-1">
                  <input
                    type="text"
                    value={tqslPath}
                    onChange={(e) => {
                      setTqslPath(e.target.value);
                      setTqslAutoDetected(false);
                    }}
                    className="flex-1 px-3 py-2 bg-background rounded border border-border"
                  />
                  <button className="p-2 bg-accent hover:bg-accent/80 rounded">
                    <Folder className="h-4 w-4" />
                  </button>
                </div>
                {tqslAutoDetected && (
                  <p className="text-xs text-green-500 mt-1">✓ Auto-detected</p>
                )}
              </div>
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="autoSync"
                  checked={autoSync}
                  onChange={(e) => setAutoSync(e.target.checked)}
                  className="rounded"
                />
                <label htmlFor="autoSync" className="text-sm">
                  Auto-sync when online
                </label>
              </div>
            </div>
          </section>

          {/* Station Info */}
          <section>
            <h3 className="font-medium mb-3">Station Info</h3>
            <div className="bg-secondary rounded-lg p-4 space-y-3">
              <div>
                <label className="text-sm text-muted-foreground">My Callsign</label>
                <input
                  type="text"
                  value={myCallsign}
                  onChange={(e) => setMyCallsign(e.target.value.toUpperCase())}
                  className="w-full mt-1 px-3 py-2 bg-background rounded border border-border"
                  placeholder="N0CALL"
                />
              </div>
              <div>
                <label className="text-sm text-muted-foreground">My Grid</label>
                <input
                  type="text"
                  value={myGrid}
                  onChange={(e) => setMyGrid(e.target.value.toUpperCase())}
                  className="w-full mt-1 px-3 py-2 bg-background rounded border border-border"
                  placeholder="EM48"
                />
              </div>
            </div>
          </section>
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-2 p-4 border-t border-border">
          <button
            onClick={onClose}
            className="px-4 py-2 bg-secondary hover:bg-accent rounded-lg transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            className="px-4 py-2 bg-primary text-primary-foreground hover:opacity-90 rounded-lg transition-colors"
          >
            Save
          </button>
        </div>
      </div>
    </div>
  );
}
