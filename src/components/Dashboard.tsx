import { useAwardsStore } from "@/stores/awardsStore";
import { Globe, MapPin, Grid3X3 } from "lucide-react";

export function Dashboard() {
  const { dxcc, was, vucc } = useAwardsStore();

  return (
    <div className="space-y-6">
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
          <div className="p-8 text-center text-muted-foreground">
            <p>No QSOs logged yet.</p>
            <p className="text-sm mt-2">
              Start WSJT-X to automatically capture FT8 contacts.
            </p>
          </div>
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
