import { useTranslation } from "react-i18next";
import { Page, WaitingPage } from "@/components/layout";
import { Button } from "@/components/ui/button";
import { useActionFeedback } from "@/hooks/useActionFeedback";
import { copyText } from "@/lib/tauri";
import { usePanel } from "@/stores/panel";
import { buildReport } from "./report";

export function DiagnosticsView() {
  const { t } = useTranslation();
  const state = usePanel((s) => s.state);
  const { feedback, run } = useActionFeedback();
  if (!state) return <WaitingPage title={t("views.diagnostics")} />;
  const report = buildReport(state);
  return (
    <Page
      title={t("views.diagnostics")}
      actions={
        <>
          {feedback && <span className="text-xs text-muted-foreground">{feedback}</span>}
          <Button
            variant="outline"
            size="sm"
            onClick={() => run(() => copyText(report).then(() => t("common.copied")))}
          >
            {t("diagnostics.copyReport")}
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
