import { useEffect, useState } from "react";
import { CheckCircle, XCircle, Info, X } from "lucide-react";

export interface ToastData {
  id: string;
  type: "success" | "error" | "info";
  title: string;
  message?: string;
  duration?: number;
}

interface ToastProps {
  toast: ToastData;
  onDismiss: (id: string) => void;
}

function Toast({ toast, onDismiss }: ToastProps) {
  useEffect(() => {
    const timer = setTimeout(() => {
      onDismiss(toast.id);
    }, toast.duration || 4000);

    return () => clearTimeout(timer);
  }, [toast.id, toast.duration, onDismiss]);

  const icons = {
    success: <CheckCircle className="h-5 w-5 text-green-400" />,
    error: <XCircle className="h-5 w-5 text-red-400" />,
    info: <Info className="h-5 w-5 text-sky-400" />,
  };

  const bgColors = {
    success: "bg-green-900/50 border-green-700",
    error: "bg-red-900/50 border-red-700",
    info: "bg-sky-900/50 border-sky-700",
  };

  return (
    <div
      className={`flex items-start gap-3 p-4 rounded-lg border ${bgColors[toast.type]} backdrop-blur-sm animate-slide-in`}
    >
      {icons[toast.type]}
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium text-zinc-100">{toast.title}</p>
        {toast.message && (
          <p className="text-sm text-zinc-400 mt-0.5">{toast.message}</p>
        )}
      </div>
      <button
        onClick={() => onDismiss(toast.id)}
        className="text-zinc-500 hover:text-zinc-300"
      >
        <X className="h-4 w-4" />
      </button>
    </div>
  );
}

// Toast container that manages the toast stack
let toastListeners: ((toast: ToastData) => void)[] = [];

export function toast(data: Omit<ToastData, "id">) {
  const id = Math.random().toString(36).slice(2, 9);
  const toastData = { ...data, id };
  toastListeners.forEach((listener) => listener(toastData));
}

export function ToastContainer() {
  const [toasts, setToasts] = useState<ToastData[]>([]);

  useEffect(() => {
    const listener = (toast: ToastData) => {
      setToasts((prev) => [...prev, toast]);
    };

    toastListeners.push(listener);
    return () => {
      toastListeners = toastListeners.filter((l) => l !== listener);
    };
  }, []);

  const dismissToast = (id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  };

  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-sm w-full">
      {toasts.map((t) => (
        <Toast key={t.id} toast={t} onDismiss={dismissToast} />
      ))}
    </div>
  );
}
