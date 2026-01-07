import { useEffect, useState, useRef, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Radio, CheckCircle2, Zap, ArrowUp, ArrowDown } from "lucide-react";
import { freqToBand } from "@/lib/utils";

interface WsjtxStatus {
  id: string;
  dial_freq: number;
  mode: string;
  dx_call: string;
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

  // NOTE: We do NOT load all band activity here.
  // Active QSO only shows: 1) My TX messages, 2) RX messages directed at me
  // Band Activity section (separate component) shows all decodes

  // Auto-scroll messages
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [state.messages]);

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
    // Listen for heartbeat to get our callsign
    const unlistenHeartbeat = listen<{ id: string; version: string }>("wsjtx-heartbeat", (event) => {
      if (event.payload.id && event.payload.id !== myCall) {
        setMyCall(event.payload.id);
      }
    });

    // Listen for WSJT-X status
    const unlistenStatus = listen<WsjtxStatus>("wsjtx-status", (event) => {
      const status = event.payload;
      setWsjtxStatus(status);

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
        
        const isCqMessage = txMsg.startsWith("CQ ");
        const newMessage: QsoMessage = {
          id: `tx-${Date.now()}`,
          time: getCurrentUtcTime(),
          direction: "tx",
          message: txMsg,
        };
        
        if (isCqMessage) {
          // CQ transmission
          setState((prev) => {
            if (prev.logged && prev.mode === "qso_complete") return prev;
            console.log(`[ActiveQso] Adding CQ message, prev messages: ${prev.messages.length}`);
            return {
              ...prev,
              mode: "calling_cq",
              freq: status.dial_freq,
              radioMode: status.mode,
              startTime: prev.startTime || Date.now(),
              dxCall: "",
              messages: [...prev.messages, newMessage],
            };
          });
        } else {
          // QSO transmission - extract target call
          const parts = txMsg.split(" ");
          let targetCall = "";
          if (parts.length >= 2) {
            targetCall = parts[0] === myCall ? parts[1] : parts[0];
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
          if (parts.length >= 2) {
            targetCall = parts[0] === myCall ? parts[1] : parts[0];
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
            return {
              ...prev,
              mode: "in_qso",
              dxCall: targetCall,
              freq: status.dial_freq,
              radioMode: status.mode,
              startTime: prev.startTime || Date.now(),
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

      // Skip our own transmissions - we handle those via Status.tx_message
      if (myCall && decode.de_call === myCall) {
        return;
      }

      // Check if this is directed at us
      if (myCall && decode.dx_call === myCall) {
        // Someone is calling us
        addMessage("rx", decode.message, decode.snr, decode.time_ms, decode.delta_freq);

        setState((prev) => {
          if (prev.logged) return prev;

          // If we're in a QSO with this station, update state
          if (prev.mode === "in_qso" && decode.de_call === prev.dxCall) {
            let newState = { ...prev };

            // Update grid/country
            if (decode.grid && !prev.dxGrid) {
              newState.dxGrid = decode.grid;
            }
            if (decode.country && !prev.dxCountry) {
              newState.dxCountry = decode.country;
            }
            if (decode.dxcc && !prev.dxDxcc) {
              newState.dxDxcc = decode.dxcc;
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

          // If they're calling us and we're calling CQ, might want to respond
          if (prev.mode === "calling_cq" && decode.de_call) {
            // Just track their call for now
            addMessage("rx", decode.message, decode.snr, decode.time_ms);
          }

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
      console.log("WSJT-X QSO logged event (backend saved to DB):", qso);
      
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
  }, [myCall, addMessage]);

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

  // Determine status badge
  const getStatusBadge = () => {
    switch (state.mode) {
      case "calling_cq":
        return (
          <div className="flex items-center gap-2 px-3 py-1 rounded-full bg-amber-500/20 text-amber-400 text-sm font-medium">
            <Radio className="h-4 w-4 animate-pulse" />
            Calling CQ
          </div>
        );
      case "in_qso":
        return (
          <div className="flex items-center gap-2 px-3 py-1 rounded-full bg-emerald-500/20 text-emerald-400 text-sm font-medium">
            <Zap className="h-4 w-4" />
            QSO with {state.dxCall}
          </div>
        );
      case "qso_complete":
        return (
          <div className="flex items-center gap-2 px-3 py-1 rounded-full bg-green-500/20 text-green-400 text-sm font-medium">
            <CheckCircle2 className="h-4 w-4" />
            QSO Complete{state.logged && " — Logged!"}
          </div>
        );
      default:
        return null;
    }
  };

  return (
    <section className="bg-zinc-900/80 rounded-lg border border-zinc-700/50 overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-zinc-700/50 bg-zinc-800/50">
        <div className="flex items-center gap-3">
          {getStatusBadge()}
          {state.dxCountry && (
            <span className="text-sm text-zinc-400">{state.dxCountry}</span>
          )}
        </div>
        <div className="flex items-center gap-4 text-sm text-zinc-500">
          {state.dxGrid && (
            <span className="font-mono">{state.dxGrid}</span>
          )}
          {state.rstRcvd && (
            <span className="flex items-center gap-1">
              <ArrowDown className="h-3 w-3 text-green-400" />
              <span className="font-mono">{state.rstRcvd}</span>
            </span>
          )}
          {state.rstSent && (
            <span className="flex items-center gap-1">
              <ArrowUp className="h-3 w-3 text-red-400" />
              <span className="font-mono">{state.rstSent}</span>
            </span>
          )}
        </div>
      </div>

      {/* QSO Message Exchange - only TX and messages directed at me */}
      <div className="max-h-32 overflow-y-auto p-2 space-y-0.5 font-mono text-sm">
        {state.messages.length === 0 ? (
          <div className="text-center text-zinc-600 py-4">
            {state.mode === "calling_cq" ? "Transmitting CQ..." : "Waiting for activity..."}
          </div>
        ) : (
          state.messages.map((m) => (
            <div
              key={m.id}
              className={`flex items-start gap-2 px-2 py-0.5 rounded ${
                m.direction === "tx"
                  ? "bg-red-950/30 text-red-300"
                  : "bg-zinc-800/50 text-zinc-300"
              }`}
            >
              <span className="text-zinc-500 shrink-0">{m.time}</span>
              <span
                className={`shrink-0 w-6 text-center ${
                  m.direction === "tx" ? "text-red-400" : "text-green-400"
                }`}
              >
                {m.direction === "tx" ? "Tx" : "Rx"}
              </span>
              {m.snr !== undefined && (
                <span className="shrink-0 w-8 text-right text-zinc-500">
                  {m.snr > 0 ? "+" : ""}
                  {m.snr}
                </span>
              )}
              <span className="flex-1">{m.message}</span>
            </div>
          ))
        )}
        <div ref={messagesEndRef} />
      </div>
    </section>
  );
}
