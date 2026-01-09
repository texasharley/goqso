import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Radio, Wifi, WifiOff, Play, Square, Settings } from "lucide-react";
import { PeriodTimer } from "./PeriodTimer";

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

// SDR-style frequency display
function FrequencyDisplay({ hz }: { hz: number }) {
  const mhz = hz / 1_000_000;
  const formatted = mhz.toFixed(6);
  const [intPart, decPart] = formatted.split(".");
  
  // Split decimal into groups: kHz (3 digits) and Hz (3 digits)
  const khzPart = decPart.slice(0, 3);
  const hzPart = decPart.slice(3, 6);
  
  return (
    <div className="flex items-baseline gap-0.5">
      {/* MHz part - brightest */}
      <span className="text-3xl font-mono font-bold tabular-nums text-white tracking-tight">
        {intPart}
      </span>
      <span className="text-3xl font-mono text-zinc-500">.</span>
      {/* kHz part */}
      <span className="text-3xl font-mono font-bold tabular-nums text-white tracking-tight">
        {khzPart}
      </span>
      <span className="text-3xl font-mono text-zinc-500">.</span>
      {/* Hz part - dimmer */}
      <span className="text-2xl font-mono tabular-nums text-zinc-400 tracking-tight">
        {hzPart}
      </span>
      {/* Unit */}
      <span className="text-sm font-medium text-zinc-500 ml-1.5 self-end mb-1">MHz</span>
    </div>
  );
}

// Mode badge component
function ModeBadge({ mode }: { mode: string }) {
  const modeColors: Record<string, string> = {
    FT8: "bg-sky-500/20 text-sky-400 border-sky-500/30",
    FT4: "bg-violet-500/20 text-violet-400 border-violet-500/30",
    JT65: "bg-amber-500/20 text-amber-400 border-amber-500/30",
    JT9: "bg-rose-500/20 text-rose-400 border-rose-500/30",
  };
  
  const colorClass = modeColors[mode] || "bg-zinc-500/20 text-zinc-400 border-zinc-500/30";
  
  return (
    <span className={`px-2.5 py-1 rounded-md text-sm font-bold border ${colorClass}`}>
      {mode}
    </span>
  );
}

// TX Status indicator - compact pill style
function TxStatus({ transmitting, enabled }: { transmitting: boolean; enabled: boolean }) {
  if (transmitting) {
    return (
      <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-red-500/20">
        <div className="w-2 h-2 rounded-full bg-red-500 animate-pulse shadow-lg shadow-red-500/50" />
        <span className="text-xs font-bold text-red-400 uppercase tracking-wider">TX</span>
      </div>
    );
  }
  if (enabled) {
    return (
      <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-zinc-800">
        <div className="w-2 h-2 rounded-full bg-emerald-500" />
        <span className="text-xs font-medium text-zinc-400">TX Ready</span>
      </div>
    );
  }
  return (
    <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-zinc-800">
      <div className="w-2 h-2 rounded-full bg-zinc-600" />
      <span className="text-xs text-zinc-500">TX Off</span>
    </div>
  );
}

