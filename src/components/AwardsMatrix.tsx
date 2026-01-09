import { useState } from "react";
import { useAwardsStore } from "@/stores/awardsStore";
import { useAwards } from "@/hooks/useAwards";

type AwardType = "dxcc" | "was" | "vucc";

export function AwardsMatrix() {
  const [selectedAward, setSelectedAward] = useState<AwardType>("dxcc");
  const { dxcc, was, vucc } = useAwards();

  return (
    <div className="space-y-6">
      {/* Award Selector */}
      <div className="flex gap-2">
        <button
          onClick={() => setSelectedAward("dxcc")}
          className={`px-4 py-2 rounded-lg transition-colors ${
            selectedAward === "dxcc"
              ? "bg-primary text-primary-foreground"
              : "bg-secondary hover:bg-accent"
          }`}
        >
          DXCC ({dxcc.worked}/{dxcc.total})
        </button>
        <button
          onClick={() => setSelectedAward("was")}
          className={`px-4 py-2 rounded-lg transition-colors ${
            selectedAward === "was"
              ? "bg-primary text-primary-foreground"
              : "bg-secondary hover:bg-accent"
          }`}
        >
          WAS ({was.worked}/{was.total})
        </button>
        <button
          onClick={() => setSelectedAward("vucc")}
          className={`px-4 py-2 rounded-lg transition-colors ${
            selectedAward === "vucc"
              ? "bg-primary text-primary-foreground"
              : "bg-secondary hover:bg-accent"
          }`}
        >
          VUCC ({vucc.worked}/{vucc.target})
        </button>
      </div>

      {/* Legend */}
      <div className="flex items-center gap-4 text-sm">
        <span className="text-muted-foreground">Legend:</span>
        <div className="flex items-center gap-1">
          <span className="w-4 h-4 bg-muted rounded" />
          <span>Needed</span>
        </div>
        <div className="flex items-center gap-1">
          <span className="w-4 h-4 bg-yellow-500 rounded" />
          <span>Worked</span>
        </div>
        <div className="flex items-center gap-1">
          <span className="w-4 h-4 bg-green-500 rounded" />
          <span>Confirmed</span>
        </div>
        <div className="flex items-center gap-1">
          <span className="w-4 h-4 bg-blue-500 rounded" />
          <span>Credited</span>
        </div>
      </div>

      {/* Matrix Content */}
      <div className="bg-card rounded-lg border border-border p-6">
        {selectedAward === "dxcc" && <DxccMatrix />}
        {selectedAward === "was" && <WasMatrix />}
        {selectedAward === "vucc" && <VuccMatrix />}
      </div>
    </div>
  );
}

function DxccMatrix() {
  const { dxcc } = useAwardsStore();
  
  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <p className="text-lg font-semibold">DXCC Entities</p>
        <div className="text-sm text-muted-foreground">
          {dxcc.worked}/{dxcc.total} worked · {dxcc.confirmed}/{dxcc.total} confirmed
        </div>
      </div>
      
      {/* Progress bar */}
      <div className="h-3 bg-muted rounded-full mb-6 overflow-hidden">
        <div className="h-full flex">
          <div 
            className="bg-green-500 transition-all" 
            style={{ width: `${(dxcc.confirmed / dxcc.total) * 100}%` }} 
          />
          <div 
            className="bg-yellow-500 transition-all" 
            style={{ width: `${((dxcc.worked - dxcc.confirmed) / dxcc.total) * 100}%` }} 
          />
        </div>
      </div>
      
      {/* Stats cards */}
      <div className="grid grid-cols-3 gap-4">
        <div className="bg-muted rounded-lg p-4 text-center">
          <div className="text-3xl font-bold">{dxcc.total - dxcc.worked}</div>
          <div className="text-sm text-muted-foreground">Needed</div>
        </div>
        <div className="bg-yellow-500/20 border border-yellow-500 rounded-lg p-4 text-center">
          <div className="text-3xl font-bold text-yellow-500">{dxcc.worked}</div>
          <div className="text-sm text-muted-foreground">Worked</div>
        </div>
        <div className="bg-green-500/20 border border-green-500 rounded-lg p-4 text-center">
          <div className="text-3xl font-bold text-green-500">{dxcc.confirmed}</div>
          <div className="text-sm text-muted-foreground">Confirmed</div>
        </div>
      </div>
      
      <p className="text-sm text-muted-foreground text-center mt-6">
        Entity × Band matrix coming soon
      </p>
    </div>
  );
}

