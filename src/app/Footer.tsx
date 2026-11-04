import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { getAppVersion, openExternalUrl } from "@/lib/tauri";
import { usePanel } from "@/stores/panel";

const REPOSITORY_URL = "https://github.com/mdiazn80/marey";

/** The GitHub mark. Lucide no longer ships brand icons. */
function GitHubIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <path d="M12 .5a11.5 11.5 0 0 0-3.64 22.41c.58.1.79-.25.79-.56v-2.15c-3.2.7-3.88-1.36-3.88-1.36-.52-1.33-1.28-1.68-1.28-1.68-1.05-.72.08-.7.08-.7 1.16.08 1.77 1.19 1.77 1.19 1.03 1.77 2.7 1.26 3.36.96.1-.75.4-1.26.73-1.55-2.55-.29-5.24-1.28-5.24-5.68 0-1.26.45-2.28 1.19-3.09-.12-.29-.52-1.46.11-3.05 0 0 .97-.31 3.17 1.18a11 11 0 0 1 5.78 0c2.2-1.49 3.17-1.18 3.17-1.18.63 1.59.23 2.76.11 3.05.74.81 1.19 1.83 1.19 3.09 0 4.41-2.69 5.39-5.25 5.67.41.36.78 1.06.78 2.14v3.17c0 .31.21.67.8.56A11.5 11.5 0 0 0 12 .5Z" />
    </svg>
  );
}

export function Footer() {
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    getAppVersion().then(setVersion, () => setVersion(null));
  }, []);

  const openRepository = () => {
    openExternalUrl(REPOSITORY_URL).catch((error) =>
      usePanel.setState({ actionError: `Could not open the repository: ${String(error)}` }),
    );
  };

  return (
    <footer className="flex h-8 shrink-0 items-center justify-between border-t border-sidebar-border bg-sidebar px-3 text-xs text-muted-foreground">
      <span className="selectable font-mono">{version ? `v${version}` : ""}</span>
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon-xs"
              aria-label="Open the Marey repository on GitHub"
              onClick={openRepository}
            >
              <GitHubIcon />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="top">GitHub repository</TooltipContent>
        </Tooltip>
      </TooltipProvider>
    </footer>
  );
}
