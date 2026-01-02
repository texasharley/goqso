import { create } from "zustand";

export interface Qso {
  id: number;
  uuid: string;
  call: string;
  qso_date: string;
  time_on: string;
  band: string;
  mode: string;
  freq?: number;
  dxcc?: number;
  country?: string;
  state?: string;
  gridsquare?: string;
  rst_sent?: string;
  rst_rcvd?: string;
  source: string;
  lotw_status?: "pending" | "uploaded" | "confirmed";
  created_at: string;
}

interface QsoStore {
  qsos: Qso[];
  isLoading: boolean;
  error: string | null;
  
  // Actions
  setQsos: (qsos: Qso[]) => void;
  addQso: (qso: Qso) => void;
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
  
  removeQso: (id) => set((state) => ({ 
    qsos: state.qsos.filter((q) => q.id !== id) 
  })),
  
  setLoading: (isLoading) => set({ isLoading }),
  
  setError: (error) => set({ error }),
}));