function WasMatrix() {
  const { was } = useAwardsStore();
  
  const states = [
    "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA",
    "HI", "ID", "IL", "IN", "IA", "KS", "KY", "LA", "ME", "MD",
    "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ",
    "NM", "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC",
    "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV", "WI", "WY",
  ];

  const getStateStatus = (state: string): "needed" | "worked" | "confirmed" => {
    if (was.confirmed_states?.includes(state)) return "confirmed";
    if (was.worked_states?.includes(state)) return "worked";
    return "needed";
  };

  const getStateColor = (status: "needed" | "worked" | "confirmed") => {
    switch (status) {
      case "confirmed": return "bg-green-500 text-white";
      case "worked": return "bg-yellow-500 text-black";
      default: return "bg-muted";
    }
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <p className="text-lg font-semibold">Worked All States</p>
        <div className="text-sm text-muted-foreground">
          {was.worked}/50 worked · {was.confirmed}/50 confirmed
        </div>
      </div>
      
      {/* Progress bar */}
      <div className="h-3 bg-muted rounded-full mb-4 overflow-hidden">
        <div className="h-full flex">
          <div 
            className="bg-green-500 transition-all" 
            style={{ width: `${(was.confirmed / 50) * 100}%` }} 
          />
          <div 
            className="bg-yellow-500 transition-all" 
            style={{ width: `${((was.worked - was.confirmed) / 50) * 100}%` }} 
          />
        </div>
      </div>
      
      <div className="grid grid-cols-10 gap-2">
        {states.map((state) => {
          const status = getStateStatus(state);
          return (
            <div
              key={state}
              className={`aspect-square flex items-center justify-center rounded text-xs font-medium transition-colors ${getStateColor(status)}`}
              title={`${state} - ${status}`}
            >
              {state}
            </div>
          );
        })}
      </div>
    </div>
  );
}

function VuccMatrix() {
  const { vucc } = useAwardsStore();
  
  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <p className="text-lg font-semibold">VHF/UHF Century Club</p>
        <div className="text-sm text-muted-foreground">
          {vucc.worked} worked · {vucc.confirmed} confirmed
        </div>
      </div>
      
      {/* Progress bar (toward 100 grids) */}
      <div className="h-3 bg-muted rounded-full mb-6 overflow-hidden">
        <div className="h-full flex">
          <div 
            className="bg-green-500 transition-all" 
            style={{ width: `${Math.min((vucc.confirmed / vucc.target) * 100, 100)}%` }} 
          />
          <div 
            className="bg-yellow-500 transition-all" 
            style={{ width: `${Math.min(((vucc.worked - vucc.confirmed) / vucc.target) * 100, 100 - (vucc.confirmed / vucc.target) * 100)}%` }} 
          />
        </div>
      </div>
      
      {/* Stats cards */}
      <div className="grid grid-cols-3 gap-4">
        <div className="bg-muted rounded-lg p-4 text-center">
          <div className="text-3xl font-bold">{Math.max(0, vucc.target - vucc.worked)}</div>
          <div className="text-sm text-muted-foreground">Need for {vucc.target}</div>
        </div>
        <div className="bg-yellow-500/20 border border-yellow-500 rounded-lg p-4 text-center">
          <div className="text-3xl font-bold text-yellow-500">{vucc.worked}</div>
          <div className="text-sm text-muted-foreground">Grids Worked</div>
        </div>
        <div className="bg-green-500/20 border border-green-500 rounded-lg p-4 text-center">
          <div className="text-3xl font-bold text-green-500">{vucc.confirmed}</div>
          <div className="text-sm text-muted-foreground">Grids Confirmed</div>
        </div>
      </div>
      
      <p className="text-sm text-muted-foreground text-center mt-6">
        Grid square map coming soon
      </p>
    </div>
  );
}
