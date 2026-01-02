import { useState } from "react";
import { useAwardsStore } from "@/stores/awardsStore";

type AwardType = "dxcc" | "was" | "vucc";

export function AwardsMatrix() {
  const [selectedAward, setSelectedAward] = useState<AwardType>("dxcc");
  const { dxcc, was, vucc } = useAwardsStore();

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
  return (
    <div className="text-center text-muted-foreground py-8">
      <p className="text-lg mb-2">DXCC Progress Matrix</p>
      <p className="text-sm">
        Entity × Band matrix will be displayed here once CTY.DAT is loaded.
      </p>
    </div>
  );
}

function WasMatrix() {
  const states = [
    "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA",
    "HI", "ID", "IL", "IN", "IA", "KS", "KY", "LA", "ME", "MD",
    "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ",
    "NM", "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC",
    "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV", "WI", "WY",
  ];

  return (
    <div>
      <p className="text-lg font-semibold mb-4">Worked All States</p>
      <div className="grid grid-cols-10 gap-2">
        {states.map((state) => (
          <div
            key={state}
            className="aspect-square flex items-center justify-center bg-muted rounded text-xs font-medium"
            title={state}
          >
            {state}
          </div>
        ))}
      </div>
    </div>
  );
}

function VuccMatrix() {
  return (
    <div className="text-center text-muted-foreground py-8">
      <p className="text-lg mb-2">VUCC Grid Progress</p>
      <p className="text-sm">
        Grid square map will be displayed here for 6m and above.
      </p>
    </div>
  );
}
