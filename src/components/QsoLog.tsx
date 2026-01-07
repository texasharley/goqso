import { useState, useMemo, useEffect, useCallback, useRef } from "react";
import { useQsoStore, Qso, parseAdifFields, CallsignHistory, QsoStatus } from "@/stores/qsoStore";
import { invoke } from "@tauri-apps/api/core";
import { listen, emit } from "@tauri-apps/api/event";
import { Search, Download, Upload, Plus, ChevronUp, ChevronDown, ChevronLeft, ChevronRight, X, Star, Sparkles, RefreshCw, History, Trash2, Edit3, Settings2, Filter, GripVertical } from "lucide-react";
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
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";

type SortField = "qso_date" | "call" | "band" | "mode" | "gridsquare" | "country";

// All available columns with their configuration
const ALL_COLUMNS = [
  { key: "status", label: "Status", required: true, defaultVisible: true },
  { key: "qso_date", label: "Start Time (UTC)", required: true, defaultVisible: true },
  { key: "call", label: "Call", required: true, defaultVisible: true },
  { key: "band", label: "Band", required: false, defaultVisible: true },
  { key: "mode", label: "Mode", required: false, defaultVisible: true },
  { key: "gridsquare", label: "Grid", required: false, defaultVisible: true },
  { key: "rst", label: "RST (Sent/Rcvd)", required: false, defaultVisible: true },
  { key: "country", label: "Entity", required: false, defaultVisible: true },
  { key: "state", label: "State/Province", required: false, defaultVisible: false },
  { key: "cont", label: "Continent", required: false, defaultVisible: false },
  { key: "cqz", label: "CQ Zone", required: false, defaultVisible: false },
  { key: "ituz", label: "ITU Zone", required: false, defaultVisible: false },
  { key: "confirm", label: "Confirm", required: false, defaultVisible: true },
] as const;

type ColumnKey = typeof ALL_COLUMNS[number]["key"];

// Available filter options
const BAND_OPTIONS = ["160m", "80m", "60m", "40m", "30m", "20m", "17m", "15m", "12m", "10m", "6m", "2m", "70cm"];
const MODE_OPTIONS = ["FT8", "FT4", "CW", "SSB", "RTTY", "PSK31", "JS8"];
const CONFIRM_OPTIONS = [
  { key: "lotw", label: "LoTW Confirmed" },
  { key: "eqsl", label: "eQSL Confirmed" },
  { key: "unconfirmed", label: "Unconfirmed" },
];

type SortDir = "asc" | "desc";

// Band sort order
const BAND_ORDER: Record<string, number> = {
  "160m": 1, "80m": 2, "60m": 3, "40m": 4, "30m": 5, "20m": 6,
  "17m": 7, "15m": 8, "12m": 9, "10m": 10, "6m": 11, "2m": 12, "70cm": 13
};

// Filter state interface
interface FilterState {
  bands: Set<string>;
  modes: Set<string>;
  confirmStatus: Set<string>;
}

// Sortable column item for drag-and-drop reordering
interface SortableColumnItemProps {
  id: ColumnKey;
  onToggle: () => void;
}

