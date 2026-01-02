import { useEffect, useCallback } from "react";
import { useAwardsStore } from "@/stores/awardsStore";
import * as api from "@/lib/tauri";

export function useAwards() {
  const { 
    dxcc, was, vucc,
    setDxccProgress, setWasProgress, setVuccProgress 
  } = useAwardsStore();

  const fetchDxccProgress = useCallback(async () => {
    try {
      const progress = await api.getDxccProgress();
      setDxccProgress(progress);
    } catch (err) {
      console.error("Failed to fetch DXCC progress:", err);
    }
  }, [setDxccProgress]);

  const fetchWasProgress = useCallback(async () => {
    try {
      const progress = await api.getWasProgress();
      setWasProgress(progress);
    } catch (err) {
      console.error("Failed to fetch WAS progress:", err);
    }
  }, [setWasProgress]);

  const fetchVuccProgress = useCallback(async (band?: string) => {
    try {
      const progress = await api.getVuccProgress(band);
      setVuccProgress(progress);
    } catch (err) {
      console.error("Failed to fetch VUCC progress:", err);
    }
  }, [setVuccProgress]);

  const fetchAllProgress = useCallback(async () => {
    await Promise.all([
      fetchDxccProgress(),
      fetchWasProgress(),
      fetchVuccProgress(),
    ]);
  }, [fetchDxccProgress, fetchWasProgress, fetchVuccProgress]);

  useEffect(() => {
    fetchAllProgress();
  }, [fetchAllProgress]);

  return {
    dxcc,
    was,
    vucc,
    fetchDxccProgress,
    fetchWasProgress,
    fetchVuccProgress,
    fetchAllProgress,
  };
}
