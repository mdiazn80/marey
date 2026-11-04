import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Page } from "@/components/layout";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useActionFeedback } from "@/hooks/useActionFeedback";
import { copyText, exportText } from "@/lib/tauri";
import { usePanel } from "@/stores/panel";
import type { LogEntry } from "@/types/generated/LogEntry";
import type { LogSeverity } from "@/types/generated/LogSeverity";

const LEVELS = ["all", "critical", "warning", "info"] as const;

const SEVERITY_COLOR: Record<LogSeverity, string> = {
  critical: "text-destructive",
  warning: "text-warning",
  info: "text-foreground/80",
  unknown: "text-muted-foreground",
};

/** Log lines stay untranslated, because they come from the backend and go into bug reports. */
function formatEntry(entry: LogEntry) {
  const time = new Date(entry.timestampMs).toISOString();
  return `${time} ${entry.severity.toUpperCase()} [${entry.source}] ${entry.message}`;
}

export function LogsView() {
  const { t } = useTranslation();
  const logs = usePanel((s) => s.logs);
  const clearLogs = usePanel((s) => s.clearLogs);
  const [level, setLevel] = useState<LogSeverity | "all">("all");
  const { feedback, run } = useActionFeedback();
  const bottom = useRef<HTMLDivElement>(null);

  const visible = useMemo(
    () => (level === "all" ? logs : logs.filter((entry) => entry.severity === level)),
    [logs, level],
  );
  const asText = () => visible.map(formatEntry).join("\n");

  useEffect(() => {
    bottom.current?.scrollIntoView();
  }, [visible.length]);

  return (
    <Page
      title={t("views.logs")}
      actions={
        <>
          {feedback && <span className="text-xs text-muted-foreground">{feedback}</span>}
          <Select value={level} onValueChange={(value) => setLevel(value as LogSeverity | "all")}>
            <SelectTrigger size="sm" aria-label={t("logs.filter")} className="w-44">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {LEVELS.map((item) => (
                <SelectItem key={item} value={item}>
                  {t(`logs.levels.${item}`)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Button
            variant="outline"
            size="sm"
            onClick={() => run(() => copyText(asText()).then(() => t("common.copied")))}
          >
            {t("logs.copy")}
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() =>
              run(() =>
                exportText("marey-logs.txt", asText()).then((path) =>
                  path ? t("logs.saved", { path }) : "",
                ),
              )
            }
          >
            {t("logs.export")}
          </Button>
          <Button variant="ghost" size="sm" onClick={clearLogs}>
            {t("logs.clear")}
          </Button>
        </>
      }
    >
      <div className="selectable min-h-0 flex-1 overflow-y-auto rounded border border-border bg-card p-3 font-mono text-xs leading-5">
        {visible.length === 0 && <p className="text-muted-foreground">{t("logs.empty")}</p>}
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
