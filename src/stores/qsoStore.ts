import { create } from "zustand";

export interface Qso {
  id: number;
  uuid: string;
  
  // Core fields
  call: string;
  qso_date: string;
  qso_date_off?: string;
  time_on: string;
  time_off?: string;
  band: string;
  mode: string;
  freq?: number;
  
  // Location
  dxcc?: number;
  country?: string;
  continent?: string;
  state?: string;
  gridsquare?: string;
  cqz?: number;
  ituz?: number;
  
  // Signal reports
  rst_sent?: string;
  rst_rcvd?: string;
  
  // My station
  station_callsign?: string;
  operator?: string;
  my_gridsquare?: string;
  tx_pwr?: number;
  
  // Extended fields (JSON blobs)
  adif_fields?: string;
  user_data?: string;
  
  // Metadata
  source: string;
  created_at: string;
  updated_at: string;
  
  // Confirmation status (from confirmations table via JOIN)
  lotw_rcvd?: string;   // "Y" if confirmed via LoTW
  eqsl_rcvd?: string;   // "Y" if confirmed via eQSL
}

// Parsed ADIF fields from the JSON blob
export interface AdifFields {
  name?: string;
  qth?: string;
  comments?: string;
  prop_mode?: string;
  sota_ref?: string;
  pota_ref?: string;
  iota?: string;
  wwff_ref?: string;
  rig?: string;
  antenna?: string;
  operator?: string;
  contest_id?: string;
  srx?: string;
  stx?: string;
  exchange_sent?: string;
  exchange_rcvd?: string;
}

// Helper to parse adif_fields JSON
export function parseAdifFields(qso: Qso): AdifFields {
  if (!qso.adif_fields) return {};
  try {
    return JSON.parse(qso.adif_fields);
  } catch {
    return {};
  }
}

// Previous QSO summary (from get_callsign_history)
export interface PreviousQso {
  id: number;
  qso_date: string;
  time_on: string;
  band: string;
  mode: string;
  rst_sent?: string;
  rst_rcvd?: string;
}

export interface CallsignHistory {
  call: string;
  total_qsos: number;
  bands_worked: string[];
  modes_worked: string[];
  first_qso?: string;
  last_qso?: string;
  previous_qsos: PreviousQso[];
}

// QSO status flags (from check_qso_status)
export interface QsoStatus {
  is_dupe: boolean;
  is_new_dxcc: boolean;
  is_new_band_dxcc: boolean;
  is_new_mode_dxcc: boolean;
  has_previous_qso: boolean;
  previous_qso_count: number;
}

interface QsoStore {
  qsos: Qso[];
  isLoading: boolean;
  error: string | null;
  
  // Actions
  setQsos: (qsos: Qso[]) => void;
  addQso: (qso: Qso) => void;
  updateQso: (id: number, updates: Partial<Qso>) => void;
  removeQso: (id: number) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
}

export const useQsoStore = create<QsoStore>((set) => ({
  qsos: [],
  isLoading: false,
  error: null,

  setQsos: (qsos) => set({ qsos }),
  
  addQso: (qso) => set((state) => ({ 
    qsos: [qso, ...state.qsos] 
  })),
  
  updateQso: (id, updates) => set((state) => ({
    qsos: state.qsos.map((q) => q.id === id ? { ...q, ...updates } : q)
  })),
  
  removeQso: (id) => set((state) => ({ 
    qsos: state.qsos.filter((q) => q.id !== id) 
  })),
  
  setLoading: (isLoading) => set({ isLoading }),
  
  setError: (error) => set({ error }),
}));
