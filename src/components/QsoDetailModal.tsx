/**
 * QSO Detail Modal Component
 * 
 * Full-screen modal showing complete QSO details with callsign history,
 * confirmation status, award impact, and navigation between QSOs.
 */

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { 
  X, 
  ChevronLeft, 
  ChevronRight, 
  Star, 
  Sparkles, 
  History, 
  Trash2, 
  Edit3 
} from "lucide-react";
import { Qso, parseAdifFields, CallsignHistory, QsoStatus } from "@/stores/qsoStore";

// =============================================================================
// Helper Functions
// =============================================================================

function formatDate(date: string): string {
  if (!date) return "";
  if (date.includes("-")) return date;
  if (date.length === 8) {
    return `${date.slice(0, 4)}-${date.slice(4, 6)}-${date.slice(6, 8)}`;
  }
  return date;
}

function formatTime(time: string): string {
  if (!time || time.length < 4) return "";
  return `${time.slice(0, 2)}:${time.slice(2, 4)}`;
}

// =============================================================================
// Detail Field Component
// =============================================================================

function DetailField({ label, value, small }: { label: string; value: string; small?: boolean }) {
  return (
    <div>
      <div className="text-xs text-zinc-500">{label}</div>
      <div className={`${small ? "text-sm text-zinc-400 font-mono truncate" : "text-zinc-100"}`}>
        {value}
      </div>
    </div>
  );
}

// =============================================================================
// QSO Detail Modal Component
// =============================================================================

export interface QsoDetailModalProps {
  qso: Qso;
  currentIndex: number;
  totalCount: number;
  onClose: () => void;
  onNavigate: (index: number) => void;
  onDelete: () => void;
}

