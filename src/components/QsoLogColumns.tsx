/**
 * QSO Log Column Definitions and Components
 * 
 * This file contains:
 * - Column configuration for the QSO log table
 * - Sortable column item component for drag-and-drop reordering
 * - Type definitions for columns
 */

import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { GripVertical } from "lucide-react";

/**
 * All available columns in the QSO log table
 * - required: Column cannot be hidden
 * - defaultVisible: Column is visible by default
 */
export const ALL_COLUMNS = [
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

/** Column key type derived from ALL_COLUMNS */
export type ColumnKey = typeof ALL_COLUMNS[number]["key"];

/** Sortable field type (subset of columns that support sorting) */
export type SortField = "qso_date" | "call" | "band" | "mode" | "gridsquare" | "country";

/** Sort direction */
export type SortDir = "asc" | "desc";

/** Get the default visible columns */
export function getDefaultColumnOrder(): ColumnKey[] {
  return ALL_COLUMNS.filter(c => c.defaultVisible).map(c => c.key);
}

/** Validate a column order array (ensures required columns are present) */
export function validateColumnOrder(order: ColumnKey[]): boolean {
  const required = ALL_COLUMNS.filter(c => c.required).map(c => c.key);
  const hasAllRequired = required.every(r => order.includes(r));
  const allValid = order.every(k => ALL_COLUMNS.some(c => c.key === k));
  return hasAllRequired && allValid;
}

// =============================================================================
// Sortable Column Item Component
// =============================================================================

interface SortableColumnItemProps {
  id: ColumnKey;
  onToggle: () => void;
}

/**
 * Sortable column item for drag-and-drop reordering in the column configuration modal
 */
export function SortableColumnItem({ id, onToggle }: SortableColumnItemProps) {
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