export function WsjtxConnection() {
  const [udpStatus, setUdpStatus] = useState<UdpStatus | null>(null);
  const [wsjtxStatus, setWsjtxStatus] = useState<WsjtxStatus | null>(null);
  const [port, setPort] = useState(2237);
  const [error, setError] = useState<string | null>(null);
  const [lastHeartbeat, setLastHeartbeat] = useState<Date | null>(null);
  const [showSettings, setShowSettings] = useState(false);

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

  const isConnected = udpStatus?.running && wsjtxStatus !== null;
  const hasRecentHeartbeat = lastHeartbeat && 
    (new Date().getTime() - lastHeartbeat.getTime()) < 30000;

  return (
    <section className="bg-gradient-to-b from-zinc-900 to-zinc-900/95 rounded-xl border border-zinc-800 overflow-hidden">
      {/* Header with connection status */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-zinc-800 bg-zinc-900/80">
        <div className="flex items-center gap-3">
          <div className={`p-1.5 rounded-lg ${isConnected ? "bg-emerald-500/20" : "bg-zinc-800"}`}>
            <Radio className={`h-5 w-5 ${isConnected ? "text-emerald-400" : "text-zinc-500"}`} />
          </div>
          <div>
            <h3 className="font-semibold text-sm">WSJT-X Connection</h3>
            {wsjtxStatus && udpStatus?.wsjtx_version && (
              <p className="text-xs text-zinc-500">v{udpStatus.wsjtx_version}</p>
            )}
          </div>
        </div>
        
        <div className="flex items-center gap-3">
          {/* Connection status badge */}
          {isConnected || hasRecentHeartbeat ? (
            <span className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-emerald-500/15 text-emerald-400 text-xs font-medium">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
              Connected
            </span>
          ) : udpStatus?.running ? (
            <span className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-amber-500/15 text-amber-400 text-xs font-medium">
              <span className="w-1.5 h-1.5 rounded-full bg-amber-400 animate-pulse" />
              Listening
            </span>
          ) : (
            <span className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-zinc-800 text-zinc-500 text-xs font-medium">
              <span className="w-1.5 h-1.5 rounded-full bg-zinc-500" />
              Offline
            </span>
          )}
          
          {/* Settings toggle */}
          <button
            onClick={() => setShowSettings(!showSettings)}
            className={`p-1.5 rounded-lg transition-colors ${
              showSettings ? "bg-zinc-700 text-zinc-300" : "hover:bg-zinc-800 text-zinc-500"
            }`}
          >
            <Settings className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Settings panel (collapsible) */}
      {showSettings && (
        <div className="px-4 py-3 border-b border-zinc-800 bg-zinc-900/50">
          <div className="flex items-center gap-3">
            <label className="text-xs text-zinc-500">UDP Port:</label>
            <input
              type="number"
              value={port}
              onChange={(e) => setPort(parseInt(e.target.value) || 2237)}
              disabled={udpStatus?.running}
              className="w-20 px-2 py-1 text-sm bg-zinc-800 border border-zinc-700 rounded-md font-mono text-zinc-300 focus:border-zinc-600 focus:outline-none disabled:opacity-50"
            />
            {udpStatus?.running ? (
              <button
                onClick={stopListener}
                className="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-red-500/20 text-red-400 border border-red-500/30 rounded-md hover:bg-red-500/30 transition-colors"
              >
                <Square className="h-3 w-3" />
                Stop
              </button>
            ) : (
              <button
                onClick={startListener}
                className="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 rounded-md hover:bg-emerald-500/30 transition-colors"
              >
                <Play className="h-3 w-3" />
                Start
              </button>
            )}
          </div>
          {error && (
            <p className="mt-2 text-xs text-red-400">{error}</p>
          )}
        </div>
      )}

      {/* Main content - SDR-style display */}
      {wsjtxStatus ? (
        <div className="p-4 space-y-3">
          {/* Frequency and mode row */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <FrequencyDisplay hz={wsjtxStatus.dial_freq} />
              <ModeBadge mode={wsjtxStatus.mode} />
            </div>
            <TxStatus 
              transmitting={wsjtxStatus.transmitting} 
              enabled={wsjtxStatus.tx_enabled} 
            />
          </div>
          
          {/* Period timer */}
          <PeriodTimer 
            isTransmitting={wsjtxStatus.transmitting} 
            mode={wsjtxStatus.mode}
          />
        </div>
      ) : udpStatus?.running ? (
        <div className="p-6 text-center">
          <div className="inline-flex items-center justify-center w-12 h-12 rounded-full bg-zinc-800 mb-3">
            <Wifi className="w-6 h-6 text-zinc-500 animate-pulse" />
          </div>
          <p className="text-sm text-zinc-400">Waiting for WSJT-X on port {port}</p>
          <p className="text-xs text-zinc-600 mt-1">
            Configure WSJT-X → Settings → Reporting → UDP Server
          </p>
        </div>
      ) : (
        <div className="p-6 text-center">
          <div className="inline-flex items-center justify-center w-12 h-12 rounded-full bg-zinc-800 mb-3">
            <WifiOff className="w-6 h-6 text-zinc-500" />
          </div>
          <p className="text-sm text-zinc-400">UDP listener not running</p>
          <button
            onClick={() => setShowSettings(true)}
            className="text-xs text-sky-400 hover:text-sky-300 mt-2"
          >
            Open settings to start
          </button>
        </div>
      )}
    </section>
  );
}