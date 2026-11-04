import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Notice } from "@/components/status";
import { connectBackend, usePanel } from "@/stores/panel";
import { Footer } from "./Footer";
import { Sidebar } from "./Sidebar";
import { VIEWS } from "./views";

export function App() {
  const { t } = useTranslation();
  const view = usePanel((s) => s.view);
  const mock = usePanel((s) => s.state?.link.mock ?? false);
  const actionError = usePanel((s) => s.actionError);
  const [startupError, setStartupError] = useState<string | null>(null);
  const { Component: CurrentView } = VIEWS[view];

  useEffect(() => {
    connectBackend().catch((error) => setStartupError(String(error)));
  }, []);

  return (
    <div className="flex h-full flex-col">
      <div className="flex min-h-0 flex-1">
        <Sidebar />
        <main className="flex min-w-0 flex-1 flex-col">
          {mock && (
            <div className="bg-primary/15 px-4 py-1.5 text-center text-xs font-medium text-primary">
              {t("app.simulationBanner")}
            </div>
          )}
          {(startupError || actionError) && (
            <div className="px-6 pt-4">
              <Notice
                tone="bad"
                title={startupError ? t("app.backendError") : t("app.actionError")}
              >
                {startupError ?? actionError}
              </Notice>
            </div>
          )}
          <div className="min-h-0 flex-1">
            <CurrentView />
          </div>
        </main>
      </div>
      <Footer />
    </div>
  );
}
