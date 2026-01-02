import { create } from "zustand";

interface Settings {
  udpPort: number;
  myCallsign: string;
  myGrid: string;
  tqslPath: string;
  autoSync: boolean;
  syncInterval: number; // minutes
}

interface SettingsStore {
  settings: Settings;
  isLoading: boolean;
  
  // Actions
  setSettings: (settings: Partial<Settings>) => void;
  setLoading: (loading: boolean) => void;
}

export const useSettingsStore = create<SettingsStore>((set) => ({
  settings: {
    udpPort: 2237,
    myCallsign: "",
    myGrid: "",
    tqslPath: "",
    autoSync: true,
    syncInterval: 15,
  },
  isLoading: false,

  setSettings: (newSettings) => set((state) => ({
    settings: { ...state.settings, ...newSettings }
  })),
  
  setLoading: (isLoading) => set({ isLoading }),
}));
