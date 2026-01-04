import { useAwardsStore } from "@/stores/awardsStore";
import { Globe, MapPin, Grid3X3, Database } from "lucide-react";
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { WsjtxConnection } from "./WsjtxConnection";
import { BandActivity } from "./BandActivity";

interface DbStats {
  qso_count: number;
  entity_count: number;
  prefix_count: number;
}

interface Qso {
  id: number;
  uuid: string;
  call: string;
  qso_date: string;
  time_on: string;
  band: string;
  mode: string;
  freq: number | null;
  dxcc: number | null;
  country: string | null;
  state: string | null;
  gridsquare: string | null;
  rst_sent: string | null;
  rst_rcvd: string | null;
  source: string;
  created_at: string;
}

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
  const { dxcc, was, vucc } = useAwardsStore();
  const [dbStats, setDbStats] = useState<DbStats | null>(null);
  const [dbError, setDbError] = useState<string | null>(null);
  const [dbReady, setDbReady] = useState(false);
  const [recentQsos, setRecentQsos] = useState<Qso[]>([]);

  const checkDbReady = async () => {
    try {
      const ready = await invoke<boolean>("is_db_ready");
      if (ready && !dbReady) {
        setDbReady(true);
        fetchStats();
        fetchQsos();
      }
      return ready;
    } catch {
      return false;
    }
  };

  const fetchQsos = async () => {
    try {
      const qsos = await invoke<Qso[]>("get_qsos", { limit: 10, offset: 0 });
      setRecentQsos(qsos);
    } catch (e) {
      // Silently ignore if DB not ready
    }
  };

  const fetchStats = async () => {
    try {
      const stats = await invoke<DbStats>("get_db_stats");
      setDbStats(stats);
      setDbError(null);
      setDbReady(true);
    } catch (e) {
      // Only set error if this isn't just "not initialized yet"
      if (dbReady) {
        setDbError(String(e));
      }
    }
  };

  useEffect(() => {
    // Listen for database ready event
    const unlistenDbReady = listen<{ success: boolean; stats?: DbStats; error?: string }>("db-ready", (event) => {
      if (event.payload.success) {
        setDbReady(true);
        if (event.payload.stats) {
          setDbStats(event.payload.stats);
        }
        setDbError(null);
        fetchQsos();
      } else {
        setDbError(event.payload.error || "Database initialization failed");
      }
    });

    // Poll for DB ready status (handles race condition where event fires before listener)
    const pollInterval = setInterval(async () => {
      if (!dbReady) {
        await checkDbReady();
      }
    }, 500);
    
    // Try immediately
    checkDbReady();
    
    // Listen for new QSOs
    const unlistenQso = listen<QsoEvent>("qso-logged", (event) => {
      console.log("New QSO logged:", event.payload);
      fetchQsos();
      fetchStats();
    });

    // Refresh stats periodically once DB is ready
    const statsInterval = setInterval(() => {
      if (dbReady) {
        fetchStats();
        fetchQsos();
      }
    }, 5000);
    
    return () => {
      clearInterval(pollInterval);
      clearInterval(statsInterval);
      unlistenQso.then((f) => f());
      unlistenDbReady.then((f) => f());
    };
  }, [dbReady]);

  return (
    <div className="space-y-6">
      {/* WSJT-X Connection */}
      <WsjtxConnection />

      {/* Band Activity - Live FT8 Decodes */}
      <BandActivity />

      {/* Database Status */}
      <section className="bg-card rounded-lg border border-border p-4">
        <div className="flex items-center gap-2 mb-2">
          <Database className="h-5 w-5 text-primary" />
          <h3 className="font-semibold">Database Status</h3>
          {dbReady && <span className="text-xs text-green-500">● Ready</span>}
        </div>
        {dbStats ? (
          <div className="grid grid-cols-3 gap-4 text-sm">
            <div>
              <span className="text-muted-foreground">QSOs: </span>
              <span className="font-mono">{dbStats.qso_count}</span>
            </div>
            <div>
              <span className="text-muted-foreground">DXCC Entities: </span>
              <span className="font-mono">{dbStats.entity_count}</span>
            </div>
            <div>
              <span className="text-muted-foreground">Prefix Rules: </span>
              <span className="font-mono">{dbStats.prefix_count}</span>
            </div>
          </div>
        ) : dbError ? (
          <p className="text-sm text-red-500">{dbError}</p>
        ) : (
          <p className="text-sm text-yellow-500">Initializing database...</p>
        )}
      </section>

      {/* Progress Cards */}
      <section>
        <h2 className="text-lg font-semibold mb-4">Your Progress</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <ProgressCard
            title="DXCC"
            icon={<Globe className="h-5 w-5" />}
            worked={dxcc.worked}
            confirmed={dxcc.confirmed}
            total={dxcc.total}
            color="text-blue-500"
          />
          <ProgressCard
            title="WAS"
            icon={<MapPin className="h-5 w-5" />}
            worked={was.worked}
            confirmed={was.confirmed}
            total={was.total}
            color="text-green-500"
          />
          <ProgressCard
            title="VUCC"
            icon={<Grid3X3 className="h-5 w-5" />}
            worked={vucc.worked}
            confirmed={vucc.confirmed}
            total={vucc.target}
            color="text-purple-500"
          />
        </div>
      </section>

      {/* Recent QSOs */}
      <section>
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold">Recent QSOs</h2>
          <button className="text-sm text-primary hover:underline">
            View All
          </button>
        </div>
        <div className="bg-card rounded-lg border border-border">
          {recentQsos.length > 0 ? (
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead className="bg-muted/50">
                  <tr>
                    <th className="text-left p-3 font-medium">Callsign</th>
                    <th className="text-left p-3 font-medium">Date/Time</th>
                    <th className="text-left p-3 font-medium">Band</th>
                    <th className="text-left p-3 font-medium">Mode</th>
                    <th className="text-left p-3 font-medium">Country</th>
                    <th className="text-left p-3 font-medium">Grid</th>
                    <th className="text-left p-3 font-medium">Source</th>
                  </tr>
                </thead>
                <tbody>
                  {recentQsos.map((qso) => (
                    <tr key={qso.id} className="border-t border-border hover:bg-muted/30">
                      <td className="p-3 font-mono font-bold">{qso.call}</td>
                      <td className="p-3 font-mono text-muted-foreground">
                        {qso.qso_date} {qso.time_on}
                      </td>
                      <td className="p-3">{qso.band}</td>
                      <td className="p-3">{qso.mode}</td>
                      <td className="p-3">{qso.country || "—"}</td>
                      <td className="p-3 font-mono">{qso.gridsquare || "—"}</td>
                      <td className="p-3">
                        <span className={`px-2 py-0.5 rounded text-xs ${
                          qso.source === "WSJT-X" 
                            ? "bg-blue-500/20 text-blue-400" 
                            : "bg-gray-500/20 text-gray-400"
                        }`}>
                          {qso.source}
                        </span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ) : (
            <div className="p-8 text-center text-muted-foreground">
              <p>No QSOs logged yet.</p>
              <p className="text-sm mt-2">
                Start the UDP listener and WSJT-X to automatically capture FT8 contacts.
              </p>
            </div>
          )}
        </div>
      </section>
    </div>
  );
}

interface ProgressCardProps {
  title: string;
  icon: React.ReactNode;
  worked: number;
  confirmed: number;
  total: number;
  color: string;
}

function ProgressCard({
  title,
  icon,
  worked,
  confirmed,
  total,
  color,
}: ProgressCardProps) {
  const percentage = Math.round((worked / total) * 100);

  return (
    <div className="bg-card rounded-lg border border-border p-4">
      <div className="flex items-center gap-2 mb-3">
        <span className={color}>{icon}</span>
        <h3 className="font-semibold">{title}</h3>
      </div>
      <div className="text-3xl font-bold mb-1">{worked}</div>
      <div className="text-sm text-muted-foreground mb-3">of {total}</div>
      <div className="h-2 bg-secondary rounded-full overflow-hidden mb-3">
        <div
          className={`h-full bg-current ${color}`}
          style={{ width: `${percentage}%` }}
        />
      </div>
      <div className="text-sm text-muted-foreground">
        Confirmed: {confirmed}
      </div>
    </div>
  );
}
