// Project-specific building blocks composed from the shadcn/ui components.
import { AlertCircle, CheckCircle2, Info, TriangleAlert } from "lucide-react";
import type { ReactNode } from "react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Card as BaseCard, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";

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

export type Tone = "good" | "warn" | "bad" | "muted" | "info";

const TONE_TEXT: Record<Tone, string> = {
  good: "text-success",
  warn: "text-warning",
  bad: "text-destructive",
  muted: "text-muted-foreground",
  info: "text-info",
};

const TONE_BADGE: Record<Tone, string> = {
  good: "border-transparent bg-success/15 text-success",
  warn: "border-transparent bg-warning/15 text-warning",
  bad: "border-transparent bg-destructive/15 text-destructive",
  muted: "border-transparent bg-muted text-muted-foreground",
  info: "border-transparent bg-info/15 text-info",
};

const TONE_ICON: Record<Tone, typeof Info> = {
  good: CheckCircle2,
  warn: TriangleAlert,
  bad: AlertCircle,
  muted: Info,
  info: Info,
};

export function StatusBadge({ tone, children }: { tone: Tone; children: ReactNode }) {
  return <Badge className={TONE_BADGE[tone]}>{children}</Badge>;
}

export function Notice({
  tone,
  title,
  children,
}: {
  tone: Tone;
  title: string;
  children?: ReactNode;
}) {
  const Icon = TONE_ICON[tone];
  return (
    <Alert role="status" className={cn("mb-4 bg-card", TONE_TEXT[tone])}>
      <Icon />
      <AlertTitle>{title}</AlertTitle>
      {children && <AlertDescription>{children}</AlertDescription>}
    </Alert>
  );
}
