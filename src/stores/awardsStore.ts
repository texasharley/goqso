import { create } from "zustand";

interface DxccProgress {
  worked: number;
  confirmed: number;
  total: number;
}

interface WasProgress {
  worked: number;
  confirmed: number;
  total: number;
  worked_states: string[];
  confirmed_states: string[];
}

interface VuccProgress {
  worked: number;
  confirmed: number;
  target: number;
  band?: string;
}

interface AwardsStore {
  dxcc: DxccProgress;
  was: WasProgress;
  vucc: VuccProgress;
  
  // Actions
  setDxccProgress: (progress: DxccProgress) => void;
  setWasProgress: (progress: WasProgress) => void;
  setVuccProgress: (progress: VuccProgress) => void;
}

export const useAwardsStore = create<AwardsStore>((set) => ({
  dxcc: { worked: 0, confirmed: 0, total: 340 },
  was: { worked: 0, confirmed: 0, total: 50, worked_states: [], confirmed_states: [] },
  vucc: { worked: 0, confirmed: 0, target: 100 },

  setDxccProgress: (dxcc) => set({ dxcc }),
  setWasProgress: (was) => set({ was }),
  setVuccProgress: (vucc) => set({ vucc }),
}));