function SortableColumnItem({ id, onToggle }: SortableColumnItemProps) {
  const col = ALL_COLUMNS.find(c => c.key === id);
  
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ 
    id, 
    disabled: col?.required ?? false 
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  if (!col) return null;

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={`flex items-center gap-3 px-3 py-2 rounded-lg transition-all ${
        col.required 
          ? "bg-zinc-800/50 opacity-60" 
          : isDragging
            ? "bg-sky-600/30 border border-sky-500 z-50 shadow-lg"
            : "bg-zinc-800 hover:bg-zinc-700"
      }`}
    >
      {/* Drag handle */}
      <div
        {...attributes}
        {...listeners}
        className={`p-1 -m-1 touch-none ${col.required ? "cursor-not-allowed" : "cursor-grab active:cursor-grabbing"}`}
      >
        <GripVertical className={`h-4 w-4 ${col.required ? "text-zinc-600" : "text-zinc-400 hover:text-zinc-200"}`} />
      </div>
      <input
        type="checkbox"
        checked={true}
        disabled={col.required}
        onChange={onToggle}
        className="rounded bg-zinc-700 border-zinc-600 text-sky-500 focus:ring-sky-500"
      />
      <span className={`text-sm flex-1 ${col.required ? "text-zinc-500" : "text-zinc-200"}`}>
        {col.label}
      </span>
      {col.required && (
        <span className="text-xs text-zinc-600">Required</span>
      )}
    </div>
  );
}

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

  const activeFilterCount = filters.bands.size + filters.modes.size + filters.confirmStatus.size;
  
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
            <div className="absolute left-0 mt-2 w-80 bg-zinc-800 border border-zinc-700 rounded-lg shadow-xl z-50">
              <div className="flex items-center justify-between px-4 py-3 border-b border-zinc-700">
                <span className="text-sm font-medium text-zinc-200">Filters</span>
                {activeFilterCount > 0 && (
                  <button 
                    onClick={clearAllFilters}
                    className="text-xs text-sky-400 hover:text-sky-300"
                  >
                    Clear all
                  </button>
                )}
              </div>
              
              <div className="p-4 space-y-4 max-h-[400px] overflow-y-auto">
                {/* Band Filter */}
                <div>
                  <div className="text-xs font-medium text-zinc-400 uppercase tracking-wide mb-2">Band</div>
                  <div className="flex flex-wrap gap-1.5">
                    {BAND_OPTIONS.map(band => (
                      <button
                        key={band}
                        onClick={() => toggleFilter("bands", band)}
                        className={`px-2 py-1 text-xs rounded transition-colors ${
                          filters.bands.has(band)
                            ? "bg-sky-600 text-white"
                            : "bg-zinc-700 text-zinc-300 hover:bg-zinc-600"
                        }`}
                      >
                        {band}
                      </button>
                    ))}
                  </div>
                </div>
                
                {/* Mode Filter */}
                <div>
                  <div className="text-xs font-medium text-zinc-400 uppercase tracking-wide mb-2">Mode</div>
                  <div className="flex flex-wrap gap-1.5">
                    {MODE_OPTIONS.map(mode => (
                      <button
                        key={mode}
                        onClick={() => toggleFilter("modes", mode)}
                        className={`px-2 py-1 text-xs rounded transition-colors ${
                          filters.modes.has(mode)
                            ? "bg-sky-600 text-white"
                            : "bg-zinc-700 text-zinc-300 hover:bg-zinc-600"
                        }`}
                      >
                        {mode}
                      </button>
                    ))}
                  </div>
                </div>
                
                {/* Confirmation Status */}
                <div>
                  <div className="text-xs font-medium text-zinc-400 uppercase tracking-wide mb-2">Confirmation</div>
                  <div className="flex flex-wrap gap-1.5">
                    {CONFIRM_OPTIONS.map(opt => (
                      <button
                        key={opt.key}
                        onClick={() => toggleFilter("confirmStatus", opt.key)}
                        className={`px-2 py-1 text-xs rounded transition-colors ${
                          filters.confirmStatus.has(opt.key)
                            ? "bg-sky-600 text-white"
                            : "bg-zinc-700 text-zinc-300 hover:bg-zinc-600"
                        }`}
                      >
                        {opt.label}
                      </button>
                    ))}
                  </div>
                </div>
              </div>
            </div>
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

interface ConfirmationBadgesProps {
  lotw?: string;
  eqsl?: string;
}

function ConfirmationBadges({ lotw, eqsl }: ConfirmationBadgesProps) {
  const lotwConfirmed = lotw === "Y";
  const eqslConfirmed = eqsl === "Y";
  
  return (
    <span className="flex items-center gap-1 text-xs font-medium">
      <span 
        className={lotwConfirmed ? "text-green-500" : "text-zinc-600"} 
        title={lotwConfirmed ? "✓ Confirmed on LoTW" : "Not confirmed on LoTW"}
      >
        L
      </span>
      <span 
        className={eqslConfirmed ? "text-green-500" : "text-zinc-600"} 
        title={eqslConfirmed ? "✓ Confirmed on eQSL" : "Not confirmed on eQSL"}
      >
        e
      </span>
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
  // Per ADIF 3.1.4 spec:
  // - COUNTRY field = DXCC entity name (e.g., "UNITED STATES OF AMERICA")
  // - STATE field = Primary Administrative Subdivision code (e.g., "MN")
  // Entity column shows just the country name, State/Province column shows state
  const country = qso.country?.trim();
  if (country) return country;
  return "-";
}

// =============================================================================
// QSO Detail Modal with History Panel
// =============================================================================

interface QsoDetailModalProps {
  qso: Qso;
  currentIndex: number;
  totalCount: number;
  onClose: () => void;
  onNavigate: (index: number) => void;
  onDelete: () => void;
}

function QsoDetailModal({ qso, currentIndex, totalCount, onClose, onNavigate, onDelete }: QsoDetailModalProps) {
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
