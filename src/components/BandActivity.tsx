import { useEffect, useState, useRef, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Radio, Zap, Target, CheckCircle } from "lucide-react";

interface Decode {
  id: string;
  time: string;
  timeMs: number;        // For Reply message
  deltaTime: number;     // For Reply message
  snr: number;
  call: string;
  grid: string | null;
  message: string;
  msgType: string;
  mode: string;          // For Reply message
  lowConfidence: boolean; // For Reply message
  dxcc: number | null;
  country: string | null;
  continent: string | null;
  cqz: number | null;
  ituz: number | null;
  deltaFreq: number;
  isNeeded: boolean;
  needReason: string | null;
  isWorked: boolean;
  isConfirmed: boolean;
}

interface DecodeEvent {
  time_ms: number;
  delta_time: number;
  snr: number;
  delta_freq: number;
  mode: string;
  message: string;
  de_call: string;        // The station sending the message
  dx_call: string | null; // The station being called (null for CQ)
  call: string;           // backwards compat - same as de_call
  grid: string | null;
  msg_type: string;
  dxcc: number | null;
  country: string | null;
  continent: string | null;
  cqz: number | null;
  ituz: number | null;
  low_confidence: boolean;
}

// US DXCC entity code - hardcoded for now, will be user setting later
const HOME_DXCC = 291;

