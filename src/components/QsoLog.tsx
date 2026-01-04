import { useState, useMemo, useEffect, useCallback } from "react";
import { useQsoStore, Qso, parseAdifFields, CallsignHistory, QsoStatus } from "@/stores/qsoStore";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Search, Download, Upload, Plus, ChevronUp, ChevronDown, X, FlaskConical, Star, Sparkles, RefreshCw, History, Trash2, Edit3 } from "lucide-react";

type SortField = "qso_date" | "call" | "band" | "mode" | "country";
type SortDir = "asc" | "desc";

// Band sort order
const BAND_ORDER: Record<string, number> = {
  "160m": 1, "80m": 2, "60m": 3, "40m": 4, "30m": 5, "20m": 6,
  "17m": 7, "15m": 8, "12m": 9, "10m": 10, "6m": 11, "2m": 12, "70cm": 13
};

export function QsoLog() {
  const { qsos, setQsos, isLoading, setLoading } = useQsoStore();
  const [searchTerm, setSearchTerm] = useState("");
  const [bandFilter, setBandFilter] = useState<string>("all");
  const [modeFilter, setModeFilter] = useState<string>("all");
  const [sortField, setSortField] = useState<SortField>("qso_date");
  const [sortDir, setSortDir] = useState<SortDir>("desc");
  const [selectedQso, setSelectedQso] = useState<Qso | null>(null);
  const [qsoStatuses, setQsoStatuses] = useState<Map<number, QsoStatus>>(new Map());

  // Load QSOs on mount
  useEffect(() => {
    const loadQsos = async () => {
      setLoading(true);
      try {
        // Wait for DB to be ready
        let ready = false;
        for (let i = 0; i < 30; i++) {
          ready = await invoke<boolean>("is_db_ready");
          if (ready) break;
          await new Promise(r => setTimeout(r, 100));
        }
        
        if (ready) {
          const data = await invoke<Qso[]>("get_qsos", { limit: 1000, offset: 0 });
          setQsos(data);
        }
      } catch (e) {
        console.error("Failed to load QSOs:", e);
      } finally {
        setLoading(false);
      }
    };
    
    loadQsos();
  }, [setQsos, setLoading]);

  // Load QSO statuses when qsos change
  useEffect(() => {
    const loadStatuses = async () => {
      const statusMap = new Map<number, QsoStatus>();
      
      // Process in batches to avoid overwhelming
      for (const qso of qsos.slice(0, 100)) { // Limit to first 100 for performance
        try {
          const status = await invoke<QsoStatus>("check_qso_status", {
            call: qso.call,
            band: qso.band,
            mode: qso.mode,
            dxcc: qso.dxcc,
            qsoDate: qso.qso_date,
            excludeId: qso.id,
          });
          statusMap.set(qso.id, status);
        } catch (e) {
          console.error(`Failed to get status for QSO ${qso.id}:`, e);
        }
      }
      
      setQsoStatuses(statusMap);
    };
    
    if (qsos.length > 0) {
      loadStatuses();
    }
  }, [qsos]);

  // Listen for new QSOs from WSJT-X
  useEffect(() => {
    const unlisten = listen("qso-logged", async () => {
      // Reload QSOs to get the newly inserted one with full data
      try {
        const data = await invoke<Qso[]>("get_qsos", { limit: 1000, offset: 0 });
        setQsos(data);
      } catch (e) {
        console.error("Failed to refresh QSOs:", e);
      }
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, [setQsos]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Don't capture if typing in input
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
      
      if (e.key === "/" && !e.ctrlKey) {
        e.preventDefault();
        document.getElementById("qso-search")?.focus();
      } else if (e.key === "Escape") {
        setSelectedQso(null);
        setSearchTerm("");
      }
    };
    
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  // Filter and sort QSOs
  const filteredQsos = useMemo(() => {
    let result = qsos.filter((qso) => {
      const term = searchTerm.toLowerCase();
      if (term) {
        const matchCall = qso.call.toLowerCase().includes(term);
        const matchCountry = qso.country?.toLowerCase().includes(term);
        const matchGrid = qso.gridsquare?.toLowerCase().includes(term);
        const matchState = qso.state?.toLowerCase().includes(term);
        const adif = parseAdifFields(qso);
        const matchName = adif.name?.toLowerCase().includes(term);
        const matchComments = adif.comments?.toLowerCase().includes(term);
        if (!matchCall && !matchCountry && !matchGrid && !matchState && !matchName && !matchComments) return false;
      }
      if (bandFilter !== "all" && qso.band !== bandFilter) return false;
      if (modeFilter !== "all" && qso.mode !== modeFilter) return false;
      return true;
    });

    // Sort
    result.sort((a, b) => {
      let cmp = 0;
      switch (sortField) {
        case "qso_date":
          cmp = (a.qso_date + a.time_on).localeCompare(b.qso_date + b.time_on);
          break;
        case "call":
          cmp = a.call.localeCompare(b.call);
          break;
        case "band":
          cmp = (BAND_ORDER[a.band] || 99) - (BAND_ORDER[b.band] || 99);
          break;
        case "mode":
          cmp = a.mode.localeCompare(b.mode);
          break;
        case "country":
          cmp = (a.country || "").localeCompare(b.country || "");
          break;
      }
      return sortDir === "asc" ? cmp : -cmp;
    });

    return result;
  }, [qsos, searchTerm, bandFilter, modeFilter, sortField, sortDir]);

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDir(sortDir === "asc" ? "desc" : "asc");
    } else {
      setSortField(field);
      setSortDir("desc");
    }
  };

  const SortIcon = ({ field }: { field: SortField }) => {
    if (sortField !== field) return null;
    return sortDir === "asc" ? 
      <ChevronUp className="inline h-3 w-3 ml-1" /> : 
      <ChevronDown className="inline h-3 w-3 ml-1" />;
  };

  const handleAddTestData = useCallback(async () => {
    try {
      let ready = false;
      for (let i = 0; i < 30; i++) {
        ready = await invoke<boolean>("is_db_ready");
        if (ready) break;
        await new Promise(r => setTimeout(r, 100));
      }
      if (!ready) {
        console.error("Database not ready after 3 seconds");
        return;
      }
      await invoke("add_test_qsos");
      const data = await invoke<Qso[]>("get_qsos", { limit: 1000, offset: 0 });
      setQsos(data);
    } catch (e) {
      console.error("Failed to add test QSOs:", e);
    }
  }, [setQsos]);

  const refreshQsos = useCallback(async () => {
    try {
      const data = await invoke<Qso[]>("get_qsos", { limit: 1000, offset: 0 });
      setQsos(data);
    } catch (e) {
      console.error("Failed to refresh QSOs:", e);
    }
  }, [setQsos]);

  return (
    <div className="space-y-4">
      {/* QSO Detail Modal */}
      {selectedQso && (
        <QsoDetailModal 
          qso={selectedQso} 
          onClose={() => setSelectedQso(null)}
          onDelete={() => {
            setSelectedQso(null);
            refreshQsos();
          }}
        />
      )}

      {/* Toolbar */}
      <div className="flex flex-wrap items-center gap-4">
        <div className="relative flex-1 min-w-[200px]">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-zinc-500" />
          <input
            id="qso-search"
            type="text"
            placeholder="Search (press / to focus)..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="w-full pl-10 pr-4 py-2 bg-zinc-800 text-zinc-100 rounded-lg border border-zinc-700 focus:outline-none focus:ring-2 focus:ring-sky-500"
          />
        </div>
        <select
          value={bandFilter}
          onChange={(e) => setBandFilter(e.target.value)}
          className="px-3 py-2 bg-zinc-800 text-zinc-100 rounded-lg border border-zinc-700"
        >
          <option value="all">All Bands</option>
          <option value="160m">160m</option>
          <option value="80m">80m</option>
          <option value="40m">40m</option>
          <option value="30m">30m</option>
          <option value="20m">20m</option>
          <option value="17m">17m</option>
          <option value="15m">15m</option>
          <option value="12m">12m</option>
          <option value="10m">10m</option>
          <option value="6m">6m</option>
          <option value="2m">2m</option>
        </select>
        <select
          value={modeFilter}
          onChange={(e) => setModeFilter(e.target.value)}
          className="px-3 py-2 bg-zinc-800 text-zinc-100 rounded-lg border border-zinc-700"
        >
          <option value="all">All Modes</option>
          <option value="FT8">FT8</option>
          <option value="FT4">FT4</option>
          <option value="CW">CW</option>
          <option value="SSB">SSB</option>
        </select>
        <div className="flex gap-2">
          <button className="flex items-center gap-2 px-3 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-100 rounded-lg transition-colors border border-zinc-700">
            <Upload className="h-4 w-4" />
            <span>Import</span>
          </button>
          <button className="flex items-center gap-2 px-3 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-100 rounded-lg transition-colors border border-zinc-700">
            <Download className="h-4 w-4" />
            <span>Export</span>
          </button>
          <button className="flex items-center gap-2 px-3 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded-lg transition-colors">
            <Plus className="h-4 w-4" />
            <span>Add QSO</span>
          </button>
          {/* Test data button - development only */}
          <button 
            onClick={handleAddTestData}
            className="flex items-center gap-2 px-3 py-2 bg-amber-700 hover:bg-amber-600 text-white rounded-lg transition-colors"
            title="Add test QSOs for development"
          >
            <FlaskConical className="h-4 w-4" />
            <span>Test Data</span>
          </button>
        </div>
      </div>

      {/* QSO Table */}
      <div className="bg-zinc-900 rounded-lg border border-zinc-800 overflow-hidden">
        <table className="w-full">
          <thead className="bg-zinc-800/60">
            <tr>
              <th className="w-12 px-2 py-3 text-center text-sm font-medium text-zinc-400">
                <span title="Status badges">🏷️</span>
              </th>
              <th 
                className="px-4 py-3 text-left text-sm font-medium text-zinc-400 cursor-pointer hover:text-zinc-200"
                onClick={() => handleSort("qso_date")}
              >
                Date/Time <SortIcon field="qso_date" />
              </th>
              <th 
                className="px-4 py-3 text-left text-sm font-medium text-zinc-400 cursor-pointer hover:text-zinc-200"
                onClick={() => handleSort("call")}
              >
                Call <SortIcon field="call" />
              </th>
              <th 
                className="px-4 py-3 text-left text-sm font-medium text-zinc-400 cursor-pointer hover:text-zinc-200"
                onClick={() => handleSort("band")}
              >
                Band <SortIcon field="band" />
              </th>
              <th 
                className="px-4 py-3 text-left text-sm font-medium text-zinc-400 cursor-pointer hover:text-zinc-200"
                onClick={() => handleSort("mode")}
              >
                Mode <SortIcon field="mode" />
              </th>
              <th className="px-4 py-3 text-left text-sm font-medium text-zinc-400">RST</th>
              <th 
                className="px-4 py-3 text-left text-sm font-medium text-zinc-400 cursor-pointer hover:text-zinc-200"
                onClick={() => handleSort("country")}
              >
                Entity <SortIcon field="country" />
              </th>
              <th className="px-4 py-3 text-left text-sm font-medium text-zinc-400">Confirm</th>
            </tr>
          </thead>
          <tbody>
            {filteredQsos.length === 0 ? (
              <tr>
                <td colSpan={8} className="px-4 py-12 text-center text-zinc-500">
                  {isLoading ? (
                    <span>Loading...</span>
                  ) : qsos.length === 0 ? (
                    <div className="flex flex-col items-center gap-2">
                      <span className="text-2xl">📻</span>
                      <span>No QSOs yet</span>
                      <span className="text-xs">Complete a QSO in WSJT-X to see it here</span>
                    </div>
                  ) : (
                    <span>No QSOs match your filters</span>
                  )}
                </td>
              </tr>
            ) : (
              filteredQsos.map((qso) => {
                const status = qsoStatuses.get(qso.id);
                return (
                  <tr 
                    key={qso.id} 
                    className="border-t border-zinc-800 hover:bg-zinc-800/50 cursor-pointer"
                    onClick={() => setSelectedQso(qso)}
                  >
                    <td className="px-2 py-3 text-center">
                      <QsoBadges status={status} />
                    </td>
                    <td className="px-4 py-3 text-sm text-zinc-300">
                      {formatDate(qso.qso_date)} {formatTime(qso.time_on)}
                    </td>
                    <td className="px-4 py-3 text-sm font-medium text-sky-400">
                      <span className="flex items-center gap-1">
                        {qso.call}
                        {status?.has_previous_qso && (
                          <span title={`Worked ${status.previous_qso_count + 1} times`}>
                            <Star className="h-3 w-3 text-amber-500 fill-amber-500" />
                          </span>
                        )}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-sm text-zinc-300">{qso.band}</td>
                    <td className="px-4 py-3 text-sm text-zinc-300">{qso.mode}</td>
                    <td className="px-4 py-3 text-sm text-zinc-400">
                      {qso.rst_sent || "-"}/{qso.rst_rcvd || "-"}
                    </td>
                    <td className="px-4 py-3 text-sm text-zinc-300">
                      {formatEntity(qso)}
                    </td>
                    <td className="px-4 py-3 text-sm">
                      <ConfirmationBadges />
                    </td>
                  </tr>
                );
              })
            )}
          </tbody>
        </table>
      </div>

      {/* Footer */}
      <div className="flex items-center justify-between text-sm text-zinc-500">
        <span>
          {filteredQsos.length === qsos.length 
            ? `${qsos.length} QSOs` 
            : `${filteredQsos.length} of ${qsos.length} QSOs`}
        </span>
        <div className="flex items-center gap-4">
          {searchTerm && (
            <button 
              onClick={() => setSearchTerm("")}
              className="text-sky-500 hover:text-sky-400"
            >
              Clear search
            </button>
          )}
          <span className="text-zinc-600">Press / to search • Click row for details</span>
        </div>
      </div>
    </div>
  );
}

// =============================================================================
// Badge Components
// =============================================================================

function QsoBadges({ status }: { status?: QsoStatus }) {
  if (!status) return <span className="text-zinc-700">-</span>;
  
  return (
    <span className="flex items-center justify-center gap-0.5">
      {status.is_dupe && (
        <span title="Duplicate">
          <RefreshCw className="h-3.5 w-3.5 text-orange-500" />
        </span>
      )}
      {status.is_new_dxcc && (
        <span title="New DXCC!">
          <Sparkles className="h-3.5 w-3.5 text-green-500" />
        </span>
      )}
      {status.is_new_band_dxcc && !status.is_new_dxcc && (
        <span className="text-xs text-emerald-400 font-bold" title="New band slot">🎯</span>
      )}
    </span>
  );
}

function ConfirmationBadges() {
  // For now, placeholder - will be enhanced when we add confirmation tracking
  return (
    <span className="flex items-center gap-1 text-xs">
      <span className="text-zinc-600" title="LoTW: Not uploaded">L</span>
      <span className="text-zinc-600" title="eQSL: Not sent">e</span>
    </span>
  );
}

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

function formatEntity(qso: { country?: string; state?: string; gridsquare?: string }): string {
  if (qso.state && qso.country === "United States") {
    return `${qso.state}, US`;
  }
  if (qso.country) return qso.country;
  if (qso.gridsquare) return qso.gridsquare;
  return "-";
}

// =============================================================================
// QSO Detail Modal with History Panel
// =============================================================================

interface QsoDetailModalProps {
  qso: Qso;
  onClose: () => void;
  onDelete: () => void;
}

function QsoDetailModal({ qso, onClose, onDelete }: QsoDetailModalProps) {
  const adif = parseAdifFields(qso);
  const [history, setHistory] = useState<CallsignHistory | null>(null);
  const [status, setStatus] = useState<QsoStatus | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

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
                    <span className="text-zinc-600">Not uploaded</span>
                  </div>
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-zinc-400">eQSL</span>
                    <span className="text-zinc-600">Not sent</span>
                  </div>
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-zinc-400">QRZ</span>
                    <span className="text-zinc-600">Not sent</span>
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
