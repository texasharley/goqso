import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { WsjtxConnection } from "./WsjtxConnection";
import { BandActivity } from "./BandActivity";
import ActiveQso from "./ActiveQso";

interface QsoEvent {
  call: string;
  grid: string;
  freq_mhz: number;
  mode: string;
  rst_sent: string;
  rst_rcvd: string;
  band: string;
}

export function Dashboard() {
  // Listen for QSO events to trigger refreshes
  useEffect(() => {
    const unlistenQso = listen<QsoEvent>("qso-logged", (event) => {
      console.log("New QSO logged:", event.payload);
    });

    return () => {
      unlistenQso.then((f) => f());
    };
  }, []);

  return (
    <div className="space-y-4">
      {/* WSJT-X Connection */}
      <WsjtxConnection />

      {/* Active QSO Panel - Shows in-progress QSO state */}
      <ActiveQso />

      {/* Band Activity - Live FT8 Decodes */}
      <BandActivity />
    </div>
  );
}
