import { useEffect, useState, useRef, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Radio, CheckCircle2, Zap, MapPin, Globe } from "lucide-react";
import { freqToBand } from "@/lib/utils";
import { lookupCallsign } from "@/hooks/useFccSync";

// US state abbreviation to full name mapping
const US_STATES: Record<string, string> = {
  AL: "Alabama", AK: "Alaska", AZ: "Arizona", AR: "Arkansas", CA: "California",
  CO: "Colorado", CT: "Connecticut", DE: "Delaware", FL: "Florida", GA: "Georgia",
  HI: "Hawaii", ID: "Idaho", IL: "Illinois", IN: "Indiana", IA: "Iowa",
  KS: "Kansas", KY: "Kentucky", LA: "Louisiana", ME: "Maine", MD: "Maryland",
  MA: "Massachusetts", MI: "Michigan", MN: "Minnesota", MS: "Mississippi", MO: "Missouri",
  MT: "Montana", NE: "Nebraska", NV: "Nevada", NH: "New Hampshire", NJ: "New Jersey",
  NM: "New Mexico", NY: "New York", NC: "North Carolina", ND: "North Dakota", OH: "Ohio",
  OK: "Oklahoma", OR: "Oregon", PA: "Pennsylvania", RI: "Rhode Island", SC: "South Carolina",
  SD: "South Dakota", TN: "Tennessee", TX: "Texas", UT: "Utah", VT: "Vermont",
  VA: "Virginia", WA: "Washington", WV: "West Virginia", WI: "Wisconsin", WY: "Wyoming",
  DC: "Washington DC", PR: "Puerto Rico", VI: "Virgin Islands", GU: "Guam",
};

interface WsjtxStatus {
  id: string;
  dial_freq: number;
  mode: string;
  dx_call: string;
  de_call: string;  // Our callsign from WSJT-X settings
  report: string;
  tx_enabled: boolean;
  transmitting: boolean;
  tx_message: string;
}

interface DecodeEvent {
  time_ms: number;
  snr: number;
  delta_freq: number;
  mode: string;
  message: string;
  de_call: string;
  dx_call: string | null;
  call: string;
  grid: string | null;
  msg_type: string;
  dxcc: number | null;
  country: string | null;
  continent: string | null;
  cqz: number | null;
  ituz: number | null;
  low_confidence: boolean;
}

type ActivityMode = "idle" | "calling_cq" | "in_qso" | "qso_complete";

interface QsoMessage {
  id: string;
  time: string;
  direction: "tx" | "rx";
  message: string;
  snr?: number;
  freq?: number;
}

interface ActiveQsoState {
  mode: ActivityMode;
  dxCall: string;
  dxGrid: string | null;
  dxCountry: string | null;
  dxDxcc: number | null;
  dxContinent: string | null;
  dxCqz: number | null;
  dxItuz: number | null;
  dxState: string | null; // US state from FCC database
  rstSent: string | null;
  rstRcvd: string | null;
  startTime: number | null;
  freq: number | null;
  radioMode: string | null;
  logged: boolean;
  messages: QsoMessage[];
}

// Extract signal report from FT8 message
function extractReport(msg: string): string | null {
  const match = msg.match(/[+-]?\d{2}\b/);
  return match ? match[0] : null;
}

// Format time from ms since midnight UTC (used for WSJT-X decode times)
function formatTimeFromMidnight(timeMs: number): string {
  const totalSeconds = Math.floor(timeMs / 1000);
  const hours = Math.floor(totalSeconds / 3600) % 24;
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  return `${hours.toString().padStart(2, "0")}${minutes.toString().padStart(2, "0")}${seconds.toString().padStart(2, "0")}`;
}

// Format current time as UTC HHMMSS (for TX messages)
function getCurrentUtcTime(): string {
  const now = new Date();
  const hours = now.getUTCHours();
  const minutes = now.getUTCMinutes();
  const seconds = now.getUTCSeconds();
  return `${hours.toString().padStart(2, "0")}${minutes.toString().padStart(2, "0")}${seconds.toString().padStart(2, "0")}`;
}

// Get current UTC date/time
function getUtcDateTime(): { date: string; time: string } {
  const now = new Date();
  const date = now.toISOString().slice(0, 10).replace(/-/g, "");
  const time = now.toISOString().slice(11, 19).replace(/:/g, "");
  return { date, time };
}

