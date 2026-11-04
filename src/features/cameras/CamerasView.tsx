import { useTranslation } from "react-i18next";
import { Page } from "@/components/layout";
import { Notice } from "@/components/status";
import { Button } from "@/components/ui/button";
import { LinkNotice } from "@/features/status/LinkNotice";
import { usePanel } from "@/stores/panel";
import { useStereoCanvases } from "./useStereoCanvases";

export function CamerasView() {
  const { t } = useTranslation();
  const imagesEnabled = usePanel((s) => s.state?.settings.imagesEnabled ?? false);
  const changeSettings = usePanel((s) => s.changeSettings);
  const { left, right, status } = useStereoCanvases(imagesEnabled);

  return (
    <Page
      title={t("views.cameras")}
      actions={
        <Button
          variant={imagesEnabled ? "outline" : "default"}
          size="sm"
          onClick={() => void changeSettings({ imagesEnabled: !imagesEnabled })}
        >
          {imagesEnabled ? t("cameras.stopImages") : t("cameras.allowImages")}
        </Button>
      }
    >
      <LinkNotice />
      {!imagesEnabled && (
        <Notice tone="muted" title={t("cameras.imagesOffTitle")}>
          {t("cameras.imagesOffBody")}
        </Notice>
      )}
      {imagesEnabled && status === "waiting" && <Notice tone="warn" title={t("cameras.waiting")} />}
      {imagesEnabled && status === "unsupported" && (
        <Notice tone="warn" title={t("cameras.unsupported")} />
      )}
      {imagesEnabled && (
        <div className="grid grid-cols-2 gap-3">
          {[
            { label: t("cameras.left"), ref: left },
            { label: t("cameras.right"), ref: right },
          ].map((camera) => (
            <figure key={camera.label}>
              <canvas
                ref={camera.ref}
                aria-label={camera.label}
                className="w-full rounded border border-border bg-black"
                style={{ imageRendering: "pixelated" }}
              />
              <figcaption className="mt-1 text-xs text-muted-foreground">{camera.label}</figcaption>
            </figure>
          ))}
        </div>
      )}
    </Page>
  );
}
