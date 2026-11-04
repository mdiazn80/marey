import { useEffect, useMemo, useRef, useState } from "react";
import { Page } from "../../app/ui";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
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
  critical: "text-destructive",
  warning: "text-warning",
  info: "text-foreground/80",
  unknown: "text-muted-foreground",
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
          {feedback && <span className="text-xs text-muted-foreground">{feedback}</span>}
          <Select value={level} onValueChange={(value) => setLevel(value as LogSeverity | "all")}>
            <SelectTrigger size="sm" aria-label="Filter by severity" className="w-36">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {LEVELS.map((item) => (
                <SelectItem key={item.value} value={item.value}>
                  {item.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Button
            variant="outline"
            size="sm"
            onClick={() => run(() => copyText(asText()).then(() => "Copied"))}
          >
            Copy
          </Button>
          <Button
            variant="outline"
            size="sm"
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
          <Button variant="ghost" size="sm" onClick={clearLogs}>
            Clear
          </Button>
        </>
      }
    >
      <div className="selectable min-h-0 flex-1 overflow-y-auto rounded border border-border bg-card p-3 font-mono text-xs leading-5">
        {visible.length === 0 && <p className="text-muted-foreground">No log entries.</p>}
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