const INITIAL_STATE: ActiveQsoState = {
  mode: "idle",
  dxCall: "",
  dxGrid: null,
  dxCountry: null,
  dxDxcc: null,
  dxContinent: null,
  dxCqz: null,
  dxItuz: null,
  dxState: null,
  rstSent: null,
  rstRcvd: null,
  startTime: null,
  freq: null,
  radioMode: null,
  logged: false,
  messages: [],
};

export default function ActiveQso() {
  const [myCall, setMyCall] = useState<string>("");
  const [state, setState] = useState<ActiveQsoState>(INITIAL_STATE);
  const [wsjtxStatus, setWsjtxStatus] = useState<WsjtxStatus | null>(null);
  const loggingRef = useRef(false);
  const wasTransmittingRef = useRef(false); // Track TX state transitions
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const myCallRef = useRef<string>(""); // Ref to track myCall for closures
  const lastActivityRef = useRef<number>(Date.now()); // Track last activity time

  // Keep ref in sync with state
  useEffect(() => {
    myCallRef.current = myCall;
  }, [myCall]);

  // Stale data timeout - reset to idle after 2 minutes of no activity
  useEffect(() => {
    const checkStale = setInterval(() => {
      const now = Date.now();
      const timeSinceActivity = now - lastActivityRef.current;
      const TWO_MINUTES = 2 * 60 * 1000;
      
      // If no activity for 2 minutes and not in a completed QSO state, reset
      if (timeSinceActivity > TWO_MINUTES && state.mode !== "idle" && state.mode !== "qso_complete") {
        console.log("[ActiveQso] Resetting stale data after 2 minutes of inactivity");
        setState(INITIAL_STATE);
      }
    }, 10000); // Check every 10 seconds
    
    return () => clearInterval(checkStale);
  }, [state.mode]);

  // NOTE: We do NOT load all band activity here.
  // Active QSO only shows: 1) My TX messages, 2) RX messages directed at me
  // Band Activity section (separate component) shows all decodes

  // Auto-scroll messages
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [state.messages]);

  // Look up FCC data when DX call changes (for US state info)
  useEffect(() => {
    if (!state.dxCall) return;
    
    // Only look up US callsigns (start with A, K, N, W)
    const firstChar = state.dxCall.charAt(0).toUpperCase();
    if (!["A", "K", "N", "W"].includes(firstChar)) return;
    
    // Capture the call we're looking up to avoid stale closure issues
    const callToLookup = state.dxCall;
    let cancelled = false;
    
    lookupCallsign(callToLookup).then((fccInfo) => {
      if (cancelled) return;
      if (fccInfo?.state) {
        setState((prev) => {
          // Only update if still the same call AND state not already set
          if (prev.dxCall === callToLookup) {
            console.log(`[ActiveQso] FCC lookup: ${callToLookup} -> ${fccInfo.state}`);
            return { ...prev, dxState: fccInfo.state };
          }
          return prev;
        });
      }
    }).catch((err) => {
      console.warn(`FCC lookup failed for ${callToLookup}:`, err);
    });
    
    // Cleanup: cancel pending lookups when call changes
    return () => {
      cancelled = true;
    };
  }, [state.dxCall]);

  // Log the QSO
  const logQso = useCallback(async (qsoState: ActiveQsoState) => {
    if (loggingRef.current || qsoState.logged || !qsoState.dxCall) return;
    loggingRef.current = true;

    const { date, time } = getUtcDateTime();
    const band = qsoState.freq ? freqToBand(qsoState.freq) : "20m";

    try {
      await invoke("add_qso", {
        qso: {
          call: qsoState.dxCall,
          qso_date: date,
          time_on: time,
          band: band,
          mode: qsoState.radioMode || "FT8",
          freq: qsoState.freq ? qsoState.freq / 1_000_000 : null,
          gridsquare: qsoState.dxGrid,
          rst_sent: qsoState.rstSent,
          rst_rcvd: qsoState.rstRcvd,
          source: "WSJT-X",
        },
      });
      console.log(`✓ QSO logged: ${qsoState.dxCall}`);
      setState((prev) => ({ ...prev, logged: true }));
    } catch (err) {
      console.error("Failed to log QSO:", err);
    } finally {
      loggingRef.current = false;
    }
  }, []);

  // Add a message to the log
  const addMessage = useCallback(
    (direction: "tx" | "rx", message: string, snr?: number, timeMs?: number, freq?: number) => {
      // Update last activity timestamp
      lastActivityRef.current = Date.now();
      
      // For RX (decode) messages, timeMs is ms since midnight UTC
      // For TX messages, timeMs is not provided so we use current UTC time
      const time = timeMs ? formatTimeFromMidnight(timeMs) : getCurrentUtcTime();
      setState((prev) => {
        // Dedupe: check if we already have this exact message (by content, direction, and approximate time)
        const isDupe = prev.messages.some(
          (m) => m.message === message && m.direction === direction && m.time.slice(0, 4) === time.slice(0, 4)
        );
        if (isDupe) {
          return prev;
        }
        return {
          ...prev,
          messages: [
            ...prev.messages,
            { id: `live-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`, time, direction, message, snr, freq },
          ],
        };
      });
    },
    []
  );

  useEffect(() => {
    // Listen for heartbeat (just for connection status, not callsign)
    const unlistenHeartbeat = listen<{ id: string; version: string }>("wsjtx-heartbeat", (event) => {
      // Note: heartbeat id is the instance name (e.g., "WSJT-X"), NOT the callsign
      // We get the actual callsign from the Status message de_call field
      console.log(`[ActiveQso] Heartbeat from WSJT-X instance: "${event.payload.id}"`);
    });

    // Listen for WSJT-X status
    const unlistenStatus = listen<WsjtxStatus>("wsjtx-status", (event) => {
      const status = event.payload;
      setWsjtxStatus(status);

      // Get our callsign from de_call field (this is our actual callsign!)
      if (status.de_call && status.de_call !== myCallRef.current) {
        console.log(`[ActiveQso] Setting myCall from status.de_call: "${status.de_call}"`);
        setMyCall(status.de_call);
        myCallRef.current = status.de_call;
      }

      const txMsg = status.tx_message?.trim() || "";
      const isTxEnabled = status.tx_enabled;
      const isTransmitting = status.transmitting;
      const wasTransmitting = wasTransmittingRef.current;
      
      // Detect TX START (transition from not transmitting to transmitting)
      const txJustStarted = isTransmitting && !wasTransmitting;
      wasTransmittingRef.current = isTransmitting;
      
      // Only process TX messages when transmission STARTS
      if (txJustStarted && txMsg) {
        console.log(`[ActiveQso] TX STARTED: "${txMsg}"`);
        
        // Update last activity timestamp
        lastActivityRef.current = Date.now();
        
        const isCqMessage = txMsg.startsWith("CQ ");
        const newMessage: QsoMessage = {
          id: `tx-${Date.now()}`,
          time: getCurrentUtcTime(),
          direction: "tx",
          message: txMsg,
        };
        
        if (isCqMessage) {
          // CQ transmission - reset all DX info, keep only CQ messages
          setState((prev) => {
            if (prev.logged && prev.mode === "qso_complete") return prev;
            console.log(`[ActiveQso] Adding CQ message, clearing previous QSO data`);
            // When calling CQ, only keep recent CQ messages (last 5)
            const cqMessages = prev.mode === "calling_cq" 
              ? [...prev.messages.slice(-4), newMessage]
              : [newMessage];
            return {
              ...INITIAL_STATE,
              mode: "calling_cq",
              freq: status.dial_freq,
              radioMode: status.mode,
              startTime: Date.now(),
              messages: cqMessages,
            };
          });
        } else {
          // QSO transmission - extract target call
          const parts = txMsg.split(" ");
          let targetCall = "";
          const currentMyCall = myCallRef.current;
          if (parts.length >= 2) {
            targetCall = parts[0] === currentMyCall ? parts[1] : parts[0];
          }
          
          if (targetCall) {
            setState((prev) => {
              if (prev.logged && prev.mode === "qso_complete") return prev;
              console.log(`[ActiveQso] Adding QSO message to ${targetCall}, prev messages: ${prev.messages.length}`);
              
              // If new station, start fresh with just this message
              if (prev.dxCall !== targetCall) {
                return {
                  ...INITIAL_STATE,
                  mode: "in_qso",
                  dxCall: targetCall,
                  freq: status.dial_freq,
                  radioMode: status.mode,
                  startTime: Date.now(),
                  messages: [newMessage],
                  rstSent: status.report || null,
                };
              }
              
              // Same station, append message
              return {
                ...prev,
                mode: "in_qso",
                messages: [...prev.messages, newMessage],
                rstSent: status.report || prev.rstSent,
              };
            });
          }
        }
        return; // Don't run the rest of the state update
      }
      
      // Non-TX state updates (mode transitions without adding messages)
      setState((prev) => {
        if (prev.logged && prev.mode === "qso_complete") return prev;

        const isCqMessage = txMsg.startsWith("CQ ");
        
        // Extract target call
        let targetCall = "";
        if (txMsg && !isCqMessage) {
          const parts = txMsg.split(" ");
          const currentMyCall = myCallRef.current;
          if (parts.length >= 2) {
            targetCall = parts[0] === currentMyCall ? parts[1] : parts[0];
          }
        }

        // Calling CQ - update mode
        if (isTxEnabled && isCqMessage) {
          if (prev.mode !== "calling_cq") {
            return {
              ...prev,
              mode: "calling_cq",
              freq: status.dial_freq,
              radioMode: status.mode,
              startTime: prev.startTime || Date.now(),
            };
          }
          return { ...prev, freq: status.dial_freq, radioMode: status.mode };
        }

        // In QSO - update mode and target
        if (isTxEnabled && txMsg && !isCqMessage && targetCall) {
          if (prev.mode !== "in_qso" || prev.dxCall !== targetCall) {
            // If switching to a new station, reset all DX info
            const isNewStation = prev.dxCall !== targetCall && prev.dxCall !== "";
            return {
              ...(isNewStation ? INITIAL_STATE : prev),
              mode: "in_qso",
              dxCall: targetCall,
              freq: status.dial_freq,
              radioMode: status.mode,
              startTime: isNewStation ? Date.now() : (prev.startTime || Date.now()),
            };
          }
          return { ...prev, freq: status.dial_freq, radioMode: status.mode };
        }

        return prev;
      });
    });

    // Listen for decodes - RX messages from other stations
    const unlistenDecode = listen<DecodeEvent>("wsjtx-decode", (event) => {
      const decode = event.payload;
      const msg = decode.message.toUpperCase();
      // Use ref to get the current myCall value (avoids stale closure)
      const currentMyCall = myCallRef.current;
      const myCallUpper = currentMyCall.toUpperCase();
      const deCallUpper = (decode.de_call || "").toUpperCase();
      const dxCallUpper = (decode.dx_call || "").toUpperCase();

      // Debug: Log all decodes to understand what's happening
      console.log(`[ActiveQso] Decode: de_call="${decode.de_call}" dx_call="${decode.dx_call}" myCall="${currentMyCall}" msg="${decode.message}"`);

      // Skip if we don't know our callsign yet
      if (!myCallUpper) {
        console.log(`[ActiveQso] Skipping decode - myCall not set yet`);
        return;
      }

      // Skip our own transmissions - we handle those via Status.tx_message
      if (deCallUpper === myCallUpper) {
        console.log(`[ActiveQso] Skipping our own TX: ${decode.message}`);
        return;
      }

      // Check if this is directed at us (case-insensitive comparison)
      if (dxCallUpper === myCallUpper) {
        console.log(`[ActiveQso] ✓ RX message directed at us from ${decode.de_call}: "${decode.message}"`);
        // Someone is calling us
        addMessage("rx", decode.message, decode.snr, decode.time_ms, decode.delta_freq);

        setState((prev) => {
          if (prev.logged) return prev;

          // If we're in a QSO with this station, update state
          const prevDxCallUpper = (prev.dxCall || "").toUpperCase();
          if (prev.mode === "in_qso" && deCallUpper === prevDxCallUpper) {
            let newState = { ...prev };

            // Update grid/country/location data
            if (decode.grid && !prev.dxGrid) {
              newState.dxGrid = decode.grid;
            }
            if (decode.country && !prev.dxCountry) {
              newState.dxCountry = decode.country;
            }
            if (decode.dxcc && !prev.dxDxcc) {
              newState.dxDxcc = decode.dxcc;
            }
            if (decode.continent && !prev.dxContinent) {
              newState.dxContinent = decode.continent;
            }
            if (decode.cqz && !prev.dxCqz) {
              newState.dxCqz = decode.cqz;
            }
            if (decode.ituz && !prev.dxItuz) {
              newState.dxItuz = decode.ituz;
            }

            // Extract their report to us
            const theirReport = extractReport(msg);
            if (theirReport && !prev.rstRcvd) {
              newState.rstRcvd = theirReport;
            }

            // Check for QSO complete (RR73/73)
            if (msg.includes("RR73") || msg.includes("RRR") || msg.match(/\b73\b/)) {
              if (prev.rstRcvd || theirReport) {
                newState.mode = "qso_complete";
              }
            }

            return newState;
          }

          // If they're calling us and we're calling CQ, the message was already added above
          // No need to add it again here

          return prev;
        });
      }
    });

    // Listen for WSJT-X QSO logged event
    // NOTE: Rust backend already logs to database, we just update UI state here
    const unlistenQso = listen<{
      call: string;
      grid: string;
      freq_mhz: number;
      mode: string;
      rst_sent: string;
      rst_rcvd: string;
      band: string;
    }>("qso-logged", (event) => {
      const qso = event.payload;
      console.log("[ActiveQso] ✓ QSO LOGGED event received from backend:", qso);
      
      setState((prev) => ({
        ...prev,
        mode: "qso_complete",
        logged: true,
        dxCall: qso.call || prev.dxCall,
        dxGrid: qso.grid || prev.dxGrid,
        rstSent: qso.rst_sent || prev.rstSent,
        rstRcvd: qso.rst_rcvd || prev.rstRcvd,
      }));
    });

    return () => {
      unlistenHeartbeat.then((f) => f());
      unlistenStatus.then((f) => f());
      unlistenDecode.then((f) => f());
      unlistenQso.then((f) => f());
    };
  }, [addMessage]); // Removed myCall - using myCallRef instead

  // Auto-log when QSO complete
  useEffect(() => {
    if (state.mode === "qso_complete" && !state.logged && state.dxCall && state.rstRcvd) {
      logQso(state);
    }
  }, [state.mode, state.logged, state.dxCall, state.rstRcvd, logQso]);

  // Reset after logging (with delay to show success)
  useEffect(() => {
    if (state.mode === "qso_complete" && state.logged) {
      const timer = setTimeout(() => {
        setState(INITIAL_STATE);
        wasTransmittingRef.current = false;
      }, 5000);
      return () => clearTimeout(timer);
    }
  }, [state.mode, state.logged]);

  // Show placeholder when idle
  if (state.mode === "idle" && state.messages.length === 0 && !wsjtxStatus?.tx_enabled) {
    return (
      <section className="bg-zinc-900/80 rounded-lg border border-zinc-700/50 overflow-hidden">
        <div className="flex items-center justify-center px-4 py-6 text-zinc-500">
          <Radio className="h-5 w-5 mr-2 opacity-50" />
          <span>No active QSO</span>
        </div>
      </section>
    );
  }

  // Build location string for display
  const getLocationDisplay = () => {
    const parts: string[] = [];
    
    // For US stations, show state name prominently
    if (state.dxState && US_STATES[state.dxState]) {
      parts.push(US_STATES[state.dxState]);
    }
    
    // Country (skip for US if we have state)
    if (state.dxCountry && (!state.dxState || !state.dxCountry.includes("UNITED STATES"))) {
      parts.push(state.dxCountry);
    } else if (state.dxCountry && state.dxState) {
      // For US, just add "USA" suffix
      parts.push("USA");
    }
    
    return parts.join(", ");
  };

  // Get mode-specific styling
  const getModeStyles = () => {
    switch (state.mode) {
      case "calling_cq":
        return {
          border: "border-amber-500/30",
          headerBg: "bg-amber-500/5",
          icon: <Radio className="h-4 w-4 animate-pulse text-amber-400" />,
          statusText: "Calling CQ",
          statusColor: "text-amber-400",
        };
      case "in_qso":
        return {
          border: "border-emerald-500/30",
          headerBg: "bg-emerald-500/5",
          icon: <Zap className="h-4 w-4 text-emerald-400" />,
          statusText: `Working ${state.dxCall}`,
          statusColor: "text-emerald-400",
        };
      case "qso_complete":
        return {
          border: "border-green-500/30",
          headerBg: "bg-green-500/5",
          icon: <CheckCircle2 className="h-4 w-4 text-green-400" />,
          statusText: state.logged ? "QSO Logged!" : "QSO Complete",
          statusColor: "text-green-400",
        };
      default:
        return {
          border: "border-zinc-700/50",
          headerBg: "bg-zinc-800/50",
          icon: <Radio className="h-4 w-4 text-zinc-400" />,
          statusText: "Standby",
          statusColor: "text-zinc-400",
        };
    }
  };

  const modeStyles = getModeStyles();
  const locationDisplay = getLocationDisplay();

  return (
    <section className={`bg-zinc-900/80 rounded-lg border ${modeStyles.border} overflow-hidden transition-colors duration-300`}>
      {/* Header - Status and QSO Info */}
      <div className={`px-4 py-3 ${modeStyles.headerBg} border-b border-zinc-700/30`}>
        <div className="flex items-center justify-between">
          {/* Left: Status + Callsign */}
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-2">
              {modeStyles.icon}
              <span className={`text-sm font-medium ${modeStyles.statusColor}`}>
                {modeStyles.statusText}
              </span>
            </div>
          </div>

          {/* Right: Signal Reports */}
          {(state.rstRcvd || state.rstSent) && (
            <div className="flex items-center gap-3">
              {state.rstRcvd && (
                <div className="flex items-center gap-1.5 px-2 py-0.5 rounded bg-zinc-800/80">
                  <span className="text-xs text-zinc-500 uppercase">Rcvd</span>
                  <span className="font-mono text-sm font-semibold text-green-400">{state.rstRcvd}</span>
                </div>
              )}
              {state.rstSent && (
                <div className="flex items-center gap-1.5 px-2 py-0.5 rounded bg-zinc-800/80">
                  <span className="text-xs text-zinc-500 uppercase">Sent</span>
                  <span className="font-mono text-sm font-semibold text-blue-400">{state.rstSent}</span>
                </div>
              )}
            </div>
          )}
        </div>

        {/* Location Row - only show if in QSO/complete mode AND have location data */}
        {state.mode !== "calling_cq" && state.mode !== "idle" && (locationDisplay || state.dxGrid || state.dxCqz || state.dxItuz) && (
          <div className="flex items-center justify-between mt-2 pt-2 border-t border-zinc-700/20">
            {/* Location */}
            <div className="flex items-center gap-2 text-sm">
              {locationDisplay && (
                <div className="flex items-center gap-1.5 text-zinc-300">
                  <MapPin className="h-3.5 w-3.5 text-zinc-500" />
                  <span>{locationDisplay}</span>
                </div>
              )}
            </div>

            {/* Grid + Zones */}
            <div className="flex items-center gap-3 text-xs text-zinc-500">
              {state.dxGrid && (
                <span className="font-mono text-zinc-400">{state.dxGrid}</span>
              )}
              {(state.dxCqz || state.dxItuz) && (
                <div className="flex items-center gap-2">
                  <Globe className="h-3 w-3" />
                  {state.dxCqz && <span>CQ {state.dxCqz}</span>}
                  {state.dxItuz && <span>ITU {state.dxItuz}</span>}
                </div>
              )}
            </div>
          </div>
        )}
      </div>

      {/* Message Exchange */}
      <div className="max-h-32 overflow-y-auto p-2 space-y-0.5">
        {state.messages.length === 0 ? (
          <div className="text-center text-zinc-600 py-4 text-sm">
            {state.mode === "calling_cq" ? "Transmitting CQ..." : "Waiting for activity..."}
          </div>
        ) : (
          state.messages.map((m) => (
            <div
              key={m.id}
              className="flex items-start gap-2 px-2 py-1 rounded hover:bg-zinc-800/30 transition-colors"
            >
              {/* Time */}
              <span className="font-mono text-xs text-zinc-600 shrink-0 pt-0.5">
                {m.time.slice(0, 2)}:{m.time.slice(2, 4)}:{m.time.slice(4, 6)}
              </span>
              
              {/* Direction indicator - subtle colored dot */}
              <span 
                className={`shrink-0 w-1.5 h-1.5 rounded-full mt-1.5 ${
                  m.direction === "tx" ? "bg-blue-400" : "bg-green-400"
                }`}
              />
              
              {/* SNR for RX messages */}
              {m.direction === "rx" && m.snr !== undefined && (
                <span className="font-mono text-xs text-zinc-500 shrink-0 w-6 text-right pt-0.5">
                  {m.snr > 0 ? "+" : ""}{m.snr}
                </span>
              )}
              {m.direction === "tx" && (
                <span className="shrink-0 w-6" /> // Spacer for alignment
              )}
              
              {/* Message */}
              <span className={`font-mono text-sm flex-1 ${
                m.direction === "tx" ? "text-zinc-400" : "text-zinc-200"
              }`}>
                {m.message}
              </span>
            </div>
          ))
        )}
        <div ref={messagesEndRef} />
      </div>
    </section>
  );
}
