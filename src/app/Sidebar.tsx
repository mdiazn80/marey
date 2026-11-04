import { useTranslation } from "react-i18next";
import lockup from "@/assets/brand/marey-lockup-on-dark.svg";
import { cn } from "@/lib/utils";
import { usePanel, type View } from "@/stores/panel";
import { VIEWS } from "./views";

export function Sidebar() {
  const { t } = useTranslation();
  const current = usePanel((s) => s.view);
  const setView = usePanel((s) => s.setView);
  return (
    <nav
      aria-label={t("app.viewsLabel")}
      className="flex w-48 shrink-0 flex-col gap-0.5 border-r border-sidebar-border bg-sidebar p-3"
    >
      <img src={lockup} alt="Marey" className="mb-5 h-8 w-fit px-2 pt-1" />
      {(Object.keys(VIEWS) as View[]).map((view) => {
        const { icon: Icon } = VIEWS[view];
        return (
          <button
            key={view}
            type="button"
            aria-current={view === current ? "page" : undefined}
            onClick={() => setView(view)}
            className={cn(
              "flex items-center gap-2.5 rounded-md px-2 py-1.5 text-left transition-colors",
              view === current
                ? "bg-sidebar-accent text-sidebar-accent-foreground"
                : "text-muted-foreground hover:bg-sidebar-accent/60 hover:text-sidebar-foreground",
            )}
          >
            <Icon className={cn("size-4", view === current && "text-primary")} />
            {t(`views.${view}`)}
          </button>
        );
      })}
    </nav>
  );
}
