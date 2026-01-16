import { useState, useMemo, useEffect, useCallback, useRef } from "react";
import { useQsoStore, Qso, QsoStatus, parseAdifFields } from "@/stores/qsoStore";
import { invoke } from "@tauri-apps/api/core";
import { listen, emit } from "@tauri-apps/api/event";
import { Search, Download, Upload, Plus, ChevronUp, ChevronDown, X, Star, Settings2, Filter, GripVertical } from "lucide-react";
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  DragEndEvent,
} from "@dnd-kit/core";
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";

// Shared constants
import { BAND_ORDER, CONFIRM_OPTIONS } from "@/lib/constants";

// Extracted components
import { FilterState, countActiveFilters, FiltersPanel } from "@/components/QsoLogFilters";
import { QsoDetailModal } from "@/components/QsoDetailModal";
import { QsoBadges, ConfirmationBadges, formatDate, formatTime, formatEntity } from "@/components/QsoLogHelpers";

// Column definitions
import { ALL_COLUMNS, ColumnKey, SortField, SortableColumnItem } from "@/components/QsoLogColumns";

type SortDir = "asc" | "desc";

export function QsoLog() {
  const { qsos, setQsos, isLoading, setLoading } = useQsoStore();
  const [searchTerm, setSearchTerm] = useState("");
  const [sortField, setSortField] = useState<SortField>("qso_date");
  const [sortDir, setSortDir] = useState<SortDir>("desc");
  const [selectedQsoIndex, setSelectedQsoIndex] = useState<number | null>(null);
  const [qsoStatuses, setQsoStatuses] = useState<Map<number, QsoStatus>>(new Map());
  
  // Filter state - multi-select
  const [filters, setFilters] = useState<FilterState>({
    bands: new Set(),
    modes: new Set(),
    confirmStatus: new Set(),
  });
  const [showFiltersPanel, setShowFiltersPanel] = useState(false);
  const filtersPanelRef = useRef<HTMLDivElement>(null);
  
  // Column configuration - ordered array of visible columns
  const [columnOrder, setColumnOrder] = useState<ColumnKey[]>(() => {
    return ALL_COLUMNS.filter(c => c.defaultVisible).map(c => c.key);
  });
  const [showColumnModal, setShowColumnModal] = useState(false);
  const [columnOrderLoaded, setColumnOrderLoaded] = useState(false);

  // Load saved column order from settings
  useEffect(() => {
    const loadColumnOrder = async () => {
      try {
        const saved = await invoke<string | null>("get_setting", { key: "qso_log_columns" });
        if (saved) {
          const parsed = JSON.parse(saved) as ColumnKey[];
          // Validate that all required columns are present
          const required = ALL_COLUMNS.filter(c => c.required).map(c => c.key);
          const hasAllRequired = required.every(r => parsed.includes(r));
          // Validate all keys are valid
          const allValid = parsed.every(k => ALL_COLUMNS.some(c => c.key === k));
          if (hasAllRequired && allValid) {
            setColumnOrder(parsed);
          }
        }
      } catch (e) {
        console.error("Failed to load column order:", e);
      } finally {
        setColumnOrderLoaded(true);
      }
    };
    loadColumnOrder();
  }, []);

  // Save column order when it changes (after initial load)
  useEffect(() => {
    if (!columnOrderLoaded) return;
    const saveColumnOrder = async () => {
      try {
        await invoke("set_setting", { key: "qso_log_columns", value: JSON.stringify(columnOrder) });
      } catch (e) {
        console.error("Failed to save column order:", e);
      }
    };
    saveColumnOrder();
  }, [columnOrder, columnOrderLoaded]);

  const toggleFilter = (type: keyof FilterState, value: string) => {
    setFilters(prev => {
      const newSet = new Set(prev[type]);
      if (newSet.has(value)) {
        newSet.delete(value);
      } else {
        newSet.add(value);
      }
      return { ...prev, [type]: newSet };
    });
  };

  const clearAllFilters = () => {
    setFilters({ bands: new Set(), modes: new Set(), confirmStatus: new Set() });
  };

  const activeFilterCount = countActiveFilters(filters);
  
  const toggleColumnVisibility = (key: ColumnKey) => {
    const col = ALL_COLUMNS.find(c => c.key === key);
    if (col?.required) return; // Can't toggle required columns
    
    setColumnOrder(prev => {
      if (prev.includes(key)) {
        return prev.filter(k => k !== key);
      } else {
        return [...prev, key];
      }
    });
  };

  // dnd-kit sensors for drag and drop
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 5, // 5px movement required before drag starts
      },
    }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    
    if (over && active.id !== over.id) {
      setColumnOrder((items) => {
        const oldIndex = items.indexOf(active.id as ColumnKey);
        const newIndex = items.indexOf(over.id as ColumnKey);
        return arrayMove(items, oldIndex, newIndex);
      });
    }
  };

  // Close filters panel when clicking outside
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (filtersPanelRef.current && !filtersPanelRef.current.contains(event.target as Node)) {
        setShowFiltersPanel(false);
      }
    }
    if (showFiltersPanel) {
      document.addEventListener("mousedown", handleClickOutside);
      return () => document.removeEventListener("mousedown", handleClickOutside);
    }
  }, [showFiltersPanel]);

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

  // Listen for new QSOs from WSJT-X or ADIF import
  useEffect(() => {
    const unlistenLogged = listen("qso-logged", async () => {
      // Reload QSOs to get the newly inserted one with full data
      try {
        const data = await invoke<Qso[]>("get_qsos", { limit: 1000, offset: 0 });
        setQsos(data);
      } catch (e) {
        console.error("Failed to refresh QSOs:", e);
      }
    });
    
    const unlistenImported = listen("qsos-imported", async () => {
      // Reload QSOs after ADIF import
      try {
        const data = await invoke<Qso[]>("get_qsos", { limit: 1000, offset: 0 });
        setQsos(data);
      } catch (e) {
        console.error("Failed to refresh QSOs:", e);
      }
    });

    return () => {
      unlistenLogged.then(fn => fn());
      unlistenImported.then(fn => fn());
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
        setSelectedQsoIndex(null);
        setSearchTerm("");
      }
    };
    
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  // Filter and sort QSOs
  const filteredQsos = useMemo(() => {
    let result = qsos.filter((qso) => {
      // Text search
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
      
      // Band filter (multi-select)
      if (filters.bands.size > 0 && !filters.bands.has(qso.band)) return false;
      
      // Mode filter (multi-select)
      if (filters.modes.size > 0 && !filters.modes.has(qso.mode)) return false;
      
      // Confirmation status filter
      if (filters.confirmStatus.size > 0) {
        const isLotwConfirmed = qso.lotw_rcvd === "Y";
        const isEqslConfirmed = qso.eqsl_rcvd === "Y";
        const isUnconfirmed = !isLotwConfirmed && !isEqslConfirmed;
        
        const matchesConfirm = 
          (filters.confirmStatus.has("lotw") && isLotwConfirmed) ||
          (filters.confirmStatus.has("eqsl") && isEqslConfirmed) ||
          (filters.confirmStatus.has("unconfirmed") && isUnconfirmed);
        
        if (!matchesConfirm) return false;
      }
      
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
        case "gridsquare":
          cmp = (a.gridsquare || "").localeCompare(b.gridsquare || "");
          break;
        case "country":
          cmp = (a.country || "").localeCompare(b.country || "");
          break;
      }
      return sortDir === "asc" ? cmp : -cmp;
    });

    return result;
  }, [qsos, searchTerm, filters, sortField, sortDir]);

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
      {selectedQsoIndex !== null && filteredQsos[selectedQsoIndex] && (
        <QsoDetailModal 
          qso={filteredQsos[selectedQsoIndex]} 
          currentIndex={selectedQsoIndex}
          totalCount={filteredQsos.length}
          onClose={() => setSelectedQsoIndex(null)}
          onNavigate={(newIndex) => setSelectedQsoIndex(newIndex)}
          onDelete={() => {
            setSelectedQsoIndex(null);
            refreshQsos();
          }}
        />
      )}

      {/* Toolbar */}
      <div className="flex flex-wrap items-center gap-3">
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
        
        {/* Filters Button with Panel */}
        {/* Filters Button with Panel */}
        <div className="relative" ref={filtersPanelRef}>
          <button 
            onClick={() => setShowFiltersPanel(!showFiltersPanel)}
            className={`flex items-center gap-2 px-3 py-2 rounded-lg transition-colors border ${
              activeFilterCount > 0 
                ? "bg-sky-600/20 text-sky-400 border-sky-600/50 hover:bg-sky-600/30" 
                : "bg-zinc-800 text-zinc-100 border-zinc-700 hover:bg-zinc-700"
            }`}
          >
            <Filter className="h-4 w-4" />
            <span>Filters</span>
            {activeFilterCount > 0 && (
              <span className="bg-sky-600 text-white text-xs px-1.5 py-0.5 rounded-full min-w-[20px] text-center">
                {activeFilterCount}
              </span>
            )}
          </button>
          
          {showFiltersPanel && (
            <FiltersPanel
              filters={filters}
              onToggleFilter={toggleFilter}
              onClearAll={clearAllFilters}
              activeFilterCount={activeFilterCount}
            />
          )}
        </div>
        
        {/* Columns Button */}
        <button 
          onClick={() => setShowColumnModal(true)}
          className="flex items-center gap-2 px-3 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-100 rounded-lg transition-colors border border-zinc-700"
        >
          <Settings2 className="h-4 w-4" />
          <span>Columns</span>
        </button>
        
        <div className="flex gap-2">
          <button 
            onClick={() => emit('open-adif-import')}
            className="flex items-center gap-2 px-3 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-100 rounded-lg transition-colors border border-zinc-700"
          >
            <Upload className="h-4 w-4" />
            <span>Import</span>
          </button>
          <button 
            onClick={() => emit('open-adif-export')}
            className="flex items-center gap-2 px-3 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-100 rounded-lg transition-colors border border-zinc-700"
          >
            <Download className="h-4 w-4" />
            <span>Export</span>
          </button>
          <button className="flex items-center gap-2 px-3 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded-lg transition-colors">
            <Plus className="h-4 w-4" />
            <span>Add QSO</span>
          </button>
        </div>
      </div>
      
      {/* Active Filter Chips */}
      {activeFilterCount > 0 && (
        <div className="flex flex-wrap gap-2">
          {Array.from(filters.bands).map(band => (
            <span key={band} className="inline-flex items-center gap-1 px-2 py-1 bg-zinc-800 text-zinc-300 rounded text-sm">
              Band: {band}
              <button onClick={() => toggleFilter("bands", band)} className="text-zinc-500 hover:text-zinc-300">
                <X className="h-3 w-3" />
              </button>
            </span>
          ))}
          {Array.from(filters.modes).map(mode => (
            <span key={mode} className="inline-flex items-center gap-1 px-2 py-1 bg-zinc-800 text-zinc-300 rounded text-sm">
              Mode: {mode}
              <button onClick={() => toggleFilter("modes", mode)} className="text-zinc-500 hover:text-zinc-300">
                <X className="h-3 w-3" />
              </button>
            </span>
          ))}
          {Array.from(filters.confirmStatus).map(status => (
            <span key={status} className="inline-flex items-center gap-1 px-2 py-1 bg-zinc-800 text-zinc-300 rounded text-sm">
              {CONFIRM_OPTIONS.find(o => o.key === status)?.label}
              <button onClick={() => toggleFilter("confirmStatus", status)} className="text-zinc-500 hover:text-zinc-300">
                <X className="h-3 w-3" />
              </button>
            </span>
          ))}
        </div>
      )}
      
      {/* Column Configuration Modal */}
      {showColumnModal && (
        <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50" onClick={() => setShowColumnModal(false)}>
          <div 
            className="bg-zinc-900 border border-zinc-700 rounded-xl shadow-2xl w-full max-w-md mx-4"
            onClick={e => e.stopPropagation()}
          >
            <div className="flex items-center justify-between px-5 py-4 border-b border-zinc-700">
              <h3 className="text-lg font-semibold text-zinc-100">Configure Columns</h3>
              <button onClick={() => setShowColumnModal(false)} className="text-zinc-400 hover:text-zinc-200">
                <X className="h-5 w-5" />
              </button>
            </div>
            
            <div className="p-5">
              <p className="text-sm text-zinc-400 mb-4">
                Select which columns to show. Drag to reorder (required columns are locked).
              </p>
              
              <DndContext
                sensors={sensors}
                collisionDetection={closestCenter}
                onDragEnd={handleDragEnd}
              >
                <SortableContext items={columnOrder} strategy={verticalListSortingStrategy}>
                  <div className="space-y-1 max-h-[400px] overflow-y-auto">
                    {columnOrder.map((key) => (
                      <SortableColumnItem
                        key={key}
                        id={key}
                        onToggle={() => toggleColumnVisibility(key)}
                      />
                    ))}
                  </div>
                </SortableContext>
              </DndContext>
              
              {/* Hidden columns */}
              {ALL_COLUMNS.filter(c => !columnOrder.includes(c.key)).length > 0 && (
                <div className="mt-4 pt-4 border-t border-zinc-700">
                  <p className="text-xs text-zinc-500 mb-2">Hidden columns (click to add)</p>
                  <div className="space-y-1">
                    {ALL_COLUMNS.filter(c => !columnOrder.includes(c.key)).map(col => (
                      <div
                        key={col.key}
                        onClick={() => toggleColumnVisibility(col.key)}
                        className="flex items-center gap-3 px-3 py-2 rounded-lg bg-zinc-800/30 hover:bg-zinc-800 cursor-pointer"
                      >
                        <GripVertical className="h-4 w-4 text-zinc-700" />
                        <input
                          type="checkbox"
                          checked={false}
                          readOnly
                          className="rounded bg-zinc-700 border-zinc-600 text-sky-500 focus:ring-sky-500 pointer-events-none"
                        />
                        <span className="text-sm flex-1 text-zinc-500">{col.label}</span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
            
            <div className="flex justify-end gap-3 px-5 py-4 border-t border-zinc-700">
              <button
                onClick={() => {
                  setColumnOrder(ALL_COLUMNS.filter(c => c.defaultVisible).map(c => c.key));
                }}
                className="px-4 py-2 text-sm text-zinc-400 hover:text-zinc-200 transition-colors"
              >
                Reset to Default
              </button>
              <button
                onClick={() => setShowColumnModal(false)}
                className="px-4 py-2 text-sm bg-sky-600 hover:bg-sky-500 text-white rounded-lg transition-colors"
              >
                Done
              </button>
            </div>
          </div>
        </div>
      )}

      {/* QSO Table */}
      <div className="bg-zinc-900 rounded-lg border border-zinc-800 overflow-hidden">
        <table className="w-full">
          <thead className="bg-zinc-800/60">
            <tr>
              {columnOrder.map(key => {
                const col = ALL_COLUMNS.find(c => c.key === key);
                if (!col) return null;
                
                // Sortable columns
                const sortableColumns: Record<string, SortField> = {
                  qso_date: "qso_date",
                  call: "call",
                  band: "band",
                  mode: "mode",
                  gridsquare: "gridsquare",
                  country: "country",
                };
                const sortField = sortableColumns[key];
                
                if (key === "status") {
                  return (
                    <th key={key} className="w-12 px-2 py-3 text-center text-sm font-medium text-zinc-400">
                      <span title="Status badges">🏷️</span>
                    </th>
                  );
                }
                
                return (
                  <th 
                    key={key}
                    className={`px-4 py-3 text-left text-sm font-medium text-zinc-400 ${
                      sortField ? "cursor-pointer hover:text-zinc-200" : ""
                    }`}
                    onClick={sortField ? () => handleSort(sortField) : undefined}
                  >
                    {col.label} {sortField && <SortIcon field={sortField} />}
                  </th>
                );
              })}
            </tr>
          </thead>
          <tbody>
            {filteredQsos.length === 0 ? (
              <tr>
                <td colSpan={columnOrder.length} className="px-4 py-12 text-center text-zinc-500">
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
              filteredQsos.map((qso, index) => {
                const status = qsoStatuses.get(qso.id);
                return (
                  <tr 
                    key={qso.id} 
                    className="border-t border-zinc-800 hover:bg-zinc-800/50 cursor-pointer"
                    onClick={() => setSelectedQsoIndex(index)}
                  >
                    {columnOrder.map(key => {
                      switch (key) {
                        case "status":
                          return (
                            <td key={key} className="px-2 py-3 text-center">
                              <QsoBadges status={status} />
                            </td>
                          );
                        case "qso_date":
                          return (
                            <td key={key} className="px-4 py-3 text-sm text-zinc-300">
                              {formatDate(qso.qso_date)} {formatTime(qso.time_on)}
                            </td>
                          );
                        case "call":
                          return (
                            <td key={key} className="px-4 py-3 text-sm font-medium text-sky-400">
                              <span className="flex items-center gap-1">
                                {qso.call}
                                {status?.has_previous_qso && (
                                  <span title={`Worked ${status.previous_qso_count + 1} times`}>
                                    <Star className="h-3 w-3 text-amber-500 fill-amber-500" />
                                  </span>
                                )}
                              </span>
                            </td>
                          );
                        case "band":
                          return <td key={key} className="px-4 py-3 text-sm text-zinc-300">{qso.band}</td>;
                        case "mode":
                          return <td key={key} className="px-4 py-3 text-sm text-zinc-300">{qso.mode}</td>;
                        case "gridsquare":
                          return <td key={key} className="px-4 py-3 text-sm text-zinc-400">{qso.gridsquare || "-"}</td>;
                        case "rst":
                          return (
                            <td key={key} className="px-4 py-3 text-sm text-zinc-400">
                              {qso.rst_sent || "-"}/{qso.rst_rcvd || "-"}
                            </td>
                          );
                        case "country":
                          return <td key={key} className="px-4 py-3 text-sm text-zinc-300">{formatEntity(qso)}</td>;
                        case "state":
                          return <td key={key} className="px-4 py-3 text-sm text-zinc-400">{qso.state || "-"}</td>;
                        case "cont":
                          return <td key={key} className="px-4 py-3 text-sm text-zinc-400">{qso.continent || "-"}</td>;
                        case "cqz":
                          return <td key={key} className="px-4 py-3 text-sm text-zinc-400">{qso.cqz || "-"}</td>;
                        case "ituz":
                          return <td key={key} className="px-4 py-3 text-sm text-zinc-400">{qso.ituz || "-"}</td>;
                        case "confirm":
                          return (
                            <td key={key} className="px-4 py-3 text-sm">
                              <ConfirmationBadges lotw={qso.lotw_rcvd} eqsl={qso.eqsl_rcvd} />
                            </td>
                          );
                        default:
                          return null;
                      }
                    })}
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
