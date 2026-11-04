import { useState } from "react";
import { Page } from "../../app/ui";
import { Button } from "@/components/ui/button";
import { copyText } from "../../lib/tauri";
import { usePanel } from "../../stores/panel";
import { buildReport } from "./report";

export function DiagnosticsView() {
  const state = usePanel((s) => s.state);
  const [feedback, setFeedback] = useState("");
  if (!state) return <Page title="Diagnostics">Waiting for the backend…</Page>;
  const report = buildReport(state);
  return (
    <Page
      title="Diagnostics"
      actions={
        <>
          {feedback && <span className="text-xs text-muted-foreground">{feedback}</span>}
          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              copyText(report).then(
                () => setFeedback("Copied"),
                (error) => setFeedback(String(error)),
              );
            }}
          >
            Copy for bug report
          </Button>
        </>
      }
    >
      <pre className="selectable overflow-x-auto rounded border border-border bg-card p-4 font-mono text-xs leading-5 text-foreground">
        {report}
      </pre>
    </Page>
  );
}