export function QsoDetailModal({ 
  qso, 
  currentIndex, 
  totalCount, 
  onClose, 
  onNavigate, 
  onDelete 
}: QsoDetailModalProps) {
  const adif = parseAdifFields(qso);
  const [history, setHistory] = useState<CallsignHistory | null>(null);
  const [status, setStatus] = useState<QsoStatus | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

  const canGoPrev = currentIndex > 0;
  const canGoNext = currentIndex < totalCount - 1;

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "ArrowLeft" && canGoPrev) {
        onNavigate(currentIndex - 1);
      } else if (e.key === "ArrowRight" && canGoNext) {
        onNavigate(currentIndex + 1);
      } else if (e.key === "Escape") {
        onClose();
      }
    };
    
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [currentIndex, canGoPrev, canGoNext, onNavigate, onClose]);

  // Load callsign history
  useEffect(() => {
    const loadHistory = async () => {
      try {
        const h = await invoke<CallsignHistory>("get_callsign_history", {
          call: qso.call,
          excludeId: qso.id,
        });
        setHistory(h);
      } catch (e) {
        console.error("Failed to load callsign history:", e);
      }
    };
    
    const loadStatus = async () => {
      try {
        const s = await invoke<QsoStatus>("check_qso_status", {
          call: qso.call,
          band: qso.band,
          mode: qso.mode,
          dxcc: qso.dxcc,
          qsoDate: qso.qso_date,
          excludeId: qso.id,
        });
        setStatus(s);
      } catch (e) {
        console.error("Failed to load QSO status:", e);
      }
    };
    
    loadHistory();
    loadStatus();
  }, [qso]);

  const handleDelete = async () => {
    try {
      await invoke("delete_qso", { id: qso.id });
      onDelete();
    } catch (e) {
      console.error("Failed to delete QSO:", e);
    }
  };
  
  return (
    <div 
      className="fixed inset-0 bg-black/70 flex items-center justify-center z-50"
      onClick={onClose}
    >
      <div 
        className="bg-zinc-900 rounded-lg border border-zinc-700 w-full max-w-4xl mx-4 max-h-[90vh] overflow-hidden flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-800 shrink-0">
          <h2 className="text-xl font-semibold text-zinc-100 flex items-center gap-3">
            QSO with <span className="text-sky-400">{qso.call}</span>
            {status?.is_new_dxcc && (
              <span className="px-2 py-0.5 text-xs bg-green-600 text-white rounded-full flex items-center gap-1">
                <Sparkles className="h-3 w-3" /> New DXCC!
              </span>
            )}
            {status?.is_new_band_dxcc && !status?.is_new_dxcc && (
              <span className="px-2 py-0.5 text-xs bg-emerald-600 text-white rounded-full">
                New on {qso.band}
              </span>
            )}
            {history && history.total_qsos > 0 && (
              <span className="px-2 py-0.5 text-xs bg-amber-700 text-white rounded-full flex items-center gap-1">
                <Star className="h-3 w-3 fill-current" /> {history.total_qsos} previous
              </span>
            )}
          </h2>
          <button 
            onClick={onClose}
            className="text-zinc-500 hover:text-zinc-300"
          >
            <X className="h-5 w-5" />
          </button>
        </div>

        {/* Content - Two columns */}
        <div className="flex-1 overflow-y-auto">
          <div className="grid grid-cols-1 lg:grid-cols-5 gap-0">
            {/* Left: QSO Details */}
            <div className="lg:col-span-3 p-6 space-y-6 border-r border-zinc-800">
              {/* Basic Info */}
              <section>
                <h3 className="text-sm font-medium text-zinc-500 mb-3 uppercase tracking-wider">Basic Info</h3>
                <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
                  <DetailField label="Date" value={formatDate(qso.qso_date)} />
                  <DetailField label="Time On (UTC)" value={formatTime(qso.time_on)} />
                  {qso.time_off && <DetailField label="Time Off" value={formatTime(qso.time_off)} />}
                  <DetailField label="Band" value={qso.band} />
                  <DetailField label="Mode" value={qso.mode} />
                  <DetailField label="Frequency" value={qso.freq ? `${qso.freq.toFixed(6)} MHz` : "-"} />
                </div>
              </section>

              {/* Signal Reports */}
              <section>
                <h3 className="text-sm font-medium text-zinc-500 mb-3 uppercase tracking-wider">Signal Reports</h3>
                <div className="grid grid-cols-2 gap-4">
                  <DetailField label="RST Sent" value={qso.rst_sent || "-"} />
                  <DetailField label="RST Received" value={qso.rst_rcvd || "-"} />
                </div>
              </section>

              {/* Location */}
              <section>
                <h3 className="text-sm font-medium text-zinc-500 mb-3 uppercase tracking-wider">Location</h3>
                <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
                  <DetailField label="Country" value={qso.country || "-"} />
                  <DetailField label="DXCC Entity" value={qso.dxcc?.toString() || "-"} />
                  <DetailField label="Continent" value={qso.continent || "-"} />
                  <DetailField label="State" value={qso.state || "-"} />
                  <DetailField label="Grid" value={qso.gridsquare || "-"} />
                  {qso.cqz && <DetailField label="CQ Zone" value={qso.cqz.toString()} />}
                  {qso.ituz && <DetailField label="ITU Zone" value={qso.ituz.toString()} />}
                </div>
              </section>

              {/* My Station */}
              {(qso.station_callsign || qso.my_gridsquare || qso.tx_pwr) && (
                <section>
                  <h3 className="text-sm font-medium text-zinc-500 mb-3 uppercase tracking-wider">My Station</h3>
                  <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
                    {qso.station_callsign && <DetailField label="My Call" value={qso.station_callsign} />}
                    {qso.my_gridsquare && <DetailField label="My Grid" value={qso.my_gridsquare} />}
                    {qso.tx_pwr && <DetailField label="TX Power" value={`${qso.tx_pwr} W`} />}
                  </div>
                </section>
              )}

              {/* Extended ADIF Fields */}
              {Object.keys(adif).some(k => adif[k as keyof typeof adif]) && (
                <section>
                  <h3 className="text-sm font-medium text-zinc-500 mb-3 uppercase tracking-wider">Additional Info</h3>
                  <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
                    {adif.name && <DetailField label="Name" value={adif.name} />}
                    {adif.qth && <DetailField label="QTH" value={adif.qth} />}
                    {adif.comments && <DetailField label="Comments" value={adif.comments} />}
                    {adif.prop_mode && <DetailField label="Propagation" value={adif.prop_mode} />}
                    {adif.sota_ref && <DetailField label="SOTA" value={adif.sota_ref} />}
                    {adif.pota_ref && <DetailField label="POTA" value={adif.pota_ref} />}
                    {adif.iota && <DetailField label="IOTA" value={adif.iota} />}
                    {adif.wwff_ref && <DetailField label="WWFF" value={adif.wwff_ref} />}
                    {adif.rig && <DetailField label="Rig" value={adif.rig} />}
                    {adif.antenna && <DetailField label="Antenna" value={adif.antenna} />}
                  </div>
                </section>
              )}

              {/* Metadata */}
              <section>
                <h3 className="text-sm font-medium text-zinc-500 mb-3 uppercase tracking-wider">Metadata</h3>
                <div className="grid grid-cols-2 gap-4">
                  <DetailField label="Source" value={qso.source || "-"} />
                  <DetailField label="Created" value={qso.created_at} small />
                  <DetailField label="Updated" value={qso.updated_at} small />
                  <DetailField label="UUID" value={qso.uuid} small />
                </div>
              </section>
            </div>

            {/* Right: History Panel */}
            <div className="lg:col-span-2 p-6 bg-zinc-950/50">
              {/* Previous QSOs */}
              <section className="mb-6">
                <h3 className="text-sm font-medium text-zinc-500 mb-3 uppercase tracking-wider flex items-center gap-2">
                  <History className="h-4 w-4" />
                  Previous QSOs with {qso.call}
                </h3>
                
                {history === null ? (
                  <div className="text-zinc-500 text-sm">Loading...</div>
                ) : history.total_qsos === 0 ? (
                  <div className="text-zinc-500 text-sm italic">First QSO with this station!</div>
                ) : (
                  <div className="space-y-2">
                    <div className="text-xs text-zinc-400 mb-2">
                      {history.total_qsos} QSO{history.total_qsos > 1 ? "s" : ""} on {history.bands_worked.join(", ")}
                    </div>
                    {history.previous_qsos.slice(0, 5).map((prev) => (
                      <div 
                        key={prev.id}
                        className="flex items-center justify-between text-sm bg-zinc-800/50 rounded px-3 py-2"
                      >
                        <span className="text-zinc-400">
                          {formatDate(prev.qso_date)}
                        </span>
                        <span className="text-zinc-300">
                          {prev.band} {prev.mode}
                        </span>
                        <span className="text-zinc-500 text-xs">
                          {prev.rst_sent}/{prev.rst_rcvd}
                        </span>
                      </div>
                    ))}
                    {history.total_qsos > 5 && (
                      <div className="text-xs text-zinc-500 text-center">
                        +{history.total_qsos - 5} more
                      </div>
                    )}
                  </div>
                )}
              </section>

              {/* Confirmation Status */}
              <section className="mb-6">
                <h3 className="text-sm font-medium text-zinc-500 mb-3 uppercase tracking-wider">
                  Confirmation Status
                </h3>
                <div className="space-y-2">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-zinc-400">LoTW</span>
                    {qso.lotw_rcvd === "Y" ? (
                      <span className="text-green-500 flex items-center gap-1">
                        <span className="w-2 h-2 bg-green-500 rounded-full"></span>
                        Confirmed
                      </span>
                    ) : (
                      <span className="text-zinc-600">Not confirmed</span>
                    )}
                  </div>
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-zinc-400">eQSL</span>
                    {qso.eqsl_rcvd === "Y" ? (
                      <span className="text-green-500 flex items-center gap-1">
                        <span className="w-2 h-2 bg-green-500 rounded-full"></span>
                        Confirmed
                      </span>
                    ) : (
                      <span className="text-zinc-600">Not confirmed</span>
                    )}
                  </div>
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-zinc-400">Paper QSL</span>
                    <span className="text-zinc-600">—</span>
                  </div>
                </div>
              </section>

              {/* Award Status */}
              {status && (
                <section>
                  <h3 className="text-sm font-medium text-zinc-500 mb-3 uppercase tracking-wider">
                    Award Impact
                  </h3>
                  <div className="space-y-2 text-sm">
                    {status.is_new_dxcc && (
                      <div className="flex items-center gap-2 text-green-400">
                        <Sparkles className="h-4 w-4" />
                        <span>New DXCC entity!</span>
                      </div>
                    )}
                    {status.is_new_band_dxcc && !status.is_new_dxcc && (
                      <div className="flex items-center gap-2 text-emerald-400">
                        <span>🎯</span>
                        <span>New band slot for {qso.country}</span>
                      </div>
                    )}
                    {status.is_new_mode_dxcc && !status.is_new_dxcc && (
                      <div className="flex items-center gap-2 text-emerald-400">
                        <span>📡</span>
                        <span>New mode for {qso.country}</span>
                      </div>
                    )}
                    {!status.is_new_dxcc && !status.is_new_band_dxcc && !status.is_new_mode_dxcc && (
                      <div className="text-zinc-500 italic">
                        Already worked on this band/mode
                      </div>
                    )}
                  </div>
                </section>
              )}
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex justify-between px-6 py-4 border-t border-zinc-800 shrink-0">
          <div>
            {isDeleting ? (
              <div className="flex items-center gap-2">
                <span className="text-red-400 text-sm">Delete this QSO?</span>
                <button
                  onClick={handleDelete}
                  className="px-3 py-1.5 bg-red-600 hover:bg-red-500 text-white text-sm rounded-lg transition-colors"
                >
                  Yes, Delete
                </button>
                <button
                  onClick={() => setIsDeleting(false)}
                  className="px-3 py-1.5 bg-zinc-700 hover:bg-zinc-600 text-zinc-100 text-sm rounded-lg transition-colors"
                >
                  Cancel
                </button>
              </div>
            ) : (
              <button
                onClick={() => setIsDeleting(true)}
                className="flex items-center gap-2 px-3 py-2 text-red-400 hover:text-red-300 hover:bg-red-950/30 rounded-lg transition-colors"
              >
                <Trash2 className="h-4 w-4" />
                <span>Delete</span>
              </button>
            )}
          </div>

          {/* Navigation controls */}
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2">
              <button
                onClick={() => canGoPrev && onNavigate(currentIndex - 1)}
                disabled={!canGoPrev}
                className={`p-2 rounded-lg transition-colors ${
                  canGoPrev 
                    ? 'bg-zinc-800 hover:bg-zinc-700 text-zinc-100' 
                    : 'bg-zinc-900 text-zinc-600 cursor-not-allowed'
                }`}
                title="Previous QSO (←)"
              >
                <ChevronLeft className="h-5 w-5" />
              </button>
              <span className="text-sm text-zinc-400 min-w-[80px] text-center">
                {currentIndex + 1} of {totalCount}
              </span>
              <button
                onClick={() => canGoNext && onNavigate(currentIndex + 1)}
                disabled={!canGoNext}
                className={`p-2 rounded-lg transition-colors ${
                  canGoNext 
                    ? 'bg-zinc-800 hover:bg-zinc-700 text-zinc-100' 
                    : 'bg-zinc-900 text-zinc-600 cursor-not-allowed'
                }`}
                title="Next QSO (→)"
              >
                <ChevronRight className="h-5 w-5" />
              </button>
            </div>
          </div>

          <div className="flex gap-3">
            <button
              onClick={onClose}
              className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-100 rounded-lg transition-colors"
            >
              Close
            </button>
            <button
              className="flex items-center gap-2 px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded-lg transition-colors"
            >
              <Edit3 className="h-4 w-4" />
              <span>Edit QSO</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
