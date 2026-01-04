import { useState, useEffect } from "react";
import { Dashboard } from "./components/Dashboard";
import { QsoLog } from "./components/QsoLog";
import { AwardsMatrix } from "./components/AwardsMatrix";
import { SettingsPanel } from "./components/SettingsPanel";
import { SyncStatus } from "./components/SyncStatus";
import { ToastContainer, toast } from "./components/Toast";
import { listen } from "@tauri-apps/api/event";
import { Trophy, List, Target, Settings, Wifi, WifiOff } from "lucide-react";

type Tab = "dashboard" | "log" | "awards";

interface QsoEvent {
  call: string;
  band: string;
  mode: string;
}

function App() {
  const [activeTab, setActiveTab] = useState<Tab>("dashboard");
  const [isOnline, _setIsOnline] = useState(true);
  const [showSettings, setShowSettings] = useState(false);

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

  return (
    <div className="min-h-screen bg-zinc-900 text-zinc-100">
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
            onClick={() => setShowSettings(true)}
            className="p-2 hover:bg-zinc-800 rounded-lg transition-colors"
          >
            <Settings className="h-5 w-5" />
          </button>
        </div>
      </header>

      {/* Navigation */}
      <nav className="border-b border-zinc-800 px-4">
        <div className="flex gap-1">
          <TabButton
            active={activeTab === "dashboard"}
            onClick={() => setActiveTab("dashboard")}
            icon={<Trophy className="h-4 w-4" />}
            label="Dashboard"
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
      <main className="p-4">
        {activeTab === "dashboard" && <Dashboard />}
        {activeTab === "log" && <QsoLog />}
        {activeTab === "awards" && <AwardsMatrix />}
      </main>

      {/* Settings Modal */}
      {showSettings && <SettingsPanel onClose={() => setShowSettings(false)} />}
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
