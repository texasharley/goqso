import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { readTextFile } from "@tauri-apps/plugin-fs";
import { X, Upload, FileText, CheckCircle2, AlertCircle, Loader2 } from "lucide-react";

interface AdifImportProps {
  onClose: () => void;
  onImportComplete: (count: number) => void;
}

interface ImportResult {
  total_records: number;
  imported: number;
  skipped: number;
  errors: number;
  error_messages: string[];
}

export function AdifImport({ onClose, onImportComplete }: AdifImportProps) {
  const [_filePath, setFilePath] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string>("");
  const [fileContent, setFileContent] = useState<string | null>(null);
  const [previewCount, setPreviewCount] = useState<number>(0);
  const [isLoading, setIsLoading] = useState(false);
  const [isImporting, setIsImporting] = useState(false);
  const [result, setResult] = useState<ImportResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const selectFile = useCallback(async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          { name: "ADIF Files", extensions: ["adi", "adif", "ADI", "ADIF"] },
          { name: "All Files", extensions: ["*"] },
        ],
      });

      if (selected && typeof selected === "string") {
        setFilePath(selected);
        setFileName(selected.split(/[\\/]/).pop() || "");
        setIsLoading(true);
        setError(null);
        setResult(null);

        try {
          const content = await readTextFile(selected);
          setFileContent(content);

          // Count records by counting <EOR> tags (case-insensitive)
          const eorMatches = content.match(/<eor>/gi);
          setPreviewCount(eorMatches ? eorMatches.length : 0);
        } catch (err) {
          setError(`Failed to read file: ${err}`);
        } finally {
          setIsLoading(false);
        }
      }
    } catch (err) {
      setError(`Failed to select file: ${err}`);
    }
  }, []);

  const handleImport = useCallback(async () => {
    if (!fileContent) return;

    setIsImporting(true);
    setError(null);

    try {
      const importResult = await invoke<ImportResult>("import_adif", {
        content: fileContent,
        skipDuplicates: true,
      });

      setResult(importResult);

      if (importResult.imported > 0) {
        onImportComplete(importResult.imported);
      }
    } catch (err) {
      setError(`Import failed: ${err}`);
    } finally {
      setIsImporting(false);
    }
  }, [fileContent, onImportComplete]);

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-zinc-900 rounded-lg border border-zinc-700 w-full max-w-md">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-zinc-700">
          <div className="flex items-center gap-2">
            <Upload className="h-5 w-5 text-sky-500" />
            <h2 className="text-lg font-semibold">Import ADIF</h2>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-zinc-800 rounded-lg transition-colors"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        {/* Content */}
        <div className="p-4 space-y-4">
          {/* File Selection */}
          <div>
            <button
              onClick={selectFile}
              disabled={isImporting}
              className="w-full p-6 border-2 border-dashed border-zinc-600 rounded-lg hover:border-sky-500 hover:bg-zinc-800/50 transition-colors flex flex-col items-center gap-2"
            >
              <FileText className="h-8 w-8 text-zinc-500" />
              <span className="text-sm text-zinc-400">
                {fileName || "Click to select ADIF file"}
              </span>
            </button>
          </div>

          {/* Loading */}
          {isLoading && (
            <div className="flex items-center justify-center gap-2 text-zinc-400">
              <Loader2 className="h-4 w-4 animate-spin" />
              <span>Reading file...</span>
            </div>
          )}

          {/* Preview */}
          {fileContent && !isLoading && !result && (
            <div className="bg-zinc-800 rounded-lg p-4 space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm text-zinc-400">File:</span>
                <span className="text-sm font-mono">{fileName}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-zinc-400">QSO Records:</span>
                <span className="text-lg font-bold text-sky-400">{previewCount}</span>
              </div>
            </div>
          )}

          {/* Error */}
          {error && (
            <div className="bg-red-900/30 border border-red-700 rounded-lg p-3 flex items-start gap-2">
              <AlertCircle className="h-5 w-5 text-red-500 flex-shrink-0 mt-0.5" />
              <p className="text-sm text-red-300">{error}</p>
            </div>
          )}

          {/* Result */}
          {result && (
            <div className="bg-zinc-800 rounded-lg p-4 space-y-3">
              <div className="flex items-center gap-2 text-green-500">
                <CheckCircle2 className="h-5 w-5" />
                <span className="font-medium">Import Complete</span>
              </div>
              <div className="grid grid-cols-2 gap-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-zinc-400">Total records:</span>
                  <span>{result.total_records}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-zinc-400">Imported:</span>
                  <span className="text-green-400">{result.imported}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-zinc-400">Skipped (dupes):</span>
                  <span className="text-yellow-400">{result.skipped}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-zinc-400">Errors:</span>
                  <span className={result.errors > 0 ? "text-red-400" : ""}>
                    {result.errors}
                  </span>
                </div>
              </div>
              {result.error_messages.length > 0 && (
                <div className="mt-2 text-xs text-red-400 max-h-24 overflow-y-auto">
                  {result.error_messages.slice(0, 5).map((msg, i) => (
                    <div key={i}>• {msg}</div>
                  ))}
                  {result.error_messages.length > 5 && (
                    <div>...and {result.error_messages.length - 5} more</div>
                  )}
                </div>
              )}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="p-4 border-t border-zinc-700 flex justify-end gap-2">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm hover:bg-zinc-800 rounded-lg transition-colors"
          >
            {result ? "Close" : "Cancel"}
          </button>
          {!result && (
            <button
              onClick={handleImport}
              disabled={!fileContent || isImporting || isLoading}
              className="px-4 py-2 text-sm bg-sky-600 hover:bg-sky-500 disabled:bg-zinc-700 disabled:text-zinc-500 rounded-lg transition-colors flex items-center gap-2"
            >
              {isImporting ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Importing...
                </>
              ) : (
                <>
                  <Upload className="h-4 w-4" />
                  Import {previewCount > 0 ? `${previewCount} QSOs` : ""}
                </>
              )}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
