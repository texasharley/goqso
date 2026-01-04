import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Radio, Wifi, WifiOff, Play, Square } from "lucide-react";

interface UdpStatus {
  running: boolean;
  port: number;
  connected: boolean;
  wsjtx_version: string | null;
}

interface WsjtxStatus {
  id: string;
  dial_freq: number;
  mode: string;
  dx_call: string;
  tx_enabled: boolean;
  transmitting: boolean;
}

export function WsjtxConnection() {
  const [udpStatus, setUdpStatus] = useState<UdpStatus | null>(null);
  const [wsjtxStatus, setWsjtxStatus] = useState<WsjtxStatus | null>(null);
  const [port, setPort] = useState(2237);
  const [error, setError] = useState<string | null>(null);
  const [lastHeartbeat, setLastHeartbeat] = useState<Date | null>(null);

  useEffect(() => {
    // Initial status check
    checkStatus();

    // Listen for WSJT-X status updates
    const unlistenStatus = listen<WsjtxStatus>("wsjtx-status", (event) => {
      setWsjtxStatus(event.payload);
    });

    // Listen for heartbeats
    const unlistenHeartbeat = listen("wsjtx-heartbeat", () => {
      setLastHeartbeat(new Date());
    });

    // Listen for connection events
    const unlistenConnected = listen("udp-connected", () => {
      setError(null);
      checkStatus();
    });

    const unlistenDisconnected = listen("udp-disconnected", () => {
      setWsjtxStatus(null);
      checkStatus();
    });

    const unlistenError = listen<string>("udp-error", (event) => {
      setError(event.payload);
    });

    // Poll for status every 5 seconds
    const interval = setInterval(checkStatus, 5000);

    return () => {
      unlistenStatus.then((f) => f());
      unlistenHeartbeat.then((f) => f());
      unlistenConnected.then((f) => f());
      unlistenDisconnected.then((f) => f());
      unlistenError.then((f) => f());
      clearInterval(interval);
    };
  }, []);

  const checkStatus = async () => {
    try {
      const status = await invoke<UdpStatus>("get_udp_status");
      setUdpStatus(status);
    } catch (e) {
      console.error("Failed to get UDP status:", e);
    }
  };

  const startListener = async () => {
    try {
      setError(null);
      await invoke("start_udp_listener", { port });
      await checkStatus();
    } catch (e) {
      setError(String(e));
    }
  };

  const stopListener = async () => {
    try {
      await invoke("stop_udp_listener");
      setWsjtxStatus(null);
      await checkStatus();
    } catch (e) {
      setError(String(e));
    }
  };

  const formatFrequency = (hz: number): string => {
    const mhz = hz / 1_000_000;
    return `${mhz.toFixed(6)} MHz`;
  };

  const isConnected = udpStatus?.running && wsjtxStatus !== null;
  const hasRecentHeartbeat = lastHeartbeat && 
    (new Date().getTime() - lastHeartbeat.getTime()) < 30000;

  return (
    <section className="bg-card rounded-lg border border-border p-4">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <Radio className="h-5 w-5 text-primary" />
          <h3 className="font-semibold">WSJT-X Connection</h3>
        </div>
        <div className="flex items-center gap-2">
          {isConnected || hasRecentHeartbeat ? (
            <span className="flex items-center gap-1 text-green-500 text-sm">
              <Wifi className="h-4 w-4" />
              Connected
            </span>
          ) : udpStatus?.running ? (
            <span className="flex items-center gap-1 text-yellow-500 text-sm">
              <Wifi className="h-4 w-4" />
              Listening...
            </span>
          ) : (
            <span className="flex items-center gap-1 text-muted-foreground text-sm">
              <WifiOff className="h-4 w-4" />
              Disconnected
            </span>
          )}
        </div>
      </div>

      {error && (
        <div className="mb-4 p-2 bg-destructive/10 text-destructive rounded text-sm">
          {error}
        </div>
      )}

      {/* Controls */}
      <div className="flex items-center gap-3 mb-4">
        <div className="flex items-center gap-2">
          <label htmlFor="port" className="text-sm text-muted-foreground">
            UDP Port:
          </label>
          <input
            id="port"
            type="number"
            value={port}
            onChange={(e) => setPort(parseInt(e.target.value) || 2237)}
            disabled={udpStatus?.running}
            className="w-20 px-2 py-1 text-sm bg-background border border-border rounded"
          />
        </div>
        {udpStatus?.running ? (
          <button
            onClick={stopListener}
            className="flex items-center gap-1 px-3 py-1.5 text-sm bg-destructive text-destructive-foreground rounded hover:bg-destructive/90"
          >
            <Square className="h-3 w-3" />
            Stop
          </button>
        ) : (
          <button
            onClick={startListener}
            className="flex items-center gap-1 px-3 py-1.5 text-sm bg-primary text-primary-foreground rounded hover:bg-primary/90"
          >
            <Play className="h-3 w-3" />
            Start
          </button>
        )}
      </div>

      {/* WSJT-X Status */}
      {wsjtxStatus && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm border-t border-border pt-4">
          <div>
            <span className="text-muted-foreground">Frequency: </span>
            <span className="font-mono">{formatFrequency(wsjtxStatus.dial_freq)}</span>
          </div>
          <div>
            <span className="text-muted-foreground">Mode: </span>
            <span className="font-mono">{wsjtxStatus.mode}</span>
          </div>
          <div>
            <span className="text-muted-foreground">DX Call: </span>
            <span className="font-mono">{wsjtxStatus.dx_call || "—"}</span>
          </div>
          <div>
            <span className="text-muted-foreground">TX: </span>
            <span className={wsjtxStatus.transmitting ? "text-red-500 font-bold" : ""}>
              {wsjtxStatus.transmitting ? "TRANSMITTING" : wsjtxStatus.tx_enabled ? "Enabled" : "Off"}
            </span>
          </div>
        </div>
      )}

      {!wsjtxStatus && udpStatus?.running && (
        <p className="text-sm text-muted-foreground border-t border-border pt-4">
          Waiting for WSJT-X to connect on port {port}...
          <br />
          <span className="text-xs">
            Ensure WSJT-X is configured to send UDP reports to this port.
          </span>
        </p>
      )}
    </section>
  );
}
