import { useState, useEffect, lazy, Suspense } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, emit } from "@tauri-apps/api/event";
import { Dashboard } from "./components/Dashboard";
import { SettingsPanel } from "./components/SettingsPanel";
import { SyncStatus } from "./components/SyncStatus";
import { AdifImport } from "./components/AdifImport";
import { LotwSync } from "./components/LotwSync";
import { ToastContainer, toast } from "./components/Toast";
import { Trophy, List, Target, Settings, Wifi, WifiOff, FileUp, Cloud, Loader2, Radio } from "lucide-react";

// Lazy load heavy tab components for faster initial render
const QsoLog = lazy(() => import("./components/QsoLog").then(m => ({ default: m.QsoLog })));
const AwardsMatrix = lazy(() => import("./components/AwardsMatrix").then(m => ({ default: m.AwardsMatrix })));

type Tab = "operate" | "log" | "awards";

interface QsoEvent {
  call: string;
  band: string;
  mode: string;
}

function App() {
  const [activeTab, setActiveTab] = useState<Tab>("operate");
  const [isOnline, _setIsOnline] = useState(true);
  const [showSettings, setShowSettings] = useState(false);
  const [showAdifImport, setShowAdifImport] = useState(false);
  const [showLotwSync, setShowLotwSync] = useState(false);
  const [dbReady, setDbReady] = useState(false);
  const [dbError, setDbError] = useState<string | null>(null);

  // Wait for database to be ready
  useEffect(() => {
    const checkDb = async () => {
      try {
        const ready = await invoke<boolean>("is_db_ready");
        if (ready) {
          setDbReady(true);
        }
      } catch {
        // Not ready yet
      }
    };

    // Listen for db-ready event
    const unlisten = listen<{ success: boolean; error?: string }>("db-ready", (event) => {
      if (event.payload.success) {
        setDbReady(true);
      } else {
        setDbError(event.payload.error || "Database initialization failed");
      }
    });

    // Poll in case event already fired
    checkDb();
    const interval = setInterval(checkDb, 200);

    return () => {
      unlisten.then(fn => fn());
      clearInterval(interval);
    };
  }, []);

  // Listen for QSO logged events and show toast
  useEffect(() => {
    const unlisten = listen<QsoEvent>("qso-logged", (event) => {
      const { call, band, mode } = event.payload;
      toast({
        type: "success",
        title: `QSO Logged: ${call}`,
        message: `${band} ${mode}`,
        duration: 5000,
      });
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  // Listen for open-adif-import event from QsoLog
  useEffect(() => {
    const unlisten = listen("open-adif-import", () => {
      setShowAdifImport(true);
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  // Show loading screen while database initializes
  if (!dbReady) {
    return (
      <div className="min-h-screen bg-zinc-900 text-zinc-100 flex flex-col items-center justify-center">
        <Trophy className="h-16 w-16 text-sky-500 mb-4" />
        <h1 className="text-2xl font-bold mb-4">GoQSO</h1>
        {dbError ? (
          <div className="text-red-400 text-center max-w-md">
            <p className="font-medium">Database Error</p>
            <p className="text-sm mt-2">{dbError}</p>
          </div>
        ) : (
          <div className="flex items-center gap-2 text-zinc-400">
            <Loader2 className="h-5 w-5 animate-spin" />
            <span>Initializing database...</span>
          </div>
        )}
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col bg-zinc-900 text-zinc-100 overflow-hidden">
      {/* Toast notifications */}
      <ToastContainer />

      {/* Header */}
      <header className="border-b border-zinc-800 px-4 py-3 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Trophy className="h-6 w-6 text-sky-500" />
          <h1 className="text-xl font-bold">GoQSO</h1>
        </div>
        <div className="flex items-center gap-4">
          <SyncStatus />
          <div className="flex items-center gap-2 text-sm">
            {isOnline ? (
              <>
                <Wifi className="h-4 w-4 text-green-500" />
                <span className="text-zinc-500">Online</span>
              </>
            ) : (
              <>
                <WifiOff className="h-4 w-4 text-yellow-500" />
                <span className="text-zinc-500">Offline</span>
              </>
            )}
          </div>
          <button
            onClick={() => setShowLotwSync(true)}
            className="p-2 hover:bg-zinc-800 rounded-lg transition-colors"
            title="LoTW Sync"
          >
            <Cloud className="h-5 w-5" />
          </button>
          <button
            onClick={() => setShowAdifImport(true)}
            className="p-2 hover:bg-zinc-800 rounded-lg transition-colors"
            title="Import ADIF"
          >
            <FileUp className="h-5 w-5" />
          </button>
          <button
            onClick={() => setShowSettings(true)}
            className="p-2 hover:bg-zinc-800 rounded-lg transition-colors"
            title="Settings"
          >
            <Settings className="h-5 w-5" />
          </button>
        </div>
      </header>

      {/* Navigation */}
      <nav className="border-b border-zinc-800 px-4">
        <div className="flex gap-1">
          <TabButton
            active={activeTab === "operate"}
            onClick={() => setActiveTab("operate")}
            icon={<Radio className="h-4 w-4" />}
            label="Operate"
          />
          <TabButton
            active={activeTab === "log"}
            onClick={() => setActiveTab("log")}
            icon={<List className="h-4 w-4" />}
            label="Log"
          />
          <TabButton
            active={activeTab === "awards"}
            onClick={() => setActiveTab("awards")}
            icon={<Target className="h-4 w-4" />}
            label="Awards"
          />
        </div>
      </nav>

      {/* Main Content */}
      <main className={`flex-1 p-4 ${activeTab === "operate" ? "flex flex-col overflow-hidden" : "overflow-y-auto"}`}>
        <Suspense fallback={<div className="flex items-center justify-center py-8"><Loader2 className="h-6 w-6 animate-spin text-zinc-500" /></div>}>
          {activeTab === "operate" && <Dashboard />}
          {activeTab === "log" && <QsoLog />}
          {activeTab === "awards" && <AwardsMatrix />}
        </Suspense>
      </main>

      {/* Settings Modal */}
      {showSettings && <SettingsPanel onClose={() => setShowSettings(false)} />}
      
      {/* ADIF Import Modal */}
      {showAdifImport && (
        <AdifImport 
          onClose={() => setShowAdifImport(false)} 
          onImportComplete={(count) => {
            toast({
              type: "success",
              title: `Imported ${count} QSOs`,
              message: "Your log has been updated",
              duration: 5000,
            });
            // Emit event to refresh QsoLog
            emit("qsos-imported", { count });
          }}
        />
      )}
      
      {/* LoTW Sync Modal */}
      {showLotwSync && (
        <LotwSync 
          onClose={() => setShowLotwSync(false)} 
          onSyncComplete={(matched) => {
            toast({
              type: "success",
              title: `Synced ${matched} QSL confirmations`,
              message: "LoTW data has been updated",
              duration: 5000,
            });
            // Emit event to refresh QsoLog
            emit("qsos-imported", { count: matched });
          }}
        />
      )}
    </div>
  );
}

interface TabButtonProps {
  active: boolean;
  onClick: () => void;
  icon: React.ReactNode;
  label: string;
}

function TabButton({ active, onClick, icon, label }: TabButtonProps) {
  return (
    <button
      onClick={onClick}
      className={`flex items-center gap-2 px-4 py-3 border-b-2 transition-colors ${
        active
          ? "border-sky-500 text-sky-500"
          : "border-transparent text-zinc-500 hover:text-zinc-300"
      }`}
    >
      {icon}
      <span>{label}</span>
    </button>
  );
}

export default App;
