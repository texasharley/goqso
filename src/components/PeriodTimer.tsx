import { useEffect, useState } from "react";

interface PeriodTimerProps {
  isTransmitting?: boolean;
  mode?: string; // FT8 = 15s, FT4 = 7.5s
}

export function PeriodTimer({ isTransmitting = false, mode = "FT8" }: PeriodTimerProps) {
  const [progress, setProgress] = useState(0);
  const [secondsInPeriod, setSecondsInPeriod] = useState(0);
  
  // Period length based on mode
  const periodLength = mode === "FT4" ? 7.5 : 15;

  useEffect(() => {
    const updateTimer = () => {
      const now = new Date();
      const totalSeconds = now.getUTCSeconds() + now.getUTCMilliseconds() / 1000;
      const secondsIntoPeriod = totalSeconds % periodLength;
      const progressPercent = (secondsIntoPeriod / periodLength) * 100;
      
      setProgress(progressPercent);
      setSecondsInPeriod(Math.floor(secondsIntoPeriod));
    };

    // Update immediately
    updateTimer();
    
    // Update every 100ms for smooth animation
    const interval = setInterval(updateTimer, 100);
    
    return () => clearInterval(interval);
  }, [periodLength]);

  return (
    <div className="flex items-center gap-3">
      {/* Period progress bar */}
      <div className="relative flex-1 h-2 bg-zinc-800 rounded-full overflow-hidden min-w-[100px]">
        <div
          className={`absolute inset-y-0 left-0 rounded-full transition-all duration-100 ${
            isTransmitting 
              ? "bg-gradient-to-r from-red-600 to-red-500" 
              : "bg-gradient-to-r from-emerald-600 to-emerald-500"
          }`}
          style={{ width: `${progress}%` }}
        />
        {/* Pulse effect at the leading edge */}
        <div
          className={`absolute top-0 bottom-0 w-1 rounded-full blur-sm ${
            isTransmitting ? "bg-red-400" : "bg-emerald-400"
          }`}
          style={{ left: `calc(${progress}% - 2px)` }}
        />
      </div>
      
      {/* Time display */}
      <div className="flex items-center gap-1.5 text-xs font-mono text-zinc-400 shrink-0">
        <span className={isTransmitting ? "text-red-400" : "text-emerald-400"}>
          {secondsInPeriod}
        </span>
        <span className="text-zinc-600">/</span>
        <span>{Math.floor(periodLength)}</span>
      </div>
      
      {/* TX/RX indicator */}
      <div 
        className={`w-6 h-6 rounded-full flex items-center justify-center text-[10px] font-bold shrink-0 ${
          isTransmitting 
            ? "bg-red-500/20 text-red-400 ring-1 ring-red-500/50" 
            : "bg-emerald-500/20 text-emerald-400 ring-1 ring-emerald-500/50"
        }`}
      >
        {isTransmitting ? "TX" : "RX"}
      </div>
    </div>
  );
}
