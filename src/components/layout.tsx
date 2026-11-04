// Page structure shared by the views, composed from the shadcn/ui components.
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { Card as BaseCard, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export function Page({
  title,
  actions,
  children,
}: {
  title: string;
  actions?: ReactNode;
  children: ReactNode;
}) {
  return (
    <div className="flex h-full flex-col overflow-y-auto p-6">
      <header className="mb-5 flex items-center justify-between gap-4">
        <h1 className="font-heading text-xl font-semibold tracking-tight">{title}</h1>
        <div className="flex items-center gap-2">{actions}</div>
      </header>
      {children}
    </div>
  );
}

/** Shown by the views that have nothing to render until the first state arrives. */
export function WaitingPage({ title }: { title: string }) {
  const { t } = useTranslation();
  return <Page title={title}>{t("common.waitingForBackend")}</Page>;
}

export function Card({ title, children }: { title?: string; children: ReactNode }) {
  return (
    <BaseCard className="mb-4">
      {title && (
        <CardHeader>
          <CardTitle>{title}</CardTitle>
        </CardHeader>
      )}
      <CardContent>{children}</CardContent>
    </BaseCard>
  );
}

export function Row({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="flex justify-between gap-6 border-b border-border py-1.5 last:border-0">
      <span className="text-muted-foreground">{label}</span>
      <span className="selectable text-right font-mono">{value}</span>
    </div>
  );
}
