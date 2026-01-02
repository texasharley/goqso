import { useEffect, useCallback } from "react";
import { useQsoStore } from "@/stores/qsoStore";
import * as api from "@/lib/tauri";

export function useQsos() {
  const { qsos, isLoading, error, setQsos, setLoading, setError } = useQsoStore();

  const fetchQsos = useCallback(async (limit = 100, offset = 0) => {
    setLoading(true);
    setError(null);
    
    try {
      const data = await api.getQsos(limit, offset);
      setQsos(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch QSOs");
    } finally {
      setLoading(false);
    }
  }, [setQsos, setLoading, setError]);

  useEffect(() => {
    fetchQsos();
  }, [fetchQsos]);

  const importAdif = useCallback(async (path: string) => {
    setLoading(true);
    try {
      const result = await api.importAdif(path);
      await fetchQsos(); // Refresh QSOs
      return result;
    } finally {
      setLoading(false);
    }
  }, [fetchQsos, setLoading]);

  const exportAdif = useCallback(async (path: string, qsoIds?: number[]) => {
    return api.exportAdif(path, qsoIds);
  }, []);

  return {
    qsos,
    isLoading,
    error,
    fetchQsos,
    importAdif,
    exportAdif,
  };
}
