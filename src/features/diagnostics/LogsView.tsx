import { useEffect, useMemo, useRef, useState } from "react";
import { Button, Page } from "../../app/ui";
import { copyText, exportText } from "../../lib/tauri";
import { usePanel } from "../../stores/panel";
import type { LogEntry } from "../../types/generated/LogEntry";
import type { LogSeverity } from "../../types/generated/LogSeverity";

const LEVELS: { value: LogSeverity | "all"; label: string }[] = [
  { value: "all", label: "All severities" },
  { value: "critical", label: "Critical" },
  { value: "warning", label: "Warning" },
  { value: "info", label: "Info" },
];

const SEVERITY_COLOR: Record<LogSeverity, string> = {
  critical: "text-red-300",
  warning: "text-amber-300",
  info: "text-white/70",
  unknown: "text-white/40",
};

function formatEntry(entry: LogEntry) {
  const time = new Date(entry.timestampMs).toISOString();
  return `${time} ${entry.severity.toUpperCase()} [${entry.source}] ${entry.message}`;
}

export function LogsView() {
  const logs = usePanel((s) => s.logs);
  const clearLogs = usePanel((s) => s.clearLogs);
  const [level, setLevel] = useState<LogSeverity | "all">("all");
  const [feedback, setFeedback] = useState("");
  const bottom = useRef<HTMLDivElement>(null);

  const visible = useMemo(
    () => (level === "all" ? logs : logs.filter((entry) => entry.severity === level)),
    [logs, level],
  );
  const asText = () => visible.map(formatEntry).join("\n");

  useEffect(() => {
    bottom.current?.scrollIntoView();
  }, [visible.length]);

  const run = (action: () => Promise<string>) => {
    action().then(setFeedback, (error) => setFeedback(String(error)));
  };

  return (
    <Page
      title="Logs"
      actions={
        <>
          {feedback && <span className="text-xs text-white/50">{feedback}</span>}
          <select
            aria-label="Filter by severity"
            className="rounded border border-white/15 bg-[#14171c] px-2 py-1 text-xs"
            value={level}
            onChange={(e) => setLevel(e.target.value as LogSeverity | "all")}
          >
            {LEVELS.map((item) => (
              <option key={item.value} value={item.value}>
                {item.label}
              </option>
            ))}
          </select>
          <Button onClick={() => run(() => copyText(asText()).then(() => "Copied"))}>Copy</Button>
          <Button
            onClick={() =>
              run(() =>
                exportText("marey-logs.txt", asText()).then((path) =>
                  path ? `Saved ${path}` : "",
                ),
              )
            }
          >
            Export
          </Button>
          <Button onClick={clearLogs}>Clear</Button>
        </>
      }
    >
      <div className="selectable min-h-0 flex-1 overflow-y-auto rounded border border-white/10 bg-black/30 p-3 font-mono text-xs leading-5">
        {visible.length === 0 && <p className="text-white/40">No log entries.</p>}
        {visible.map((entry, index) => (
          <div key={index} className={SEVERITY_COLOR[entry.severity]}>
            {formatEntry(entry)}
          </div>
        ))}
        <div ref={bottom} />
      </div>
    </Page>
  );
}
