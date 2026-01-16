/**
 * QSO Log Helper Components and Functions
 * 
 * Contains badge components and formatting functions used by QsoLog table.
 */

import { RefreshCw, Sparkles } from "lucide-react";
import { QsoStatus } from "@/stores/qsoStore";

// =============================================================================
// Badge Components
// =============================================================================

interface QsoBadgesProps {
  status?: QsoStatus;
}

/**
 * Badge component showing QSO status (duplicate, new DXCC, new band slot)
 */
export function QsoBadges({ status }: QsoBadgesProps) {
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

/**
 * Badge component showing LoTW and eQSL confirmation status
 */
export function ConfirmationBadges({ lotw, eqsl }: ConfirmationBadgesProps) {
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
// Formatting Functions
// =============================================================================

/**
 * Format ADIF date (YYYYMMDD) to display format (YYYY-MM-DD)
 */
export function formatDate(date: string): string {
  if (!date) return "";
  if (date.includes("-")) return date;
  if (date.length === 8) {
    return `${date.slice(0, 4)}-${date.slice(4, 6)}-${date.slice(6, 8)}`;
  }
  return date;
}

/**
 * Format ADIF time (HHMMSS) to display format (HH:MM)
 */
export function formatTime(time: string): string {
  if (!time || time.length < 4) return "";
  return `${time.slice(0, 2)}:${time.slice(2, 4)}`;
}

/**
 * Format entity name for display
 * Per ADIF 3.1.4 spec:
 * - COUNTRY field = DXCC entity name
 * - STATE field = Primary Administrative Subdivision code
 */
export function formatEntity(qso: { country?: string; state?: string; gridsquare?: string }): string {
  const country = qso.country?.trim();
  if (country) return country;
  return "-";
}