// Convert milliseconds since midnight to HH:MM:SS
function msToTime(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  return `${hours.toString().padStart(2, "0")}:${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
}

export function BandActivity() {
  const [decodes, setDecodes] = useState<Decode[]>([]);
  const [workedCalls, setWorkedCalls] = useState<Set<string>>(new Set());
  const [workedDxcc, setWorkedDxcc] = useState<Set<number>>(new Set());
  // Note: We don't track worked states from decodes because STATE cannot be
  // reliably derived from grid (portable operators may be in different states).
  // WAS tracking is based on LoTW confirmations only.
  const [priorityOnly, setPriorityOnly] = useState(false);
  const [cqOnly, setCqOnly] = useState(false);
  const [dbReady, setDbReady] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  // Load worked callsigns from database
  useEffect(() => {
    const loadWorkedData = async () => {
      // First check if DB is ready
      try {
        const ready = await invoke<boolean>("is_db_ready");
        if (!ready) return;
      } catch {
        return;
      }

      try {
        const qsos = await invoke<Array<{call: string, dxcc: number | null}>>("get_qsos", { limit: 10000, offset: 0 });
        const calls = new Set(qsos.map(q => q.call));
        const dxccs = new Set(qsos.filter(q => q.dxcc).map(q => q.dxcc as number));
        setWorkedCalls(calls);
        setWorkedDxcc(dxccs);
        setDbReady(true);
      } catch {
        // Silently ignore
      }
    };

    // Listen for database ready event
    const unlistenDbReady = listen("db-ready", () => {
      loadWorkedData();
    });

    // Poll until DB is ready
    const pollInterval = setInterval(() => {
      if (!dbReady) {
        loadWorkedData();
      }
    }, 500);

    // Try immediately
    loadWorkedData();

    return () => {
      clearInterval(pollInterval);
      unlistenDbReady.then((f) => f());
    };
  }, [dbReady]);

  useEffect(() => {
    const unlistenDecode = listen<DecodeEvent>("wsjtx-decode", (event) => {
      const d = event.payload;
      
      // Use de_call - the station that transmitted this message
      const call = d.de_call;
      
      // Determine if this is a "needed" contact
      const isWorked = workedCalls.has(call);
      const isHomeCountry = d.dxcc === HOME_DXCC;
      const isDxccWorked = d.dxcc ? workedDxcc.has(d.dxcc) : true;
      // Note: We don't check for "NEW STATE" from decodes because STATE cannot
      // be reliably derived from grid (portable operators may be elsewhere).
      // WAS needs detection will be added when we have better data sources.
      
      let needReason: string | null = null;
      let isNeeded = false;
      
      // For DX stations, check if DXCC is needed
      if (!isHomeCountry && !isDxccWorked && d.dxcc) {
        needReason = d.country ? `NEW DXCC: ${d.country}` : "NEW DXCC";
        isNeeded = true;
      }
      // TODO: Add VUCC need checking based on grid
      
      const decode: Decode = {
        id: `${d.time_ms}-${call}-${d.delta_freq}`,
        time: msToTime(d.time_ms),
        timeMs: d.time_ms,
        deltaTime: d.delta_time,
        snr: d.snr,
        call: call,
        grid: d.grid,
        message: d.message,
        msgType: d.msg_type,
        mode: d.mode,
        lowConfidence: d.low_confidence,
        dxcc: d.dxcc,
        country: d.country,
        continent: d.continent,
        cqz: d.cqz,
        ituz: d.ituz,
        deltaFreq: d.delta_freq,
        isNeeded,
        needReason,
        isWorked,
        isConfirmed: false, // TODO: check LoTW confirmations
      };

      setDecodes((prev) => {
        // Remove old decodes from same callsign (keep latest)
        const filtered = prev.filter((p) => p.call !== call);
        // Add new decode at top, limit to 50 entries
        return [decode, ...filtered].slice(0, 50);
      });
    });

    // Listen for new QSOs to update worked status
    const unlistenQso = listen("qso-logged", () => {
      // Refresh worked data
      invoke<Array<{call: string, dxcc: number | null}>>("get_qsos", { limit: 10000, offset: 0 })
        .then((qsos) => {
          setWorkedCalls(new Set(qsos.map(q => q.call)));
          setWorkedDxcc(new Set(qsos.filter(q => q.dxcc).map(q => q.dxcc as number)));
        });
    });

    // Listen for clear message from WSJT-X to sync band activity
    const unlistenClear = listen<{ id: string; window: number }>("wsjtx-clear", (event) => {
      // Window 0 = Band Activity, Window 1 = Rx Frequency
      // Clear our decodes when Band Activity is cleared (window 0 or 2)
      if (event.payload.window === 0 || event.payload.window === 2) {
        setDecodes([]);
      }
    });

    return () => {
      unlistenDecode.then((f) => f());
      unlistenQso.then((f) => f());
      unlistenClear.then((f) => f());
    };
  }, [workedCalls, workedDxcc]);

  const filteredDecodes = decodes.filter((d) => {
    if (priorityOnly && !(d.isNeeded || d.msgType === "Cq")) return false;
    if (cqOnly && d.msgType !== "Cq") return false;
    return true;
  });

  const priorityDecodes = decodes.filter((d) => d.isNeeded && d.msgType === "Cq");

  // Double-click to call a station via WSJT-X
  const handleCallStation = useCallback(async (decode: Decode) => {
    try {
      await invoke("call_station", {
        timeMs: decode.timeMs,
        snr: decode.snr,
        deltaTime: decode.deltaTime,
        deltaFreq: decode.deltaFreq,
        mode: decode.mode,
        message: decode.message,
        lowConfidence: decode.lowConfidence,
      });
    } catch (err) {
      console.error("Failed to call station:", err);
    }
  }, []);

  return (
    <div className="space-y-4">
      {/* Priority Queue - Stations You Need */}
      {priorityDecodes.length > 0 && (
        <section className="bg-card rounded-lg border border-border p-4">
          <div className="flex items-center gap-2 mb-3">
            <Target className="h-5 w-5 text-primary" />
            <h3 className="font-semibold">Priority Queue - Stations You Need!</h3>
          </div>
          <div className="space-y-2">
            {priorityDecodes.slice(0, 5).map((d) => (
              <div 
                key={d.id} 
                className="flex items-center justify-between bg-background/50 rounded p-2 border border-border"
              >
                <div className="flex items-center gap-3">
                  <span className="font-mono font-bold text-lg text-sky-400">{d.call}</span>
                  <span className="text-sm text-muted-foreground">
                    {d.country || "Unknown"}
                    {d.continent && <span className="text-xs ml-1">({d.continent})</span>}
                  </span>
                  {d.grid && <span className="font-mono text-xs text-muted-foreground">{d.grid}</span>}
                </div>
                <div className="flex items-center gap-3">
                  <span className="text-sm">{d.snr > 0 ? "+" : ""}{d.snr} dB</span>
                  <span className="px-2 py-0.5 rounded text-xs text-yellow-400 font-semibold">
                    {d.needReason}
                  </span>
                  <button 
                    onClick={() => handleCallStation(d)}
                    className="px-3 py-1 bg-primary text-primary-foreground rounded text-sm hover:bg-primary/90"
                  >
                    Call
                  </button>
                </div>
              </div>
            ))}
          </div>
        </section>
      )}

      {/* Band Activity */}
      <section className="bg-card rounded-lg border border-border">
        <div className="flex items-center justify-between p-4 border-b border-border">
          <div className="flex items-center gap-2">
            <Radio className="h-5 w-5 text-primary" />
            <h3 className="font-semibold">Band Activity</h3>
            <span className="text-sm text-muted-foreground">({decodes.length} stations)</span>
          </div>
          <div className="flex items-center gap-4">
            <label className="flex items-center gap-2 text-sm cursor-pointer">
              <input
                type="checkbox"
                checked={cqOnly}
                onChange={(e) => setCqOnly(e.target.checked)}
                className="rounded"
              />
              CQ only
            </label>
            <label className="flex items-center gap-2 text-sm cursor-pointer">
              <input
                type="checkbox"
                checked={priorityOnly}
                onChange={(e) => setPriorityOnly(e.target.checked)}
                className="rounded"
              />
              Priority only
            </label>
          </div>
        </div>

        <div ref={containerRef} className="max-h-[400px] overflow-y-auto">
          <table className="w-full text-sm">
            <thead className="bg-muted/50 sticky top-0">
              <tr>
                <th className="text-left p-2 font-medium w-16">Time</th>
                <th className="text-left p-2 font-medium w-12">dB</th>
                <th className="text-left p-2 font-medium w-16">Freq</th>
                <th className="text-left p-2 font-medium">Callsign</th>
                <th className="text-left p-2 font-medium">Grid</th>
                <th className="text-left p-2 font-medium">Entity</th>
                <th className="text-left p-2 font-medium">Need</th>
                <th className="text-left p-2 font-medium">Message</th>
              </tr>
            </thead>
            <tbody>
              {filteredDecodes.map((d) => (
                <tr 
                  key={d.id} 
                  className="border-t border-border hover:bg-muted/50 cursor-pointer select-none"
                  onDoubleClick={() => handleCallStation(d)}
                  title="Double-click to call this station"
                >
                  <td className="p-2 font-mono text-muted-foreground text-xs">{d.time}</td>
                  <td className={`p-2 font-mono ${d.snr >= -10 ? "text-green-500" : d.snr >= -15 ? "text-yellow-500" : "text-red-500"}`}>
                    {d.snr > 0 ? "+" : ""}{d.snr}
                  </td>
                  <td className="p-2 font-mono text-xs text-muted-foreground">{d.deltaFreq}</td>
                  <td className="p-2">
                    <span className={`font-mono font-bold ${
                      d.isNeeded ? "text-sky-300" : d.isWorked ? "text-muted-foreground" : ""
                    }`}>
                      {d.call}
                    </span>
                  </td>
                  <td className="p-2 font-mono text-xs">{d.grid || "—"}</td>
                  <td className="p-2 text-xs truncate max-w-[120px]" title={d.country || undefined}>
                    {d.country || "—"}
                  </td>
                  <td className="p-2">
                    {d.isNeeded ? (
                      <span className="flex items-center gap-1 text-yellow-400">
                        <Zap className="h-3 w-3" />
                        <span className="text-xs">{d.needReason}</span>
                      </span>
                    ) : d.isConfirmed ? (
                      <span className="flex items-center gap-1 text-green-500">
                        <CheckCircle className="h-3 w-3" />
                        <span className="text-xs">Confirmed</span>
                      </span>
                    ) : (
                      <span className="text-xs text-muted-foreground">—</span>
                    )}
                  </td>
                  <td className="p-2 font-mono text-xs text-muted-foreground truncate max-w-[200px]" title={d.message}>
                    {d.message}
                  </td>
                </tr>
              ))}
              {filteredDecodes.length === 0 && (
                <tr>
                  <td colSpan={8} className="p-8 text-center text-muted-foreground">
                    Waiting for FT8 decodes...
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </section>
    </div>
  );
}
