/**
 * QSO Log Filter Components
 * 
 * This file contains the filter panel for the QSO log table,
 * allowing users to filter by band, mode, and confirmation status.
 */

import { BAND_OPTIONS, MODE_OPTIONS, CONFIRM_OPTIONS } from "@/lib/constants";

// =============================================================================
// Types
// =============================================================================

/** Filter state for multi-select filters */
export interface FilterState {
  bands: Set<string>;
  modes: Set<string>;
  confirmStatus: Set<string>;
}

/** Create an empty filter state */
export function createEmptyFilterState(): FilterState {
  return {
    bands: new Set(),
    modes: new Set(),
    confirmStatus: new Set(),
  };
}

/** Count total active filters */
export function countActiveFilters(filters: FilterState): number {
  return filters.bands.size + filters.modes.size + filters.confirmStatus.size;
}

// =============================================================================
// Filter Panel Component
// =============================================================================

interface FiltersPanelProps {
  filters: FilterState;
  onToggleFilter: (type: keyof FilterState, value: string) => void;
  onClearAll: () => void;
  activeFilterCount: number;
}

/**
 * Dropdown panel containing filter options for bands, modes, and confirmation status
 */
export function FiltersPanel({ 
  filters, 
  onToggleFilter, 
  onClearAll,
  activeFilterCount 
}: FiltersPanelProps) {
  return (
    <div className="absolute left-0 mt-2 w-80 bg-zinc-800 border border-zinc-700 rounded-lg shadow-xl z-50">
      <div className="flex items-center justify-between px-4 py-3 border-b border-zinc-700">
        <span className="text-sm font-medium text-zinc-200">Filters</span>
        {activeFilterCount > 0 && (
          <button 
            onClick={onClearAll}
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
                onClick={() => onToggleFilter("bands", band)}
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
                onClick={() => onToggleFilter("modes", mode)}
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
                onClick={() => onToggleFilter("confirmStatus", opt.key)}
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
  );
}

