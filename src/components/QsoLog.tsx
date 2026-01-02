import { useState } from "react";
import { useQsoStore } from "@/stores/qsoStore";
import { Search, Download, Upload, Plus } from "lucide-react";

export function QsoLog() {
  const { qsos, isLoading } = useQsoStore();
  const [searchTerm, setSearchTerm] = useState("");
  const [bandFilter, setBandFilter] = useState<string>("all");
  const [modeFilter, setModeFilter] = useState<string>("all");

  const filteredQsos = qsos.filter((qso) => {
    if (searchTerm && !qso.call.toLowerCase().includes(searchTerm.toLowerCase())) {
      return false;
    }
    if (bandFilter !== "all" && qso.band !== bandFilter) {
      return false;
    }
    if (modeFilter !== "all" && qso.mode !== modeFilter) {
      return false;
    }
    return true;
  });

  return (
    <div className="space-y-4">
      {/* Toolbar */}
      <div className="flex flex-wrap items-center gap-4">
        <div className="relative flex-1 min-w-[200px]">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <input
            type="text"
            placeholder="Search callsigns..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="w-full pl-10 pr-4 py-2 bg-secondary rounded-lg border border-border focus:outline-none focus:ring-2 focus:ring-ring"
          />
        </div>
        <select
          value={bandFilter}
          onChange={(e) => setBandFilter(e.target.value)}
          className="px-3 py-2 bg-secondary rounded-lg border border-border"
        >
          <option value="all">All Bands</option>
          <option value="160m">160m</option>
          <option value="80m">80m</option>
          <option value="40m">40m</option>
          <option value="30m">30m</option>
          <option value="20m">20m</option>
          <option value="17m">17m</option>
          <option value="15m">15m</option>
          <option value="12m">12m</option>
          <option value="10m">10m</option>
          <option value="6m">6m</option>
          <option value="2m">2m</option>
        </select>
        <select
          value={modeFilter}
          onChange={(e) => setModeFilter(e.target.value)}
          className="px-3 py-2 bg-secondary rounded-lg border border-border"
        >
          <option value="all">All Modes</option>
          <option value="FT8">FT8</option>
          <option value="FT4">FT4</option>
          <option value="CW">CW</option>
          <option value="SSB">SSB</option>
        </select>
        <div className="flex gap-2">
          <button className="flex items-center gap-2 px-3 py-2 bg-secondary hover:bg-accent rounded-lg transition-colors">
            <Upload className="h-4 w-4" />
            <span>Import</span>
          </button>
          <button className="flex items-center gap-2 px-3 py-2 bg-secondary hover:bg-accent rounded-lg transition-colors">
            <Download className="h-4 w-4" />
            <span>Export</span>
          </button>
          <button className="flex items-center gap-2 px-3 py-2 bg-primary text-primary-foreground hover:opacity-90 rounded-lg transition-colors">
            <Plus className="h-4 w-4" />
            <span>Add QSO</span>
          </button>
        </div>
      </div>

      {/* QSO Table */}
      <div className="bg-card rounded-lg border border-border overflow-hidden">
        <table className="w-full">
          <thead className="bg-secondary">
            <tr>
              <th className="px-4 py-3 text-left text-sm font-medium">Date</th>
              <th className="px-4 py-3 text-left text-sm font-medium">Time</th>
              <th className="px-4 py-3 text-left text-sm font-medium">Call</th>
              <th className="px-4 py-3 text-left text-sm font-medium">Band</th>
              <th className="px-4 py-3 text-left text-sm font-medium">Mode</th>
              <th className="px-4 py-3 text-left text-sm font-medium">RST</th>
              <th className="px-4 py-3 text-left text-sm font-medium">Grid</th>
              <th className="px-4 py-3 text-left text-sm font-medium">LoTW</th>
            </tr>
          </thead>
          <tbody>
            {filteredQsos.length === 0 ? (
              <tr>
                <td colSpan={8} className="px-4 py-8 text-center text-muted-foreground">
                  {isLoading ? "Loading..." : "No QSOs found"}
                </td>
              </tr>
            ) : (
              filteredQsos.map((qso) => (
                <tr key={qso.id} className="border-t border-border hover:bg-accent/50">
                  <td className="px-4 py-3 text-sm">{formatDate(qso.qso_date)}</td>
                  <td className="px-4 py-3 text-sm">{formatTime(qso.time_on)}</td>
                  <td className="px-4 py-3 text-sm font-medium">{qso.call}</td>
                  <td className="px-4 py-3 text-sm">{qso.band}</td>
                  <td className="px-4 py-3 text-sm">{qso.mode}</td>
                  <td className="px-4 py-3 text-sm">{qso.rst_rcvd || "-"}</td>
                  <td className="px-4 py-3 text-sm">{qso.gridsquare || "-"}</td>
                  <td className="px-4 py-3 text-sm">
                    <LotwStatus status={qso.lotw_status} />
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      <div className="flex items-center justify-between text-sm text-muted-foreground">
        <span>Showing {filteredQsos.length} QSOs</span>
      </div>
    </div>
  );
}

function formatDate(date: string): string {
  if (!date || date.length !== 8) return date;
  return `${date.slice(0, 4)}-${date.slice(4, 6)}-${date.slice(6, 8)}`;
}

function formatTime(time: string): string {
  if (!time || time.length < 4) return time;
  return `${time.slice(0, 2)}:${time.slice(2, 4)}`;
}

function LotwStatus({ status }: { status?: string }) {
  switch (status) {
    case "confirmed":
      return <span className="text-green-500">✓</span>;
    case "pending":
      return <span className="text-yellow-500">⏳</span>;
    case "uploaded":
      return <span className="text-blue-500">↑</span>;
    default:
      return <span className="text-muted-foreground">-</span>;
  }
}
