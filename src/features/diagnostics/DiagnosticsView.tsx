import { useState } from "react";
import { Button, Page } from "../../app/ui";
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
          {feedback && <span className="text-xs text-white/50">{feedback}</span>}
          <Button
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
      <pre className="selectable overflow-x-auto rounded border border-white/10 bg-black/30 p-4 font-mono text-xs leading-5 text-white/85">
        {report}
      </pre>
    </Page>
  );
}
