import { useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { useQsoStore } from "@/stores/qsoStore";
import * as api from "@/lib/tauri";

interface QsoLoggedPayload {
  call: string;
  grid: string;
  mode: string;
  freq: number;
  band: string;
  time_on: string;
  qso_date: string;
  rst_sent: string;
  rst_rcvd: string;
  dxcc?: number;
  country?: string;
  state?: string;
}

export function useUdpListener() {
  const { addQso } = useQsoStore();

  useEffect(() => {
    // Listen for QSO logged events from WSJT-X
    const unlisten = listen<QsoLoggedPayload>("qso-logged", (event) => {
      console.log("QSO logged:", event.payload);
      
      // Add to store (will also be persisted by backend)
      addQso({
        id: Date.now(), // Temporary ID until we get the real one
        uuid: crypto.randomUUID(),
        call: event.payload.call,
        qso_date: event.payload.qso_date,
        time_on: event.payload.time_on,
        band: event.payload.band,
        mode: event.payload.mode,
        freq: event.payload.freq,
        gridsquare: event.payload.grid,
        rst_sent: event.payload.rst_sent,
        rst_rcvd: event.payload.rst_rcvd,
        dxcc: event.payload.dxcc,
        country: event.payload.country,
        state: event.payload.state,
        source: "wsjt-x",
        lotw_status: "pending",
        created_at: new Date().toISOString(),
      });
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [addQso]);

  const startListener = useCallback(async (port: number) => {
    await api.startUdpListener(port);
  }, []);

  const stopListener = useCallback(async () => {
    await api.stopUdpListener();
  }, []);

  const getStatus = useCallback(async () => {
    return api.getUdpStatus();
  }, []);

  return { startListener, stopListener, getStatus };
}
